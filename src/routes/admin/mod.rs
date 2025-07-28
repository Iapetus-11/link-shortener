mod api;
mod dashboard;

pub fn routes() -> poem::Route {
    poem::Route::new()
        .nest("/api/", api::routes())
        .nest("/dashboard/", dashboard::routes())
}
