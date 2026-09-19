use rocket::http::Header;
use rocket::request::Request;
use rocket::response::{self, Responder};

/// Wraps a Responder and adds a `Cache-Control: no-cache` header,
/// so browsers always revalidate served content.
pub struct NoCache<R>(pub R);

impl<'r, 'o: 'r, R: Responder<'r, 'o>> Responder<'r, 'o> for NoCache<R> {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'o> {
        let mut response = self.0.respond_to(request)?;
        response.set_header(Header::new("Cache-Control", "no-cache"));
        Ok(response)
    }
}
