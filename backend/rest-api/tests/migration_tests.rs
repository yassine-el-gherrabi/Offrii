use sqlx::PgPool;
use sqlx::migrate::Migrator;
use testcontainers::ContainerAsync;
use testcontainers::ImageExt;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

// ---------------------------------------------------------------------------
// MigrationDb – lightweight helper (Postgres only, no Redis/Router)
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

        // Fetch column sets for each unique index on the table, grouped by index OID.
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

    /// Check a column is NOT NULL via is_nullable = 'NO'
    async fn assert_not_null(&self, table: &str, column: &str) {
        let info = self.column_info(table, column).await;
        let (_, nullable, _) = info.unwrap_or_else(|| panic!("{table}.{column} should exist"));
        assert_eq!(
            nullable, "NO",
            "{table}.{column} should be NOT NULL, got nullable={nullable}"
        );
    }

    /// Check a column is nullable via is_nullable = 'YES'
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
// Migration 1 : create_users
// ===========================================================================

#[tokio::test]
async fn migration_001_creates_users_table() {
    let mdb = MigrationDb::new().await;

    assert!(mdb.table_exists("users").await);
    assert_eq!(mdb.column_count("users").await, 12);

    // Column types & nullability
    mdb.assert_not_null("users", "id").await;
    mdb.assert_not_null("users", "email").await;
    mdb.assert_not_null("users", "password_hash").await;
    mdb.assert_nullable("users", "display_name").await;
    mdb.assert_not_null("users", "reminder_freq").await;
    mdb.assert_not_null("users", "reminder_time").await;
    mdb.assert_not_null("users", "timezone").await;
    mdb.assert_not_null("users", "utc_reminder_hour").await;
    mdb.assert_not_null("users", "locale").await;
    mdb.assert_not_null("users", "token_version").await;
    mdb.assert_not_null("users", "created_at").await;
    mdb.assert_not_null("users", "updated_at").await;

    // data types
    let (dt, _, _) = mdb.column_info("users", "id").await.unwrap();
    assert_eq!(dt, "uuid");
    let (dt, _, _) = mdb.column_info("users", "email").await.unwrap();
    assert_eq!(dt, "character varying");
    let (dt, _, _) = mdb.column_info("users", "reminder_freq").await.unwrap();
    assert_eq!(dt, "character varying");
    let (dt, _, _) = mdb.column_info("users", "reminder_time").await.unwrap();
    assert!(dt.starts_with("time"), "expected time type, got {dt}");
    let (dt, _, _) = mdb.column_info("users", "created_at").await.unwrap();
    assert!(dt.contains("timestamp"), "expected timestamptz, got {dt}");

    // defaults
    let (_, _, default) = mdb.column_info("users", "reminder_freq").await.unwrap();
    assert!(
        default.as_deref().unwrap_or("").contains("weekly"),
        "expected default 'weekly', got {default:?}"
    );

    // UNIQUE on email
    assert!(mdb.unique_constraint_exists("users", &["email"]).await);

    // Trigger
    assert!(mdb.trigger_exists("trg_users_updated_at").await);
    assert!(mdb.function_exists("set_updated_at").await);
}

// ===========================================================================
// Migration 2 : create_categories
// ===========================================================================

#[tokio::test]
async fn migration_002_creates_categories_table() {
    let mdb = MigrationDb::new().await;

    assert!(mdb.table_exists("categories").await);
    assert_eq!(mdb.column_count("categories").await, 7);

    mdb.assert_not_null("categories", "id").await;
    mdb.assert_nullable("categories", "user_id").await;
    mdb.assert_not_null("categories", "name").await;
    mdb.assert_nullable("categories", "icon").await;
    mdb.assert_not_null("categories", "is_default").await;
    mdb.assert_not_null("categories", "position").await;
    mdb.assert_not_null("categories", "created_at").await;

    // defaults
    let (_, _, default) = mdb.column_info("categories", "is_default").await.unwrap();
    assert!(
        default.as_deref().unwrap_or("").contains("false"),
        "expected default false, got {default:?}"
    );
    let (_, _, default) = mdb.column_info("categories", "position").await.unwrap();
    assert!(
        default.as_deref().unwrap_or("").contains('0'),
        "expected default 0, got {default:?}"
    );

    // FK user_id → users.id
    assert!(mdb.fk_exists("categories", "user_id").await);

    // Partial unique indexes
    assert!(mdb.index_exists("uq_categories_user_name").await);
    assert!(mdb.index_exists("uq_categories_default_name").await);
}

// ===========================================================================
// Migration 3 : create_items
// ===========================================================================

