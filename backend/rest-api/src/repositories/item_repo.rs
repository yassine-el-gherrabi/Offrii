use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{PgExecutor, PgPool, QueryBuilder, Row};
use uuid::Uuid;

use crate::models::Item;
use crate::traits;

/// Shared column list for all item queries (avoids duplication).
const ITEM_COLS: &str = "id, user_id, name, description, url, estimated_price, \
                         priority, category_id, status, purchased_at, created_at, updated_at, \
                         claimed_by, claimed_at, image_url, links, og_image_url, og_title, og_site_name, \
                         is_private, claimed_via, claimed_name, claimed_via_link_id, web_claim_token";

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgItemRepo {
    pool: PgPool,
}

impl PgItemRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl traits::ItemRepo for PgItemRepo {
    async fn create(
        &self,
        user_id: Uuid,
        name: &str,
        description: Option<&str>,
        url: Option<&str>,
        estimated_price: Option<Decimal>,
        priority: i16,
        category_id: Option<Uuid>,
        image_url: Option<&str>,
        links: Option<&[String]>,
        is_private: bool,
    ) -> Result<Item> {
        create(
            &self.pool,
            user_id,
            name,
            description,
            url,
            estimated_price,
            priority,
            category_id,
            image_url,
            links,
            is_private,
        )
        .await
    }

    async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> Result<Option<Item>> {
        find_by_id(&self.pool, id, user_id).await
    }

    async fn list(
        &self,
        user_id: Uuid,
        status: Option<&str>,
        category_ids: Option<&[Uuid]>,
        sort: &str,
        order: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Item>> {
        list(
            &self.pool,
            user_id,
            status,
            category_ids,
            sort,
            order,
            limit,
            offset,
        )
        .await
    }

    async fn count(
        &self,
        user_id: Uuid,
        status: Option<&str>,
        category_ids: Option<&[Uuid]>,
    ) -> Result<i64> {
        count(&self.pool, user_id, status, category_ids).await
    }

    async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        url: Option<&str>,
        estimated_price: Option<Decimal>,
        priority: Option<i16>,
        category_id: Option<Option<Uuid>>,
        status: Option<&str>,
        image_url: Option<&str>,
        links: Option<&[String]>,
        is_private: Option<bool>,
    ) -> Result<Option<Item>> {
        update(
            &self.pool,
            id,
            user_id,
            name,
            description,
            url,
            estimated_price,
            priority,
            category_id,
            status,
            image_url,
            links,
            is_private,
        )
        .await
    }

    async fn update_og_metadata(
        &self,
        id: Uuid,
        og_image_url: Option<&str>,
        og_title: Option<&str>,
        og_site_name: Option<&str>,
    ) -> Result<bool> {
        update_og_metadata(&self.pool, id, og_image_url, og_title, og_site_name).await
    }

    async fn soft_delete(&self, id: Uuid, user_id: Uuid) -> Result<bool> {
        soft_delete(&self.pool, id, user_id).await
    }

    async fn find_active_older_than(
        &self,
        user_id: Uuid,
        cutoff: DateTime<Utc>,
    ) -> Result<Vec<Item>> {
        find_active_older_than(&self.pool, user_id, cutoff).await
    }

    async fn find_by_id_any_user(&self, id: Uuid) -> Result<Option<Item>> {
        find_by_id_any_user(&self.pool, id).await
    }

    async fn claim_item(&self, id: Uuid, claimer_id: Uuid) -> Result<Option<Uuid>> {
        claim_item(&self.pool, id, claimer_id).await
    }

    async fn unclaim_item(&self, id: Uuid, claimer_id: Uuid) -> Result<Option<Uuid>> {
        unclaim_item(&self.pool, id, claimer_id).await
    }

    async fn find_by_ids(&self, user_id: Uuid, ids: &[Uuid]) -> Result<Vec<Item>> {
        find_by_ids(&self.pool, user_id, ids).await
    }

    async fn find_by_ids_any_user(&self, ids: &[Uuid]) -> Result<Vec<Item>> {
        find_by_ids_any_user(&self.pool, ids).await
    }

    async fn web_claim_item(
        &self,
        id: Uuid,
        name: &str,
        link_id: Uuid,
    ) -> Result<Option<(Uuid, Uuid)>> {
        web_claim_item(&self.pool, id, name, link_id).await
    }

