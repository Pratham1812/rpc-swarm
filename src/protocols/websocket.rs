use crate::error::Result;
use hyper::{Request, Response};
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
pub async fn handle_ws_connection(_req: Request<Incoming>) -> Result<Response<Full<Bytes>>> {
    // Create a new body using the `Full` and `Bytes` types.
    let response_body = Full::new(Bytes::from("Response from HTTP handler"));
    Ok(Response::new(response_body))
}