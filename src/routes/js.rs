use crate::constants;
use crate::errors;
use crate::tera_utils;
use rocket::get;
use rocket::response::content;

#[get("/js/<script_name>")]
pub async fn get(
    script_name: &str,
) -> Result<content::RawJavaScript<String>, errors::AppError> {
    let context = tera::Context::new();
    let path = format!("js/{}", script_name);
    let js = tera_utils::render_template_with_logging(&path, &context)?;
    Ok(content::RawJavaScript(js))
}
