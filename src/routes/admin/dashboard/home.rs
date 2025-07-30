use askama::Template;
use poem::{
    EndpointExt, Response,
    endpoint::DynEndpoint,
    get,
    http::StatusCode,
    post,
    session::{CookieConfig, MemoryStorage, ServerSession, Session},
    web::{Data, Form, Html, Query, Redirect, cookie::SameSite},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    common::platform_auth::{PlatformApiKeyAndHash, generate_platform_api_key},
    db::platforms::{
        Platform, UpdatePlatformData, create_platform, get_platform_by_name, get_platforms,
        update_platform,
    },
};

// TODO: Add auth

pub fn routes() -> Box<dyn DynEndpoint<Output = Response>> {
    poem::Route::new()
        .at("", get(get_view))
        .at("/reset-api-key/", post(post_reset_api_key))
        .at("/create-platform/", post(post_create_platform))
        .with(ServerSession::new(
            CookieConfig::new()
                .secure(true)
                .http_only(true)
                .same_site(SameSite::Strict), // TODO: Set domain
            MemoryStorage::new(),
        ))
        .boxed()
}

const PAGE_STATE_KEY: &str = "SESSION_STATE";

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum PageActionResult {
    ShowNewPlatformApiKey { platform_id: Uuid, api_key: String },
    CreateNameAlreadyInUse { name: String },
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PageState {
    action_result: Option<PageActionResult>,
    selected_platform_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct HomeViewQueryParams {
    // #[serde(default)]
    selected_platform: Option<Uuid>,
}

#[derive(askama::Template)]
#[template(path = "views/admin/dashboard/home.html")]
struct HomeViewTemplate<'a> {
    platforms: &'a Vec<Platform>,
    state: &'a PageState,
}

#[poem::handler]
pub async fn get_view(
    db_pool: Data<&sqlx::PgPool>,
    session: &Session,
    Query(HomeViewQueryParams {
        selected_platform: selected_platform_id,
    }): Query<HomeViewQueryParams>,
) -> poem::Result<Html<String>> {
    let mut db = db_pool.acquire().await.unwrap();

    let platforms = get_platforms(&mut db).await.unwrap();

    let mut page_state = session.get::<PageState>(PAGE_STATE_KEY).unwrap_or_default();
    page_state.selected_platform_id = selected_platform_id;
    session.set(
        PAGE_STATE_KEY,
        PageState {
            action_result: None,
            ..page_state
        },
    );

    Ok(Html(
        HomeViewTemplate {
            platforms: &platforms,
            state: &page_state,
        }
        .render()
        .unwrap(),
    ))
}

#[derive(Debug, Deserialize)]
struct PostResetAPIKeyRequest {
    platform_id: Uuid,
}

#[poem::handler]
pub async fn post_reset_api_key(
    db_pool: Data<&sqlx::PgPool>,
    Form(PostResetAPIKeyRequest { platform_id }): Form<PostResetAPIKeyRequest>,
    session: &Session,
) -> poem::Result<Redirect> {
    let mut db = db_pool.acquire().await.unwrap();

    let PlatformApiKeyAndHash {
        api_key,
        api_key_hash,
    } = generate_platform_api_key();

    let platform = update_platform(
        &mut db,
        &platform_id,
        &UpdatePlatformData {
            api_key_hash: Some(api_key_hash),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    if platform.is_none() {
        return Err(poem::Error::from_status(StatusCode::BAD_REQUEST));
    };

    let mut page_state = session.get::<PageState>(PAGE_STATE_KEY).unwrap_or_default();
    page_state.action_result = Some(PageActionResult::ShowNewPlatformApiKey {
        platform_id,
        api_key,
    });
    session.set(PAGE_STATE_KEY, &page_state);

    Ok(Redirect::see_other("/admin/dashboard/"))
}

#[derive(Debug, Deserialize)]
pub struct PostCreatePlatformRequest {
    name: String,
}

#[poem::handler]
pub async fn post_create_platform(
    db_pool: Data<&sqlx::PgPool>,
    Form(PostCreatePlatformRequest { name }): Form<PostCreatePlatformRequest>,
    session: &Session,
) -> poem::Result<Redirect> {
    let mut db = db_pool.acquire().await.unwrap();

    let mut page_state = session.get::<PageState>(PAGE_STATE_KEY).unwrap_or_default();

    let existing_platform = get_platform_by_name(&mut db, &name).await.unwrap();
    if existing_platform.is_some() {
        page_state.action_result = Some(PageActionResult::CreateNameAlreadyInUse { name });
        session.set(PAGE_STATE_KEY, &page_state);
        return Ok(Redirect::see_other("/admin/dashboard/"));
    }

    let (api_key, platform) = create_platform(&mut db, &name).await.unwrap();

    page_state.action_result = Some(PageActionResult::ShowNewPlatformApiKey {
        platform_id: platform.id,
        api_key,
    });
    session.set(PAGE_STATE_KEY, &page_state);

    Ok(Redirect::see_other("/admin/dashboard/"))
}
