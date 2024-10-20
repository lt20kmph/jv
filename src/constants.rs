use lazy_static::lazy_static;
use std::collections::HashMap;
use tera::Tera;
use log::error;

pub static SESSION_LENGTH: i64 = 60 * 60 * 24 * 7; // 1 week
pub static MAX_FILE_SIZE: u64 = 1024 * 1024 * 10; // 10 MB

pub static IMG_PATH: &str = "./img";

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
