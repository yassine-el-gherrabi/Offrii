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

// Circles (groups)
const C1: Uuid = uuid("c0000000-0000-4000-a000-000000000001");
const C2: Uuid = uuid("c0000000-0000-4000-a000-000000000002");

// Circles (direct / 1-1 friend circles)
const CD1: Uuid = uuid("c1000000-0000-4000-a000-000000000001"); // Emma ↔ Marie
const CD2: Uuid = uuid("c1000000-0000-4000-a000-000000000002"); // Emma ↔ Lucas
const CD3: Uuid = uuid("c1000000-0000-4000-a000-000000000003"); // Emma ↔ Sophie
const CD4: Uuid = uuid("c1000000-0000-4000-a000-000000000004"); // Emma ↔ Camille
const CD5: Uuid = uuid("c1000000-0000-4000-a000-000000000005"); // Emma ↔ Sarah

// Community wishes
const D1: Uuid = uuid("d0000000-0000-4000-a000-000000000001");
const D2: Uuid = uuid("d0000000-0000-4000-a000-000000000002");
const D3: Uuid = uuid("d0000000-0000-4000-a000-000000000003");

// Wish messages
const M1: Uuid = uuid("af000000-0000-4000-a000-000000000001");
const M2: Uuid = uuid("af000000-0000-4000-a000-000000000002");
const M3: Uuid = uuid("af000000-0000-4000-a000-000000000003");
const M4: Uuid = uuid("af000000-0000-4000-a000-000000000004");

// Notifications
const N1: Uuid = uuid("ae000000-0000-4000-a000-000000000001");
const N2: Uuid = uuid("ae000000-0000-4000-a000-000000000002");
const N3: Uuid = uuid("ae000000-0000-4000-a000-000000000003");
const N4: Uuid = uuid("ae000000-0000-4000-a000-000000000004");
const N5: Uuid = uuid("ae000000-0000-4000-a000-000000000005");

// Share links
const E1: Uuid = uuid("e0000000-0000-4000-a000-000000000001");

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
    )
    .await?;

    // ── 3. Friendships ──────────────────────────────────────────────────────
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
    seed_circle_share_rules(&mut *tx).await?;

    // ── 7. Share links ──────────────────────────────────────────────────────
    println!("[seed] inserting share links...");
    seed_share_links(&mut *tx).await?;

    // ── 8. Community wishes ─────────────────────────────────────────────────
    println!("[seed] inserting community wishes...");
    seed_community_wishes(&mut *tx).await?;

    // ── 9. Wish messages ────────────────────────────────────────────────────
    println!("[seed] inserting wish messages...");
    seed_wish_messages(&mut *tx).await?;

    // ── 10. Notifications ───────────────────────────────────────────────────
    println!("[seed] inserting notifications...");
    seed_notifications(&mut *tx).await?;

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

