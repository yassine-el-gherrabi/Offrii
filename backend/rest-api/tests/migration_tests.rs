use sqlx::PgPool;
use sqlx::migrate::Migrator;
use testcontainers::ContainerAsync;
use testcontainers::ImageExt;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

// ---------------------------------------------------------------------------
// MigrationDb -- lightweight helper (Postgres only, no Redis/Router)
// ---------------------------------------------------------------------------

struct MigrationDb {
    _pg_container: ContainerAsync<Postgres>,
    db: PgPool,
}

impl MigrationDb {
    async fn new() -> Self {
        let pg_container = Postgres::default()
            .with_tag("16-alpine")
            .start()
            .await
            .unwrap();

        let host = pg_container.get_host().await.unwrap();
        let port = pg_container.get_host_port_ipv4(5432).await.unwrap();
        let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

        let db = PgPool::connect(&url).await.unwrap();
        MIGRATOR.run(&db).await.unwrap();

        Self {
            _pg_container: pg_container,
            db,
        }
    }

    // -- information_schema helpers ------------------------------------------

    async fn table_exists(&self, table: &str) -> bool {
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_schema = 'public' AND table_name = $1
            )",
        )
        .bind(table)
        .fetch_one(&self.db)
        .await
        .unwrap();
        row.0
    }

    async fn column_info(
        &self,
        table: &str,
        column: &str,
    ) -> Option<(String, String, Option<String>)> {
        sqlx::query_as::<_, (String, String, Option<String>)>(
            "SELECT data_type, is_nullable, column_default
             FROM information_schema.columns
             WHERE table_schema = 'public' AND table_name = $1 AND column_name = $2",
        )
        .bind(table)
        .bind(column)
        .fetch_optional(&self.db)
        .await
        .unwrap()
    }

    async fn column_count(&self, table: &str) -> i64 {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM information_schema.columns
             WHERE table_schema = 'public' AND table_name = $1",
        )
        .bind(table)
        .fetch_one(&self.db)
        .await
        .unwrap();
        row.0
    }

    async fn index_exists(&self, name: &str) -> bool {
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS (
                SELECT 1 FROM pg_indexes
                WHERE schemaname = 'public' AND indexname = $1
            )",
        )
        .bind(name)
        .fetch_one(&self.db)
        .await
        .unwrap();
        row.0
    }

    async fn trigger_exists(&self, name: &str) -> bool {
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.triggers
                WHERE trigger_schema = 'public' AND trigger_name = $1
            )",
        )
        .bind(name)
        .fetch_one(&self.db)
        .await
        .unwrap();
        row.0
    }

    async fn function_exists(&self, name: &str) -> bool {
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS (
                SELECT 1 FROM pg_proc p
                JOIN pg_namespace n ON p.pronamespace = n.oid
                WHERE n.nspname = 'public' AND p.proname = $1
            )",
        )
        .bind(name)
        .fetch_one(&self.db)
        .await
        .unwrap();
        row.0
    }

    async fn fk_exists(&self, table: &str, column: &str) -> bool {
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.key_column_usage kcu
                JOIN information_schema.table_constraints tc
                  ON kcu.constraint_name = tc.constraint_name
                  AND kcu.table_schema = tc.table_schema
                WHERE tc.constraint_type = 'FOREIGN KEY'
                  AND kcu.table_schema = 'public'
                  AND kcu.table_name = $1
                  AND kcu.column_name = $2
            )",
        )
        .bind(table)
        .bind(column)
        .fetch_one(&self.db)
        .await
        .unwrap();
        row.0
    }

    async fn unique_constraint_exists(&self, table: &str, columns: &[&str]) -> bool {
        let mut expected: Vec<&str> = columns.to_vec();
        expected.sort();

        let rows: Vec<(Vec<String>,)> = sqlx::query_as(
            "SELECT array_agg(a.attname ORDER BY a.attname) AS cols
                FROM pg_index i
                JOIN pg_class c ON c.oid = i.indrelid
                JOIN pg_namespace n ON n.oid = c.relnamespace
                JOIN pg_attribute a ON a.attrelid = c.oid AND a.attnum = ANY(i.indkey)
                WHERE n.nspname = 'public'
                  AND c.relname = $1
                  AND i.indisunique = true
                GROUP BY i.indexrelid",
        )
        .bind(table)
        .fetch_all(&self.db)
        .await
        .unwrap();

        rows.iter().any(|(cols,)| {
            let sorted: Vec<&str> = cols.iter().map(|s| s.as_str()).collect();
            sorted == expected
        })
    }

    async fn assert_not_null(&self, table: &str, column: &str) {
        let info = self.column_info(table, column).await;
        let (_, nullable, _) = info.unwrap_or_else(|| panic!("{table}.{column} should exist"));
        assert_eq!(
            nullable, "NO",
            "{table}.{column} should be NOT NULL, got nullable={nullable}"
        );
    }

    async fn assert_nullable(&self, table: &str, column: &str) {
        let info = self.column_info(table, column).await;
        let (_, nullable, _) = info.unwrap_or_else(|| panic!("{table}.{column} should exist"));
        assert_eq!(
            nullable, "YES",
            "{table}.{column} should be nullable, got nullable={nullable}"
        );
    }
}

