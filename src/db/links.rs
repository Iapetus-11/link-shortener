use chrono::{DateTime, Utc};
use rand::distr::{Alphanumeric, SampleString};
use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Link {
    pub slug: String,
    pub platform_id: Uuid,
    pub url: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

pub async fn create_link(
    db: &mut PgConnection,
    platform_id: Uuid,
    mut slug: Option<String>,
    url: String,
    metadata: Option<serde_json::Value>,
) -> sqlx::Result<Link> {
    let mut result: Option<sqlx::Result<Link>> = None;

    let autogenerate_slug = slug.is_none();

    while match result {
        None => true,
        // Only retry if we're autogenerating a slug, otherwise it won't change and we'll have an infinite loop :)
        Some(Err(sqlx::Error::Database(ref db_err))) => {
            autogenerate_slug && db_err.is_unique_violation()
        }
        _ => false,
    } {
        if autogenerate_slug {
            slug = Some(Alphanumeric.sample_string(&mut rand::rng(), 7)) // TODO: Make this configurable
        }

        result = Some(
            sqlx::query_as!(
                Link,
                r#"
                    INSERT INTO links (slug, platform_id, url, metadata, created_at)
                    VALUES ($1, $2, $3, $4, NOW())
                    RETURNING slug, platform_id, url, metadata, created_at;
                "#,
                slug,
                platform_id,
                url,
                metadata,
            )
            .fetch_one(&mut *db)
            .await,
        );
    }

    result.unwrap()
}

pub async fn get_link(db: &mut PgConnection, slug: &str) -> sqlx::Result<Option<Link>> {
    sqlx::query_as!(
        Link,
        r#"
            SELECT slug, platform_id, url, metadata, created_at FROM links WHERE slug = $1
        "#,
        slug,
    )
    .fetch_optional(&mut *db)
    .await
}
