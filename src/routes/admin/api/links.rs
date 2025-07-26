use chrono::{DateTime, Utc};
use poem::{
    http::StatusCode, web::{Data, Json}, ResponseBuilder, Route
};

use crate::{common::platform_auth::AuthedPlatform, db::links::{create_link, get_link}};

pub fn routes() -> Route {
    Route::new().at("", poem::post(post_create_link))
}

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
#[serde(tag = "error_type")]
enum PostCreateLinkError {
    #[error("slug {slug:#?} is already in use")]
    SlugAlreadyUsed { slug: String }
}

#[derive(serde::Deserialize)]
struct PostCreateLinkRequest {
    slug: Option<String>,
    url: String,
    metadata: Option<serde_json::Value>,
}

#[derive(serde::Serialize)]
struct LinkDetailsView {
    id: String,
    slug: String,
    url: String,
    metadata: Option<serde_json::Value>,
    created_at: DateTime<Utc>,
}

#[poem::handler]
pub async fn post_create_link(
    db: Data<&sqlx::PgPool>,
    Json(create_request): Json<PostCreateLinkRequest>,
    AuthedPlatform(platform): AuthedPlatform,
) -> poem::Result<Json<LinkDetailsView>> {
    let mut db = db.acquire().await.unwrap();

    // create_link(db, platform_id, slug, url, metadata);

    if get_link(&mut *db, create_request.slug).await.unwrap().is_some() {
        return poem::Error::from_response(poem::Response::builder().status(StatusCode::BAD_REQUEST).body(Json(PostLink)))
    }

    todo!();
}