// ===========================================================================
// Users table (final state: 17 columns, no reminder/timezone/locale)
// ===========================================================================

#[tokio::test]
async fn users_table_final_state() {
    let mdb = MigrationDb::new().await;

    assert!(mdb.table_exists("users").await);
    assert_eq!(mdb.column_count("users").await, 17);

    mdb.assert_not_null("users", "id").await;
    mdb.assert_not_null("users", "email").await;
    mdb.assert_not_null("users", "username").await;
    mdb.assert_nullable("users", "password_hash").await;
    mdb.assert_nullable("users", "display_name").await;
    mdb.assert_nullable("users", "oauth_provider").await;
    mdb.assert_nullable("users", "oauth_provider_id").await;
    mdb.assert_not_null("users", "email_verified").await;
    mdb.assert_not_null("users", "token_version").await;
    mdb.assert_not_null("users", "is_admin").await;
    mdb.assert_not_null("users", "username_customized").await;
    mdb.assert_nullable("users", "avatar_url").await;
    mdb.assert_nullable("users", "terms_accepted_at").await;
    mdb.assert_nullable("users", "last_active_at").await;
    mdb.assert_nullable("users", "inactivity_notice_sent_at")
        .await;
    mdb.assert_not_null("users", "created_at").await;
    mdb.assert_not_null("users", "updated_at").await;

    // Reminder columns must NOT exist
    assert!(mdb.column_info("users", "reminder_freq").await.is_none());
    assert!(mdb.column_info("users", "timezone").await.is_none());
    assert!(mdb.column_info("users", "locale").await.is_none());

    // Data types
    let (dt, _, _) = mdb.column_info("users", "id").await.unwrap();
    assert_eq!(dt, "uuid");
    let (dt, _, _) = mdb.column_info("users", "email").await.unwrap();
    assert_eq!(dt, "character varying");
    let (dt, _, _) = mdb.column_info("users", "created_at").await.unwrap();
    assert!(dt.contains("timestamp"), "expected timestamptz, got {dt}");

    // UNIQUE on email and username
    assert!(mdb.unique_constraint_exists("users", &["email"]).await);
    assert!(mdb.unique_constraint_exists("users", &["username"]).await);

    // OAuth partial unique index
    assert!(mdb.index_exists("idx_users_oauth").await);

    // Trigger
    assert!(mdb.trigger_exists("trg_users_updated_at").await);
    assert!(mdb.function_exists("set_updated_at").await);

    // Cleanup trigger for community wishes
    assert!(mdb.trigger_exists("trg_cleanup_matched_wishes").await);
    assert!(
        mdb.function_exists("cleanup_matched_wishes_on_user_delete")
            .await
    );
}

// ===========================================================================
// Related user tables
// ===========================================================================

#[tokio::test]
async fn user_related_tables_exist() {
    let mdb = MigrationDb::new().await;

    assert!(mdb.table_exists("connection_logs").await);
    assert_eq!(mdb.column_count("connection_logs").await, 5);
    assert!(mdb.index_exists("idx_connection_logs_user").await);
    assert!(mdb.index_exists("idx_connection_logs_created").await);

    assert!(mdb.table_exists("email_verification_tokens").await);
    assert_eq!(mdb.column_count("email_verification_tokens").await, 5);

    assert!(mdb.table_exists("email_change_tokens").await);
    assert_eq!(mdb.column_count("email_change_tokens").await, 6);
    assert!(mdb.index_exists("idx_email_change_user").await);
}

