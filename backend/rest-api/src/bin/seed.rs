#![allow(clippy::explicit_auto_deref)] // &mut *tx is idiomatic for sqlx transactions
//! Dev seed binary — inserts fixture data using direct SQL with compile-time checked queries.
//!
//! Uses the same models and database connection as the main app. If a schema change
//! breaks a query here, the build will fail — unlike the old `dev_seed.sql`.
//!
//! All inserts use `ON CONFLICT DO NOTHING` so the seed is idempotent.
//! Password for all email+password users: `DemoPass123x`

use std::str::FromStr;

use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;

// Re-use the app's password hashing so the seed stays in sync with auth logic.
use rest_api::utils::hash::hash_password;

// ---------------------------------------------------------------------------
// Fixed UUIDs (deterministic seed)
// ---------------------------------------------------------------------------

// Users
const U1: Uuid = uuid("a0000000-0000-4000-a000-000000000001");
const U2: Uuid = uuid("a0000000-0000-4000-a000-000000000002");
const U3: Uuid = uuid("a0000000-0000-4000-a000-000000000003");
const U4: Uuid = uuid("a0000000-0000-4000-a000-000000000004");
const U5: Uuid = uuid("a0000000-0000-4000-a000-000000000005");
const U6: Uuid = uuid("a0000000-0000-4000-a000-000000000006");
const U7: Uuid = uuid("a0000000-0000-4000-a000-000000000007");
const U8: Uuid = uuid("a0000000-0000-4000-a000-000000000008");

// Items
const B01: Uuid = uuid("b0000000-0000-4000-a000-000000000001");
const B02: Uuid = uuid("b0000000-0000-4000-a000-000000000002");
const B03: Uuid = uuid("b0000000-0000-4000-a000-000000000003");
const B04: Uuid = uuid("b0000000-0000-4000-a000-000000000004");
const B05: Uuid = uuid("b0000000-0000-4000-a000-000000000005");
const B06: Uuid = uuid("b0000000-0000-4000-a000-000000000006");
const B07: Uuid = uuid("b0000000-0000-4000-a000-000000000007");
const B08: Uuid = uuid("b0000000-0000-4000-a000-000000000008");
const B09: Uuid = uuid("b0000000-0000-4000-a000-000000000009");
const B10: Uuid = uuid("b0000000-0000-4000-a000-000000000010");
const B11: Uuid = uuid("b0000000-0000-4000-a000-000000000011");
const B12: Uuid = uuid("b0000000-0000-4000-a000-000000000012");
const B13: Uuid = uuid("b0000000-0000-4000-a000-000000000013");
const B14: Uuid = uuid("b0000000-0000-4000-a000-000000000014");
const B15: Uuid = uuid("b0000000-0000-4000-a000-000000000015");
const B16: Uuid = uuid("b0000000-0000-4000-a000-000000000016");
const B17: Uuid = uuid("b0000000-0000-4000-a000-000000000017");
const B18: Uuid = uuid("b0000000-0000-4000-a000-000000000018");
const B19: Uuid = uuid("b0000000-0000-4000-a000-000000000019");
const B20: Uuid = uuid("b0000000-0000-4000-a000-000000000020");

// Circles
const C1: Uuid = uuid("c0000000-0000-4000-a000-000000000001");
const C2: Uuid = uuid("c0000000-0000-4000-a000-000000000002");
const C3: Uuid = uuid("c0000000-0000-4000-a000-000000000003");
const C4: Uuid = uuid("c0000000-0000-4000-a000-000000000004");

// Community wishes
const D1: Uuid = uuid("d0000000-0000-4000-a000-000000000001");
const D2: Uuid = uuid("d0000000-0000-4000-a000-000000000002");
const D3: Uuid = uuid("d0000000-0000-4000-a000-000000000003");
const D4: Uuid = uuid("d0000000-0000-4000-a000-000000000004");
const D5: Uuid = uuid("d0000000-0000-4000-a000-000000000005");
const D6: Uuid = uuid("d0000000-0000-4000-a000-000000000006");
const D7: Uuid = uuid("d0000000-0000-4000-a000-000000000007");
const D8: Uuid = uuid("d0000000-0000-4000-a000-000000000008");

// Share links
const E1: Uuid = uuid("e0000000-0000-4000-a000-000000000001");
const E2: Uuid = uuid("e0000000-0000-4000-a000-000000000002");
const E3: Uuid = uuid("e0000000-0000-4000-a000-000000000003");
const E4: Uuid = uuid("e0000000-0000-4000-a000-000000000004");

