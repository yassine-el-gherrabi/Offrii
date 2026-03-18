use std::sync::Arc;

use async_trait::async_trait;

use crate::dto::categories::CategoryResponse;
use crate::errors::AppError;
use crate::traits;

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgCategoryService {
    category_repo: Arc<dyn traits::CategoryRepo>,
}

impl PgCategoryService {
    pub fn new(category_repo: Arc<dyn traits::CategoryRepo>) -> Self {
        Self { category_repo }
    }
}

#[async_trait]
impl traits::CategoryService for PgCategoryService {
    #[tracing::instrument(skip(self))]
    async fn list_categories(&self) -> Result<Vec<CategoryResponse>, AppError> {
        let cats = self
            .category_repo
            .list_all()
            .await
            .map_err(AppError::Internal)?;

        Ok(cats.into_iter().map(CategoryResponse::from).collect())
    }
}