// ===========================================================================
// Categories table (global only, no user_id)
// ===========================================================================

#[tokio::test]
async fn categories_table_final_state() {
    let mdb = MigrationDb::new().await;

    assert!(mdb.table_exists("categories").await);
    assert_eq!(mdb.column_count("categories").await, 6);

    mdb.assert_not_null("categories", "id").await;
    mdb.assert_not_null("categories", "name").await;
    mdb.assert_nullable("categories", "icon").await;
    mdb.assert_not_null("categories", "is_default").await;
    mdb.assert_not_null("categories", "position").await;
    mdb.assert_not_null("categories", "created_at").await;

    // No user_id column
    assert!(mdb.column_info("categories", "user_id").await.is_none());
}

// ===========================================================================
// Seed categories
// ===========================================================================

#[tokio::test]
async fn seed_categories_inserted() {
    let mdb = MigrationDb::new().await;

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM categories WHERE is_default = true")
        .fetch_one(&mdb.db)
        .await
        .unwrap();
    assert_eq!(count.0, 6, "expected 6 default categories");

    let names: Vec<(String, String)> = sqlx::query_as(
        "SELECT name, icon FROM categories
         WHERE is_default = true
         ORDER BY position",
    )
    .fetch_all(&mdb.db)
    .await
    .unwrap();

    let expected = [
        ("Tech", "laptop"),
        ("Mode", "tshirt"),
        ("Maison", "home"),
        ("Loisirs", "gamepad"),
        ("Sant\u{00e9}", "heart"),
        ("Autre", "tag"),
    ];

    for (i, (name, icon)) in names.iter().enumerate() {
        assert_eq!(
            name, expected[i].0,
            "category name mismatch at position {i}"
        );
        assert_eq!(
            icon, expected[i].1,
            "category icon mismatch at position {i}"
        );
    }
}

// ===========================================================================
// Items table (final state: no url column, has links[], claim fields, OG)
// ===========================================================================

#[tokio::test]
async fn items_table_final_state() {
    let mdb = MigrationDb::new().await;

    assert!(mdb.table_exists("items").await);
    // 23 columns: id, user_id, name, description, estimated_price, priority,
    // category_id, status, purchased_at, created_at, updated_at,
    // claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
    // image_url, links, og_image_url, og_title, og_site_name, is_private
    assert_eq!(mdb.column_count("items").await, 23);

    // No url column
    assert!(mdb.column_info("items", "url").await.is_none());

    mdb.assert_not_null("items", "id").await;
    mdb.assert_not_null("items", "user_id").await;
    mdb.assert_not_null("items", "name").await;
    mdb.assert_nullable("items", "description").await;
    mdb.assert_nullable("items", "estimated_price").await;
    mdb.assert_not_null("items", "priority").await;
    mdb.assert_nullable("items", "category_id").await;
    mdb.assert_not_null("items", "status").await;
    mdb.assert_nullable("items", "purchased_at").await;
    mdb.assert_not_null("items", "created_at").await;
    mdb.assert_not_null("items", "updated_at").await;
    mdb.assert_nullable("items", "claimed_by").await;
    mdb.assert_nullable("items", "claimed_at").await;
    mdb.assert_nullable("items", "claimed_via").await;
    mdb.assert_nullable("items", "claimed_name").await;
    mdb.assert_nullable("items", "claimed_via_link_id").await;
    mdb.assert_nullable("items", "web_claim_token").await;
    mdb.assert_nullable("items", "image_url").await;
    mdb.assert_nullable("items", "links").await;
    mdb.assert_nullable("items", "og_image_url").await;
    mdb.assert_nullable("items", "og_title").await;
    mdb.assert_nullable("items", "og_site_name").await;
    mdb.assert_not_null("items", "is_private").await;

    // FKs
    assert!(mdb.fk_exists("items", "user_id").await);
    assert!(mdb.fk_exists("items", "category_id").await);
    assert!(mdb.fk_exists("items", "claimed_by").await);
    assert!(mdb.fk_exists("items", "claimed_via_link_id").await);

    // Indexes
    assert!(mdb.index_exists("idx_items_user_status").await);
    assert!(mdb.index_exists("idx_items_user_priority").await);
    assert!(mdb.index_exists("idx_items_created_at").await);
    assert!(mdb.index_exists("idx_items_category_id").await);
    assert!(mdb.index_exists("idx_items_claimed_by").await);
    assert!(mdb.index_exists("idx_items_web_claim_token").await);

    // Triggers
    assert!(mdb.trigger_exists("trg_items_updated_at").await);
    assert!(mdb.trigger_exists("trg_items_set_purchased_at").await);
    assert!(mdb.function_exists("set_purchased_at").await);
}

