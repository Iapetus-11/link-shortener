use chrono::{DateTime, TimeDelta, Utc};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{
    common::{
        argon2::check_key_against_hash,
        dashboard_auth::{GenerateLoginTokenAndHash, generate_login_token},
    },
    config::CONFIG,
};

pub struct DashboardLoginToken {
    pub id: Uuid,
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
}

/// Creates a dashboard login token in the database, returning the unhashed token
/// and an instance of DashboardLoginToken
pub async fn create_dashboard_login_token(
    db: &mut PgConnection,
) -> sqlx::Result<(String, DashboardLoginToken)> {
    let GenerateLoginTokenAndHash {
        token,
        hash: token_hash,
    } = generate_login_token();

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

    Ok((token, token_row))
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

pub fn check_dashboard_login_token(token_row: &DashboardLoginToken, token_to_check: &str) -> bool {
    check_key_against_hash(token_to_check, &token_row.token_hash)
}