/// Compile-time UUID parsing (const fn).
const fn uuid(s: &str) -> Uuid {
    match Uuid::try_parse(s) {
        Ok(u) => u,
        Err(_) => panic!("invalid UUID literal"),
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;

    println!("[seed] connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .context("failed to connect to database")?;

    println!("[seed] running migrations...");
    sqlx::migrate!()
        .run(&pool)
        .await
        .context("migrations failed")?;

    println!("[seed] hashing password (Argon2id — this takes a moment)...");
    let pw_hash = hash_password("DemoPass123x").context("password hashing failed")?;

    // Look up global category IDs (seeded by migration)
    println!("[seed] looking up categories...");
    let cat_tech = cat_id(&pool, "Tech").await?;
    let cat_mode = cat_id(&pool, "Mode").await?;
    let cat_maison = cat_id(&pool, "Maison").await?;
    let cat_loisirs = cat_id(&pool, "Loisirs").await?;
    let cat_sante = cat_id(&pool, "Santé").await?;
    let cat_autre = cat_id(&pool, "Autre").await?;

    // Everything inside a transaction for atomicity.
    let mut tx = pool.begin().await?;

    // ── 1. Users ────────────────────────────────────────────────────────────
    println!("[seed] inserting users...");
    seed_users(&mut *tx, &pw_hash).await?;

    // ── 2. Items ────────────────────────────────────────────────────────────
    println!("[seed] inserting items...");
    seed_items(
        &mut *tx,
        cat_tech,
        cat_mode,
        cat_maison,
        cat_loisirs,
        cat_sante,
        cat_autre,
    )
    .await?;

    // ── 3. Friend requests & friendships ────────────────────────────────────
    println!("[seed] inserting friendships...");
    seed_friends(&mut *tx).await?;

    // ── 4. Circles ──────────────────────────────────────────────────────────
    println!("[seed] inserting circles...");
    seed_circles(&mut *tx).await?;

    // ── 5. Circle members ───────────────────────────────────────────────────
    println!("[seed] inserting circle members...");
    seed_circle_members(&mut *tx).await?;

    // ── 6. Circle share rules ───────────────────────────────────────────────
    println!("[seed] inserting circle share rules...");
    seed_circle_share_rules(&mut *tx, cat_tech, cat_mode).await?;

    // ── 7. Circle items ─────────────────────────────────────────────────────
    println!("[seed] inserting circle items...");
    seed_circle_items(&mut *tx).await?;

    // ── 8. Circle events ────────────────────────────────────────────────────
    println!("[seed] inserting circle events...");
    seed_circle_events(&mut *tx).await?;

    // ── 9. Circle invites ───────────────────────────────────────────────────
    println!("[seed] inserting circle invites...");
    seed_circle_invites(&mut *tx).await?;

    // ── 10. Share links ─────────────────────────────────────────────────────
    println!("[seed] inserting share links...");
    seed_share_links(&mut *tx, cat_tech).await?;

    // ── Back-fill claimed_via_link_id ───────────────────────────────────────
    sqlx::query(
        "UPDATE items SET claimed_via_link_id = $1
         WHERE id = $2 AND claimed_via_link_id IS NULL",
    )
    .bind(E1)
    .bind(B06)
    .execute(&mut *tx)
    .await?;

    // ── 11. Community wishes ────────────────────────────────────────────────
    println!("[seed] inserting community wishes...");
    seed_community_wishes(&mut *tx).await?;

    // ── 12. Wish messages ───────────────────────────────────────────────────
    println!("[seed] inserting wish messages...");
    seed_wish_messages(&mut *tx).await?;

    // ── 13. Wish reports ────────────────────────────────────────────────────
    println!("[seed] inserting wish reports...");
    seed_wish_reports(&mut *tx).await?;

    // ── 14. Wish blocks ─────────────────────────────────────────────────────
    println!("[seed] inserting wish blocks...");
    seed_wish_blocks(&mut *tx).await?;

    // ── 15. Notifications ───────────────────────────────────────────────────
    println!("[seed] inserting notifications...");
    seed_notifications(&mut *tx).await?;

    // ── 16. Push tokens ─────────────────────────────────────────────────────
    println!("[seed] inserting push tokens...");
    seed_push_tokens(&mut *tx).await?;

    // ── 17. Refresh tokens ──────────────────────────────────────────────────
    println!("[seed] inserting refresh tokens...");
    seed_refresh_tokens(&mut *tx).await?;

    // ── 18. Email verification tokens ───────────────────────────────────────
    println!("[seed] inserting email verification tokens...");
    seed_email_verification_tokens(&mut *tx).await?;

    tx.commit().await.context("transaction commit failed")?;

    println!("[seed] done — all dev fixture data seeded.");
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn cat_id(pool: &PgPool, name: &str) -> Result<Uuid> {
    let row: (Uuid,) = sqlx::query_as("SELECT id FROM categories WHERE name = $1 LIMIT 1")
        .bind(name)
        .fetch_one(pool)
        .await
        .with_context(|| format!("category '{name}' not found — have migrations run?"))?;
    Ok(row.0)
}

/// Parse a decimal literal. Panics on invalid input (fine for compile-time-known literals).
fn d(s: &str) -> Decimal {
    Decimal::from_str(s).expect("invalid decimal literal")
}

/// Shorthand for `Utc::now() - Duration::days(d)`.
fn ago_days(d: i64) -> chrono::DateTime<Utc> {
    Utc::now() - Duration::days(d)
}
fn ago_hours(h: i64) -> chrono::DateTime<Utc> {
    Utc::now() - Duration::hours(h)
}
fn ago_minutes(m: i64) -> chrono::DateTime<Utc> {
    Utc::now() - Duration::minutes(m)
}
fn from_now_days(d: i64) -> chrono::DateTime<Utc> {
    Utc::now() + Duration::days(d)
}
fn from_now_hours(h: i64) -> chrono::DateTime<Utc> {
    Utc::now() + Duration::hours(h)
}

// ---------------------------------------------------------------------------
// 1. Users
// ---------------------------------------------------------------------------
#[allow(clippy::type_complexity)]
async fn seed_users(tx: &mut sqlx::PgConnection, pw_hash: &str) -> Result<()> {
    // (id, email, username, password_hash, display_name,
    //  oauth_provider, oauth_provider_id, email_verified,
    //  is_admin, username_customized, avatar_url, created_at)
    let users: Vec<(
        Uuid,
        &str,
        &str,
        Option<&str>,
        Option<&str>,
        Option<&str>,
        Option<&str>,
        bool,
        bool,
        bool,
        Option<&str>,
        chrono::DateTime<Utc>,
    )> = vec![
        (
            U1,
            "yassine@demo.com",
            "yassine",
            Some(pw_hash),
            Some("Yassine"),
            None,
            None,
            true,
            true,
            true,
            Some("https://cdn.offrii.com/avatars/demo-yassine.jpg"),
            ago_days(30),
        ),
        (
            U2,
            "marie@demo.com",
            "marie_dupont",
            Some(pw_hash),
            Some("Marie Dupont"),
            None,
            None,
            true,
            false,
            true,
            Some("https://cdn.offrii.com/avatars/demo-marie.jpg"),
            ago_days(14),
        ),
        (
            U3,
            "lucas@demo.com",
            "lucas123",
            Some(pw_hash),
            None,
            None,
            None,
            false,
            false,
            false,
            None,
            ago_days(7),
        ),
        (
            U4,
            "sophie@gmail.com",
            "sophie_martin",
            None,
            Some("Sophie Martin"),
            Some("google"),
            Some("google_sophie_123"),
            true,
            false,
            true,
            Some("https://lh3.googleusercontent.com/demo-sophie"),
            ago_days(10),
        ),
        (
            U5,
            "thomas@icloud.com",
            "thomas_b",
            None,
            None,
            Some("apple"),
            Some("apple_thomas_456"),
            true,
            false,
            true,
            None,
            ago_days(5),
        ),
        (
            U6,
            "camille@demo.com",
            "camille_r",
            Some(pw_hash),
            Some("Camille R."),
            Some("google"),
            Some("google_camille_789"),
            true,
            false,
            true,
            None,
            ago_days(12),
        ),
        (
            U7,
            "newuser@demo.com",
            "new_user",
            Some(pw_hash),
            Some("Nouveau"),
            None,
            None,
            true,
            false,
            true,
            None,
            ago_hours(1),
        ),
        (
            U8,
            "reporter@demo.com",
            "reporter_user",
            Some(pw_hash),
            Some("Reporter"),
            None,
            None,
            true,
            false,
            true,
            None,
            ago_days(10),
        ),
    ];

    for (
        id,
        email,
        username,
        password_hash,
        display_name,
        oauth_provider,
        oauth_provider_id,
        email_verified,
        is_admin,
        username_customized,
        avatar_url,
        created_at,
    ) in users
    {
        sqlx::query(
            "INSERT INTO users (id, email, username, password_hash, display_name,
                                oauth_provider, oauth_provider_id, email_verified,
                                token_version, is_admin, username_customized, avatar_url,
                                created_at, updated_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,1,$9,$10,$11,$12,$12)
             ON CONFLICT (id) DO NOTHING",
        )
        .bind(id)
        .bind(email)
        .bind(username)
        .bind(password_hash)
        .bind(display_name)
        .bind(oauth_provider)
        .bind(oauth_provider_id)
        .bind(email_verified)
        .bind(is_admin)
        .bind(username_customized)
        .bind(avatar_url)
        .bind(created_at)
        .execute(&mut *tx)
        .await?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// 2. Items
// ---------------------------------------------------------------------------
async fn seed_items(
    tx: &mut sqlx::PgConnection,
    cat_tech: Uuid,
    cat_mode: Uuid,
    cat_maison: Uuid,
    cat_loisirs: Uuid,
    cat_sante: Uuid,
    cat_autre: Uuid,
) -> Result<()> {
    // Insert each item individually for clarity and type safety.
    // Using a helper macro to reduce boilerplate.

    // b01: MacBook Pro M4 — Yassine, Tech, active, high priority, with links+OG
    sqlx::query(
        "INSERT INTO items (id, user_id, name, description, estimated_price,
         priority, category_id, status, is_private,
         image_url, links, og_image_url, og_title, og_site_name,
         claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(B01)
    .bind(U1)
    .bind("MacBook Pro M4")
    .bind(Some("Le nouveau MacBook avec puce M4 Max"))
    .bind(Some(d("2999.00")))
    .bind(1_i16)
    .bind(Some(cat_tech))
    .bind("active")
    .bind(false)
    .bind(None::<String>) // image_url
    .bind(Some(vec![
        "https://www.apple.com/fr/macbook-pro/".to_owned(),
    ]))
    .bind(Some("https://www.apple.com/v/macbook-pro/og.jpg"))
    .bind(Some("MacBook Pro"))
    .bind(Some("Apple"))
    .bind(None::<Uuid>)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<Uuid>)
    .bind(ago_days(28))
    .bind(ago_days(28))
    .execute(&mut *tx)
    .await?;

    // b02: Veste en cuir Sandro — Yassine, Mode, active, medium
    sqlx::query(
        "INSERT INTO items (id, user_id, name, description, estimated_price,
         priority, category_id, status, is_private,
         image_url, links, og_image_url, og_title, og_site_name,
         claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(B02)
    .bind(U1)
    .bind("Veste en cuir Sandro")
    .bind(None::<String>)
    .bind(Some(d("450.00")))
    .bind(2_i16)
    .bind(Some(cat_mode))
    .bind("active")
    .bind(false)
    .bind(Some("https://cdn.offrii.com/items/demo-veste.jpg"))
    .bind(None::<Vec<String>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<Uuid>)
    .bind(ago_days(20))
    .bind(ago_days(20))
    .execute(&mut *tx)
    .await?;

    // b03: Journal intime Moleskine — Yassine, no cat, private, low
    sqlx::query(
        "INSERT INTO items (id, user_id, name, description, estimated_price,
         priority, category_id, status, is_private,
         image_url, links, og_image_url, og_title, og_site_name,
         claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(B03)
    .bind(U1)
    .bind("Journal intime Moleskine")
    .bind(Some("Un beau carnet pour ecrire"))
    .bind(None::<rust_decimal::Decimal>)
    .bind(3_i16)
    .bind(None::<Uuid>)
    .bind("active")
    .bind(true)
    .bind(None::<String>)
    .bind(None::<Vec<String>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<Uuid>)
    .bind(ago_days(15))
    .bind(ago_days(15))
    .execute(&mut *tx)
    .await?;

    // b04: Casque Sony WH-1000XM5 — Yassine, Tech, purchased, claimed by Marie (app)
    sqlx::query(
        "INSERT INTO items (id, user_id, name, description, estimated_price,
         priority, category_id, status, is_private,
         image_url, links, og_image_url, og_title, og_site_name,
         claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(B04)
    .bind(U1)
    .bind("Casque Sony WH-1000XM5")
    .bind(Some("Casque a reduction de bruit"))
    .bind(Some(d("350.00")))
    .bind(1_i16)
    .bind(Some(cat_tech))
    .bind("purchased")
    .bind(false)
    .bind(None::<String>)
    .bind(Some(vec![
        "https://www.sony.fr/headphones/wh-1000xm5".to_owned(),
    ]))
    .bind(Some("https://www.sony.fr/og-xm5.jpg"))
    .bind(Some("WH-1000XM5"))
    .bind(Some("Sony"))
    .bind(Some(U2))
    .bind(Some(ago_days(3)))
    .bind(Some("app"))
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<Uuid>)
    .bind(ago_days(25))
    .bind(ago_days(3))
    .execute(&mut *tx)
    .await?;

    // b05: Ancien souhait supprime — Yassine, deleted
    sqlx::query(
        "INSERT INTO items (id, user_id, name, description, estimated_price,
         priority, category_id, status, is_private,
         image_url, links, og_image_url, og_title, og_site_name,
         claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(B05)
    .bind(U1)
    .bind("Ancien souhait supprime")
    .bind(None::<String>)
    .bind(None::<rust_decimal::Decimal>)
    .bind(2_i16)
    .bind(None::<Uuid>)
    .bind("deleted")
    .bind(false)
    .bind(None::<String>)
    .bind(None::<Vec<String>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<Uuid>)
    .bind(ago_days(29))
    .bind(ago_days(10))
    .execute(&mut *tx)
    .await?;

    // b06: Lampe Dyson Solarcycle — Yassine, Maison, active
    sqlx::query(
        "INSERT INTO items (id, user_id, name, description, estimated_price,
         priority, category_id, status, is_private,
         image_url, links, og_image_url, og_title, og_site_name,
         claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(B06)
    .bind(U1)
    .bind("Lampe Dyson Solarcycle")
    .bind(None::<String>)
    .bind(Some(d("599.00")))
    .bind(2_i16)
    .bind(Some(cat_maison))
    .bind("active")
    .bind(false)
    .bind(Some("https://cdn.offrii.com/items/demo-lampe.jpg"))
    .bind(Some(vec![
        "https://www.dyson.fr/eclairage/lampes-de-bureau/solarcycle".to_owned(),
    ]))
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<Uuid>)
    .bind(ago_days(5))
    .bind(ago_days(5))
    .execute(&mut *tx)
    .await?;

    // b07: Zelda TOTK — Yassine, Loisirs, web-claimed by "Maman"
    sqlx::query(
        "INSERT INTO items (id, user_id, name, description, estimated_price,
         priority, category_id, status, is_private,
         image_url, links, og_image_url, og_title, og_site_name,
         claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(B07)
    .bind(U1)
    .bind("Zelda Tears of the Kingdom")
    .bind(Some("Edition collector Switch"))
    .bind(Some(d("69.99")))
    .bind(2_i16)
    .bind(Some(cat_loisirs))
    .bind("active")
    .bind(false)
    .bind(None::<String>)
    .bind(Some(vec!["https://www.nintendo.fr/zelda-totk".to_owned()]))
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(Some(ago_days(1)))
    .bind(Some("web"))
    .bind(Some("Maman"))
    .bind(None::<Uuid>)
    .bind(Some(uuid("f0000000-0000-4000-a000-000000000001")))
    .bind(ago_days(18))
    .bind(ago_days(1))
    .execute(&mut *tx)
    .await?;

    // b08: Tapis de yoga Lululemon — Yassine, Sante, high
    sqlx::query(
        "INSERT INTO items (id, user_id, name, description, estimated_price,
         priority, category_id, status, is_private,
         image_url, links, og_image_url, og_title, og_site_name,
         claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(B08)
    .bind(U1)
    .bind("Tapis de yoga Lululemon")
    .bind(Some("Le modele Reversible 5mm"))
    .bind(Some(d("88.00")))
    .bind(1_i16)
    .bind(Some(cat_sante))
    .bind("active")
    .bind(false)
    .bind(None::<String>)
    .bind(None::<Vec<String>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<Uuid>)
    .bind(ago_days(2))
    .bind(ago_days(2))
    .execute(&mut *tx)
    .await?;

    // b09: Carte cadeau FNAC — Yassine, Autre, low
    sqlx::query(
        "INSERT INTO items (id, user_id, name, description, estimated_price,
         priority, category_id, status, is_private,
         image_url, links, og_image_url, og_title, og_site_name,
         claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(B09)
    .bind(U1)
    .bind("Carte cadeau FNAC 50EUR")
    .bind(None::<String>)
    .bind(Some(d("50.00")))
    .bind(3_i16)
    .bind(Some(cat_autre))
    .bind("active")
    .bind(false)
    .bind(None::<String>)
    .bind(None::<Vec<String>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<Uuid>)
    .bind(ago_days(1))
    .bind(ago_days(1))
    .execute(&mut *tx)
    .await?;

    // b10: Sac Longchamp — Marie, Mode, high, with OG
    sqlx::query(
        "INSERT INTO items (id, user_id, name, description, estimated_price,
         priority, category_id, status, is_private,
         image_url, links, og_image_url, og_title, og_site_name,
         claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(B10)
    .bind(U2)
    .bind("Sac Longchamp Le Pliage")
    .bind(Some("Modele grand format en noir"))
    .bind(Some(d("145.00")))
    .bind(1_i16)
    .bind(Some(cat_mode))
    .bind("active")
    .bind(false)
    .bind(Some("https://cdn.offrii.com/items/demo-sac.jpg"))
    .bind(Some(vec![
        "https://www.longchamp.com/fr/le-pliage".to_owned(),
    ]))
    .bind(Some("https://www.longchamp.com/og.jpg"))
    .bind(Some("Le Pliage"))
    .bind(Some("Longchamp"))
    .bind(None::<Uuid>)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<Uuid>)
    .bind(ago_days(12))
    .bind(ago_days(12))
    .execute(&mut *tx)
    .await?;

    // b11: Bougie Diptyque — Marie, Maison, claimed by Yassine (app)
    sqlx::query(
        "INSERT INTO items (id, user_id, name, description, estimated_price,
         priority, category_id, status, is_private,
         image_url, links, og_image_url, og_title, og_site_name,
         claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(B11)
    .bind(U2)
    .bind("Bougie Diptyque Baies")
    .bind(Some("La grande 300g"))
    .bind(Some(d("68.00")))
    .bind(2_i16)
    .bind(Some(cat_maison))
    .bind("active")
    .bind(false)
    .bind(None::<String>)
    .bind(None::<Vec<String>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(Some(U1))
    .bind(Some(ago_days(2)))
    .bind(Some("app"))
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<Uuid>)
    .bind(ago_days(10))
    .bind(ago_days(2))
    .execute(&mut *tx)
    .await?;

    // b12: AirPods Pro 3 — Marie, Tech, low
    sqlx::query(
        "INSERT INTO items (id, user_id, name, description, estimated_price,
         priority, category_id, status, is_private,
         image_url, links, og_image_url, og_title, og_site_name,
         claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(B12)
    .bind(U2)
    .bind("AirPods Pro 3")
    .bind(None::<String>)
    .bind(Some(d("279.00")))
    .bind(3_i16)
    .bind(Some(cat_tech))
    .bind("active")
    .bind(false)
    .bind(None::<String>)
    .bind(Some(vec![
        "https://www.apple.com/fr/airpods-pro/".to_owned(),
    ]))
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<Uuid>)
    .bind(ago_days(8))
    .bind(ago_days(8))
    .execute(&mut *tx)
    .await?;

    // b13: Livre Devenir — Marie, purchased (self), no category
    sqlx::query(
        "INSERT INTO items (id, user_id, name, description, estimated_price,
         priority, category_id, status, is_private,
         image_url, links, og_image_url, og_title, og_site_name,
         claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(B13)
    .bind(U2)
    .bind("Livre \"Devenir\" de Michelle Obama")
    .bind(None::<String>)
    .bind(Some(d("24.90")))
    .bind(2_i16)
    .bind(None::<Uuid>)
    .bind("purchased")
    .bind(false)
    .bind(None::<String>)
    .bind(None::<Vec<String>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(Some(ago_days(5)))
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<Uuid>)
    .bind(ago_days(13))
    .bind(ago_days(5))
    .execute(&mut *tx)
    .await?;

    // b14: Surprise pour anniversaire — Marie, private
    sqlx::query(
        "INSERT INTO items (id, user_id, name, description, estimated_price,
         priority, category_id, status, is_private,
         image_url, links, og_image_url, og_title, og_site_name,
         claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(B14)
    .bind(U2)
    .bind("Surprise pour anniversaire Yassine")
    .bind(Some("Ne pas montrer!"))
    .bind(Some(d("200.00")))
    .bind(1_i16)
    .bind(None::<Uuid>)
    .bind("active")
    .bind(true)
    .bind(None::<String>)
    .bind(None::<Vec<String>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<Uuid>)
    .bind(ago_days(6))
    .bind(ago_days(6))
    .execute(&mut *tx)
    .await?;

    // b15: Un truc cool — Lucas, minimal
    sqlx::query(
        "INSERT INTO items (id, user_id, name, description, estimated_price,
         priority, category_id, status, is_private,
         image_url, links, og_image_url, og_title, og_site_name,
         claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(B15)
    .bind(U3)
    .bind("Un truc cool")
    .bind(None::<String>)
    .bind(None::<rust_decimal::Decimal>)
    .bind(2_i16)
    .bind(None::<Uuid>)
    .bind("active")
    .bind(false)
    .bind(None::<String>)
    .bind(None::<Vec<String>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<Uuid>)
    .bind(ago_days(6))
    .bind(ago_days(6))
    .execute(&mut *tx)
    .await?;

    // b16: Manette PS5 — Lucas, Loisirs
    sqlx::query(
        "INSERT INTO items (id, user_id, name, description, estimated_price,
         priority, category_id, status, is_private,
         image_url, links, og_image_url, og_title, og_site_name,
         claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(B16)
    .bind(U3)
    .bind("Manette PS5 DualSense")
    .bind(Some("Couleur Cosmic Red"))
    .bind(Some(d("69.99")))
    .bind(1_i16)
    .bind(Some(cat_loisirs))
    .bind("active")
    .bind(false)
    .bind(None::<String>)
    .bind(None::<Vec<String>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<Uuid>)
    .bind(ago_days(4))
    .bind(ago_days(4))
    .execute(&mut *tx)
    .await?;

    // b17: Parfum Chanel N5 — Sophie, Mode
    sqlx::query(
        "INSERT INTO items (id, user_id, name, description, estimated_price,
         priority, category_id, status, is_private,
         image_url, links, og_image_url, og_title, og_site_name,
         claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(B17)
    .bind(U4)
    .bind("Parfum Chanel N5")
    .bind(Some("Eau de parfum 100ml"))
    .bind(Some(d("135.00")))
    .bind(1_i16)
    .bind(Some(cat_mode))
    .bind("active")
    .bind(false)
    .bind(None::<String>)
    .bind(Some(vec![
        "https://www.chanel.com/fr/parfums/n5/".to_owned(),
    ]))
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<Uuid>)
    .bind(ago_days(9))
    .bind(ago_days(9))
    .execute(&mut *tx)
    .await?;

    // b18: Plaid en laine — Camille, Maison
    sqlx::query(
        "INSERT INTO items (id, user_id, name, description, estimated_price,
         priority, category_id, status, is_private,
         image_url, links, og_image_url, og_title, og_site_name,
         claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(B18)
    .bind(U6)
    .bind("Plaid en laine Zara Home")
    .bind(None::<String>)
    .bind(Some(d("59.99")))
    .bind(2_i16)
    .bind(Some(cat_maison))
    .bind("active")
    .bind(false)
    .bind(None::<String>)
    .bind(None::<Vec<String>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<Uuid>)
    .bind(ago_days(11))
    .bind(ago_days(11))
    .execute(&mut *tx)
    .await?;

    // b19: Mon premier souhait — New User
    sqlx::query(
        "INSERT INTO items (id, user_id, name, description, estimated_price,
         priority, category_id, status, is_private,
         image_url, links, og_image_url, og_title, og_site_name,
         claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(B19)
    .bind(U7)
    .bind("Mon premier souhait")
    .bind(Some("Je decouvre Offrii!"))
    .bind(None::<rust_decimal::Decimal>)
    .bind(2_i16)
    .bind(None::<Uuid>)
    .bind("active")
    .bind(false)
    .bind(None::<String>)
    .bind(None::<Vec<String>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<Uuid>)
    .bind(ago_minutes(30))
    .bind(ago_minutes(30))
    .execute(&mut *tx)
    .await?;

    // b20: Sneakers Nike Air Max 90 — Yassine, Mode, multiple links
    sqlx::query(
        "INSERT INTO items (id, user_id, name, description, estimated_price,
         priority, category_id, status, is_private,
         image_url, links, og_image_url, og_title, og_site_name,
         claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(B20)
    .bind(U1)
    .bind("Sneakers Nike Air Max 90")
    .bind(Some("Coloris blanc/gris taille 43"))
    .bind(Some(d("150.00")))
    .bind(2_i16)
    .bind(Some(cat_mode))
    .bind("active")
    .bind(false)
    .bind(Some("https://cdn.offrii.com/items/demo-sneakers.jpg"))
    .bind(Some(vec![
        "https://www.nike.com/fr/air-max-90".to_owned(),
        "https://www.zalando.fr/nike-air-max-90".to_owned(),
    ]))
    .bind(Some("https://www.nike.com/og-am90.jpg"))
    .bind(Some("Air Max 90"))
    .bind(Some("Nike"))
    .bind(None::<Uuid>)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<Uuid>)
    .bind(ago_days(3))
    .bind(ago_days(3))
    .execute(&mut *tx)
    .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// 3. Friendships
// ---------------------------------------------------------------------------
async fn seed_friends(tx: &mut sqlx::PgConnection) -> Result<()> {
    // Friend requests
    sqlx::query(
        "INSERT INTO friend_requests (id, from_user_id, to_user_id, status, created_at)
         VALUES
           ($1, $2, $3, 'accepted', $8),
           ($4, $2, $5, 'accepted', $9),
           ($6, $3, $7, 'accepted', $10),
           ($11, $12, $2, 'pending', $13),
           ($14, $5, $3, 'declined', $15)
         ON CONFLICT (from_user_id, to_user_id) DO NOTHING",
    )
    .bind(uuid("f1000000-0000-4000-a000-000000000001")) // $1
    .bind(U1) // $2
    .bind(U2) // $3
    .bind(uuid("f1000000-0000-4000-a000-000000000002")) // $4
    .bind(U3) // $5
    .bind(uuid("f1000000-0000-4000-a000-000000000003")) // $6
    .bind(U6) // $7
    .bind(ago_days(13)) // $8
    .bind(ago_days(6)) // $9
    .bind(ago_days(11)) // $10
    .bind(uuid("f1000000-0000-4000-a000-000000000004")) // $11
    .bind(U4) // $12
    .bind(ago_days(2)) // $13
    .bind(uuid("f1000000-0000-4000-a000-000000000005")) // $14
    .bind(ago_days(5)) // $15
    .execute(&mut *tx)
    .await?;

    // Actual friendships (canonical ordering: user_a_id < user_b_id)
    sqlx::query(
        "INSERT INTO friendships (user_a_id, user_b_id, created_at)
         VALUES ($1, $2, $3), ($1, $4, $5), ($2, $6, $7)
         ON CONFLICT (user_a_id, user_b_id) DO NOTHING",
    )
    .bind(U1)
    .bind(U2)
    .bind(ago_days(13))
    .bind(U3)
    .bind(ago_days(6))
    .bind(U6)
    .bind(ago_days(11))
    .execute(&mut *tx)
    .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// 4. Circles
// ---------------------------------------------------------------------------
async fn seed_circles(tx: &mut sqlx::PgConnection) -> Result<()> {
    sqlx::query(
        "INSERT INTO circles (id, name, owner_id, is_direct, image_url, created_at)
         VALUES
           ($1, NULL, $5, TRUE, NULL, $9),
           ($2, NULL, $5, TRUE, NULL, $10),
           ($3, 'Famille', $6, FALSE, 'https://cdn.offrii.com/circles/demo-famille.jpg', $11),
           ($4, 'Amis proches', $5, FALSE, NULL, $12)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(C1)
    .bind(C2)
    .bind(C3)
    .bind(C4) // $1-$4
    .bind(U1)
    .bind(U2) // $5-$6
    .bind(U1)
    .bind(U1) // $7-$8 (unused padding)
    .bind(ago_days(13)) // $9
    .bind(ago_days(6)) // $10
    .bind(ago_days(10)) // $11
    .bind(ago_days(8)) // $12
    .execute(&mut *tx)
    .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// 5. Circle members
// ---------------------------------------------------------------------------
async fn seed_circle_members(tx: &mut sqlx::PgConnection) -> Result<()> {
    // Owner is auto-inserted by trigger, so we only add non-owner members.
    sqlx::query(
        "INSERT INTO circle_members (circle_id, user_id, role, joined_at)
         VALUES
           ($1, $5, 'member', $12),
           ($2, $6, 'member', $13),
           ($3, $7, 'member', $14),
           ($3, $6, 'member', $15),
           ($3, $8, 'member', $15),
           ($4, $5, 'member', $16),
           ($4, $9, 'member', $17)
         ON CONFLICT (circle_id, user_id) DO NOTHING",
    )
    .bind(C1) // $1
    .bind(C2) // $2
    .bind(C3) // $3
    .bind(C4) // $4
    .bind(U2) // $5
    .bind(U3) // $6
    .bind(U1) // $7
    .bind(U6) // $8
    .bind(U4) // $9
    .bind(U4) // $10 (unused)
    .bind(U4) // $11 (unused)
    .bind(ago_days(13)) // $12 c1: u2
    .bind(ago_days(6)) // $13 c2: u3
    .bind(ago_days(10)) // $14 c3: u1
    .bind(ago_days(9)) // $15 c3: u3, u6
    .bind(ago_days(8)) // $16 c4: u2
    .bind(ago_days(7)) // $17 c4: u4
    .execute(&mut *tx)
    .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// 6. Circle share rules
// ---------------------------------------------------------------------------
async fn seed_circle_share_rules(
    tx: &mut sqlx::PgConnection,
    cat_tech: Uuid,
    cat_mode: Uuid,
) -> Result<()> {
    // c1: Yassine shares 'all' with Marie
    sqlx::query(
        "INSERT INTO circle_share_rules (circle_id, user_id, share_mode, category_ids, created_at, updated_at)
         VALUES ($1, $2, 'all', '{}', $3, $3)
         ON CONFLICT (circle_id, user_id) DO NOTHING"
    )
    .bind(C1).bind(U1).bind(ago_days(13))
    .execute(&mut *tx).await?;

    // c1: Marie shares 'categories' (Tech + Mode) with Yassine
    sqlx::query(
        "INSERT INTO circle_share_rules (circle_id, user_id, share_mode, category_ids, created_at, updated_at)
         VALUES ($1, $2, 'categories', $3, $4, $4)
         ON CONFLICT (circle_id, user_id) DO NOTHING"
    )
    .bind(C1).bind(U2)
    .bind(vec![cat_tech, cat_mode])
    .bind(ago_days(12))
    .execute(&mut *tx).await?;

    // c2: Yassine shares 'selection' with Lucas
    sqlx::query(
        "INSERT INTO circle_share_rules (circle_id, user_id, share_mode, category_ids, created_at, updated_at)
         VALUES ($1, $2, 'selection', '{}', $3, $3)
         ON CONFLICT (circle_id, user_id) DO NOTHING"
    )
    .bind(C2).bind(U1).bind(ago_days(6))
    .execute(&mut *tx).await?;

    // c2: Lucas shares 'none'
    sqlx::query(
        "INSERT INTO circle_share_rules (circle_id, user_id, share_mode, category_ids, created_at, updated_at)
         VALUES ($1, $2, 'none', '{}', $3, $3)
         ON CONFLICT (circle_id, user_id) DO NOTHING"
    )
    .bind(C2).bind(U3).bind(ago_days(6))
    .execute(&mut *tx).await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// 7. Circle items
// ---------------------------------------------------------------------------
async fn seed_circle_items(tx: &mut sqlx::PgConnection) -> Result<()> {
    sqlx::query(
        "INSERT INTO circle_items (circle_id, item_id, shared_by, shared_at)
         VALUES
           ($1, $3, $7, $8),
           ($1, $4, $7, $9),
           ($2, $3, $7, $10),
           ($5, $6, $11, $12)
         ON CONFLICT (circle_id, item_id) DO NOTHING",
    )
    .bind(C2) // $1
    .bind(C4) // $2
    .bind(B01) // $3
    .bind(B06) // $4
    .bind(C3) // $5
    .bind(B10) // $6
    .bind(U1) // $7
    .bind(ago_days(5)) // $8
    .bind(ago_days(4)) // $9
    .bind(ago_days(7)) // $10
    .bind(U2) // $11
    .bind(ago_days(9)) // $12
    .execute(&mut *tx)
    .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// 8. Circle events
// ---------------------------------------------------------------------------
async fn seed_circle_events(tx: &mut sqlx::PgConnection) -> Result<()> {
    let events = [
        // (id, circle_id, actor_id, event_type, target_item_id, target_user_id, created_at)
        (
            uuid("ca000000-0000-4000-a000-000000000001"),
            C3,
            U2,
            "member_joined",
            None,
            Some(U2),
            ago_days(10),
        ),
        (
            uuid("ca000000-0000-4000-a000-000000000002"),
            C3,
            U1,
            "member_joined",
            None,
            Some(U1),
            ago_days(10),
        ),
        (
            uuid("ca000000-0000-4000-a000-000000000003"),
            C3,
            U2,
            "item_shared",
            Some(B10),
            None,
            ago_days(9),
        ),
        (
            uuid("ca000000-0000-4000-a000-000000000004"),
            C2,
            U1,
            "item_shared",
            Some(B01),
            None,
            ago_days(5),
        ),
        (
            uuid("ca000000-0000-4000-a000-000000000005"),
            C1,
            U1,
            "item_claimed",
            Some(B11),
            None,
            ago_days(2),
        ),
        (
            uuid("ca000000-0000-4000-a000-000000000006"),
            C1,
            U2,
            "item_received",
            Some(B04),
            None,
            ago_days(3),
        ),
    ];

    for (id, circle_id, actor_id, event_type, target_item, target_user, created_at) in events {
        sqlx::query(
            "INSERT INTO circle_events (id, circle_id, actor_id, event_type, target_item_id, target_user_id, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (id) DO NOTHING"
        )
        .bind(id).bind(circle_id).bind(actor_id).bind(event_type)
        .bind(target_item).bind(target_user).bind(created_at)
        .execute(&mut *tx).await?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// 9. Circle invites
// ---------------------------------------------------------------------------
async fn seed_circle_invites(tx: &mut sqlx::PgConnection) -> Result<()> {
    // Active invite to Famille (c3)
    sqlx::query(
        "INSERT INTO circle_invites (id, circle_id, token, created_by, expires_at, max_uses, use_count, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (id) DO NOTHING"
    )
    .bind(uuid("c1000000-0000-4000-a000-000000000001"))
    .bind(C3)
    .bind("inv_famille_abc123def456")
    .bind(U2)
    .bind(from_now_days(7))
    .bind(5_i32).bind(2_i32)
    .bind(ago_days(3))
    .execute(&mut *tx).await?;

    // Expired invite to Amis proches (c4)
    sqlx::query(
        "INSERT INTO circle_invites (id, circle_id, token, created_by, expires_at, max_uses, use_count, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (id) DO NOTHING"
    )
    .bind(uuid("c1000000-0000-4000-a000-000000000002"))
    .bind(C4)
    .bind("inv_amis_expired_xyz789")
    .bind(U1)
    .bind(ago_days(1)) // expired
    .bind(1_i32).bind(0_i32)
    .bind(ago_days(8))
    .execute(&mut *tx).await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// 10. Share links
// ---------------------------------------------------------------------------
async fn seed_share_links(tx: &mut sqlx::PgConnection, cat_tech: Uuid) -> Result<()> {
    // e1: Yassine's "all items" link
    sqlx::query(
        "INSERT INTO share_links (id, user_id, token, label, permissions, scope, scope_data, is_active, expires_at, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (id) DO NOTHING"
    )
    .bind(E1).bind(U1)
    .bind("sl_yassine_all_abc123456789")
    .bind(Some("Ma liste complete"))
    .bind("view_and_claim").bind("all")
    .bind(None::<serde_json::Value>)
    .bind(true).bind(None::<chrono::DateTime<Utc>>)
    .bind(ago_days(20))
    .execute(&mut *tx).await?;

    // e2: Marie's category link (Tech only)
    let tech_scope = serde_json::json!({ "category_ids": [cat_tech.to_string()] });
    sqlx::query(
        "INSERT INTO share_links (id, user_id, token, label, permissions, scope, scope_data, is_active, expires_at, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (id) DO NOTHING"
    )
    .bind(E2).bind(U2)
    .bind("sl_marie_tech_def456789012")
    .bind(Some("Idees tech"))
    .bind("view_only").bind("category")
    .bind(Some(tech_scope))
    .bind(true).bind(Some(from_now_days(30)))
    .bind(ago_days(10))
    .execute(&mut *tx).await?;

    // e3: Yassine's selection link
    let sel_scope = serde_json::json!({ "item_ids": [B01.to_string(), B06.to_string()] });
    sqlx::query(
        "INSERT INTO share_links (id, user_id, token, label, permissions, scope, scope_data, is_active, expires_at, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (id) DO NOTHING"
    )
    .bind(E3).bind(U1)
    .bind("sl_yassine_sel_ghi789012345")
    .bind(Some("Pour Noel"))
    .bind("view_and_claim").bind("selection")
    .bind(Some(sel_scope))
    .bind(true).bind(None::<chrono::DateTime<Utc>>)
    .bind(ago_days(5))
    .execute(&mut *tx).await?;

    // e4: Yassine's expired/deactivated link
    sqlx::query(
        "INSERT INTO share_links (id, user_id, token, label, permissions, scope, scope_data, is_active, expires_at, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (id) DO NOTHING"
    )
    .bind(E4).bind(U1)
    .bind("sl_yassine_old_jkl012345678")
    .bind(Some("Ancien lien"))
    .bind("view_only").bind("all")
    .bind(None::<serde_json::Value>)
    .bind(false).bind(Some(ago_days(10)))
    .bind(ago_days(25))
    .execute(&mut *tx).await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// 11. Community wishes
// ---------------------------------------------------------------------------
#[allow(clippy::type_complexity)]
async fn seed_community_wishes(tx: &mut sqlx::PgConnection) -> Result<()> {
    let wishes: Vec<(
        Uuid,
        Uuid,
        &str,
        Option<&str>,
        &str,
        &str,
        bool,
        Option<Uuid>,
        Option<chrono::DateTime<Utc>>,
        Option<chrono::DateTime<Utc>>,
        Option<chrono::DateTime<Utc>>,
        i32,
        i32,
        Option<chrono::DateTime<Utc>>,
        Option<&str>,
        Option<&str>,
        Option<Vec<String>>,
        chrono::DateTime<Utc>,
        chrono::DateTime<Utc>,
    )> = vec![
        // d1: PENDING
        (
            D1,
            U3,
            "Manuels scolaires pour le lycee",
            Some("J ai besoin de manuels pour la rentree de septembre"),
            "education",
            "pending",
            false,
            None,
            None,
            None,
            None,
            0,
            0,
            None,
            None,
            None,
            None,
            ago_days(1),
            ago_days(1),
        ),
        // d2: OPEN
        (
            D2,
            U7,
            "Vetements chauds pour l hiver",
            Some("Taille M, manteau et echarpe"),
            "clothing",
            "open",
            false,
            None,
            None,
            None,
            None,
            0,
            0,
            None,
            None,
            Some("https://cdn.offrii.com/wishes/demo-vetements.jpg"),
            None,
            ago_days(3),
            ago_days(3),
        ),
        // d3: MATCHED
        (
            D3,
            U3,
            "Medicaments pour allergie",
            Some("Antihistaminiques en pharmacie"),
            "health",
            "matched",
            true,
            Some(U2),
            Some(ago_days(1)),
            None,
            None,
            0,
            0,
            None,
            None,
            None,
            Some(vec!["https://www.doctissimo.fr/allergie".to_owned()]),
            ago_days(5),
            ago_days(1),
        ),
        // d4: FULFILLED
        (
            D4,
            U6,
            "Articles de cuisine pour premier appartement",
            Some("Casseroles, poeles, ustensiles de base"),
            "home",
            "fulfilled",
            false,
            Some(U1),
            Some(ago_days(7)),
            Some(ago_days(2)),
            None,
            0,
            0,
            None,
            None,
            None,
            None,
            ago_days(10),
            ago_days(2),
        ),
        // d5: CLOSED
        (
            D5,
            U3,
            "Jouets pour ma petite soeur",
            Some("Elle a 5 ans, aime les puzzles"),
            "children",
            "closed",
            false,
            None,
            None,
            None,
            Some(ago_days(4)),
            0,
            0,
            None,
            None,
            None,
            None,
            ago_days(12),
            ago_days(4),
        ),
        // d6: REJECTED
        (
            D6,
            U8,
            "Demande non conforme",
            Some("Contenu problematique"),
            "other",
            "rejected",
            false,
            None,
            None,
            None,
            None,
            0,
            0,
            None,
            Some("Wish does not meet community guidelines"),
            None,
            None,
            ago_days(8),
            ago_days(7),
        ),
        // d7: FLAGGED
        (
            D7,
            U7,
            "Demande signale",
            Some("Ce souhait a ete signale par la communaute"),
            "other",
            "flagged",
            false,
            None,
            None,
            None,
            None,
            3,
            0,
            None,
            None,
            None,
            None,
            ago_days(6),
            ago_days(2),
        ),
        // d8: OPEN (reopened)
        (
            D8,
            U6,
            "Fournitures scolaires pour la rentree",
            Some("Cahiers, stylos, trousse"),
            "education",
            "open",
            true,
            None,
            None,
            None,
            None,
            0,
            1,
            Some(ago_days(1)),
            None,
            None,
            Some(vec![
                "https://www.amazon.fr/fournitures-scolaires".to_owned(),
            ]),
            ago_days(14),
            ago_days(1),
        ),
    ];

    for (
        id,
        owner_id,
        title,
        description,
        category,
        status,
        is_anonymous,
        matched_with,
        matched_at,
        fulfilled_at,
        closed_at,
        report_count,
        reopen_count,
        last_reopen_at,
        moderation_note,
        image_url,
        links,
        created_at,
        updated_at,
    ) in wishes
    {
        sqlx::query(
            "INSERT INTO community_wishes (id, owner_id, title, description, category, status,
             is_anonymous, matched_with, matched_at, fulfilled_at, closed_at,
             report_count, reopen_count, last_reopen_at, moderation_note,
             image_url, links, og_image_url, og_title, og_site_name,
             created_at, updated_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,NULL,NULL,NULL,$18,$19)
             ON CONFLICT (id) DO NOTHING"
        )
        .bind(id).bind(owner_id).bind(title).bind(description)
        .bind(category).bind(status).bind(is_anonymous)
        .bind(matched_with).bind(matched_at).bind(fulfilled_at).bind(closed_at)
        .bind(report_count).bind(reopen_count).bind(last_reopen_at).bind(moderation_note)
        .bind(image_url).bind(links)
        .bind(created_at).bind(updated_at)
        .execute(&mut *tx).await?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// 12. Wish messages
// ---------------------------------------------------------------------------
async fn seed_wish_messages(tx: &mut sqlx::PgConnection) -> Result<()> {
    let messages = [
        (
            uuid("aa000000-0000-4000-a000-000000000001"),
            D3,
            Some(U2),
            "Bonjour, je peux vous aider avec les medicaments. Quelle marque preferez-vous?",
            ago_days(1),
        ),
        (
            uuid("aa000000-0000-4000-a000-000000000002"),
            D3,
            Some(U3),
            "Merci beaucoup! Cetirizine si possible.",
            ago_hours(23),
        ),
        (
            uuid("aa000000-0000-4000-a000-000000000003"),
            D4,
            Some(U1),
            "J ai un set de cuisine complet a donner, ca vous interesse?",
            ago_days(6),
        ),
        (
            uuid("aa000000-0000-4000-a000-000000000004"),
            D4,
            Some(U6),
            "Oui, c est parfait! Merci enormement!",
            ago_days(6),
        ),
    ];

    for (id, wish_id, sender_id, body, created_at) in messages {
        sqlx::query(
            "INSERT INTO wish_messages (id, wish_id, sender_id, body, created_at)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (id) DO NOTHING",
        )
        .bind(id)
        .bind(wish_id)
        .bind(sender_id)
        .bind(body)
        .bind(created_at)
        .execute(&mut *tx)
        .await?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// 13. Wish reports
// ---------------------------------------------------------------------------
async fn seed_wish_reports(tx: &mut sqlx::PgConnection) -> Result<()> {
    let reports = [
        (
            uuid("ab000000-0000-4000-a000-000000000001"),
            D7,
            U8,
            "inappropriate",
            Some("Le contenu semble inapproprie pour la plateforme"),
            ago_days(3),
        ),
        (
            uuid("ab000000-0000-4000-a000-000000000002"),
            D7,
            U1,
            "spam",
            None,
            ago_days(2),
        ),
        (
            uuid("ab000000-0000-4000-a000-000000000003"),
            D7,
            U2,
            "other",
            Some("Pas clair ce qui est demande"),
            ago_days(2),
        ),
    ];

    for (id, wish_id, reporter_id, reason, details, created_at) in reports {
        sqlx::query(
            "INSERT INTO wish_reports (id, wish_id, reporter_id, reason, details, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (wish_id, reporter_id) DO NOTHING",
        )
        .bind(id)
        .bind(wish_id)
        .bind(reporter_id)
        .bind(reason)
        .bind(details)
        .bind(created_at)
        .execute(&mut *tx)
        .await?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// 14. Wish blocks
// ---------------------------------------------------------------------------
async fn seed_wish_blocks(tx: &mut sqlx::PgConnection) -> Result<()> {
    sqlx::query(
        "INSERT INTO wish_blocks (id, wish_id, user_id, created_at)
         VALUES ($1, $2, $3, $4), ($5, $6, $7, $8)
         ON CONFLICT (wish_id, user_id) DO NOTHING",
    )
    .bind(uuid("ac000000-0000-4000-a000-000000000001"))
    .bind(D7)
    .bind(U8)
    .bind(ago_days(3))
    .bind(uuid("ac000000-0000-4000-a000-000000000002"))
    .bind(D6)
    .bind(U1)
    .bind(ago_days(7))
    .execute(&mut *tx)
    .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// 15. Notifications
// ---------------------------------------------------------------------------
async fn seed_notifications(tx: &mut sqlx::PgConnection) -> Result<()> {
    #[allow(clippy::type_complexity)]
    let notifs: Vec<(
        Uuid,
        Uuid,
        &str,
        &str,
        &str,
        bool,
        Option<Uuid>,
        Option<Uuid>,
        Option<Uuid>,
        Option<Uuid>,
        chrono::DateTime<Utc>,
    )> = vec![
        // bb01: friend_request u1 from u4
        (
            uuid("bb000000-0000-4000-a000-000000000001"),
            U1,
            "friend_request",
            "Nouvelle demande d ami",
            "Sophie Martin vous a envoye une demande d ami",
            false,
            None,
            None,
            None,
            Some(U4),
            ago_days(2),
        ),
        // bb02: friend_accepted u2 from u1
        (
            uuid("bb000000-0000-4000-a000-000000000002"),
            U2,
            "friend_accepted",
            "Demande acceptee",
            "Yassine a accepte votre demande d ami",
            true,
            None,
            None,
            None,
            Some(U1),
            ago_days(13),
        ),
        // bb03: item_claimed u1
        (
            uuid("bb000000-0000-4000-a000-000000000003"),
            U1,
            "item_claimed",
            "Souhait reserve",
            "Marie Dupont a reserve \"Casque Sony WH-1000XM5\"",
            false,
            Some(C1),
            Some(B04),
            None,
            Some(U2),
            ago_days(3),
        ),
        // bb04: item_shared u2
        (
            uuid("bb000000-0000-4000-a000-000000000004"),
            U2,
            "item_shared",
            "Nouvel article partage",
            "Yassine a partage \"MacBook Pro M4\" dans Amis proches",
            true,
            Some(C4),
            Some(B01),
            None,
            Some(U1),
            ago_days(7),
        ),
        // bb05: circle_member_joined u2
        (
            uuid("bb000000-0000-4000-a000-000000000005"),
            U2,
            "circle_member_joined",
            "Nouveau membre",
            "Sophie Martin a rejoint Amis proches",
            true,
            Some(C4),
            None,
            None,
            Some(U4),
            ago_days(7),
        ),
        // bb06: wish_message u3
        (
            uuid("bb000000-0000-4000-a000-000000000006"),
            U3,
            "wish_message",
            "Nouveau message",
            "Vous avez recu un nouveau message concernant votre souhait",
            false,
            None,
            None,
            Some(D3),
            Some(U2),
            ago_days(1),
        ),
        // bb07: wish_offer u3
        (
            uuid("bb000000-0000-4000-a000-000000000007"),
            U3,
            "wish_offer",
            "Nouvelle proposition d aide",
            "Quelqu un souhaite vous aider avec \"Medicaments pour allergie\"",
            true,
            None,
            None,
            Some(D3),
            Some(U2),
            ago_days(1),
        ),
        // bb08: wish_confirmed u1
        (
            uuid("bb000000-0000-4000-a000-000000000008"),
            U1,
            "wish_confirmed",
            "Souhait confirme",
            "Camille R. a confirme la reception de votre aide",
            true,
            None,
            None,
            Some(D4),
            Some(U6),
            ago_days(2),
        ),
        // bb09: wish_moderation_flagged u7
        (
            uuid("bb000000-0000-4000-a000-000000000009"),
            U7,
            "wish_moderation_flagged",
            "Souhait signale",
            "Votre souhait a ete signale et est en cours de revision",
            false,
            None,
            None,
            Some(D7),
            None,
            ago_days(2),
        ),
        // bb10: wish_rejected u8
        (
            uuid("bb000000-0000-4000-a000-000000000010"),
            U8,
            "wish_rejected",
            "Souhait refuse",
            "Votre souhait ne respecte pas les regles de la communaute",
            true,
            None,
            None,
            Some(D6),
            None,
            ago_days(7),
        ),
        // bb11: web item_claimed u1
        (
            uuid("bb000000-0000-4000-a000-000000000011"),
            U1,
            "item_claimed",
            "Souhait reserve depuis le web",
            "Maman a reserve \"Zelda Tears of the Kingdom\" via votre lien de partage",
            false,
            None,
            Some(B07),
            None,
            None,
            ago_days(1),
        ),
    ];

    for (
        id,
        user_id,
        ntype,
        title,
        body,
        read,
        circle_id,
        item_id,
        wish_id,
        actor_id,
        created_at,
    ) in notifs
    {
        sqlx::query(
            "INSERT INTO notifications (id, user_id, type, title, body, read,
             circle_id, item_id, wish_id, actor_id, created_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
             ON CONFLICT (id) DO NOTHING",
        )
        .bind(id)
        .bind(user_id)
        .bind(ntype)
        .bind(title)
        .bind(body)
        .bind(read)
        .bind(circle_id)
        .bind(item_id)
        .bind(wish_id)
        .bind(actor_id)
        .bind(created_at)
        .execute(&mut *tx)
        .await?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// 16. Push tokens
// ---------------------------------------------------------------------------
async fn seed_push_tokens(tx: &mut sqlx::PgConnection) -> Result<()> {
    let tokens = [
        (
            uuid("ad000000-0000-4000-a000-000000000001"),
            U1,
            "apns_demo_token_yassine_iphone_abc123",
            "ios",
            ago_days(28),
        ),
        (
            uuid("ad000000-0000-4000-a000-000000000002"),
            U1,
            "apns_demo_token_yassine_ipad_def456",
            "ios",
            ago_days(15),
        ),
        (
            uuid("ad000000-0000-4000-a000-000000000003"),
            U2,
            "apns_demo_token_marie_iphone_ghi789",
            "ios",
            ago_days(12),
        ),
        (
            uuid("ad000000-0000-4000-a000-000000000004"),
            U3,
            "fcm_demo_token_lucas_android_jkl012",
            "android",
            ago_days(6),
        ),
    ];

    for (id, user_id, token, platform, created_at) in tokens {
        sqlx::query(
            "INSERT INTO push_tokens (id, user_id, token, platform, created_at)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (user_id, token) DO NOTHING",
        )
        .bind(id)
        .bind(user_id)
        .bind(token)
        .bind(platform)
        .bind(created_at)
        .execute(&mut *tx)
        .await?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// 17. Refresh tokens
// ---------------------------------------------------------------------------
async fn seed_refresh_tokens(tx: &mut sqlx::PgConnection) -> Result<()> {
    let tokens = [
        (
            uuid("ae000000-0000-4000-a000-000000000001"),
            U1,
            "sha256_demo_active_yassine_abc123def456ghi789",
            from_now_days(30),
            None,
            ago_days(1),
        ),
        (
            uuid("ae000000-0000-4000-a000-000000000002"),
            U1,
            "sha256_demo_revoked_yassine_jkl012mno345pqr678",
            from_now_days(15),
            Some(ago_days(5)),
            ago_days(20),
        ),
        (
            uuid("ae000000-0000-4000-a000-000000000003"),
            U2,
            "sha256_demo_active_marie_stu901vwx234yz567",
            from_now_days(30),
            None,
            ago_days(2),
        ),
        (
            uuid("ae000000-0000-4000-a000-000000000004"),
            U3,
            "sha256_demo_expired_lucas_abc789def012ghi345",
            ago_days(2),
            None,
            ago_days(32),
        ),
    ];

    for (id, user_id, token_hash, expires_at, revoked_at, created_at) in tokens {
        sqlx::query(
            "INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, revoked_at, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (id) DO NOTHING"
        )
        .bind(id).bind(user_id).bind(token_hash)
        .bind(expires_at).bind(revoked_at).bind(created_at)
        .execute(&mut *tx).await?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// 18. Email verification tokens
// ---------------------------------------------------------------------------
async fn seed_email_verification_tokens(tx: &mut sqlx::PgConnection) -> Result<()> {
    let tokens = [
        (
            uuid("af000000-0000-4000-a000-000000000001"),
            U3,
            "verify_lucas_demo_token_abc123def456ghi789jkl012mno345pqr678st",
            from_now_hours(24),
            ago_hours(1),
        ),
        (
            uuid("af000000-0000-4000-a000-000000000002"),
            U1,
            "verify_yassine_expired_uvw456xyz789abc012def345ghi678jkl901mn",
            ago_days(29),
            ago_days(30),
        ),
    ];

    for (id, user_id, token, expires_at, created_at) in tokens {
        sqlx::query(
            "INSERT INTO email_verification_tokens (id, user_id, token, expires_at, created_at)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (id) DO NOTHING",
        )
        .bind(id)
        .bind(user_id)
        .bind(token)
        .bind(expires_at)
        .bind(created_at)
        .execute(&mut *tx)
        .await?;
    }

    Ok(())
}
