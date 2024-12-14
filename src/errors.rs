use argon2::password_hash;
use reqwest;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::response::{self, Responder, Response};
use rocket::Request;
use rocket_db_pools::sqlx;
use std::fmt;
use std::io;
use tera;

pub struct AppError {
    pub code: u16,
    pub message: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err_msg = match self.code {
            404 => "Sorry, Can not find the Page!",
            _ => "Sorry, something is wrong! Please Try Again!",
        };

        write!(f, "{}", err_msg)
    }
}

impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AppError {{ code: {}, message: {} }}",
            self.code, self.message
        )
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        AppError {
            code: 500,
            message: error.to_string(),
        }
    }
}

impl From<password_hash::Error> for AppError {
    fn from(error: password_hash::Error) -> Self {
        AppError {
            code: 500,
            message: error.to_string(),
        }
    }
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        AppError {
            code: 500,
            message: error.to_string(),
        }
    }
}

impl From<tera::Error> for AppError {
    fn from(error: tera::Error) -> Self {
        AppError {
            code: 500,
            message: error.to_string(),
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> Self {
        AppError {
            code: 500,
            message: error.to_string(),
        }
    }
}

impl From<String> for AppError {
    fn from(error: String) -> Self {
        AppError {
            code: 500,
            message: error,
        }
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .status(Status::from_code(self.code).unwrap())
            .header(ContentType::Plain)
            .sized_body(self.message.len(), io::Cursor::new(self.message))
            .ok()
    }
}
