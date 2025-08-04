use serde::{Deserialize, Serialize};
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
pub async fn get_platform(db: &mut PgConnection, id: &Uuid) -> sqlx::Result<Option<Platform>> {
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

/// Retrieve a platform by its name, case insensitively
pub async fn get_platform_by_name(
    db: &mut PgConnection,
    name: &str,
) -> sqlx::Result<Option<Platform>> {
    sqlx::query_as!(
        Platform,
        r#"
            SELECT id, name, api_key_hash FROM platforms WHERE UPPER(name) = UPPER($1)
        "#,
        name,
    )
    .fetch_optional(&mut *db)
    .await
}

/// Fetch all platforms from the DB
pub async fn get_platforms(db: &mut PgConnection) -> sqlx::Result<Vec<Platform>> {
    sqlx::query_as!(
        Platform,
        r#"
            SELECT id, name, api_key_hash FROM platforms ORDER BY name;
        "#,
    )
    .fetch_all(&mut *db)
    .await
}

#[derive(Serialize, Deserialize, Default)]
pub struct UpdatePlatformData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_hash: Option<String>,
}

/// Updates a platform with the provided values, if fields are set as None then they are not updated.
/// Returns the updated Platform, or if no platform exists with the specified ID, None.
pub async fn update_platform(
    db: &mut PgConnection,
    id: &Uuid,
    update_data: &UpdatePlatformData,
) -> sqlx::Result<Option<Platform>> {
    sqlx::query_as!(
        Platform,
        r#"
            UPDATE platforms
            SET
                name = CASE
                    WHEN $2 ? 'name'
                    THEN ($2->>'name')::VARCHAR
                    ELSE name END,
                api_key_hash = CASE
                    WHEN $2 ? 'api_key_hash'
                    THEN ($2->>'api_key_hash')::VARCHAR
                    ELSE api_key_hash END
            WHERE id = $1
            RETURNING id, name, api_key_hash
        "#,
        id,
        serde_json::to_value(update_data).unwrap(),
    )
    .fetch_optional(&mut *db)
    .await
}

/// Delete a platform by its ID, returning None if no platform for the specified ID was found
pub async fn delete_platform(db: &mut PgConnection, id: &Uuid) -> sqlx::Result<Option<Platform>> {
    sqlx::query_as!(
        Platform,
        r#"
            DELETE FROM platforms WHERE id = $1 RETURNING id, name, api_key_hash;
        "#,
        id,
    )
    .fetch_optional(db)
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

        let platform = get_platform(&mut db, &platform.id).await.unwrap().unwrap();
        assert_eq!(platform.name, "Some Platform");
    }

    #[sqlx::test]
    async fn test_get_platform_by_name(mut db: PgPoolConn) {
        let (_, platform_a) = create_platform(&mut db, "Platform A").await.unwrap();
        create_platform(&mut db, "Test B").await.unwrap();

        let retrieved_platform_a = get_platform_by_name(&mut db, "Platform A")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(platform_a.id, retrieved_platform_a.id);

        assert!(
            get_platform_by_name(&mut db, "womp")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[sqlx::test]
    async fn test_get_platforms(mut db: PgPoolConn) {
        let (_, platform_a) = create_platform(&mut db, "Platform A").await.unwrap();
        let (_, platform_b) = create_platform(&mut db, "Platform B").await.unwrap();

        let platforms = get_platforms(&mut db).await.unwrap();

        assert_eq!(platforms.len(), 2);
        assert_eq!(platforms[0].id, platform_a.id);
        assert_eq!(platforms[1].id, platform_b.id);
    }

    #[sqlx::test]
    async fn test_update_whole_platform(mut db: PgPoolConn) {
        let (_, platform) = create_platform(&mut db, "Test").await.unwrap();

        let updated_platform = update_platform(
            &mut db,
            &platform.id,
            &UpdatePlatformData {
                name: Some("New Name".to_string()),
                api_key_hash: Some("Not a real hash but new".to_string()),
            },
        )
        .await
        .unwrap()
        .unwrap();

        assert_eq!(platform.id, updated_platform.id);

        assert_eq!(updated_platform.name, "New Name");
        assert_ne!(updated_platform.name, platform.name);

        assert_eq!(updated_platform.api_key_hash, "Not a real hash but new");
        assert_ne!(updated_platform.api_key_hash, platform.api_key_hash);
    }

    #[sqlx::test]
    async fn test_update_nonexistent_platform(mut db: PgPoolConn) {
        let updated_platform = update_platform(
            &mut db,
            &Uuid::now_v7(),
            &UpdatePlatformData {
                name: Some("New Name".to_string()),
                api_key_hash: Some("BLAH".to_string()),
            },
        )
        .await
        .unwrap();

        assert!(updated_platform.is_none());
    }

    #[sqlx::test]
    async fn test_update_platform_missing_all_fields(mut db: PgPoolConn) {
        let (_, platform) = create_platform(&mut db, "minecraft.global").await.unwrap();

        let updated_platform = update_platform(
            &mut db,
            &platform.id,
            &UpdatePlatformData {
                name: None,
                api_key_hash: None,
            },
        )
        .await
        .unwrap()
        .unwrap();

        assert_eq!(updated_platform.id, platform.id);
        assert_eq!(updated_platform.name, platform.name);
        assert_eq!(updated_platform.api_key_hash, platform.api_key_hash);
    }

    #[sqlx::test]
    async fn test_delete_platform(mut db: PgPoolConn) {
        let (_, platform) = create_platform(&mut db, "Villager Bot").await.unwrap();

        let deleted_platform = delete_platform(&mut db, &platform.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            sqlx::query!("SELECT COUNT(*) FROM platforms")
                .fetch_one(&mut *db)
                .await
                .unwrap()
                .count,
            Some(0)
        );

        assert_eq!(deleted_platform.id, platform.id);
        assert_eq!(deleted_platform.name, platform.name);
        assert_eq!(deleted_platform.api_key_hash, platform.api_key_hash);
    }

    #[sqlx::test]
    async fn test_delete_nonexistent_platform(mut db: PgPoolConn) {
        let deleted_platform = delete_platform(&mut db, &Uuid::now_v7()).await.unwrap();

        assert!(deleted_platform.is_none());
    }
}
