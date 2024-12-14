use crate::models::models::Session;
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
