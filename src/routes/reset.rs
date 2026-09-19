use crate::auth::pw_utils;
use crate::constants::{self, CONFIG};
use crate::db::queries;
use crate::db::queries::Db;
use crate::email::send_email;
use crate::errors;
use crate::models::{ForgotPasswordRequest, ResetPasswordRequest};
use crate::tera_utils;
use chrono::Utc;
use log::info;
use rocket::form::Form;
use rocket::response::content;
use rocket::{get, post};
use uuid::Uuid;

#[get("/forgot_password")]
pub async fn get() -> Result<content::RawHtml<String>, errors::AppError> {
    let page = tera_utils::render_template_with_logging("forgot_password.html", &tera::Context::new())?;
    Ok(content::RawHtml(page))
}

#[post("/forgot_password", data = "<request>")]
pub async fn post(
    request: Form<ForgotPasswordRequest>,
    db: &Db,
) -> Result<content::RawHtml<String>, errors::AppError> {
    // The response is identical whether or not the email exists,
    // so we don't leak which addresses have accounts.
    if let Some((user_id, email)) =
        queries::get_verified_user_by_email(db, &request.email).await?
    {
        let token = Uuid::new_v4();
        let expires_at =
            Utc::now().timestamp() + constants::RESET_TOKEN_TTL_SECONDS;
        queries::create_reset_token(db, user_id, &token.to_string(), expires_at).await?;

        let reset_link = format!(
            "https://{}/reset_password/{}",
            CONFIG.jv_host.clone(),
            token
        );

        let mut context = tera::Context::new();
        context.insert("reset_link", &reset_link);

        let email_body =
            tera_utils::render_template_with_logging("reset_email.html", &context)?;

        send_email(
            &email,
            constants::RESET_PASSWORD_SUBJECT,
            &email_body,
            constants::RESET_CATEGORY,
        )
        .await?;

        info!("Password reset requested for user id {}", user_id);
    }

    let page =
        tera_utils::render_template_with_logging("check_email.html", &tera::Context::new())?;
    Ok(content::RawHtml(page))
}

#[get("/reset_password/<token>")]
pub async fn get_reset(
    token: String,
    db: &Db,
) -> Result<content::RawHtml<String>, errors::AppError> {
    let valid = reset_token_valid(db, &token).await?;
    render_reset_page(&token, valid, None)
}

#[post("/reset_password/<token>", data = "<request>")]
pub async fn post_reset(
    token: String,
    request: Form<ResetPasswordRequest<'_>>,
    db: &Db,
) -> Result<content::RawHtml<String>, errors::AppError> {
    let valid = reset_token_valid(db, &token).await?;

    if !valid {
        return render_reset_page(&token, false, None);
    }

    if request.password != request.confirm {
        return render_reset_page(&token, true, Some("Passwords do not match"));
    }

    if request.password.is_empty() {
        return render_reset_page(&token, true, Some("Please choose a password"));
    }

    let hashed = pw_utils::hash_and_salt_password(request.password)
        .map_err(errors::AppError::from)?;

    let (user_id, _, _) = queries::get_reset_token(db, &token)
        .await?
        .ok_or(errors::AppError {
            code: 400,
            message: "Invalid reset link".to_string(),
        })?;

    queries::update_user_password(db, user_id, &hashed.password_hash).await?;
    queries::delete_reset_token(db, &token).await?;
    queries::delete_user_sessions(db, user_id).await?;

    info!("Password reset completed for user id {}", user_id);

    render_reset_success()
}

async fn reset_token_valid(db: &Db, token: &str) -> Result<bool, errors::AppError> {
    let record = queries::get_reset_token(db, token).await?;

    Ok(match record {
        None => false,
        Some((_, _, expires_at)) => Utc::now().timestamp() < expires_at,
    })
}

fn render_reset_page(
    token: &str,
    valid: bool,
    error: Option<&str>,
) -> Result<content::RawHtml<String>, errors::AppError> {
    let mut context = tera::Context::new();
    context.insert("token", token);
    context.insert("valid", &valid);
    context.insert("error", &error);

    let page = tera_utils::render_template_with_logging("reset_password.html", &context)?;
    Ok(content::RawHtml(page))
}

fn render_reset_success() -> Result<content::RawHtml<String>, errors::AppError> {
    let page = tera_utils::render_template_with_logging("reset_success.html", &tera::Context::new())?;
    Ok(content::RawHtml(page))
}
