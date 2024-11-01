use lazy_static::lazy_static;
use std::collections::HashMap;
use tera::Tera;
use log::error;

pub static SESSION_LENGTH: i64 = 60 * 60 * 24 * 7; // 1 week
pub static MAX_FILE_SIZE: u64 = 1024 * 1024 * 10; // 10 MB

pub static IMG_PATH: &str = "./img";
pub static MAILTRAP_SEND: &str = "https://send.api.mailtrap.io/api/send";
pub static JV_EMAIL: &str = "jv@lt20kmph.co.uk";
pub static VERIFY_NEW_USER_SUBJECT: &str = "Verify new user";
pub static VERIFY_NEW_USER_CATEGORY: &str = "verify_new_user";
pub static WELCOME_SUBJECT: &str = "Welcome to JV";
pub static WELCOME_CATEGORY: &str = "welcome";

lazy_static! {
    pub static ref COLORS: HashMap<&'static str, &'static str> = [
        ("black", "#252422"),
        ("brown", "#ff9b42"),
        ("vanilla", "#e3e7af"),
        ("blue", "#6a8eae"),
        ("sugar", "#be9e46"),
    ]
    .iter()
    .copied()
    .collect();
}

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                error!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".html", ".sql"]);
        tera
    };
}