// ===========================================================================
// Circles and related tables
// ===========================================================================

#[tokio::test]
async fn circles_tables_final_state() {
    let mdb = MigrationDb::new().await;

    // circles: 6 columns (id, name, owner_id, is_direct, image_url, created_at)
    assert!(mdb.table_exists("circles").await);
    assert_eq!(mdb.column_count("circles").await, 6);
    mdb.assert_not_null("circles", "id").await;
    mdb.assert_nullable("circles", "name").await;
    mdb.assert_not_null("circles", "owner_id").await;
    mdb.assert_not_null("circles", "is_direct").await;
    mdb.assert_nullable("circles", "image_url").await;
    mdb.assert_not_null("circles", "created_at").await;
    assert!(mdb.fk_exists("circles", "owner_id").await);

    // circle_members: 4 columns
    assert!(mdb.table_exists("circle_members").await);
    assert_eq!(mdb.column_count("circle_members").await, 4);
    assert!(
        mdb.unique_constraint_exists("circle_members", &["circle_id", "user_id"])
            .await
    );
    assert!(mdb.index_exists("idx_circle_members_user").await);

    // circle_items
    assert!(mdb.table_exists("circle_items").await);
    assert!(mdb.index_exists("idx_circle_items_item_id").await);

    // circle_events
    assert!(mdb.table_exists("circle_events").await);
    assert!(mdb.index_exists("idx_circle_events_circle_created").await);

    // circle_share_rules
    assert!(mdb.table_exists("circle_share_rules").await);
    assert!(mdb.index_exists("idx_circle_share_rules_user").await);

    // circle_invites
    assert!(mdb.table_exists("circle_invites").await);
    assert!(mdb.index_exists("idx_circle_invites_circle_id").await);

    // Triggers
    assert!(mdb.trigger_exists("trg_circles_add_owner_member").await);
    assert!(mdb.function_exists("add_circle_owner_as_member").await);
    assert!(
        mdb.trigger_exists("trg_check_direct_circle_member_limit")
            .await
    );
    assert!(
        mdb.function_exists("fn_check_direct_circle_member_limit")
            .await
    );
}

// ===========================================================================
// Social tables
// ===========================================================================

#[tokio::test]
async fn social_tables_final_state() {
    let mdb = MigrationDb::new().await;

    assert!(mdb.table_exists("friend_requests").await);
    assert!(mdb.index_exists("idx_friend_requests_to_user").await);
    assert!(mdb.index_exists("idx_friend_requests_from_user").await);
    assert!(
        mdb.unique_constraint_exists("friend_requests", &["from_user_id", "to_user_id"])
            .await
    );

    assert!(mdb.table_exists("friendships").await);
    assert!(mdb.index_exists("idx_friendships_a").await);
    assert!(mdb.index_exists("idx_friendships_b").await);
}

// ===========================================================================
// Community tables
// ===========================================================================