    async fn web_unclaim_item(&self, id: Uuid, token: Uuid) -> Result<Option<Uuid>> {
        web_unclaim_item(&self.pool, id, token).await
    }

    async fn owner_unclaim_web_item(&self, id: Uuid, owner_id: Uuid) -> Result<Option<Uuid>> {
        owner_unclaim_web_item(&self.pool, id, owner_id).await
    }
}

// ── Free functions (kept pub(crate) for transactional use) ───────────

#[allow(clippy::too_many_arguments)]
pub(crate) async fn create(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
    name: &str,
    description: Option<&str>,
    url: Option<&str>,
    estimated_price: Option<Decimal>,
    priority: i16,
    category_id: Option<Uuid>,
    image_url: Option<&str>,
    links: Option<&[String]>,
    is_private: bool,
) -> Result<Item> {
    let sql = format!(
        "INSERT INTO items (user_id, name, description, url, estimated_price, priority, category_id, image_url, links, is_private) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
         RETURNING {ITEM_COLS}"
    );
    let item = sqlx::query_as::<_, Item>(&sql)
        .bind(user_id)
        .bind(name)
        .bind(description)
        .bind(url)
        .bind(estimated_price)
        .bind(priority)
        .bind(category_id)
        .bind(image_url)
        .bind(links)
        .bind(is_private)
        .fetch_one(exec)
        .await?;

    Ok(item)
}

pub(crate) async fn find_by_id(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    user_id: Uuid,
) -> Result<Option<Item>> {
    let sql = format!(
        "SELECT {ITEM_COLS} FROM items WHERE id = $1 AND user_id = $2 AND status != 'deleted'"
    );
    let item = sqlx::query_as::<_, Item>(&sql)
        .bind(id)
        .bind(user_id)
        .fetch_optional(exec)
        .await?;

    Ok(item)
}

