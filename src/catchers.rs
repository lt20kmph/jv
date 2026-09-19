use crate::constants;
use rocket::catch;
use rocket::response::content;
use rocket::response::Redirect;

#[catch(401)]
pub fn not_authorized() -> Redirect {
    Redirect::to("/login")
}

#[catch(403)]
pub fn forbidden() -> &'static str {
    "Access Denied: You need Writer role to perform this action"
}

#[catch(404)]
pub fn not_found() -> content::RawHtml<String> {
    let context = tera::Context::new();
    let html = constants::TEMPLATES
        .render("error404.html", &context)
        .unwrap_or_else(|_| {
            "<html><body><h2>Page not found</h2><p><a href=\"/galleries\">Back to galleries</a></p></body></html>".to_string()
        });
    content::RawHtml(html)
}
