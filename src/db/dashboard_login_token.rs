use chrono::{DateTime, TimeDelta, Utc};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::config::CONFIG;

pub struct DashboardLoginToken {
    pub id: Uuid,
    pub token_hash: String,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

/// Creates a dashboard login token in the database, returning the unhashed token
/// and an instance of DashboardLoginToken
pub async fn create_dashboard_login_token(
    db: &mut PgConnection,
    token_hash: &str,
) -> sqlx::Result<DashboardLoginToken> {
    let token_row = sqlx::query_as!(
        DashboardLoginToken,
        r#"
            INSERT INTO dashboard_login_tokens (id, token_hash, created_at)
            VALUES ($1, $2, NOW())
            RETURNING id, token_hash, created_at;
        "#,
        Uuid::now_v7(),
        token_hash,
    )
    .fetch_one(&mut *db)
    .await?;

    Ok(token_row)
}

pub async fn get_dashboard_login_token(
    db: &mut PgConnection,
    id: &Uuid,
) -> sqlx::Result<Option<DashboardLoginToken>> {
    let expiration_barrier =
        Utc::now() - TimeDelta::seconds(CONFIG.admin_login_expires_after_seconds as i64);

    sqlx::query_as!(
        DashboardLoginToken,
        "SELECT id, token_hash, created_at FROM dashboard_login_tokens WHERE id = $1 AND created_at > $2",
        id,
        expiration_barrier,
    ).fetch_optional(&mut *db).await
}
