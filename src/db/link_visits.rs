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
    use super::*;

    use crate::{
        common::testing::db::PgPoolConn,
        db::{links::create_link, platforms::create_platform},
    };

    #[sqlx::test]
    async fn test_create_link_visit(mut db: PgPoolConn) {
        let (_, platform) = create_platform(&mut db, "Guacamole").await.unwrap();

        let link = create_link(
            &mut db,
            &platform.id,
            None,
            "https://iapetus11.me/fractals".to_string(),
            None,
        )
        .await
        .unwrap();

        create_link_visit(&mut db, &link.slug, HashMap::new(), Some("0.0.0.0"))
            .await
            .unwrap();
        create_link_visit(&mut db, &link.slug, HashMap::new(), Some("0.0.0.0"))
            .await
            .unwrap();

        let visit_count = sqlx::query!(
            "SELECT COUNT(*) FROM link_visits WHERE link_slug = $1",
            link.slug
        )
        .fetch_one(&mut *db)
        .await
        .unwrap();
        assert_eq!(visit_count.count, Some(2));
    }
}
