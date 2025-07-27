use std::collections::HashMap;

use sqlx::PgConnection;

pub async fn create_link_visit(
    db: &mut PgConnection,
    slug: &str,
    headers: HashMap<String, Vec<String>>,
    ip_address: Option<&str>,
) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO link_visits (link_slug, at, headers, ip_address) VALUES ($1, NOW(), $2, $3)",
        slug,
        serde_json::to_value(headers).unwrap(),
        ip_address,
    )
    .execute(&mut *db)
    .await
    .map(|_| ())
}

#[cfg(test)]
pub mod tests {
    #[sqlx::test]
    async fn test_create_link_visit() {}
}
