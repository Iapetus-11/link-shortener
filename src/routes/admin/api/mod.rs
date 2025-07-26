use poem::Route;

mod links;

pub fn routes() -> Route {
    Route::new().nest("/links/", links::routes())
}
