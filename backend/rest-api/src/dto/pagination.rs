use serde::{Deserialize, Serialize};

pub const DEFAULT_PAGE: i64 = 1;
pub const DEFAULT_LIMIT: i64 = 20;
pub const MAX_LIMIT: i64 = 100;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PaginationMeta {
    pub total: i64,
    pub page: i64,
    pub limit: i64,
    pub total_pages: i64,
    pub has_more: bool,
}

impl PaginationMeta {
    pub fn new(total: i64, page: i64, limit: i64) -> Self {
        let total_pages = if limit > 0 {
            (total + limit - 1) / limit
        } else {
            0
        };
        Self {
            total,
            page,
            limit,
            total_pages,
            has_more: page < total_pages,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, page: i64, limit: i64) -> Self {
        Self {
            data,
            pagination: PaginationMeta::new(total, page, limit),
        }
    }
}

/// Normalise and clamp pagination params.
/// Returns (page, limit, offset) ready for SQL queries.
pub fn normalize_pagination(page: Option<i64>, limit: Option<i64>) -> (i64, i64, i64) {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let page = page.unwrap_or(DEFAULT_PAGE).max(1);
    let offset = (page - 1) * limit;
    (page, limit, offset)
}
