use std::net::SocketAddr;

use axum::{
    extract::{Path, State},
    http::{uri::Uri, Request, Response},
    routing::{any, get},
    Router,
};
use hyper::{client::HttpConnector, Body};

type Client = hyper::client::Client<HttpConnector, Body>;

#[tokio::main]
async fn main() {
    let client = Client::new();

    let app = Router::with_state(client)
        .route("/healthz", get(healthz))
        .route("/api/:service/*path", any(handler));

    let addr = SocketAddr::from(([0,0,0,0], 80));
    println!("reverse proxy listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn healthz() {}

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

    println!("path_query: {}", path_query);

    let uri = format!("http://{}:{}{}", service, port, path_query);

    *req.uri_mut() = Uri::try_from(uri).unwrap();

    client.request(req).await.unwrap()
}
