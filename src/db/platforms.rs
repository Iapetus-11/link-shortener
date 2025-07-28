use sqlx::PgConnection;
use uuid::Uuid;

use crate::common::platform_auth::{PlatformApiKeyAndHash, generate_platform_api_key};

#[derive(Debug, Clone)]
pub struct Platform {
    pub id: Uuid,
    pub name: String,
    pub api_key_hash: String,
}

/// Creates a Platform, returning the unhashed API key and an object holding the Platform's data
pub async fn create_platform(
    db: &mut PgConnection,
    name: &str,
) -> sqlx::Result<(String, Platform)> {
    let PlatformApiKeyAndHash {
        api_key,
        api_key_hash,
    } = generate_platform_api_key();

    let platform = sqlx::query_as!(
        Platform,
        "INSERT INTO platforms (id, name, api_key_hash) VALUES ($1, $2, $3) RETURNING id, name, api_key_hash;",
        uuid::Uuid::now_v7(),
        name,
        api_key_hash,
    ).fetch_one(&mut *db).await?;

    Ok((api_key, platform))
}

/// Fetch a platform from the DB by its ID
pub async fn get_platform(db: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<Platform>> {
    sqlx::query_as!(
        Platform,
        r#"
            SELECT id, name, api_key_hash FROM platforms WHERE id = $1;
        "#,
        id,
    )
    .fetch_optional(&mut *db)
    .await
}

pub async fn get_platforms(db: &mut PgConnection) -> sqlx::Result<Vec<Platform>> {
    sqlx::query_as!(
        Platform,
        r#"
            SELECT id, name, api_key_hash FROM platforms;
        "#,
    )
    .fetch_all(&mut *db)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::testing::db::PgPoolConn;

    #[sqlx::test]
    async fn test_create_and_get_platform(mut db: PgPoolConn) {
        let (api_key, platform) = create_platform(&mut db, "Some Platform").await.unwrap();

        assert!(api_key.len() == 69);
        assert!(platform.name == "Some Platform");

        let platform = get_platform(&mut db, platform.id).await.unwrap().unwrap();
        assert_eq!(platform.name, "Some Platform");
    }
}
