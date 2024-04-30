use cloudevents::binding::http::builder::adapter::to_response;
use cloudevents::binding::http::to_event;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use hyper::{Body, Method, Request, Response, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::result::Result;

mod handler;

#[allow(clippy::redundant_closure)]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 9000));
    let make_svc = make_service_fn(|_| async move {
        Ok::<_, Infallible>(service_fn(move |req| handle_request(req)))
    });
    let server = Server::bind(&addr).serve(make_svc);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
    Ok(())
}
async fn handle_request(req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/") => {
            let headers = req.headers().clone();
            let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
            let body = body_bytes.to_vec();
            let reqevt = to_event(&headers, body)?;
            let _respevt = handler::handle_event(reqevt).await?;

            to_response(_respevt).map_err(|err| err.into())
        }
        (&Method::GET, "/health/readiness") => Ok(Response::new(Body::from(""))),
        (&Method::GET, "/health/liveness") => Ok(Response::new(Body::from(""))),
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}