#[tokio::test]
async fn migration_003_creates_items_table() {
    let mdb = MigrationDb::new().await;

    assert!(mdb.table_exists("items").await);
    assert_eq!(mdb.column_count("items").await, 12);

    mdb.assert_not_null("items", "id").await;
    mdb.assert_not_null("items", "user_id").await;
    mdb.assert_not_null("items", "name").await;
    mdb.assert_nullable("items", "description").await;
    mdb.assert_nullable("items", "url").await;
    mdb.assert_nullable("items", "estimated_price").await;
    mdb.assert_not_null("items", "priority").await;
    mdb.assert_nullable("items", "category_id").await;
    mdb.assert_not_null("items", "status").await;
    mdb.assert_nullable("items", "purchased_at").await;
    mdb.assert_not_null("items", "created_at").await;
    mdb.assert_not_null("items", "updated_at").await;

    // defaults
    let (_, _, default) = mdb.column_info("items", "priority").await.unwrap();
    assert!(
        default.as_deref().unwrap_or("").contains('2'),
        "expected default 2, got {default:?}"
    );
    let (_, _, default) = mdb.column_info("items", "status").await.unwrap();
    assert!(
        default.as_deref().unwrap_or("").contains("active"),
        "expected default 'active', got {default:?}"
    );

    // FKs
    assert!(mdb.fk_exists("items", "user_id").await);
    assert!(mdb.fk_exists("items", "category_id").await);

    // Indexes
    assert!(mdb.index_exists("idx_items_user_status").await);
    assert!(mdb.index_exists("idx_items_user_priority").await);
    assert!(mdb.index_exists("idx_items_created_at").await);

    // Trigger
    assert!(mdb.trigger_exists("trg_items_updated_at").await);
}

// ===========================================================================
// Migration 4 : create_push_tokens
// ===========================================================================

#[tokio::test]
async fn migration_004_creates_push_tokens_table() {
    let mdb = MigrationDb::new().await;

    assert!(mdb.table_exists("push_tokens").await);
    assert_eq!(mdb.column_count("push_tokens").await, 5);

    mdb.assert_not_null("push_tokens", "id").await;
    mdb.assert_not_null("push_tokens", "user_id").await;
    mdb.assert_not_null("push_tokens", "token").await;
    mdb.assert_not_null("push_tokens", "platform").await;
    mdb.assert_not_null("push_tokens", "created_at").await;

    // UNIQUE(user_id, token)
    assert!(
        mdb.unique_constraint_exists("push_tokens", &["user_id", "token"])
            .await
    );

    // FK
    assert!(mdb.fk_exists("push_tokens", "user_id").await);
}

// ===========================================================================
// Migration 5 : create_circles + circle_members
// ===========================================================================

#[tokio::test]
async fn migration_005_creates_circles_and_members() {
    let mdb = MigrationDb::new().await;

    // circles
    assert!(mdb.table_exists("circles").await);
    assert_eq!(mdb.column_count("circles").await, 4);
    mdb.assert_not_null("circles", "id").await;
    mdb.assert_not_null("circles", "name").await;
    mdb.assert_not_null("circles", "owner_id").await;
    mdb.assert_not_null("circles", "created_at").await;
    assert!(mdb.fk_exists("circles", "owner_id").await);

    // circle_members
    assert!(mdb.table_exists("circle_members").await);
    assert_eq!(mdb.column_count("circle_members").await, 4);
    mdb.assert_not_null("circle_members", "circle_id").await;
    mdb.assert_not_null("circle_members", "user_id").await;
    mdb.assert_not_null("circle_members", "role").await;
    mdb.assert_not_null("circle_members", "joined_at").await;

    // Composite PK acts as unique constraint
    assert!(
        mdb.unique_constraint_exists("circle_members", &["circle_id", "user_id"])
            .await
    );

    // FKs
    assert!(mdb.fk_exists("circle_members", "circle_id").await);
    assert!(mdb.fk_exists("circle_members", "user_id").await);
}

// ===========================================================================
// Migration 6 : seed_default_categories
// ===========================================================================

