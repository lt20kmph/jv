use crate::constants::{self, CONFIG};
use crate::db::queries;
use crate::db::queries::Db;
use crate::errors;
use crate::models::{self, UserSignup};
use crate::tera_utils;
use log::info;
use reqwest;
use rocket::form::Form;
use rocket::response::content;
use rocket::{get, post};
use rocket_db_pools::Connection;
use serde_json::json;

async fn send_email(
    to: &str,
    subject: &str,
    body: &str,
    category: &str,
) -> Result<(), errors::AppError> {
    let email_payload = json!({
        "from": {"email" : constants::JV_EMAIL},
        "to": [{"email": to}],
        "subject": subject,
        "text": body,
        "html": body,
        "category": category
    });

    let client = reqwest::Client::new();
    let response = client
        .post(constants::MAILTRAP_SEND)
        .header("Content-Type", "application/json")
        .header("Api-Token", CONFIG.mailtrap_api_key.clone())
        .body(email_payload.to_string()) // Serialize the JSON payload to a string
        .send()
        .await?;

    if response.status().is_success() {
        info!("Email sent successfully!");
    } else {
        info!("Failed to send email. Status: {:?}", response.status());

        // Print the response body for additional information
        let body = response.text().await?;
        info!("Response body: {}", body);
    }

    Ok(())
}

#[get("/signup")]
pub async fn get() -> Result<content::RawHtml<String>, errors::AppError> {
    let signup = tera_utils::render_template_with_logging("signup.html", &tera::Context::new())?;
    Ok(content::RawHtml(signup))
}

#[post("/signup", data = "<user_signup>")]
pub async fn post(
    user_signup: Form<UserSignup>,
    conn: Connection<Db>,
) -> Result<content::RawHtml<String>, errors::AppError> {
    let verification_id =
        queries::insert_user(conn, &user_signup.email, &user_signup.password).await?;
    let host = CONFIG.jv_host.clone();
    let verification_link = format!("https://{}/signup/{}", host, verification_id);
    let mut context = tera::Context::new();
    context.insert("verification_link", &verification_link);
    context.insert("new_user_email", &user_signup.email);

    let email_body = tera_utils::render_template_with_logging("verify_signup.html", &context)?;
    let admin_email = CONFIG.jv_admin_email.clone();

    send_email(
        &admin_email,
        constants::VERIFY_NEW_USER_SUBJECT,
        &email_body,
        constants::VERIFY_NEW_USER_CATEGORY,
    )
    .await?;

    let html = tera_utils::render_template_with_logging("awaiting_verification.html", &tera::Context::new())?;
    Ok(content::RawHtml(html))
}

#[get("/signup/<verification_id>")]
pub async fn verify(
    verification_id: String,
    db: &Db,
) -> Result<content::RawHtml<String>, errors::AppError> {
    let user = queries::get_user_by_verification(db, &verification_id).await?;

    let outcome = match user {
        // Unknown or already-used link
        None => models::VerificationOutcome::Invalid,
        Some((email, time_created, is_verified)) => {
            if is_verified {
                models::VerificationOutcome::AlreadyVerified
            } else if is_verification_expired(&time_created) {
                models::VerificationOutcome::Expired
            } else {
                queries::verify_user(db, &verification_id).await?;

                let host = CONFIG.jv_host.clone();
                let login_link = format!("https://{}/login", host);

                let mut context = tera::Context::new();
                context.insert("login_link", &login_link);

                let email_body =
                    tera_utils::render_template_with_logging("welcome.html", &context)?;

                send_email(
                    &email,
                    constants::WELCOME_SUBJECT,
                    &email_body,
                    constants::WELCOME_CATEGORY,
                )
                .await?;

                models::VerificationOutcome::Verified
            }
        }
    };

    let mut context = tera::Context::new();
    context.insert("outcome", &outcome);

    let page =
        tera_utils::render_template_with_logging("verification_result.html", &context)?;
    Ok(content::RawHtml(page))
}

fn is_verification_expired(time_created: &str) -> bool {
    match chrono::NaiveDateTime::parse_from_str(time_created, "%Y-%m-%d %H:%M:%S") {
        Ok(created) => {
            let age = chrono::Utc::now().naive_utc() - created;
            age > chrono::Duration::days(constants::VERIFICATION_LINK_TTL_DAYS)
        }
        Err(e) => {
            info!("Couldn't parse time_created '{}': {}", time_created, e);
            // Fail closed: treat unparseable timestamps as expired
            true
        }
    }
}
