use std::net::SocketAddr;

use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::{Body, Client, Method, Request, Response, StatusCode};
use tokio::net::TcpListener;

use url::form_urlencoded;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    let https = wasmedge_hyper_rustls::connector::new_https_connector(
        wasmedge_rustls_api::ClientConfig::default(),
    );
    
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);
    loop {
        let (stream, _) = listener.accept().await?;
        
        tokio::task::spawn(
            async move {
            
            if let Err(err) = Http::new().serve_connection(stream, service_fn(
                move |req| request_handler(req)
            )).await {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

/// This is our service handler. It receives a Request, routes on its
/// path, and returns a Future of a Response.
async fn request_handler(req: Request<Body>) -> Result<Response<Body>, reqwest::Error> {
    
    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::GET, "/") => Ok(Response::new(Body::from(
            "Try POSTing data to /test such as: `curl localhost:8080/test -XPOST -d 'hello world'`",
        ))),

        // test endpoint.
        (&Method::POST, "/test") => {
            let (parts, body) = req.into_parts();
            let body_bytes = hyper::body::to_bytes(body).await.unwrap();
            let encoded: String = form_urlencoded::byte_serialize(&body_bytes).collect();
            println!("encoded: {}", encoded);
            let url = format!("https://httpbin.org/get?msg={}", encoded);

            eprintln!("Fetching {:?}...", url);
        
            let res = reqwest::get(url).await?;
        
            eprintln!("Response: {:?} {}", res.version(), res.status());
            eprintln!("Headers: {:#?}\n", res.headers());
        
            let body = res.text().await?;
            println!("GET: {}", body);
     
            let mut resp = Response::new(body);
            *resp.status_mut() = StatusCode::OK;
            return Ok(resp);   
        },

        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}
