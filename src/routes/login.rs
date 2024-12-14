use crate::constants;
use crate::db::queries;
use crate::db::queries::Db;
use crate::errors;
use crate::models::models::UserLogin;

use rocket::form::Form;
use rocket::get;
use rocket::http::{Cookie, CookieJar};
use rocket::post;
use rocket::response::content;
use rocket::response::Redirect;

#[get("/login")]
pub async fn get() -> Result<content::RawHtml<String>, errors::AppError> {
    let login = constants::TEMPLATES.render("login.html", &tera::Context::new())?;
    Ok(content::RawHtml(login))
}

#[post("/login", data = "<user_login>")]
pub async fn post(
    user_login: Form<UserLogin<'_>>,
    cookies: &CookieJar<'_>,
    jv_db: &Db,
) -> Result<Redirect, errors::AppError> {
    let is_valid = queries::verify_password(jv_db, user_login.email, user_login.password).await?;

    if is_valid {
        let session_token = queries::create_user_session(jv_db, user_login.email).await?;
        cookies.add_private(Cookie::new("s_id", session_token));
    } else {
        return Err(errors::AppError {
            code: 401,
            message: "Invalid Credentials".to_string(),
        });
    }
    Ok(Redirect::to("/galleries"))
}
