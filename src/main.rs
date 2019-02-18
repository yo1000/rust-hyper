#![deny(warnings)]
extern crate futures;
extern crate hyper;
extern crate pretty_env_logger;
extern crate serde;
#[macro_use]
extern crate serde_derive;
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
<h1>Hello, World!</h1>
<h2>Usage</h2>
<p>
<h3>GET: <code>/query_as_json</code></h3>
<pre><code>curl 'localhost:1337/query_as_json?a=XYZ&b=123&b=456&c'</code></pre>
<h3>POST: <code>/param_as_json</code></h3>
<pre><code>curl -X POST -d 'a=XYZ' -d 'b=123' -d 'b=456' -d 'c' 'localhost:1337/param_as_json'</code></pre>
<h3>PUT: <code>/json_as_json</code></h3>
<pre><code>curl -X PUT -d '{ "a": "XYZ", "b": [ 123, 456 ], "c": null }' 'localhost:1337/json_as_json'</code></pre>
</p>
</body>
</html>
"#;

#[derive(Serialize, Deserialize, Debug)]
struct  Point {
    x: i32,
    y: i32,
}

// Using service_fn, we can turn this function into a `Service`.
fn param_example(req: Request<Body>) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::GET, "/hello") => {
            Box::new(future::ok(Response::builder()
                .status(StatusCode::OK)
                .header("X-HELLO", "world")
                .body(Body::from(INDEX))
                .unwrap()))
        },
        (&Method::GET, "/query_as_json") => {
            let query_as_map = match req.uri().query() {
                Some(it) => {
                    form_urlencoded::parse(it.as_ref())
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
                        )
                }
                None => { HashMap::new() }
            };

            println!("{:?}", query_as_map);

            Box::new(future::ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json; charset=utf-8")
                .body(Body::from(json!(query_as_map).to_string()))
                .unwrap()))
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

                println!("{:?}", params_as_map);

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

                println!("{:?}", json_as_value);

                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json; charset=utf-8")
                    .body(Body::from(json!(json_as_value).to_string()))
                    .unwrap()
            }))
        },
        (&Method::PUT, "/json_as_point") => {
            Box::new(req.into_body().concat2().map(|b| {
                let json_str = String::from_utf8(b.as_ref().to_vec()).unwrap();
                let json_as_point: Point = serde_json::from_str(json_str.as_str()).unwrap();

                println!("{:?}", json_as_point);

                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json; charset=utf-8")
                    .body(Body::from(json!(json_as_point).to_string()))
                    .unwrap()
            }))
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
