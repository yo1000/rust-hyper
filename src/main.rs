#![deny(warnings)]
extern crate futures;
extern crate hyper;
extern crate pretty_env_logger;
#[macro_use]
extern crate serde_json;
extern crate url;

use std::collections::HashMap;

use futures::{future, Future, Stream};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::service_fn;
use serde_json::Value;
use url::form_urlencoded;

static INDEX: &[u8] = br#"
<html>
<body>
<form action="post" method="post">
    Name: <input type="text" name="name"><br>
    Number: <input type="text" name="number"><br>
    <input type="submit">
</form>
</body>
</html>
"#;

// Using service_fn, we can turn this function into a `Service`.
fn param_example(req: Request<Body>) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::GET, "/post") => {
            Box::new(future::ok(Response::new(INDEX.into())))
        },
        (&Method::POST, "/param_as_json") => {
            Box::new(req.into_body().concat2().map(|b| {
                let params_as_map = form_urlencoded::parse(b.as_ref())
                    .into_owned().into_iter()
                    .fold(
                        HashMap::<String, Vec<_>>::new(),
                        |mut acc: HashMap<String, Vec<_>>, pair: (String, String)| {
                            match acc.get_mut(pair.0.as_str()) {
                                Some(vec) => {
                                    vec.push(pair.1);
                                }
                                None => {
                                    acc.insert(pair.0, vec![pair.1]);
                                }
                            }
                            acc
                        }
                    );

                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json; charset=utf-8")
                    .body(Body::from(json!(params_as_map).to_string()))
                    .unwrap()
            }))
        },
        (&Method::PUT, "/json_as_json") => {
            Box::new(req.into_body().concat2().map(|b| {
                let json_str = String::from_utf8(b.as_ref().to_vec()).unwrap();
                let json_as_value: Value = serde_json::from_str(json_str.as_str()).unwrap();

                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json; charset=utf-8")
                    .body(Body::from(json!(json_as_value).to_string()))
                    .unwrap()
            }))
        },
        (&Method::GET, "/hello") => {
            Box::new(future::ok(Response::builder()
                .status(StatusCode::OK)
                .header("X-HELLO", "world")
                .body(Body::from(json!({
                    "message": "Hello, World!"}).to_string()))
                .unwrap()))
        },
        (&Method::GET, "/query_as_json") => {
            let query_as_map = match req.uri().query() {
                Some(it) => {
                    it.split('&')
                        .map(|q| q.split('=')
                            .collect::<Vec<_>>())
                        .filter(|q| q.len() >= 1)
                        .map(|q| match q.len() {
                            1 => { (q[0], "") }
                            _ => { (q[0], q[1]) }
                        })
                        .collect::<HashMap<_, _>>()
                }
                None => { HashMap::new() }
            };

            Box::new(future::ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json; charset=utf-8")
                .body(Body::from(json!(query_as_map).to_string()))
                .unwrap()))
        },
        _ => {
            Box::new(future::ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap()))
        }
    }
}

fn main() {
    pretty_env_logger::init();

    let addr = ([127, 0, 0, 1], 1337).into();

    let server = Server::bind(&addr)
        .serve(|| service_fn(param_example))
        .map_err(|e| eprintln!("server error: {}", e));

    hyper::rt::run(server);
}