#[tokio::test]
async fn community_tables_final_state() {
    let mdb = MigrationDb::new().await;

    assert!(mdb.table_exists("community_wishes").await);
    assert!(mdb.index_exists("idx_cw_status_created").await);
    assert!(mdb.index_exists("idx_cw_owner").await);
    assert!(mdb.index_exists("idx_cw_category_open").await);
    assert!(mdb.index_exists("idx_cw_matched").await);
    assert!(mdb.index_exists("idx_cw_pending").await);
    assert!(mdb.index_exists("idx_cw_fulfilled").await);
    assert!(mdb.trigger_exists("set_community_wishes_updated_at").await);

    // OG columns exist
    mdb.assert_nullable("community_wishes", "og_image_url")
        .await;
    mdb.assert_nullable("community_wishes", "og_title").await;
    mdb.assert_nullable("community_wishes", "og_site_name")
        .await;
    mdb.assert_nullable("community_wishes", "image_url").await;
    mdb.assert_nullable("community_wishes", "links").await;

    assert!(mdb.table_exists("wish_reports").await);
    assert!(mdb.index_exists("idx_wr_wish").await);
    mdb.assert_nullable("wish_reports", "details").await;
    assert!(
        mdb.unique_constraint_exists("wish_reports", &["reporter_id", "wish_id"])
            .await
    );

    assert!(mdb.table_exists("wish_messages").await);
    assert!(mdb.index_exists("idx_wm_wish_created").await);
    mdb.assert_nullable("wish_messages", "sender_id").await;

    assert!(mdb.table_exists("wish_blocks").await);
    assert!(
        mdb.unique_constraint_exists("wish_blocks", &["user_id", "wish_id"])
            .await
    );
}

// ===========================================================================
// Infrastructure tables
// ===========================================================================

#[tokio::test]
async fn infra_tables_final_state() {
    let mdb = MigrationDb::new().await;

    // push_tokens
    assert!(mdb.table_exists("push_tokens").await);
    assert_eq!(mdb.column_count("push_tokens").await, 5);
    assert!(
        mdb.unique_constraint_exists("push_tokens", &["user_id", "token"])
            .await
    );

    // refresh_tokens
    assert!(mdb.table_exists("refresh_tokens").await);
    assert_eq!(mdb.column_count("refresh_tokens").await, 6);
    assert!(
        mdb.unique_constraint_exists("refresh_tokens", &["token_hash"])
            .await
    );
    assert!(mdb.index_exists("idx_refresh_tokens_user_id").await);
    assert!(mdb.index_exists("idx_refresh_tokens_active_expires").await);

    // notifications
    assert!(mdb.table_exists("notifications").await);
    mdb.assert_nullable("notifications", "wish_id").await;
    assert!(mdb.index_exists("idx_notifications_user_unread").await);
    assert!(mdb.index_exists("idx_notifications_user_created").await);

    // share_links
    assert!(mdb.table_exists("share_links").await);
    assert!(mdb.index_exists("idx_share_links_user_id").await);
    assert!(
        mdb.unique_constraint_exists("share_links", &["token"])
            .await
    );
}

// ===========================================================================
// Behavioral: triggers
// ===========================================================================

