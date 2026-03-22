use std::sync::Arc;

use tokio_cron_scheduler::{Job, JobScheduler};

use crate::traits::EmailService;

/// Register all cron jobs and start the scheduler.
pub async fn start_scheduler(
    db: sqlx::PgPool,
    email_service: Arc<dyn EmailService>,
) -> anyhow::Result<()> {
    let sched = JobScheduler::new().await?;

    // Cleanup expired refresh tokens (daily at 3:00 AM UTC)
    let cleanup_pool = db.clone();
    sched
        .add(Job::new_async("0 0 3 * * *", move |_, _| {
            let pool = cleanup_pool.clone();
            Box::pin(async move {
                let result: Result<sqlx::postgres::PgQueryResult, sqlx::Error> =
                    sqlx::query("DELETE FROM refresh_tokens WHERE expires_at < NOW()")
                        .execute(&pool)
                        .await;
                match result {
                    Ok(r) => tracing::info!(
                        deleted = r.rows_affected(),
                        "cleaned up expired refresh tokens"
                    ),
                    Err(e) => {
                        tracing::error!(error = %e, "failed to cleanup expired refresh tokens")
                    }
                }
            })
        })?)
        .await?;

    // Cleanup expired email verification tokens (daily at 3:05 AM)
    let cleanup_pool2 = db.clone();
    sched
        .add(Job::new_async("0 5 3 * * *", move |_, _| {
            let pool = cleanup_pool2.clone();
            Box::pin(async move {
                let r1 =
                    sqlx::query("DELETE FROM email_verification_tokens WHERE expires_at < NOW()")
                        .execute(&pool)
                        .await;
                let r2 = sqlx::query("DELETE FROM email_change_tokens WHERE expires_at < NOW()")
                    .execute(&pool)
                    .await;
                match (r1, r2) {
                    (Ok(a), Ok(b)) => tracing::info!(
                        verification = a.rows_affected(),
                        email_change = b.rows_affected(),
                        "expired token cleanup complete"
                    ),
                    _ => tracing::warn!("expired token cleanup encountered errors"),
                }
            })
        })?)
        .await?;

    // Cleanup old notifications > 6 months (daily at 3:10 AM)
    let cleanup_pool3 = db.clone();
    sched
        .add(Job::new_async("0 10 3 * * *", move |_, _| {
            let pool = cleanup_pool3.clone();
            Box::pin(async move {
                match sqlx::query(
                    "DELETE FROM notifications WHERE created_at < NOW() - INTERVAL '6 months'",
                )
                .execute(&pool)
                .await
                {
                    Ok(r) => tracing::info!(
                        rows = r.rows_affected(),
                        "old notifications cleanup complete"
                    ),
                    Err(e) => {
                        tracing::warn!(error = %e, "old notifications cleanup failed")
                    }
                }
            })
        })?)
        .await?;

    // Purge connection_logs > 12 months (monthly, 1st at 4:00 AM)
    let cleanup_pool_logs = db.clone();
    sched
        .add(Job::new_async("0 0 4 1 * *", move |_, _| {
            let pool = cleanup_pool_logs.clone();
            Box::pin(async move {
                match sqlx::query(
                    "DELETE FROM connection_logs WHERE created_at < NOW() - INTERVAL '12 months'",
                )
                .execute(&pool)
                .await
                {
                    Ok(r) => {
                        tracing::info!(rows = r.rows_affected(), "connection logs purge complete")
                    }
                    Err(e) => tracing::warn!(error = %e, "connection logs purge failed"),
                }
            })
        })?)
        .await?;

    // Inactivity warning (monthly, 1st at 4:30 AM)
    let inactivity_pool = db.clone();
    let inactivity_email = email_service.clone();
    sched
        .add(Job::new_async("0 30 4 1 * *", move |_, _| {
            let pool = inactivity_pool.clone();
            let email_svc = inactivity_email.clone();
            Box::pin(async move {
                let users: Vec<(uuid::Uuid, String)> = sqlx::query_as(
                    "SELECT id, email FROM users \
                     WHERE last_active_at < NOW() - INTERVAL '23 months' \
                     AND inactivity_notice_sent_at IS NULL",
                )
                .fetch_all(&pool)
                .await
                .unwrap_or_default();

                for (user_id, email) in &users {
                    let _ = email_svc.send_inactivity_warning(email).await;
                    let _ = sqlx::query(
                        "UPDATE users SET inactivity_notice_sent_at = NOW() WHERE id = $1",
                    )
                    .bind(user_id)
                    .execute(&pool)
                    .await;
                }

                if !users.is_empty() {
                    tracing::info!(count = users.len(), "inactivity warnings sent");
                }
            })
        })?)
        .await?;

    // Delete inactive accounts (monthly, 1st at 5:00 AM)
    let deletion_pool = db.clone();
    sched
        .add(Job::new_async("0 0 5 1 * *", move |_, _| {
            let pool = deletion_pool.clone();
            Box::pin(async move {
                match sqlx::query(
                    "DELETE FROM users \
                     WHERE inactivity_notice_sent_at IS NOT NULL \
                     AND inactivity_notice_sent_at < NOW() - INTERVAL '30 days' \
                     AND last_active_at < NOW() - INTERVAL '24 months'",
                )
                .execute(&pool)
                .await
                {
                    Ok(r) => {
                        if r.rows_affected() > 0 {
                            tracing::info!(rows = r.rows_affected(), "inactive accounts deleted");
                        }
                    }
                    Err(e) => tracing::warn!(error = %e, "inactive account deletion failed"),
                }
            })
        })?)
        .await?;

    sched.start().await?;

    Ok(())
}
