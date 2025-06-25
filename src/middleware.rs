use crate::models::models::{Session, Role};
use crate::queries;
use crate::queries::Db;
use log::debug;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Session {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Session, ()> {
        debug!("Checking for session token");

        let session_token: Option<String> = request
            .cookies()
            .get_private("s_id")
            .and_then(|cookie| cookie.value().parse().ok());

        debug!("Session token: {:?}", session_token);

        let session_token = match session_token {
            Some(session_token) => session_token,
            None => return Outcome::Error((Status::Unauthorized, ())),
        };

        let pool = request.rocket().state::<Db>();

        let pool = match pool {
            Some(pool) => pool,
            None => return Outcome::Error((Status::InternalServerError, ())),
        };

        debug!("Getting user id from session token");

        let user = queries::get_user_from_session_token(&session_token, pool).await;

        let user = match user {
            Ok(user) => user,
            Err(_) => return Outcome::Error((Status::Unauthorized, ())),
        };

        debug!("Getting session for user: (user_id: {:?})", user);

        let session = Session {
            session_token,
            user,
        };

        debug!("Session: {:?}", session);

        Outcome::Success(session)
    }
}

/// Request guard that ensures the user has Writer role authorization
/// 
/// This guard combines authentication (via Session) with authorization,
/// ensuring that only users with Writer role can access protected routes.
/// 
/// Returns:
/// - Success: WriterSession containing the authenticated session
/// - Forbidden (403): User is authenticated but lacks Writer role  
/// - Unauthorized (401): User is not authenticated
pub struct WriterSession {
    pub session: Session,
}

impl WriterSession {
    /// Access the underlying session
    pub fn session(&self) -> &Session {
        &self.session
    }
    
    /// Access the user from the session
    pub fn user(&self) -> &crate::models::models::User {
        &self.session.user
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for WriterSession {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<WriterSession, ()> {
        debug!("Checking WriterSession authorization");
        
        // First, ensure user is authenticated by getting their session
        let session = match Session::from_request(request).await {
            Outcome::Success(session) => session,
            Outcome::Error(e) => {
                debug!("Authentication failed: {:?}", e);
                return Outcome::Error(e);
            }
            Outcome::Forward(status) => {
                debug!("Authentication forwarded");
                return Outcome::Forward(status);
            }
        };
        
        // Then check if user has Writer role authorization
        match session.user.role {
            Role::Writer => {
                debug!("Authorization successful for Writer user: {}", session.user.email);
                Outcome::Success(WriterSession { session })
            }
            Role::Reader => {
                debug!("Authorization denied for Reader user: {}", session.user.email);
                Outcome::Error((Status::Forbidden, ()))
            }
        }
    }
}
