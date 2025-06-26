use crate::db::queries;
use crate::db::queries::Db;
use crate::errors;

use rocket::get;
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;

#[get("/logout")]
pub async fn get(cookies: &CookieJar<'_>, jv_db: &Db) -> Result<Redirect, errors::AppError> {
    // Get the session token from the cookie
    if let Some(cookie) = cookies.get_private("s_id") {
        let session_token = cookie.value();
        
        // Delete the session from the database
        queries::delete_user_session(jv_db, session_token).await?;
        
        // Remove the session cookie
        cookies.remove_private(Cookie::named("s_id"));
    }
    
    // Redirect to login page
    Ok(Redirect::to("/login"))
}