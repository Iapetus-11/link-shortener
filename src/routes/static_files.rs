use std::time::Duration;

use poem::{
    Body, EndpointExt, Response,
    endpoint::DynEndpoint,
    get,
    http::StatusCode,
    middleware::{NormalizePath, TrailingSlash},
    web::{Path, headers::CacheControl},
};

pub fn routes() -> Box<dyn DynEndpoint<Output = Response>> {
    poem::Route::new()
        .at(":file", get(get_file))
        .with(NormalizePath::new(TrailingSlash::Trim))
        .boxed()
}

#[poem::handler]
pub async fn get_file(Path((file,)): Path<(String,)>) -> poem::Result<Response> {
    let (content_type, data) = match file.as_str() {
        "InterVariable.woff2" => (
            "font/woff2",
            include_bytes!("../static/InterVariable.woff2"),
        ),
        _ => return Err(poem::Error::from_status(StatusCode::NOT_FOUND)),
    };

    Ok(Response::builder()
        .typed_header(
            CacheControl::new()
                .with_public()
                .with_max_age(Duration::from_secs(3600)),
        )
        .content_type(content_type)
        .body(Body::from_bytes((data as &[u8]).into())))
}
