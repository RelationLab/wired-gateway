use std::net::SocketAddr;

use axum::{
    extract::{Path, State},
    http::{uri::Uri, HeaderValue, Request, Response},
    routing::get,
    Router,
};
use hyper::{client::HttpConnector, Body};

type Client = hyper::client::Client<HttpConnector, Body>;

#[tokio::main]
async fn main() {
    let client = Client::new();

    let app = Router::with_state(client)
        .route("/healthz", get(healthz))
        .route(
            "/api/:service/*path",
            get(handler).post(handler).options(option_handler),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 80));
    println!("reverse proxy listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn healthz() {}

async fn option_handler() -> Response<Body> {
    let mut builder = Response::builder().status(200);
    let mut headers_mut = builder.headers_mut().unwrap();
    headers_mut.insert("access-control-allow-origin", HeaderValue::from_static("*"));
    headers_mut.insert(
        "access-control-allow-credentials",
        HeaderValue::from_static("true"),
    );
    headers_mut.insert(
        "access-control-allow-methods",
        HeaderValue::from_static("PUT, GET, POST, DELETE, PATCH, OPTIONS"),
    );
    headers_mut.insert("access-control-allow-headers", HeaderValue::from_static("DNT,Keep-Alive,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range,Authorization,X-Api-Key,x-hasura-admin-secret,X-Sign"));
    builder.body(Body::empty()).unwrap()
}

async fn handler(
    State(client): State<Client>,
    Path((service, path)): Path<(String, String)>,
    mut req: Request<Body>,
) -> Response<Body> {
    let port = match service.as_str() {
        "hasura-cache" => 80,
        "ugc-gateway" => 10000,
        _ => return Response::builder().status(404).body(Body::empty()).unwrap(),
    };

    let path_query = req
        .uri()
        .query()
        .map(|query| format!("{}?{}", path, query))
        .unwrap_or(path);

    let uri = format!("http://{}:{}/{}", service, port, path_query);

    *req.uri_mut() = Uri::try_from(uri).unwrap();

    let mut response = client.request(req).await.unwrap();
    let headers_mut = response.headers_mut();
    headers_mut.insert("access-control-allow-origin", HeaderValue::from_static("*"));
    headers_mut.insert(
        "access-control-allow-credentials",
        HeaderValue::from_static("true"),
    );
    headers_mut.insert(
        "access-control-allow-methods",
        HeaderValue::from_static("PUT, GET, POST, DELETE, PATCH, OPTIONS"),
    );
    headers_mut.insert("access-control-allow-headers", HeaderValue::from_static("DNT,Keep-Alive,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range,Authorization,X-Api-Key,x-hasura-admin-secret,X-Sign"));
    response
}
