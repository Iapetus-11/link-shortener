use askama::Template;
use poem::{
    EndpointExt, Response,
    endpoint::DynEndpoint,
    get,
    http::StatusCode,
    post,
    session::Session,
    web::{Data, Form, Html, Query, Redirect},
};
use serde::{Deserialize, Serialize};
use serde_valid::{Validate, json::ToJsonString};
use uuid::Uuid;

use crate::{
    common::{
        dashboard_auth::dashboard_auth_middleware,
        platform_auth::{PlatformApiKeyAndHash, generate_platform_api_key},
        validation::validate_to_poem_error,
    },
    db::{
        links::{Link, create_link, delete_link, get_links},
        platforms::{
            Platform, UpdatePlatformData, create_platform, delete_platform, get_platform,
            get_platform_by_name, get_platforms, update_platform,
        },
    },
};

// TODO: Add auth

pub fn routes() -> Box<dyn DynEndpoint<Output = Response>> {
    poem::Route::new()
        .at("", get(get_view))
        .at("/reset-api-key/", post(post_reset_api_key))
        .at("/create-platform/", post(post_create_platform))
        .at("/delete-platform/", post(post_delete_platform))
        .at("/create-link/", post(post_create_link))
        .at("/delete-link/", post(post_delete_link))
        .around(dashboard_auth_middleware)
        .boxed()
}

const PAGE_STATE_KEY: &str = "__Host-PS";

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum PageActionResult {
    ShowNewPlatformApiKey { platform_id: Uuid, api_key: String },
    CreateNameAlreadyInUse { name: String },
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PageState {
    action_result: Option<PageActionResult>,
}

#[derive(Deserialize)]
pub struct HomeViewQueryParams {
    platform: Option<Uuid>,
}

#[derive(askama::Template)]
#[template(path = "views/admin/dashboard/home.html")]
struct HomeViewTemplate<'a> {
    platforms: &'a Vec<Platform>,
    links: &'a Vec<Link>,

    state: &'a PageState,

    selected_platform: Option<&'a Platform>,
}

#[poem::handler]
pub async fn get_view(
    db_pool: Data<&sqlx::PgPool>,
    session: &Session,
    Query(HomeViewQueryParams {
        platform: selected_platform_id,
    }): Query<HomeViewQueryParams>,
) -> poem::Result<Html<String>> {
    let mut db = db_pool.acquire().await.unwrap();

    let platforms = get_platforms(&mut db).await.unwrap();

    let links: Vec<Link>;
    let selected_platform: Option<&Platform>;
    if let Some(selected_platform_id) = selected_platform_id {
        links = get_links(&mut db, &selected_platform_id).await.unwrap();
        selected_platform = platforms.iter().find(|p| p.id == selected_platform_id);
    } else {
        links = vec![];
        selected_platform = None;
    };

    let mut page_state = session.get::<PageState>(PAGE_STATE_KEY).unwrap_or_default();
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
            links: &links,
            state: &page_state,
            selected_platform,
        }
        .render()
        .unwrap(),
    ))
}

#[derive(Deserialize)]
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

    Ok(Redirect::see_other(format!(
        "/admin/dashboard/?platform={}",
        platform_id
    )))
}

#[derive(Validate, Deserialize)]
pub struct PostCreatePlatformRequest {
    #[validate(min_length = 2)]
    #[validate(max_length = 28)]
    name: String,
}

#[poem::handler]
pub async fn post_create_platform(
    db_pool: Data<&sqlx::PgPool>,
    Form(create_platform_request): Form<PostCreatePlatformRequest>,
    session: &Session,
) -> poem::Result<Redirect> {
    let PostCreatePlatformRequest { name } = validate_to_poem_error(create_platform_request)?;

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

    Ok(Redirect::see_other(format!(
        "/admin/dashboard/?platform={}",
        platform.id
    )))
}

#[derive(Deserialize)]
pub struct PostDeletePlatformRequest {
    platform_id: Uuid,
}

