use crate::error::Result;
use hyper::{Request,Response,Body};
pub async fn handle_ws_connection(_req:Request<Body>) -> Result<()>{
    //ToDo implement ws protocol logic
    Ok((Response::new(Body::from("Response from WS handler"))));
}