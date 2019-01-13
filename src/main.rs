extern crate hyper;
#[macro_use]
extern crate serde_json;

use hyper::{Body, Response, Server};
use hyper::Method;
use hyper::rt::Future;  // Used for map_err
use hyper::service::service_fn_ok;
use hyper::StatusCode;

fn main() {
    let addr = ([127, 0, 0, 1], 3000).into();

    let new_svc = || {
        service_fn_ok(|_req|{
            match(_req.method(), _req.uri().path()) {
                (&Method::GET, "/hello") => {
                    Response::builder()
                        .status(StatusCode::OK)
                        .header("X-HELLO", "world")
                        .body(Body::from(json!({
                            "message": "Hello, World!"}).to_string()))
                        .unwrap()
                },
                (_, _) => {
                    Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::from("Not found"))
                        .unwrap()
                }
            }
        })
    };

    let server = Server::bind(&addr)
        .serve(new_svc)
        .map_err(|e| eprintln!("server error: {}", e));

    hyper::rt::run(server);
}