#[poem::handler]
pub async fn post_delete_platform(
    db_pool: Data<&sqlx::PgPool>,
    Form(PostDeletePlatformRequest { platform_id }): Form<PostDeletePlatformRequest>,
) -> poem::Result<Redirect> {
    let mut db = db_pool.acquire().await.unwrap();

    let deleted_platform = delete_platform(&mut db, &platform_id).await.unwrap();

    match deleted_platform {
        None => Err(poem::Error::from_status(StatusCode::BAD_REQUEST)),
        Some(_) => Ok(Redirect::see_other("/admin/dashboard/")),
    }
}

#[derive(Validate, Deserialize)]
pub struct PostCreateLinkRequest {
    platform_id: Uuid,

    #[validate(min_length = 7)]
    #[validate(max_length = 1000)]
    #[validate(
        pattern = r"https?:\/\/(.+)?[-a-zA-Z0-9@:%._\+~#=]{1,256}(\.[a-zA-Z0-9()]{1,6})?\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)"
    )]
    url: String,

    #[validate(min_length = 2)]
    #[validate(max_length = 100)]
    #[validate(pattern = r"^[\w\-]{2,28}$")]
    slug: Option<String>,

    metadata: Option<serde_json::Value>,
}

#[poem::handler]
pub async fn post_create_link(
    db_pool: Data<&sqlx::PgPool>,
    Form(mut create_link_request): Form<PostCreateLinkRequest>,
) -> poem::Result<Redirect> {
    if create_link_request
        .slug
        .as_ref()
        .is_some_and(|s| s.is_empty())
    {
        create_link_request.slug = None;
    }

    if let Some(ref metadata) = create_link_request.metadata {
        if match metadata {
            serde_json::Value::Null => true,
            serde_json::Value::Bool(_) => false,
            serde_json::Value::Number(_) => false,
            serde_json::Value::String(string) => string.is_empty(),
            serde_json::Value::Array(array) => array.is_empty(),
            serde_json::Value::Object(object) => object.is_empty(),
        } {
            create_link_request.metadata = None;
        // Take string from text input and re-parse it
        } else if let serde_json::Value::String(string) = metadata {
            if let Ok(parsed_str) = serde_json::from_str::<serde_json::Value>(string) {
                create_link_request.metadata = Some(parsed_str);
            }
        }
    }

    let PostCreateLinkRequest {
        platform_id,
        url,
        mut slug,
        mut metadata,
    } = validate_to_poem_error(create_link_request)?;

    let mut db = db_pool.acquire().await.unwrap();

    // See src/routes/mod.rs::routes
    const BLACKLISTED_SLUGS: &[&str] = &["admin", "static"];
    if slug
        .as_ref()
        .is_some_and(|slug| BLACKLISTED_SLUGS.contains(&slug.as_str()))
    {
        return Err(poem::Error::from_string(
            format!("Slug cannot be one of: {}", BLACKLISTED_SLUGS.join(", ")),
            StatusCode::BAD_REQUEST,
        ));
    }

    if get_platform(&mut db, &platform_id).await.unwrap().is_none() {
        return Err(poem::Error::from_string(
            format!("Can not find platform for ID: {:?}", platform_id),
            StatusCode::BAD_REQUEST,
        ));
    }

    create_link(&mut db, &platform_id, slug, url, metadata)
        .await
        .unwrap();

    Ok(Redirect::see_other(format!(
        "/admin/dashboard/?platform={platform_id}"
    )))
}

#[derive(Deserialize)]
pub struct PostDeleteLinkRequest {
    link_slug: String,
}

#[poem::handler]
pub async fn post_delete_link(
    db_pool: Data<&sqlx::PgPool>,
    Form(PostDeleteLinkRequest { link_slug }): Form<PostDeleteLinkRequest>,
) -> poem::Result<Redirect> {
    let mut db = db_pool.acquire().await.unwrap();

    let deleted_link = delete_link(&mut db, &link_slug).await.unwrap();

    match deleted_link {
        None => Err(poem::Error::from_string(
            "Link for specified slug does not exist",
            StatusCode::NOT_FOUND,
        )),
        Some(deleted_link) => Ok(Redirect::see_other(format!(
            "/admin/dashboard/?platform={}",
            deleted_link.platform_id
        ))),
    }
}
