//! The Iron web framework request handler and helper function

// System modules:
use std::sync::Mutex;

// External modules:
use iron::prelude::{Request, IronResult, Response};
use iron::headers::{Headers, ContentType};
use iron::mime::{Mime, TopLevel, SubLevel};
use iron::status;

// Internal modules:
use slurm_status::{status_to_html, SlurmStatus};

/// Accepts a HTML string and returns a IronResult response with correct mime type
fn string_to_response(page: &str) -> IronResult<Response> {
    let mut res = Response::new();

    res.status = Some(status::Ok);
    res.headers = Headers::new();
    res.headers.set(ContentType(Mime(TopLevel::Text, SubLevel::Html, vec![])));
    res.body = Some(Box::new(page.to_string()));

    Ok(res)
}

/// Handles Iron requests and shows the slurm status as a HTML web page
/// TODO: better error handling
pub fn handle_request(req: &mut Request, shared_slurm_status: &Mutex<SlurmStatus>) -> IronResult<Response> {
    debug!("req: {:?}", req);

    match shared_slurm_status.lock() {
        Ok(status) => {
            string_to_response(&status_to_html(&status))
        },
        Err(err) => {
            error!("Could not lock Mutex: {}", err);
            string_to_response("<h1>Could not lock Mutex!</h1>")
        }
    }
}
