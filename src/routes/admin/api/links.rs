use chrono::{DateTime, Utc};
use poem::{
    Body, Route,
    http::StatusCode,
    web::{Data, Json},
};

use crate::{
    common::platform_auth::AuthedPlatform,
    db::links::{Link, create_link, get_link},
};

pub fn routes() -> Route {
    Route::new().at("", poem::post(post_create_link))
}

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
#[serde(tag = "error_type")]
enum PostCreateLinkError {
    #[error("slug is already in use for existing link")]
    SlugAlreadyUsed(LinkDetailsView),
}

#[derive(serde::Deserialize)]
struct PostCreateLinkRequest {
    slug: Option<String>,
    url: String,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, serde::Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
struct LinkDetailsView {
    slug: String,
    url: String,
    metadata: Option<serde_json::Value>,
    created_at: DateTime<Utc>,
}

impl From<Link> for LinkDetailsView {
    fn from(value: Link) -> Self {
        LinkDetailsView {
            slug: value.slug,
            url: value.url,
            metadata: value.metadata,
            created_at: value.created_at,
        }
    }
}

#[poem::handler]
pub async fn post_create_link(
    db: Data<&sqlx::PgPool>,
    Json(create_request): Json<PostCreateLinkRequest>,
    AuthedPlatform(platform): AuthedPlatform,
) -> poem::Result<Json<LinkDetailsView>> {
    let mut db = db.acquire().await.unwrap();

    if let Some(custom_slug) = &create_request.slug {
        let link_for_slug = get_link(&mut db, custom_slug.as_str()).await.unwrap();

        if let Some(link_for_slug) = link_for_slug {
            return Err(poem::Error::from_response(
                poem::Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(
                        Body::from_json(PostCreateLinkError::SlugAlreadyUsed(
                            LinkDetailsView::from(link_for_slug),
                        ))
                        .unwrap(),
                    ),
            ));
        }
    }

    let link = create_link(
        &mut db,
        &platform.id,
        create_request.slug,
        create_request.url,
        create_request.metadata,
    )
    .await
    .unwrap();

    Ok(Json(LinkDetailsView::from(link)))
}
