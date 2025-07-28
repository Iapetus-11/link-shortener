use std::sync::{Arc, OnceLock};

use poem::{
    EndpointExt,
    endpoint::BoxEndpoint,
    middleware::{AddData, NormalizePath, TrailingSlash},
    test::TestClient,
    web::headers::{Authorization, authorization::Basic},
};
use uuid::Uuid;

use crate::routes::routes;

// Cache API test client to improve test execution speed
static API_TEST_CLIENT: OnceLock<Arc<BoxEndpoint<'static>>> = OnceLock::new();

pub fn api_test_client(db_pool: sqlx::PgPool) -> TestClient<BoxEndpoint<'static>> {
    let cached_app = API_TEST_CLIENT.get_or_init(|| {
        let app = routes().with(NormalizePath::new(TrailingSlash::Always));

        Arc::new(app.boxed())
    });

    TestClient::new(cached_app.clone().with(AddData::new(db_pool)).boxed())
}

pub fn platform_auth_header(platform_id: &Uuid, api_key: &str) -> Authorization<Basic> {
    Authorization::basic(&platform_id.to_string(), api_key)
}