// ---------------------------------------------------------------------------
// 1. Users
// ---------------------------------------------------------------------------
#[allow(clippy::type_complexity)]
async fn seed_users(tx: &mut sqlx::PgConnection, pw_hash: &str) -> Result<()> {
    // (id, email, username, password_hash, display_name, email_verified,
    //  avatar_url, created_at, terms_accepted_at, last_active_at)
    let users: Vec<(
        Uuid,
        &str,
        &str,
        &str,
        &str,
        bool,
        &str,
        chrono::DateTime<Utc>,
        chrono::DateTime<Utc>,
        chrono::DateTime<Utc>,
    )> = vec![
        (
            U1,
            "emma@demo.com",
            "emma_b",
            pw_hash,
            "Emma",
            true,
            "https://cdn.offrii.com/demo/avatar-emma.jpg",
            ago_days(60),
            ago_days(60),
            ago_hours(1),
        ),
        (
            U2,
            "marie@demo.com",
            "marie_dupont",
            pw_hash,
            "Marie Dupont",
            true,
            "https://cdn.offrii.com/demo/avatar-marie.jpg",
            ago_days(50),
            ago_days(50),
            ago_hours(3),
        ),
        (
            U3,
            "lucas@demo.com",
            "lucas_d",
            pw_hash,
            "Lucas",
            true,
            "https://cdn.offrii.com/demo/avatar-lucas.jpg",
            ago_days(45),
            ago_days(45),
            ago_hours(6),
        ),
        (
            U4,
            "sophie@demo.com",
            "sophie_martin",
            pw_hash,
            "Sophie Martin",
            true,
            "https://cdn.offrii.com/demo/avatar-sophie.jpg",
            ago_days(40),
            ago_days(40),
            ago_hours(12),
        ),
        (
            U5,
            "camille@demo.com",
            "camille_r",
            pw_hash,
            "Camille R.",
            true,
            "https://cdn.offrii.com/demo/avatar-camille.jpg",
            ago_days(35),
            ago_days(35),
            ago_days(1),
        ),
        (
            U6,
            "sarah@demo.com",
            "sarah_l",
            pw_hash,
            "Sarah L.",
            true,
            "https://cdn.offrii.com/demo/avatar-sarah.jpg",
            ago_days(30),
            ago_days(30),
            ago_days(1),
        ),
    ];

    for (
        id,
        email,
        username,
        password_hash,
        display_name,
        email_verified,
        avatar_url,
        created_at,
        terms_accepted_at,
        last_active_at,
    ) in users
    {
        sqlx::query(
            "INSERT INTO users (id, email, username, password_hash, display_name,
                                oauth_provider, oauth_provider_id, email_verified,
                                token_version, is_admin, username_customized, avatar_url,
                                terms_accepted_at, last_active_at,
                                created_at, updated_at)
             VALUES ($1,$2,$3,$4,$5,NULL,NULL,$6,1,FALSE,TRUE,$7,$8,$9,$10,$10)
             ON CONFLICT (id) DO NOTHING",
        )
        .bind(id)
        .bind(email)
        .bind(username)
        .bind(password_hash)
        .bind(display_name)
        .bind(email_verified)
        .bind(avatar_url)
        .bind(terms_accepted_at)
        .bind(last_active_at)
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
) -> Result<()> {
    // B01: Ecouteurs sans fil — Emma, Tech, priority 1
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
    .bind("Écouteurs sans fil")
    .bind(Some("Casque audio premium avec réduction de bruit"))
    .bind(Some(d("349.00")))
    .bind(1_i16)
    .bind(Some(cat_tech))
    .bind("active")
    .bind(false)
    .bind(Some("https://cdn.offrii.com/demo/headphones.jpg"))
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
    .bind(ago_days(50))
    .bind(ago_days(50))
    .execute(&mut *tx)
    .await?;

    // B02: Sac en cuir italien — Emma, Mode, priority 1, CLAIMED by Marie (U2)
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
    .bind("Sac en cuir italien")
    .bind(Some("Cuir pleine fleur, couleur cognac"))
    .bind(Some(d("290.00")))
    .bind(1_i16)
    .bind(Some(cat_mode))
    .bind("active")
    .bind(false)
    .bind(Some("https://cdn.offrii.com/demo/bag.jpg"))
    .bind(None::<Vec<String>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(Some(U2))
    .bind(Some(ago_days(5)))
    .bind(Some("app"))
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<Uuid>)
    .bind(ago_days(45))
    .bind(ago_days(5))
    .execute(&mut *tx)
    .await?;

    // B03: Demain est un autre jour — Emma, Loisirs, priority 2
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
    .bind("Demain est un autre jour")
    .bind(Some("Le dernier roman de Guillaume Musso"))
    .bind(Some(d("22.90")))
    .bind(2_i16)
    .bind(Some(cat_loisirs))
    .bind("active")
    .bind(false)
    .bind(Some("https://cdn.offrii.com/demo/book.jpg"))
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
    .bind(ago_days(40))
    .bind(ago_days(40))
    .execute(&mut *tx)
    .await?;

    // B04: Eau de parfum boisée — Emma, Santé, priority 2
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
    .bind("Eau de parfum boisée")
    .bind(Some("Notes de santal, cèdre et vanille"))
    .bind(Some(d("95.00")))
    .bind(2_i16)
    .bind(Some(cat_sante))
    .bind("active")
    .bind(false)
    .bind(Some("https://cdn.offrii.com/demo/perfume.jpg"))
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
    .bind(ago_days(35))
    .bind(ago_days(35))
    .execute(&mut *tx)
    .await?;

    // B05: Sneakers blanches — Emma, Mode, priority 3
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
    .bind("Sneakers blanches")
    .bind(Some("Taille 39, cuir blanc"))
    .bind(Some(d("119.00")))
    .bind(3_i16)
    .bind(Some(cat_mode))
    .bind("active")
    .bind(false)
    .bind(Some("https://cdn.offrii.com/demo/sneakers.jpg"))
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
    .bind(ago_days(30))
    .bind(ago_days(30))
    .execute(&mut *tx)
    .await?;

    // B06: Bougie parfumée artisanale — Emma, Maison, priority 3, status 'purchased', claimed by Sophie (U4)
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
    .bind("Bougie parfumée artisanale")
    .bind(Some("Senteur figue et ambre, cire de soja"))
    .bind(Some(d("45.00")))
    .bind(3_i16)
    .bind(Some(cat_maison))
    .bind("purchased")
    .bind(false)
    .bind(Some("https://cdn.offrii.com/demo/candle.jpg"))
    .bind(None::<Vec<String>>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(Some(U4))
    .bind(Some(ago_days(3)))
    .bind(Some("app"))
    .bind(None::<String>)
    .bind(None::<Uuid>)
    .bind(None::<Uuid>)
    .bind(ago_days(25))
    .bind(ago_days(3))
    .execute(&mut *tx)
    .await?;

    // B07: Cours de poterie — Emma, Loisirs, priority 2
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
    .bind("Cours de poterie")
    .bind(Some("Bon cadeau pour un atelier découverte"))
    .bind(Some(d("85.00")))
    .bind(2_i16)
    .bind(Some(cat_loisirs))
    .bind("active")
    .bind(false)
    .bind(Some("https://cdn.offrii.com/demo/pottery.jpg"))
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

    // B08: Plaid en laine — Emma, Maison, priority 3
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
    .bind("Plaid en laine")
    .bind(Some("Laine mérinos, coloris beige naturel"))
    .bind(Some(d("130.00")))
    .bind(3_i16)
    .bind(Some(cat_maison))
    .bind("active")
    .bind(false)
    .bind(Some("https://cdn.offrii.com/demo/blanket.jpg"))
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

    // B09: AirPods Pro 3 — Marie (U2), Tech, priority 1
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
    .bind(U2)
    .bind("AirPods Pro 3")
    .bind(Some("Dernier modèle avec réduction de bruit adaptative"))
    .bind(Some(d("279.00")))
    .bind(1_i16)
    .bind(Some(cat_tech))
    .bind("active")
    .bind(false)
    .bind(Some("https://cdn.offrii.com/demo/airpods.jpg"))
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
    .bind(ago_days(40))
    .bind(ago_days(40))
    .execute(&mut *tx)
    .await?;

    // B10: Sac Longchamp Le Pliage — Marie (U2), Mode, priority 2, CLAIMED by Emma (U1)
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
    .bind(Some("Taille M, couleur noir"))
    .bind(Some(d("145.00")))
    .bind(2_i16)
    .bind(Some(cat_mode))
    .bind("active")
    .bind(false)
    .bind(Some("https://cdn.offrii.com/demo/longchamp.jpg"))
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
    .bind(ago_days(35))
    .bind(ago_days(2))
    .execute(&mut *tx)
    .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// 3. Friendships
// ---------------------------------------------------------------------------
async fn seed_friends(tx: &mut sqlx::PgConnection) -> Result<()> {
    // Emma (U1) is friends with everyone (U2..U6).
    // Canonical ordering: user_a_id < user_b_id — all U1 < U2..U6 since UUIDs are sequential.
    sqlx::query(
        "INSERT INTO friendships (user_a_id, user_b_id, created_at)
         VALUES ($1, $2, $7),
                ($1, $3, $8),
                ($1, $4, $9),
                ($1, $5, $10),
                ($1, $6, $11)
         ON CONFLICT (user_a_id, user_b_id) DO NOTHING",
    )
    .bind(U1) // $1
    .bind(U2) // $2
    .bind(U3) // $3
    .bind(U4) // $4
    .bind(U5) // $5
    .bind(U6) // $6
    .bind(ago_days(45)) // $7  — U1-U2
    .bind(ago_days(40)) // $8  — U1-U3
    .bind(ago_days(35)) // $9  — U1-U4
    .bind(ago_days(30)) // $10 — U1-U5
    .bind(ago_days(25)) // $11 — U1-U6
    .execute(&mut *tx)
    .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// 4. Circles
// ---------------------------------------------------------------------------
async fn seed_circles(tx: &mut sqlx::PgConnection) -> Result<()> {
    // Group circles
    sqlx::query(
        "INSERT INTO circles (id, name, owner_id, is_direct, image_url, created_at)
         VALUES
           ($1, 'Famille', $3, FALSE, 'https://cdn.offrii.com/demo/circle-famille.jpg', $5),
           ($2, 'Copines', $4, FALSE, 'https://cdn.offrii.com/demo/circle-copines.jpg', $6)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(C1) // $1
    .bind(C2) // $2
    .bind(U1) // $3 — owner of C1
    .bind(U1) // $4 — owner of C2
    .bind(ago_days(40)) // $5 — C1 created_at
    .bind(ago_days(30)) // $6 — C2 created_at
    .execute(&mut *tx)
    .await?;

    // Direct (1-1) friend circles — one per friendship
    sqlx::query(
        "INSERT INTO circles (id, owner_id, is_direct, created_at)
         VALUES
           ($1, $6, TRUE, $7),
           ($2, $6, TRUE, $8),
           ($3, $6, TRUE, $9),
           ($4, $6, TRUE, $10),
           ($5, $6, TRUE, $11)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(CD1) // $1
    .bind(CD2) // $2
    .bind(CD3) // $3
    .bind(CD4) // $4
    .bind(CD5) // $5
    .bind(U1) // $6 — owner (Emma)
    .bind(ago_days(45)) // $7  — CD1
    .bind(ago_days(40)) // $8  — CD2
    .bind(ago_days(35)) // $9  — CD3
    .bind(ago_days(30)) // $10 — CD4
    .bind(ago_days(25)) // $11 — CD5
    .execute(&mut *tx)
    .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// 5. Circle members
// ---------------------------------------------------------------------------
async fn seed_circle_members(tx: &mut sqlx::PgConnection) -> Result<()> {
    // Owner is auto-inserted by trigger, so we only add non-owner members.
    // C1 (Famille): U1 (owner), U2 (Marie), U3 (Lucas), U4 (Sophie)
    // C2 (Copines): U1 (owner), U5 (Camille), U6 (Sarah)
    sqlx::query(
        "INSERT INTO circle_members (circle_id, user_id, role, joined_at)
         VALUES
           ($1, $3, 'member', $7),
           ($1, $4, 'member', $8),
           ($1, $5, 'member', $9),
           ($2, $6, 'member', $10),
           ($2, $11, 'member', $12)
         ON CONFLICT (circle_id, user_id) DO NOTHING",
    )
    .bind(C1) // $1
    .bind(C2) // $2
    .bind(U2) // $3 — Marie in Famille
    .bind(U3) // $4 — Lucas in Famille
    .bind(U4) // $5 — Sophie in Famille
    .bind(U5) // $6 — Camille in Copines
    .bind(ago_days(39)) // $7
    .bind(ago_days(38)) // $8
    .bind(ago_days(37)) // $9
    .bind(ago_days(29)) // $10
    .bind(U6) // $11 — Sarah in Copines
    .bind(ago_days(28)) // $12
    .execute(&mut *tx)
    .await?;

    // Direct circle members — both users in each 1-1 circle
    // Owner (Emma/U1) is auto-inserted by trigger; we add the friend.
    sqlx::query(
        "INSERT INTO circle_members (circle_id, user_id, role, joined_at)
         VALUES
           ($1, $6, 'member', $11),
           ($2, $7, 'member', $12),
           ($3, $8, 'member', $13),
           ($4, $9, 'member', $14),
           ($5, $10, 'member', $15)
         ON CONFLICT (circle_id, user_id) DO NOTHING",
    )
    .bind(CD1) // $1
    .bind(CD2) // $2
    .bind(CD3) // $3
    .bind(CD4) // $4
    .bind(CD5) // $5
    .bind(U2) // $6  — Marie in CD1
    .bind(U3) // $7  — Lucas in CD2
    .bind(U4) // $8  — Sophie in CD3
    .bind(U5) // $9  — Camille in CD4
    .bind(U6) // $10 — Sarah in CD5
    .bind(ago_days(45)) // $11
    .bind(ago_days(40)) // $12
    .bind(ago_days(35)) // $13
    .bind(ago_days(30)) // $14
    .bind(ago_days(25)) // $15
    .execute(&mut *tx)
    .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// 6. Circle share rules
// ---------------------------------------------------------------------------
async fn seed_circle_share_rules(tx: &mut sqlx::PgConnection) -> Result<()> {
    // Emma shares ALL items to both circles
    sqlx::query(
        "INSERT INTO circle_share_rules (circle_id, user_id, share_mode, category_ids, created_at, updated_at)
         VALUES ($1, $3, 'all', '{}', $4, $4)
         ON CONFLICT (circle_id, user_id) DO NOTHING",
    )
    .bind(C1)
    .bind(C2) // unused
    .bind(U1)
    .bind(ago_days(40))
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO circle_share_rules (circle_id, user_id, share_mode, category_ids, created_at, updated_at)
         VALUES ($1, $2, 'all', '{}', $3, $3)
         ON CONFLICT (circle_id, user_id) DO NOTHING",
    )
    .bind(C2)
    .bind(U1)
    .bind(ago_days(30))
    .execute(&mut *tx)
    .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// 7. Share links
// ---------------------------------------------------------------------------
async fn seed_share_links(tx: &mut sqlx::PgConnection) -> Result<()> {
    // E1: Emma's "all items" link
    sqlx::query(
        "INSERT INTO share_links (id, user_id, token, label, permissions, scope, scope_data, is_active, expires_at, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(E1)
    .bind(U1)
    .bind("emma-wishlist")
    .bind(Some("Ma liste"))
    .bind("view_and_claim")
    .bind("all")
    .bind(None::<serde_json::Value>)
    .bind(true)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(ago_days(50))
    .execute(&mut *tx)
    .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// 8. Community wishes
// ---------------------------------------------------------------------------
async fn seed_community_wishes(tx: &mut sqlx::PgConnection) -> Result<()> {
    // D1: Sarah — matched with Emma
    sqlx::query(
        "INSERT INTO community_wishes (id, owner_id, title, description, category, status,
         is_anonymous, matched_with, matched_at, fulfilled_at, closed_at,
         report_count, reopen_count, last_reopen_at, moderation_note,
         image_url, links, og_image_url, og_title, og_site_name,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,NULL,NULL,NULL,$18,$19)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(D1)
    .bind(U6) // Sarah
    .bind("Vêtements chauds pour l'hiver")
    .bind(Some(
        "Taille M, manteau et écharpe si possible. Merci beaucoup !",
    ))
    .bind("clothing")
    .bind("matched")
    .bind(false)
    .bind(Some(U1)) // matched with Emma
    .bind(Some(ago_days(2))) // matched_at
    .bind(None::<chrono::DateTime<Utc>>) // fulfilled_at
    .bind(None::<chrono::DateTime<Utc>>) // closed_at
    .bind(0_i32) // report_count
    .bind(0_i32) // reopen_count
    .bind(None::<chrono::DateTime<Utc>>) // last_reopen_at
    .bind(None::<String>) // moderation_note
    .bind(Some("https://cdn.offrii.com/demo/winter-clothes.jpg"))
    .bind(None::<Vec<String>>) // links
    .bind(ago_days(3))
    .bind(ago_days(2))
    .execute(&mut *tx)
    .await?;

    // D2: Camille — open
    sqlx::query(
        "INSERT INTO community_wishes (id, owner_id, title, description, category, status,
         is_anonymous, matched_with, matched_at, fulfilled_at, closed_at,
         report_count, reopen_count, last_reopen_at, moderation_note,
         image_url, links, og_image_url, og_title, og_site_name,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,NULL,NULL,NULL,$18,$19)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(D2)
    .bind(U5) // Camille
    .bind("Jouets pour enfants 3-5 ans")
    .bind(Some(
        "Pour une association du quartier, jeux éducatifs ou peluches en bon état",
    ))
    .bind("children")
    .bind("open")
    .bind(false)
    .bind(None::<Uuid>) // matched_with
    .bind(None::<chrono::DateTime<Utc>>) // matched_at
    .bind(None::<chrono::DateTime<Utc>>) // fulfilled_at
    .bind(None::<chrono::DateTime<Utc>>) // closed_at
    .bind(0_i32)
    .bind(0_i32)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(None::<String>)
    .bind(Some("https://cdn.offrii.com/demo/toys.jpg"))
    .bind(None::<Vec<String>>)
    .bind(ago_days(2))
    .bind(ago_days(2))
    .execute(&mut *tx)
    .await?;

    // D3: Marie — open, no image
    sqlx::query(
        "INSERT INTO community_wishes (id, owner_id, title, description, category, status,
         is_anonymous, matched_with, matched_at, fulfilled_at, closed_at,
         report_count, reopen_count, last_reopen_at, moderation_note,
         image_url, links, og_image_url, og_title, og_site_name,
         created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,NULL,NULL,NULL,$18,$19)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(D3)
    .bind(U2) // Marie
    .bind("Livres scolaires lycée")
    .bind(Some("Manuels de terminale, toutes matières acceptées"))
    .bind("education")
    .bind("open")
    .bind(false)
    .bind(None::<Uuid>)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(0_i32)
    .bind(0_i32)
    .bind(None::<chrono::DateTime<Utc>>)
    .bind(None::<String>)
    .bind(None::<String>) // no image
    .bind(None::<Vec<String>>)
    .bind(ago_days(1))
    .bind(ago_days(1))
    .execute(&mut *tx)
    .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// 9. Wish messages
// ---------------------------------------------------------------------------
async fn seed_wish_messages(tx: &mut sqlx::PgConnection) -> Result<()> {
    let messages = [
        (
            M1,
            D1,
            Some(U1),
            "Bonjour ! J'ai un manteau taille M en très bon état, ça vous intéresse ?",
            ago_hours(23),
        ),
        (
            M2,
            D1,
            Some(U6),
            "Oh oui avec plaisir ! C'est vraiment gentil. Il est de quelle couleur ?",
            ago_hours(22),
        ),
        (
            M3,
            D1,
            Some(U1),
            "Bleu marine, avec une capuche. J'ai aussi une écharpe assortie !",
            ago_hours(21),
        ),
        (
            M4,
            D1,
            Some(U6),
            "Parfait, c'est exactement ce qu'il me faut. Merci infiniment !",
            ago_hours(20),
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
// 10. Notifications
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
        // N1: friend_accepted — Sarah accepted Emma's request (unread)
        (
            N1,
            U1,
            "friend_accepted",
            "Nouvelle amie",
            "Sarah a accepté votre demande",
            false,
            None,
            None,
            None,
            Some(U6),
            ago_hours(2),
        ),
        // N2: item_claimed — someone claimed B02 (unread)
        (
            N2,
            U1,
            "item_claimed",
            "Souhait réservé",
            "Quelqu'un a réservé un de vos souhaits",
            false,
            None,
            Some(B02),
            None,
            None,
            ago_hours(5),
        ),
        // N3: circle_member_joined — Sarah joined Copines (read)
        (
            N3,
            U1,
            "circle_member_joined",
            "Nouveau membre",
            "Sarah a rejoint Copines",
            true,
            Some(C2),
            None,
            None,
            Some(U6),
            ago_days(1),
        ),
        // N4: wish_offer — someone offers help for D1 (read)
        (
            N4,
            U1,
            "wish_offer",
            "Offre d'aide",
            "Quelqu'un propose son aide pour votre besoin",
            true,
            None,
            None,
            Some(D1),
            None,
            ago_days(2),
        ),
        // N5: item_received — Emma marked B06 as received (read)
        (
            N5,
            U1,
            "item_received",
            "Cadeau reçu",
            "Vous avez marqué un souhait comme reçu",
            true,
            None,
            Some(B06),
            None,
            None,
            ago_days(3),
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
