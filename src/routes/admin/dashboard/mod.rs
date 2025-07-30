mod home;

pub fn routes() -> poem::Route {
    poem::Route::new().nest("", home::routes())
}
