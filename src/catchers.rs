use rocket::catch;
use rocket::response::Redirect;

#[catch(401)]
pub fn not_authorized() -> Redirect {
    Redirect::to("/login")
}

#[catch(403)]
pub fn forbidden() -> &'static str {
    "Access Denied: You need Writer role to perform this action"
}