#[tokio::test]
async fn triggers_behave_correctly() {
    let mdb = MigrationDb::new().await;

    // -- setup: insert a user ------------------------------------------------
    let user_id: (sqlx::types::Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, password_hash, username) VALUES ('trigger@test.com', 'hash123', 'trigger_user')
         RETURNING id",
    )
    .fetch_one(&mdb.db)
    .await
    .unwrap();
    let uid = user_id.0;

    // -- users: updated_at trigger -------------------------------------------
    let before: (chrono::DateTime<chrono::Utc>,) =
        sqlx::query_as("SELECT updated_at FROM users WHERE id = $1")
            .bind(uid)
            .fetch_one(&mdb.db)
            .await
            .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    sqlx::query("UPDATE users SET display_name = 'Updated' WHERE id = $1")
        .bind(uid)
        .execute(&mdb.db)
        .await
        .unwrap();

    let after: (chrono::DateTime<chrono::Utc>,) =
        sqlx::query_as("SELECT updated_at FROM users WHERE id = $1")
            .bind(uid)
            .fetch_one(&mdb.db)
            .await
            .unwrap();

    assert!(after.0 > before.0, "updated_at should change after UPDATE");

    // -- items: INSERT with status='purchased' -> purchased_at auto-set -------
    let cat_id: (sqlx::types::Uuid,) = sqlx::query_as("SELECT id FROM categories LIMIT 1")
        .fetch_one(&mdb.db)
        .await
        .unwrap();

    let item: (sqlx::types::Uuid, Option<chrono::DateTime<chrono::Utc>>) = sqlx::query_as(
        "INSERT INTO items (user_id, name, category_id, status)
         VALUES ($1, 'Purchased Item', $2, 'purchased')
         RETURNING id, purchased_at",
    )
    .bind(uid)
    .bind(cat_id.0)
    .fetch_one(&mdb.db)
    .await
    .unwrap();
    assert!(
        item.1.is_some(),
        "purchased_at should be auto-set on INSERT with status='purchased'"
    );

    // -- items: UPDATE active -> purchased -> purchased_at set -----------------
    let item2: (sqlx::types::Uuid,) = sqlx::query_as(
        "INSERT INTO items (user_id, name, status) VALUES ($1, 'Active Item', 'active') RETURNING id",
    )
    .bind(uid)
    .fetch_one(&mdb.db)
    .await
    .unwrap();

    sqlx::query("UPDATE items SET status = 'purchased' WHERE id = $1")
        .bind(item2.0)
        .execute(&mdb.db)
        .await
        .unwrap();

    let purchased_at: (Option<chrono::DateTime<chrono::Utc>>,) =
        sqlx::query_as("SELECT purchased_at FROM items WHERE id = $1")
            .bind(item2.0)
            .fetch_one(&mdb.db)
            .await
            .unwrap();
    assert!(
        purchased_at.0.is_some(),
        "purchased_at should be set when status changes to 'purchased'"
    );

    // -- items: UPDATE purchased -> active -> purchased_at NULL ----------------
    sqlx::query("UPDATE items SET status = 'active' WHERE id = $1")
        .bind(item2.0)
        .execute(&mdb.db)
        .await
        .unwrap();

    let cleared: (Option<chrono::DateTime<chrono::Utc>>,) =
        sqlx::query_as("SELECT purchased_at FROM items WHERE id = $1")
            .bind(item2.0)
            .fetch_one(&mdb.db)
            .await
            .unwrap();
    assert!(
        cleared.0.is_none(),
        "purchased_at should be NULL when status reverts to 'active'"
    );

    // -- circles: auto-add owner as member -----------------------------------
    let circle: (sqlx::types::Uuid,) = sqlx::query_as(
        "INSERT INTO circles (name, owner_id) VALUES ('Test Circle', $1) RETURNING id",
    )
    .bind(uid)
    .fetch_one(&mdb.db)
    .await
    .unwrap();

    let member: (String,) =
        sqlx::query_as("SELECT role FROM circle_members WHERE circle_id = $1 AND user_id = $2")
            .bind(circle.0)
            .bind(uid)
            .fetch_one(&mdb.db)
            .await
            .unwrap();
    assert_eq!(
        member.0, "owner",
        "owner should be auto-added as member with role='owner'"
    );
}

// ===========================================================================
// Behavioral: CHECK constraints reject invalid data
// ===========================================================================

#[tokio::test]
async fn check_constraints_reject_invalid_data() {
    let mdb = MigrationDb::new().await;

    let user_id: (sqlx::types::Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, password_hash, username) VALUES ('check@test.com', 'hash123', 'check_user')
         RETURNING id",
    )
    .fetch_one(&mdb.db)
    .await
    .unwrap();
    let uid = user_id.0;

    // items: invalid priority (0)
    let result =
        sqlx::query("INSERT INTO items (user_id, name, priority) VALUES ($1, 'Bad Priority', 0)")
            .bind(uid)
            .execute(&mdb.db)
            .await;
    assert!(
        result.is_err(),
        "priority=0 should violate CHECK constraint"
    );

    // items: invalid status
    let result = sqlx::query(
        "INSERT INTO items (user_id, name, status) VALUES ($1, 'Bad Status', 'unknown')",
    )
    .bind(uid)
    .execute(&mdb.db)
    .await;
    assert!(
        result.is_err(),
        "status='unknown' should violate CHECK constraint"
    );

    // push_tokens: invalid platform
    let result = sqlx::query(
        "INSERT INTO push_tokens (user_id, token, platform)
         VALUES ($1, 'some-token', 'windows')",
    )
    .bind(uid)
    .execute(&mdb.db)
    .await;
    assert!(
        result.is_err(),
        "platform='windows' should violate CHECK constraint"
    );

    // circle_members: invalid role
    let circle: (sqlx::types::Uuid,) = sqlx::query_as(
        "INSERT INTO circles (name, owner_id) VALUES ('Check Circle', $1) RETURNING id",
    )
    .bind(uid)
    .fetch_one(&mdb.db)
    .await
    .unwrap();

    let user2: (sqlx::types::Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, password_hash, username) VALUES ('check2@test.com', 'hash123', 'check2_user')
         RETURNING id",
    )
    .fetch_one(&mdb.db)
    .await
    .unwrap();

    let result = sqlx::query(
        "INSERT INTO circle_members (circle_id, user_id, role)
         VALUES ($1, $2, 'admin')",
    )
    .bind(circle.0)
    .bind(user2.0)
    .execute(&mdb.db)
    .await;
    assert!(
        result.is_err(),
        "role='admin' should violate CHECK constraint"
    );

    // items: invalid claimed_via
    let result = sqlx::query(
        "INSERT INTO items (user_id, name, claimed_via) VALUES ($1, 'Bad Claim', 'email')",
    )
    .bind(uid)
    .execute(&mdb.db)
    .await;
    assert!(
        result.is_err(),
        "claimed_via='email' should violate CHECK constraint"
    );

    // notifications: invalid type
    let result = sqlx::query(
        "INSERT INTO notifications (user_id, type, title, body) VALUES ($1, 'bad_type', 'Title', 'Body')",
    )
    .bind(uid)
    .execute(&mdb.db)
    .await;
    assert!(
        result.is_err(),
        "notification type='bad_type' should violate CHECK constraint"
    );
}

