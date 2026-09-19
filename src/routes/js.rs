use crate::errors;
use crate::http_utils::NoCache;
use crate::tera_utils;
use rocket::get;
use rocket::response::content;

#[get("/js/<script_name>")]
pub async fn get(
    script_name: &str,
) -> Result<NoCache<content::RawJavaScript<String>>, errors::AppError> {
    let context = tera::Context::new();
    let path = format!("js/{}", script_name);
    let js = tera_utils::render_template_with_logging(&path, &context)?;
    Ok(NoCache(content::RawJavaScript(js)))
}
