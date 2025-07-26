use chrono::{DateTime, Utc};
use poem::{
    Route,
    web::{Data, Json},
};

#[derive(serde::Deserialize)]
struct LinkCreateRequest {
    slug: Option<String>,
    url: String,
    metadata: Option<serde_json::Value>,
}

#[derive(serde::Serialize)]
struct LinkDetailsResponse {
    id: String,
    slug: String,
    url: String,
    metadata: Option<serde_json::Value>,
    created_at: DateTime<Utc>,
}

pub fn routes() -> Route {
    Route::new().at("", poem::post(post_create_link))
}

#[poem::handler]
pub async fn post_create_link(
    db: Data<&sqlx::PgPool>,
    Json(create_request): Json<LinkCreateRequest>,
) -> poem::Result<Json<LinkDetailsResponse>> {
    todo!();
}