// ===========================================================================
// Helper correctness: composite unique constraint detection
// ===========================================================================

#[tokio::test]
async fn unique_constraint_is_composite_not_individual() {
    let mdb = MigrationDb::new().await;

    assert!(
        !mdb.unique_constraint_exists("push_tokens", &["user_id"])
            .await,
        "user_id alone is not uniquely constrained on push_tokens"
    );
    assert!(
        !mdb.unique_constraint_exists("push_tokens", &["token"])
            .await,
        "token alone is not uniquely constrained on push_tokens"
    );
    assert!(
        mdb.unique_constraint_exists("push_tokens", &["user_id", "token"])
            .await
    );
}

// ===========================================================================
// Down-migration: up -> down -> up roundtrip
// ===========================================================================

#[tokio::test]
async fn down_migrations_then_re_up_succeeds() {
    let pg_container = Postgres::default()
        .with_tag("16-alpine")
        .start()
        .await
        .unwrap();

    let host = pg_container.get_host().await.unwrap();
    let port = pg_container.get_host_port_ipv4(5432).await.unwrap();
    let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");
    let db = PgPool::connect(&url).await.unwrap();

    // Step 1: Apply all migrations UP
    MIGRATOR.run(&db).await.unwrap();

    // Step 2: Undo all migrations (one by one, from newest to oldest)
    for _ in MIGRATOR.migrations.iter().rev() {
        MIGRATOR.undo(&db, 1).await.unwrap();
    }

    // Step 3: Verify all tables are gone
    let table_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.tables \
         WHERE table_schema = 'public' AND table_type = 'BASE TABLE' \
         AND table_name != '_sqlx_migrations'",
    )
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(
        table_count.0, 0,
        "all tables should be dropped after full undo, found {}",
        table_count.0
    );

    // Step 4: Re-apply all migrations UP
    MIGRATOR.run(&db).await.unwrap();

    // Step 5: Verify key tables exist again
    let mdb = MigrationDb {
        _pg_container: pg_container,
        db,
    };
    assert!(mdb.table_exists("users").await);
    assert!(mdb.table_exists("categories").await);
    assert!(mdb.table_exists("items").await);
    assert!(mdb.table_exists("refresh_tokens").await);
    assert!(mdb.table_exists("circles").await);
    assert!(mdb.table_exists("circle_members").await);
    assert!(mdb.table_exists("push_tokens").await);
    assert!(mdb.table_exists("community_wishes").await);
    assert!(mdb.table_exists("notifications").await);
    assert!(mdb.table_exists("share_links").await);

    // Verify constraints survive roundtrip
    assert!(mdb.unique_constraint_exists("users", &["email"]).await);
    assert!(
        mdb.unique_constraint_exists("refresh_tokens", &["token_hash"])
            .await
    );
}