/// Build the shared WHERE clause for list/count queries.
fn push_list_where<'args>(
    qb: &mut QueryBuilder<'args, sqlx::Postgres>,
    user_id: Uuid,
    status: Option<&'args str>,
    category_ids: Option<&'args [Uuid]>,
) {
    qb.push(" WHERE user_id = ");
    qb.push_bind(user_id);
    qb.push(" AND status != 'deleted'");

    if let Some(s) = status {
        qb.push(" AND status = ");
        qb.push_bind(s);
    }

    if let Some(ids) = category_ids
        && !ids.is_empty()
    {
        qb.push(" AND category_id = ANY(");
        qb.push_bind(ids);
        qb.push(")");
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn list(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
    status: Option<&str>,
    category_ids: Option<&[Uuid]>,
    sort: &str,
    order: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<Item>> {
    let select = format!("SELECT {ITEM_COLS} FROM items");
    let mut qb: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(select);

    push_list_where(&mut qb, user_id, status, category_ids);

    // Defense-in-depth: validate sort/order even though service layer whitelists them.
    const REPO_ALLOWED_SORTS: &[&str] = &["created_at", "priority", "name"];
    const REPO_ALLOWED_ORDERS: &[&str] = &["asc", "desc"];
    if !REPO_ALLOWED_SORTS.contains(&sort) {
        anyhow::bail!("invalid sort column: {sort}");
    }
    if !REPO_ALLOWED_ORDERS.contains(&order) {
        anyhow::bail!("invalid order direction: {order}");
    }

    qb.push(" ORDER BY ");
    qb.push(sort);
    qb.push(" ");
    qb.push(order);

    qb.push(" LIMIT ");
    qb.push_bind(limit);
    qb.push(" OFFSET ");
    qb.push_bind(offset);

    let items = qb.build_query_as::<Item>().fetch_all(exec).await?;

    Ok(items)
}

pub(crate) async fn count(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
    status: Option<&str>,
    category_ids: Option<&[Uuid]>,
) -> Result<i64> {
    let mut qb: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new("SELECT COUNT(*) FROM items");

    push_list_where(&mut qb, user_id, status, category_ids);

    let row = qb.build().fetch_one(exec).await?;
    let total: i64 = row.get(0);

    Ok(total)
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn update(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    user_id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    url: Option<&str>,
    estimated_price: Option<Decimal>,
    priority: Option<i16>,
    category_id: Option<Option<Uuid>>,
    status: Option<&str>,
    image_url: Option<&str>,
    links: Option<&[String]>,
    is_private: Option<bool>,
) -> Result<Option<Item>> {
    let mut qb: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new("UPDATE items SET ");
    let mut separated = qb.separated(", ");

    // Only push fields that are provided
    if let Some(n) = name {
        separated.push("name = ");
        separated.push_bind_unseparated(n);
    }
    if let Some(d) = description {
        separated.push("description = ");
        separated.push_bind_unseparated(d);
    }
    if let Some(u) = url {
        separated.push("url = ");
        separated.push_bind_unseparated(u);
    }
    if let Some(p) = estimated_price {
        separated.push("estimated_price = ");
        separated.push_bind_unseparated(p);
    }
    if let Some(p) = priority {
        separated.push("priority = ");
        separated.push_bind_unseparated(p);
    }
    if let Some(cid) = category_id {
        separated.push("category_id = ");
        separated.push_bind_unseparated(cid);
    }
    if let Some(s) = status {
        separated.push("status = ");
        separated.push_bind_unseparated(s);
    }
    if let Some(img) = image_url {
        separated.push("image_url = ");
        separated.push_bind_unseparated(img);
    }
    if let Some(l) = links {
        separated.push("links = ");
        separated.push_bind_unseparated(l);
    }
    if let Some(p) = is_private {
        separated.push("is_private = ");
        separated.push_bind_unseparated(p);
    }

    qb.push(" WHERE id = ");
    qb.push_bind(id);
    qb.push(" AND user_id = ");
    qb.push_bind(user_id);
    qb.push(" AND status != 'deleted'");

    // Atomic guard: reject update if item already has the target status.
    if let Some(s) = status {
        qb.push(" AND status != ");
        qb.push_bind(s);
    }

    qb.push(format!(" RETURNING {ITEM_COLS}"));

    let item = qb.build_query_as::<Item>().fetch_optional(exec).await?;

    Ok(item)
}

pub(crate) async fn update_og_metadata(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    og_image_url: Option<&str>,
    og_title: Option<&str>,
    og_site_name: Option<&str>,
) -> Result<bool> {
    let result = sqlx::query(
        "UPDATE items SET og_image_url = $2, og_title = $3, og_site_name = $4 \
         WHERE id = $1 AND status != 'deleted'",
    )
    .bind(id)
    .bind(og_image_url)
    .bind(og_title)
    .bind(og_site_name)
    .execute(exec)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub(crate) async fn soft_delete(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    user_id: Uuid,
) -> Result<bool> {
    let result = sqlx::query(
        "UPDATE items SET status = 'deleted' WHERE id = $1 AND user_id = $2 AND status != 'deleted'",
    )
    .bind(id)
    .bind(user_id)
    .execute(exec)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub(crate) async fn find_active_older_than(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
    cutoff: DateTime<Utc>,
) -> Result<Vec<Item>> {
    let sql = format!(
        "SELECT {ITEM_COLS} FROM items \
         WHERE user_id = $1 AND status = 'active' AND created_at < $2"
    );
    let items = sqlx::query_as::<_, Item>(&sql)
        .bind(user_id)
        .bind(cutoff)
        .fetch_all(exec)
        .await?;

    Ok(items)
}

pub(crate) async fn find_by_id_any_user(
    exec: impl PgExecutor<'_>,
    id: Uuid,
) -> Result<Option<Item>> {
    let sql = format!("SELECT {ITEM_COLS} FROM items WHERE id = $1 AND status != 'deleted'");
    let item = sqlx::query_as::<_, Item>(&sql)
        .bind(id)
        .fetch_optional(exec)
        .await?;

    Ok(item)
}

pub(crate) async fn claim_item(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    claimer_id: Uuid,
) -> Result<Option<Uuid>> {
    let row = sqlx::query(
        "UPDATE items \
         SET claimed_by = $2, claimed_at = NOW(), claimed_via = 'app', updated_at = NOW() \
         WHERE id = $1 \
           AND user_id != $2 \
           AND claimed_by IS NULL \
           AND claimed_via IS NULL \
           AND status = 'active' \
         RETURNING user_id",
    )
    .bind(id)
    .bind(claimer_id)
    .fetch_optional(exec)
    .await?;

    Ok(row.map(|r| r.get("user_id")))
}

pub(crate) async fn unclaim_item(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    claimer_id: Uuid,
) -> Result<Option<Uuid>> {
    let row = sqlx::query(
        "UPDATE items \
         SET claimed_by = NULL, claimed_at = NULL, claimed_via = NULL, \
             claimed_name = NULL, claimed_via_link_id = NULL, web_claim_token = NULL, \
             updated_at = NOW() \
         WHERE id = $1 \
           AND claimed_by = $2 \
           AND status != 'deleted' \
         RETURNING user_id",
    )
    .bind(id)
    .bind(claimer_id)
    .fetch_optional(exec)
    .await?;

    Ok(row.map(|r| r.get("user_id")))
}

pub(crate) async fn find_by_ids(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
    ids: &[Uuid],
) -> Result<Vec<Item>> {
    let sql = format!(
        "SELECT {ITEM_COLS} FROM items \
         WHERE user_id = $1 AND id = ANY($2) AND status = 'active'"
    );
    let items = sqlx::query_as::<_, Item>(&sql)
        .bind(user_id)
        .bind(ids)
        .fetch_all(exec)
        .await?;

    Ok(items)
}

pub(crate) async fn find_by_ids_any_user(
    exec: impl PgExecutor<'_>,
    ids: &[Uuid],
) -> Result<Vec<Item>> {
    let sql = format!(
        "SELECT {ITEM_COLS} FROM items \
         WHERE id = ANY($1) AND status != 'deleted'"
    );
    let items = sqlx::query_as::<_, Item>(&sql)
        .bind(ids)
        .fetch_all(exec)
        .await?;

    Ok(items)
}

/// Web claim: anonymous user claims an item via a share link.
/// Returns (user_id, web_claim_token) on success.
pub(crate) async fn web_claim_item(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    name: &str,
    link_id: Uuid,
) -> Result<Option<(Uuid, Uuid)>> {
    let token = Uuid::new_v4();
    let row = sqlx::query(
        "UPDATE items \
         SET claimed_via = 'web', claimed_name = $2, claimed_via_link_id = $3, \
             claimed_at = NOW(), web_claim_token = $4, updated_at = NOW() \
         WHERE id = $1 \
           AND claimed_by IS NULL \
           AND claimed_via IS NULL \
           AND status = 'active' \
         RETURNING user_id",
    )
    .bind(id)
    .bind(name)
    .bind(link_id)
    .bind(token)
    .fetch_optional(exec)
    .await?;

    Ok(row.map(|r| (r.get::<Uuid, _>("user_id"), token)))
}

/// Web unclaim: anonymous user cancels their claim using the web_claim_token.
pub(crate) async fn web_unclaim_item(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    token: Uuid,
) -> Result<Option<Uuid>> {
    let row = sqlx::query(
        "UPDATE items \
         SET claimed_via = NULL, claimed_name = NULL, claimed_via_link_id = NULL, \
             claimed_at = NULL, web_claim_token = NULL, updated_at = NOW() \
         WHERE id = $1 \
           AND web_claim_token = $2 \
           AND claimed_via = 'web' \
           AND status != 'deleted' \
         RETURNING user_id",
    )
    .bind(id)
    .bind(token)
    .fetch_optional(exec)
    .await?;

    Ok(row.map(|r| r.get("user_id")))
}

/// Owner unclaim for web claims: item owner can remove a web claim.
pub(crate) async fn owner_unclaim_web_item(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    owner_id: Uuid,
) -> Result<Option<Uuid>> {
    let row = sqlx::query(
        "UPDATE items \
         SET claimed_via = NULL, claimed_name = NULL, claimed_via_link_id = NULL, \
             claimed_at = NULL, web_claim_token = NULL, updated_at = NOW() \
         WHERE id = $1 \
           AND user_id = $2 \
           AND claimed_via = 'web' \
           AND status != 'deleted' \
         RETURNING user_id",
    )
    .bind(id)
    .bind(owner_id)
    .fetch_optional(exec)
    .await?;

    Ok(row.map(|r| r.get("user_id")))
}
