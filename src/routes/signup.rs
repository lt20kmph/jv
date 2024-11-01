use crate::constants;
use crate::db::queries;
use crate::db::queries::Db;
use crate::errors;
use crate::models::models::UserSignup;
use log::info;
use reqwest;
use rocket::form::Form;
use rocket::response::content;
use rocket::{get, post};
use rocket_db_pools::Connection;
use serde_json::json;
use std::env;

async fn send_email(
    to: &str,
    subject: &str,
    body: &str,
    category: &str,
) -> Result<(), errors::AppError> {
    let api_key = env::var("MAILTRAP_API_KEY").expect("MAILTRAP_API_KEY must be set");
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
        .header("Api-Token", api_key)
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
    let signup = constants::TEMPLATES.render("signup.html", &tera::Context::new())?;
    Ok(content::RawHtml(signup))
}

#[post("/signup", data = "<user_signup>")]
pub async fn post(
    user_signup: Form<UserSignup>,
    conn: Connection<Db>,
) -> Result<content::RawHtml<String>, errors::AppError> {
    let verification_id =
        queries::insert_user(conn, &user_signup.email, &user_signup.password).await?;
    let host = env::var("JV_HOST").expect("JV_HOST must be set");
    let verification_link = format!("https://{}/signup/{}", host, verification_id);
    let mut context = tera::Context::new();
    context.insert("verification_link", &verification_link);
    context.insert("new_user_email", &user_signup.email);

    let email_body = constants::TEMPLATES.render("verify_signup.html", &context)?;
    let admin_email = env::var("JV_ADMIN_EMAIL").expect("JV_ADMIN_EMAIL must be set");

    send_email(
        &admin_email,
        constants::VERIFY_NEW_USER_SUBJECT,
        &email_body,
        constants::VERIFY_NEW_USER_CATEGORY,
    )
    .await?;

    let html = constants::TEMPLATES.render("awaiting_verification.html", &tera::Context::new())?;
    Ok(content::RawHtml(html))
}

#[get("/signup/<verification_id>")]
pub async fn verify(
    verification_id: String,
    db: &Db,
) -> Result<content::RawHtml<String>, errors::AppError> {
    // TODO: Add TTL for verification links
    let email = queries::verify_user(db, &verification_id).await?;
    let host = env::var("JV_HOST").expect("JV_HOST must be set");
    let login_link = format!("https://{}/login", host);

    let mut context = tera::Context::new();
    context.insert("login_link", &login_link);

    let email_body = constants::TEMPLATES.render("welcome.html", &context)?;

    send_email(
        &email,
        constants::WELCOME_SUBJECT,
        &email_body,
        constants::WELCOME_CATEGORY,
    )
    .await?;

    let login = "";
    Ok(content::RawHtml(login.to_string()))
}
