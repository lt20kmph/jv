use crate::constants::{self, CONFIG};
use crate::errors;
use log::info;
use serde_json::json;

pub async fn send_email(
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
