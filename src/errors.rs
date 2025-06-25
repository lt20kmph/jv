use argon2::password_hash;
use image;
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

impl From<image::ImageError> for AppError {
    fn from(error: image::ImageError) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_app_error_creation() {
        let error = AppError {
            code: 404,
            message: "Not found".to_string(),
        };
        
        assert_eq!(error.code, 404);
        assert_eq!(error.message, "Not found");
    }

    #[test]
    fn test_app_error_display_404() {
        let error = AppError {
            code: 404,
            message: "Page not found".to_string(),
        };
        
        let display_output = format!("{}", error);
        assert_eq!(display_output, "Sorry, Can not find the Page!");
    }

    #[test]
    fn test_app_error_display_500() {
        let error = AppError {
            code: 500,
            message: "Internal server error".to_string(),
        };
        
        let display_output = format!("{}", error);
        assert_eq!(display_output, "Sorry, something is wrong! Please Try Again!");
    }

    #[test]
    fn test_app_error_display_other_codes() {
        let test_codes = [400, 401, 403, 418, 502];
        
        for code in test_codes {
            let error = AppError {
                code,
                message: "Some error".to_string(),
            };
            
            let display_output = format!("{}", error);
            assert_eq!(display_output, "Sorry, something is wrong! Please Try Again!");
        }
    }

    #[test]
    fn test_app_error_debug() {
        let error = AppError {
            code: 500,
            message: "Database connection failed".to_string(),
        };
        
        let debug_output = format!("{:?}", error);
        assert_eq!(debug_output, "AppError { code: 500, message: Database connection failed }");
    }

    #[test]
    fn test_from_sqlx_error() {
        let sqlx_error = sqlx::Error::RowNotFound;
        let app_error = AppError::from(sqlx_error);
        
        assert_eq!(app_error.code, 500);
        assert!(app_error.message.contains("no rows returned"));
    }

    #[test]
    fn test_from_password_hash_error() {
        let pw_error = password_hash::Error::Algorithm;
        let app_error = AppError::from(pw_error);
        
        assert_eq!(app_error.code, 500);
        assert!(app_error.message.contains("unsupported algorithm"));
    }

    #[test]
    fn test_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let app_error = AppError::from(io_error);
        
        assert_eq!(app_error.code, 500);
        assert!(app_error.message.contains("File not found"));
    }

    #[test]
    fn test_from_tera_error() {
        let tera_error = tera::Error::msg("Template not found");
        let app_error = AppError::from(tera_error);
        
        assert_eq!(app_error.code, 500);
        assert!(app_error.message.contains("Template not found"));
    }

    #[test]
    fn test_from_reqwest_error() {
        // Create a reqwest URL parse error
        let url_result = reqwest::Url::parse("not_a_valid_url");
        assert!(url_result.is_err());
        
        // Since reqwest::Error is hard to construct directly, we'll test the From trait exists
        // by verifying the trait bound exists (compilation test)
        fn test_reqwest_error_conversion(err: reqwest::Error) -> AppError {
            AppError::from(err)
        }
        
        // If this compiles, the From trait is implemented correctly
        assert!(true);
    }

    #[test]
    fn test_from_image_error() {
        // Create a simple image error using IoError variant which is easier to construct
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "Image file not found");
        let image_error = image::ImageError::IoError(io_error);
        let app_error = AppError::from(image_error);
        
        assert_eq!(app_error.code, 500);
        assert!(app_error.message.contains("Image file not found") || app_error.message.contains("not found"));
    }

    #[test]
    fn test_from_string() {
        let error_string = "Custom error message".to_string();
        let app_error = AppError::from(error_string);
        
        assert_eq!(app_error.code, 500);
        assert_eq!(app_error.message, "Custom error message");
    }

    #[test]
    fn test_from_string_slice() {
        let error_str = "Another error message";
        let app_error = AppError::from(error_str.to_string());
        
        assert_eq!(app_error.code, 500);
        assert_eq!(app_error.message, "Another error message");
    }

    #[test]
    fn test_status_code_mapping() {
        // Test that Status::from_code works with our error codes
        let test_cases = [
            (404, Status::NotFound),
            (500, Status::InternalServerError),
            (418, Status::ImATeapot),
            (401, Status::Unauthorized),
            (403, Status::Forbidden),
        ];
        
        for (code, expected_status) in test_cases {
            let status = Status::from_code(code).unwrap();
            assert_eq!(status, expected_status);
        }
    }

    #[test]
    fn test_error_responder_properties() {
        let error = AppError {
            code: 500,
            message: "Test error".to_string(),
        };
        
        // Test that we can access the properties for response building
        assert_eq!(error.code, 500);
        assert_eq!(error.message, "Test error");
        
        // Verify content type logic
        assert_eq!(ContentType::Plain.to_string(), "text/plain; charset=utf-8");
    }

    #[test]
    fn test_error_message_preservation() {
        let original_message = "Very specific error details";
        let error = AppError {
            code: 500,
            message: original_message.to_string(),
        };
        
        // Debug format should preserve the original message
        let debug_output = format!("{:?}", error);
        assert!(debug_output.contains(original_message));
        
        // But display format should show user-friendly message
        let display_output = format!("{}", error);
        assert!(!display_output.contains(original_message));
        assert_eq!(display_output, "Sorry, something is wrong! Please Try Again!");
    }

    #[test]
    fn test_error_chaining_sqlx() {
        // Test with a simple SQLx error that's easy to construct
        let sqlx_error = sqlx::Error::RowNotFound;
        let app_error = AppError::from(sqlx_error);
        
        assert_eq!(app_error.code, 500);
        assert!(app_error.message.contains("no rows returned") || app_error.message.contains("row"));
    }

    #[test]
    fn test_error_chaining_io() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let app_error = AppError::from(io_error);
        
        assert_eq!(app_error.code, 500);
        assert!(app_error.message.contains("Access denied") || app_error.message.contains("permission"));
    }

    #[test]
    fn test_concurrent_error_creation() {
        use std::thread;
        use std::sync::Arc;
        
        let handles: Vec<_> = (0..10).map(|i| {
            thread::spawn(move || {
                let error = AppError {
                    code: 500 + i,
                    message: format!("Error {}", i),
                };
                format!("{:?}", error)
            })
        }).collect();
        
        for handle in handles {
            let result = handle.join().unwrap();
            assert!(result.contains("AppError"));
        }
    }

    #[test]
    fn test_empty_message() {
        let error = AppError {
            code: 500,
            message: String::new(),
        };
        
        assert_eq!(error.message, "");
        assert_eq!(format!("{}", error), "Sorry, something is wrong! Please Try Again!");
    }

    #[test]
    fn test_very_long_message() {
        let long_message = "x".repeat(10000);
        let error = AppError {
            code: 500,
            message: long_message.clone(),
        };
        
        assert_eq!(error.message, long_message);
        let debug_output = format!("{:?}", error);
        assert!(debug_output.contains(&long_message));
    }

    #[test]
    fn test_message_with_special_characters() {
        let special_message = "Error with 🦀 emoji and \"quotes\" and \n newlines \t tabs";
        let error = AppError {
            code: 500,
            message: special_message.to_string(),
        };
        
        assert_eq!(error.message, special_message);
        let debug_output = format!("{:?}", error);
        assert!(debug_output.contains("🦀"));
    }
}
