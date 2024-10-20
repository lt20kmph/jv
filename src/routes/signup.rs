use crate::constants;
use crate::db::queries;
use crate::db::queries::Db;
use crate::errors;
use crate::models::models::UserSignup;
use rocket::form::Form;
use rocket::response::content;
use rocket::{get, post};
use rocket_db_pools::Connection;

#[get("/signup")]
pub async fn get() -> Result<content::RawHtml<String>, errors::AppError> {
    let signup = constants::TEMPLATES.render("signup.html", &tera::Context::new())?;
    Ok(content::RawHtml(signup))
}

#[post("/signup", data = "<user_signup>")]
pub async fn post(
    user_signup: Form<UserSignup>,
    conn: Connection<Db>,
) -> Result<content::RawHtml<String>, errors::AppError> {
    queries::insert_user(conn, &user_signup.email, &user_signup.password).await?;
    let login = constants::TEMPLATES.render("login.html", &tera::Context::new())?;
    Ok(content::RawHtml(login))
}
