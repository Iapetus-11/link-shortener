use poem::get;

mod home;

pub fn routes() -> poem::Route {
    poem::Route::new().nest("", get(home::get_view))
}
