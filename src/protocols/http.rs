use crate::error::Result;
use hyper::{Request,Response,Body};
pub async fn handle_http_request(_req:Request<Body>) -> Result<()>{
    //ToDo implement HTTP request handling logic
    Ok((Response::new(Body::from("Response from HTTP handler"))));
}