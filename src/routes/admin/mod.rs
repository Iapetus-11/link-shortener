mod api;

pub fn routes() -> poem::Route {
    poem::Route::new().nest("/api/", api::routes())
}
