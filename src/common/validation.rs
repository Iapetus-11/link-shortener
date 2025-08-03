use poem::{Body, http::StatusCode};
use serde_valid::Validate;

/// Validates a struct with the Validate trait, returning a properly formatted BAD_REQUEST response
/// if validation fails.
pub fn validate_to_poem_error<T: Validate>(value: T) -> poem::Result<T> {
    value.validate().map(|_| value).map_err(|e| {
        poem::Error::from_response(
            poem::Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .content_type("application/json")
                .body(Body::from_string(e.to_string())),
        )
    })
}