#[tokio::test]
async fn migration_006_seeds_default_categories() {
    let mdb = MigrationDb::new().await;

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM categories WHERE user_id IS NULL AND is_default = true",
    )
    .fetch_one(&mdb.db)
    .await
    .unwrap();
    assert_eq!(count.0, 6, "expected 6 default categories");

    let names: Vec<(String, String)> = sqlx::query_as(
        "SELECT name, icon FROM categories
         WHERE user_id IS NULL AND is_default = true
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
        ("Santé", "heart"),
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
// Migration 7 : add_items_improvements
// ===========================================================================

#[tokio::test]
async fn migration_007_adds_items_improvements() {
    let mdb = MigrationDb::new().await;

    assert!(mdb.index_exists("idx_items_category_id").await);
    assert!(mdb.trigger_exists("trg_items_set_purchased_at").await);
    assert!(mdb.function_exists("set_purchased_at").await);
}

// ===========================================================================
// Migration 8 : create_refresh_tokens
// ===========================================================================

#[tokio::test]
async fn migration_008_creates_refresh_tokens_table() {
    let mdb = MigrationDb::new().await;

    assert!(mdb.table_exists("refresh_tokens").await);
    assert_eq!(mdb.column_count("refresh_tokens").await, 6);

    mdb.assert_not_null("refresh_tokens", "id").await;
    mdb.assert_not_null("refresh_tokens", "user_id").await;
    mdb.assert_not_null("refresh_tokens", "token_hash").await;
    mdb.assert_not_null("refresh_tokens", "expires_at").await;
    mdb.assert_nullable("refresh_tokens", "revoked_at").await;
    mdb.assert_not_null("refresh_tokens", "created_at").await;

    // UNIQUE on token_hash
    assert!(
        mdb.unique_constraint_exists("refresh_tokens", &["token_hash"])
            .await
    );

    // Indexes
    assert!(mdb.index_exists("idx_refresh_tokens_user_id").await);
    assert!(mdb.index_exists("idx_refresh_tokens_active_expires").await);

    // FK
    assert!(mdb.fk_exists("refresh_tokens", "user_id").await);
}

// ===========================================================================
// Migration 9 : add_circle_owner_member trigger
// ===========================================================================

#[tokio::test]
async fn migration_009_adds_circle_owner_auto_member() {
    let mdb = MigrationDb::new().await;

    assert!(mdb.trigger_exists("trg_circles_add_owner_member").await);
    assert!(mdb.function_exists("add_circle_owner_as_member").await);
}

// ===========================================================================
// Behavioral : triggers
// ===========================================================================

#[tokio::test]
async fn triggers_behave_correctly() {
    let mdb = MigrationDb::new().await;

    // -- setup: insert a user ------------------------------------------------
    let user_id: (sqlx::types::Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, password_hash) VALUES ('trigger@test.com', 'hash123')
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

    // Small delay so timestamps differ
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

    // -- items: INSERT with status='purchased' → purchased_at auto-set -------
    let cat_id: (sqlx::types::Uuid,) = sqlx::query_as(
        "INSERT INTO categories (user_id, name) VALUES ($1, 'Test Cat') RETURNING id",
    )
    .bind(uid)
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

    // -- items: UPDATE active → purchased → purchased_at set -----------------
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

    // -- items: UPDATE purchased → active → purchased_at NULL ----------------
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
// Behavioral : CHECK constraints reject invalid data
// ===========================================================================

#[tokio::test]
async fn check_constraints_reject_invalid_data() {
    let mdb = MigrationDb::new().await;

    // Setup user
    let user_id: (sqlx::types::Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, password_hash) VALUES ('check@test.com', 'hash123')
         RETURNING id",
    )
    .fetch_one(&mdb.db)
    .await
    .unwrap();
    let uid = user_id.0;

    // -- users: invalid reminder_freq ----------------------------------------
    let result = sqlx::query(
        "INSERT INTO users (email, password_hash, reminder_freq)
         VALUES ('bad_freq@test.com', 'hash', 'hourly')",
    )
    .execute(&mdb.db)
    .await;
    assert!(
        result.is_err(),
        "reminder_freq='hourly' should violate CHECK constraint"
    );

    // -- items: invalid priority (0) -----------------------------------------
    let result =
        sqlx::query("INSERT INTO items (user_id, name, priority) VALUES ($1, 'Bad Priority', 0)")
            .bind(uid)
            .execute(&mdb.db)
            .await;
    assert!(
        result.is_err(),
        "priority=0 should violate CHECK constraint"
    );

    // -- items: invalid status -----------------------------------------------
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

    // -- push_tokens: invalid platform ---------------------------------------
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

    // -- circle_members: invalid role ----------------------------------------
    let circle: (sqlx::types::Uuid,) = sqlx::query_as(
        "INSERT INTO circles (name, owner_id) VALUES ('Check Circle', $1) RETURNING id",
    )
    .bind(uid)
    .fetch_one(&mdb.db)
    .await
    .unwrap();

    // The owner trigger already inserted (circle_id, uid) with role='owner',
    // so use a second user to test the CHECK constraint.
    let user2: (sqlx::types::Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, password_hash) VALUES ('check2@test.com', 'hash123')
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
}

// ===========================================================================
// Helper correctness: composite unique constraint detection
// ===========================================================================

#[tokio::test]
async fn unique_constraint_is_composite_not_individual() {
    let mdb = MigrationDb::new().await;

    // push_tokens has UNIQUE(user_id, token) — a composite constraint.
    // Asking for just one column should return false: there is no single-column
    // unique index on user_id alone.
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

    // But the composite should still pass
    assert!(
        mdb.unique_constraint_exists("push_tokens", &["user_id", "token"])
            .await
    );
}

// ===========================================================================
// Down-migration: up → down → up roundtrip
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

    // Verify constraints survive roundtrip
    assert!(mdb.unique_constraint_exists("users", &["email"]).await);
    assert!(
        mdb.unique_constraint_exists("refresh_tokens", &["token_hash"])
            .await
    );
}
