use std::{
    convert::Infallible,
    net::SocketAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use futures::{Future, StreamExt, TryStreamExt};
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server, StatusCode,
};
use tokio::fs::File;
use tokio_util::compat::FuturesAsyncReadCompatExt;

struct Context {
    next_id: AtomicUsize,
}

struct Error {
    status: StatusCode,
    message: String,
}

impl Error {
    fn new(status: StatusCode, message: impl Into<String>) -> Error {
        Error {
            status,
            message: message.into(),
        }
    }

    fn into_response(self) -> Response<Body> {
        let mut response = Response::new(self.message.into());
        *response.status_mut() = self.status;
        response
    }
}

impl From<tokio::io::Error> for Error {
    fn from(src: tokio::io::Error) -> Self {
        Error::new(StatusCode::INTERNAL_SERVER_ERROR, src.to_string())
    }
}

fn response<E>(code: StatusCode, body: impl Into<Body>) -> Result<Response<Body>, E> {
    let mut response = Response::new(body.into());
    *response.status_mut() = code;
    Ok(response)
}

fn instant<T>(v: T) -> impl Future<Output = T> {
    async { v }
}

fn ok<T>(v: T) -> Result<T, Infallible> {
    Ok(v)
}

async fn hello_world(
    context: Arc<Context>,
    request: Request<Body>,
) -> Result<Response<Body>, Error> {
    if request.method() != Method::POST {
        return response(StatusCode::BAD_REQUEST, "invalid method");
    }

    let body = request
        .into_body()
        .map(|result| result.map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Error!")))
        .into_async_read();

    let mut body = FuturesAsyncReadCompatExt::compat(body);

    let id = context.next_id.fetch_add(1, Ordering::Relaxed);
    let path = format!("data/{id}");

    let mut file = File::create(path).await?;

    tokio::io::copy(&mut body, &mut file).await?;

    response(StatusCode::OK, id.to_string())
}

#[tokio::main]
async fn main() {
    let port = std::env::args()
        .nth(1)
        .expect("missing port")
        .parse()
        .expect("invalid port");
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let context = Arc::new(Context {
        next_id: Default::default(),
    });

    let make_svc = make_service_fn(move |_conn| {
        let context = context.clone();

        instant(ok(service_fn(move |request| {
            let context = context.clone();

            async move {
                match hello_world(context.clone(), request).await {
                    Ok(v) => ok(v),
                    Err(e) => ok(e.into_response()),
                }
            }
        })))
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
