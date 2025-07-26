use chrono::{DateTime, Utc};
use rand::distr::{Alphanumeric, SampleString};
use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Link {
    pub id: Uuid,
    pub platform_id: Uuid,
    pub slug: String,
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
                    INSERT INTO links (platform_id, slug, url, metadata, created_at)
                    VALUES ($1, $2, $3, $4, NOW())
                    RETURNING id, platform_id, slug, url, metadata, created_at;
                "#,
                platform_id,
                slug,
                url,
                metadata,
            )
            .fetch_one(&mut *db)
            .await,
        );
    }

    result.unwrap()
}
