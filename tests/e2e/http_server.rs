use http::header::IntoHeaderName;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::rt::ReadBufCursor;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use pin_project_lite::pin_project;
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

#[derive(Debug)]
pub enum Error {
    ConnectionFailed(String),
    IO(String),
    InvalidRequest(String),
    InvalidRequestEndpoint(String),
    InvalidEndpoint(String),
    BadRequest(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "HTTP Server Error: {self:?}")
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IO(format!("{err}"))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

fn format_relative_url(url: &str) -> Option<String> {
    let dedup_url = Regex::new(r"/+")
        .unwrap()
        .replace_all(url, "/")
        .into_owned();
    Some(
        dedup_url
            .strip_suffix('/')
            .map(String::from)
            .unwrap_or(dedup_url),
    )
}

// ==============================

pub trait ServerContext {
    fn should_terminate_server(&self) -> bool;
}

type ServerEndpointHandle<T> = fn(
    HttpRequest,
    Arc<Mutex<T>>,
) -> Pin<
    Box<
        dyn Future<Output = HttpResponse> // future API / pollable
            + Send, // required by non-single-threaded executors
    >,
>;

pub struct Server<T: ServerContext> {
    addr: SocketAddr,

    handles: HashMap<String, ServerEndpointHandle<T>>,
    else_handle: fn(HttpRequest) -> HttpResponse,
    pub context: Arc<Mutex<T>>,
}

#[derive(Clone)]
pub struct HttpRequest {
    /// An absolute or relative URL describing the requested target (endpoint).
    /// Most of the time, this is in relative form (also called origin form), e.g. /api/success
    pub endpoint: http::Uri,

    /// The actual content of the request
    pub body: Bytes,

    /// The headers of the request
    pub headers: http::HeaderMap,
}

#[derive(Clone)]
pub struct HttpResponse {
    pub response: http::Response<Vec<u8>>,
}

impl HttpResponse {
    fn new() -> Self {
        Self {
            response: Response::new(Vec::new()),
        }
    }
    pub fn from_status(status: http::StatusCode) -> Self {
        let mut s = Self::new();
        *s.response.status_mut() = status;
        s
    }
    pub fn not_found() -> Self {
        Self::from_status(http::StatusCode::NOT_FOUND)
    }
    pub fn ok() -> Self {
        Self::from_status(http::StatusCode::OK)
    }
    pub fn bad_request() -> Self {
        Self::from_status(http::StatusCode::BAD_REQUEST)
    }
    pub fn internal_server_error() -> Self {
        Self::from_status(http::StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub fn insert_header<T: IntoHeaderName>(mut self, key: T, value: &str) -> Self {
        self.response
            .headers_mut()
            .insert(key, http::header::HeaderValue::from_str(value).unwrap());
        self
    }
    pub fn content_type(self, content_type: &str) -> Self {
        self.insert_header(http::header::CONTENT_TYPE, content_type)
    }

    pub fn text(mut self, body: &str) -> Self {
        *self.response.body_mut() = body.to_owned().into_bytes();
        self
    }
    pub fn json<T: Serialize>(mut self, content: &T) -> Self {
        *self.response.body_mut() = serde_json::to_string(content).unwrap().into_bytes(); // this should not panic
        self.response.headers_mut().insert(
            http::header::CONTENT_TYPE,
            http::header::HeaderValue::from_static("application/json"),
        );
        self
    }
    pub fn body(mut self, body: Bytes) -> Self {
        *self.response.body_mut() = body.to_vec();
        self
    }
}

pin_project! {
    struct ServerIO {
        #[pin]
        tcp: TcpStream,
    }
}

impl ServerIO {
    pub fn new(tcp_stream: TcpStream) -> Self {
        Self { tcp: tcp_stream }
    }
}

impl hyper::rt::Read for ServerIO {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: ReadBufCursor<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        let buf_len = unsafe {
            let mut tokio_buf = tokio::io::ReadBuf::uninit(buf.as_mut());
            match tokio::io::AsyncRead::poll_read(self.project().tcp, cx, &mut tokio_buf) {
                Poll::Ready(Ok(())) => tokio_buf.filled().len(),
                other => return other,
            }
        };
        unsafe {
            buf.advance(buf_len);
        }
        Poll::Ready(Ok(()))
    }
}
impl hyper::rt::Write for ServerIO {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        tokio::io::AsyncWrite::poll_write(self.project().tcp, cx, buf)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        tokio::io::AsyncWrite::poll_flush(self.project().tcp, cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        tokio::io::AsyncWrite::poll_shutdown(self.project().tcp, cx)
    }
}

impl<T: ServerContext + std::marker::Send + Clone + 'static> Server<T> {
    pub fn new(addr: SocketAddr, context: T) -> Self {
        Self {
            addr,
            handles: HashMap::new(),
            else_handle: |_| HttpResponse::not_found(),
            context: Arc::new(Mutex::new(context)),
        }
    }

    /// Set handle for a specific endpoint.
    pub fn with_endpoint(
        mut self,
        target: &str,
        callback: ServerEndpointHandle<T>,
    ) -> Result<Self> {
        self.handles.insert(
            format_relative_url(target).ok_or(Error::InvalidEndpoint(target.to_owned()))?,
            callback,
        );
        Ok(self)
    }

    /// Set handle which is called when the endpoint is not recognized.
    pub fn with_else_handle(mut self, handle: fn(HttpRequest) -> HttpResponse) -> Self {
        self.else_handle = handle;
        self
    }

    async fn handle_request(
        request: Request<hyper::body::Incoming>,
        handles: HashMap<String, ServerEndpointHandle<T>>,
        else_handle: fn(HttpRequest) -> HttpResponse,
        context: Arc<Mutex<T>>,
    ) -> hyper::Result<Response<Full<Bytes>>> {
        let uri = request.uri().clone();
        let headers = request.headers().clone();
        let whole_body = request.into_body().collect().await?.to_bytes();
        let internal_request = HttpRequest {
            endpoint: uri.clone(),
            headers,
            body: whole_body,
        };

        Ok(
            // NOTE this should not panic as it is directly from a valid Uri
            match handles.get(&format_relative_url(uri.path()).unwrap()) {
                Some(handle) => handle(internal_request.clone(), context).await,
                None => (else_handle)(internal_request),
            }
            .response
            .map(http_body_util::Full::from),
        )
    }

    /// Starts the server, consuming it. It runs until `context.should_terminate_server()` returns `true`.
    /// The context is returned on shutdown.
    pub async fn run(self) -> Result<T> {
        let listener = TcpListener::bind(self.addr).await?;

        loop {
            let (tcp_stream, _) = match listener.accept().await {
                Ok(v) => v,
                Err(err) => {
                    return Err::<T, Error>(Error::ConnectionFailed(format!("{err}")));
                }
            };

            let io = ServerIO::new(tcp_stream);
            let else_handle_single = self.else_handle;

            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(|request| {
                        let handles = self.handles.clone();
                        let context = self.context.clone();
                        async move {
                            Self::handle_request(request, handles, else_handle_single, context)
                                .await
                        }
                    }),
                )
                .await
            {
                return Err(Error::ConnectionFailed(format!("{err}")));
            }

            let context_unlocked = self.context.lock().await;
            if context_unlocked.should_terminate_server() {
                return Ok(context_unlocked.clone());
            }
        }
    }
}
