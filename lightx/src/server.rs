use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use bytes::Bytes;
use http_body_util::{BodyExt, Full, Limited};
use hyper::body::Incoming;
use hyper::server::conn::http1;

use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;

use std::fs::File;
use std::io::BufReader;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::rustls::{ServerConfig, pki_types::CertificateDer, pki_types::PrivateKeyDer};

use crate::core::AppError;

/// Maximum allowed body size in bytes (80 KiB).
/// Any request body exceeding this limit triggers an immediate `HTTP 413 Payload Too Large`.
const MAX_BODY_SIZE: usize = 81_920;

/// Maximum time allowed for a single HTTP connection before forceful termination (anti-Slowloris).
const CONNECTION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// The `Router` trait allows the generated application to inject its `O(1)` static router
/// into the Hyper server in an agnostic and type-safe manner.
///
/// # Examples
///
/// ```
/// use lightx::server::Router;
/// use lightx::core::AppError;
/// use std::future::Future;
/// use std::pin::Pin;
/// use bytes::Bytes;
/// use http_body_util::Full;
/// use hyper::Response;
///
/// struct MockRouter;
/// impl Router for MockRouter {
///     type Context = ();
///     fn route<'a>(
///         &'a self,
///         method: &'a str,
///         uri: &'a str,
///         ctx: &'a mut Self::Context,
///     ) -> Pin<Box<dyn Future<Output = Result<Response<Full<Bytes>>, AppError>> + Send + 'a>> {
///         Box::pin(async {
///             Ok(Response::new(Full::new(Bytes::from(r#"{"status":"mocked"}"#))))
///         })
///     }
/// }
/// ```
pub trait ContextFactory: Send + Sync + 'static {
    type Context: Send + 'static;

    fn create_context(
        &self,
        peer_addr: std::net::IpAddr,
        headers: hyper::HeaderMap,
        raw_body: Bytes,
        raw_req: Option<Request<Incoming>>,
    ) -> Self::Context;

    fn commit_context<'a>(
        &'a self,
        ctx: &'a mut Self::Context,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>>;
}

pub trait Router: Send + Sync + 'static {
    type Context: Send + 'static;
    #[allow(clippy::type_complexity)]
    fn route<'a>(
        &'a self,
        method: &'a str,
        uri: &'a str,
        ctx: &'a mut Self::Context,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Full<Bytes>>, AppError>> + Send + 'a>>;
}

async fn handle_request<C: Send + 'static>(
    mut req: Request<Incoming>,
    factory: Arc<dyn ContextFactory<Context = C>>,
    router: Arc<dyn Router<Context = C>>,
    peer_addr: std::net::IpAddr,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let is_ws = {
        let has_upgrade = req
            .headers()
            .get_all(hyper::header::UPGRADE)
            .iter()
            .any(|v| {
                if let Ok(s) = v.to_str() {
                    s.split(',')
                        .any(|part| part.trim().eq_ignore_ascii_case("websocket"))
                } else {
                    false
                }
            });
        let has_connection = req
            .headers()
            .get_all(hyper::header::CONNECTION)
            .iter()
            .any(|v| {
                if let Ok(s) = v.to_str() {
                    s.split(',')
                        .any(|part| part.trim().eq_ignore_ascii_case("upgrade"))
                } else {
                    false
                }
            });
        has_upgrade && has_connection
    };

    let mut client_ip = peer_addr;
    if let Some(xff) = req.headers().get("x-forwarded-for") {
        if let Ok(s) = xff.to_str()
            && let Some(first) = s.split(',').next()
            && let Ok(ip) = first.trim().parse()
        {
            client_ip = ip;
        }
    } else if let Some(xreal) = req.headers().get("x-real-ip")
        && let Ok(s) = xreal.to_str()
        && let Ok(ip) = s.trim().parse()
    {
        client_ip = ip;
    }

    let method;
    let uri;
    let mut ctx;

    if is_ws {
        // Refactored to ZER0-ALLOCATION: we extract headers structurally via mem::take.
        let headers = std::mem::take(req.headers_mut());
        method = req.method().clone();
        uri = req.uri().clone();
        ctx = factory.create_context(client_ip, headers, Bytes::new(), Some(req));
    } else {
        let (parts, body) = req.into_parts();

        // 1. Read the HTTP request body with strict size limit (anti-OOM / Buffer Overflow)
        let body_bytes = match Limited::new(body, MAX_BODY_SIZE).collect().await {
            Ok(collected) => collected.to_bytes(),
            Err(e) => {
                // Distinguish between a size limit error and a generic body read error
                let err_str = format!("{e}");
                if err_str.contains("length limit") {
                    return Ok(payload_too_large());
                }
                return Ok(bad_request(
                    "{\"code\":400,\"error\":\"Invalid body payload\"}",
                ));
            }
        };

        method = parts.method;
        uri = parts.uri;
        let headers = parts.headers;
        ctx = factory.create_context(client_ip, headers, body_bytes, None);
    }

    // 4. Delegate to the statically generated O(1) router
    let response = match router.route(method.as_str(), uri.path(), &mut ctx).await {
        Ok(mut raw_resp) => {
            // 5. Transaction Management (COMMIT via Factory)
            if let Err(e) = factory.commit_context(&mut ctx).await {
                crate::logger::error(format!("[COMMIT ERROR]: {}", e));
                return Ok(internal_error(
                    "{\"code\":500,\"error\":\"Internal Server Error\"}",
                ));
            }

            // Re-inject core security headers unconditionally
            let headers = raw_resp.headers_mut();
            if !headers.contains_key("Content-Type") {
                headers.insert(
                    "Content-Type",
                    hyper::header::HeaderValue::from_static("application/json"),
                );
            }
            headers.insert(
                "Strict-Transport-Security",
                hyper::header::HeaderValue::from_static("max-age=63072000"),
            );
            headers.insert(
                "X-Content-Type-Options",
                hyper::header::HeaderValue::from_static("nosniff"),
            );
            headers.insert(
                "X-Frame-Options",
                hyper::header::HeaderValue::from_static("DENY"),
            );

            raw_resp
        }
        Err(e) => {
            // ROLLBACK: The transaction will be automatically rolled back when dropped
            // if it wasn't committed.
            match e {
                AppError::ParameterError { field, msg, .. } => {
                    let json = serde_json::json!({
                        "code": 400,
                        "field": field,
                        "error": msg
                    })
                    .to_string();
                    bad_request(&json)
                }
                AppError::BusinessError { field, msg, .. } => {
                    let json = if field == "global" {
                        serde_json::json!({"code": 422, "error": msg})
                    } else {
                        serde_json::json!({"code": 422, "field": field, "error": msg})
                    }
                    .to_string();
                    build_response(StatusCode::UNPROCESSABLE_ENTITY, &json)
                }
                AppError::AuthenticationError { msg, .. } => {
                    let json = serde_json::json!({"code": 401, "error": msg}).to_string();
                    build_response(StatusCode::UNAUTHORIZED, &json)
                }
                AppError::PermissionError { msg, .. } => {
                    let json = serde_json::json!({"code": 403, "error": msg}).to_string();
                    build_response(StatusCode::FORBIDDEN, &json)
                }
                AppError::DatabaseError { msg, .. } | AppError::SystemError { msg, .. } => {
                    // CRITICAL: Never leak internal/database error details to the client
                    crate::logger::error(msg);
                    internal_error("{\"code\":500,\"error\":\"Internal Server Error\"}")
                }
                AppError::TooManyRequests => build_response(
                    StatusCode::TOO_MANY_REQUESTS,
                    "{\"code\":429,\"error\":\"Too Many Requests\"}",
                ),
                AppError::RouteNotFound => not_found(),
            }
        }
    };

    Ok(response)
}

/// Builds the formal `tower::Service` handling the LightX pipeline natively.
/// This prevents any per-request Heap allocations (`Box<dyn Future>`) thanks to
/// `tower::service_fn` inferring an opaque `impl Future` seamlessly.
///
/// # Examples
///
/// ```no_run
/// use lightx::server::{build_tower_service, Router, ContextFactory};
/// use lightx::core::AppError;
/// use std::sync::Arc;
/// use std::future::Future;
/// use std::pin::Pin;
/// use bytes::Bytes;
/// use http_body_util::Full;
/// use hyper::Response;
///
/// struct DummyRouter;
/// impl Router for DummyRouter {
///     type Context = ();
///     fn route<'a>(&'a self, method: &'a str, uri: &'a str, ctx: &'a mut Self::Context) -> Pin<Box<dyn Future<Output = Result<Response<Full<Bytes>>, AppError>> + Send + 'a>> {
///         Box::pin(async { Ok(Response::new(Full::new(Bytes::new()))) })
///     }
/// }
///
/// struct DummyFactory;
/// impl ContextFactory for DummyFactory {
///     type Context = ();
///     fn create_context(&self, _peer: std::net::IpAddr, _map: hyper::HeaderMap, _body: Bytes, _req: Option<hyper::Request<hyper::body::Incoming>>) -> () { () }
///     fn commit_context<'a>(&'a self, _ctx: &'a mut ()) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>> {
///         Box::pin(async { Ok(()) })
///     }
/// }
///
/// async fn init_server() {
///     let factory = Arc::new(DummyFactory);
///     let router = Arc::new(DummyRouter);
///     let ip: std::net::IpAddr = "127.0.0.1".parse().unwrap();
///     let service = build_tower_service(factory, router, ip);
/// }
/// ```
pub fn build_tower_service<C: Send + 'static>(
    factory: Arc<dyn ContextFactory<Context = C>>,
    router: Arc<dyn Router<Context = C>>,
    peer_addr: std::net::IpAddr,
) -> impl tower::Service<
    Request<Incoming>,
    Response = Response<Full<Bytes>>,
    Error = Infallible,
    Future = impl Future<Output = Result<Response<Full<Bytes>>, Infallible>> + Send,
> + Clone {
    let svc = tower::service_fn(move |req: Request<Incoming>| {
        let factory = factory.clone();
        let router = router.clone();
        // The async block returns the exact opaque Future type without Boxing
        async move { handle_request(req, factory, router, peer_addr).await }
    });

    tower::ServiceBuilder::new()
        // Seamless composability with tower-http ecosystem (CORS, TraceLayer, etc.) is now native.
        .service(svc)
}

/// Standard security headers injected into every HTTP response.
fn inject_security_headers(
    builder: hyper::http::response::Builder,
) -> hyper::http::response::Builder {
    builder
        .header("Content-Type", "application/json")
        .header("Strict-Transport-Security", "max-age=63072000")
        .header("X-Content-Type-Options", "nosniff")
        .header("X-Frame-Options", "DENY")
}

fn build_response(status: StatusCode, body: &str) -> Response<Full<Bytes>> {
    inject_security_headers(Response::builder().status(status))
        .body(Full::new(Bytes::from(body.to_owned())))
        .unwrap_or_else(|_| Response::new(Full::new(Bytes::from("{\"error\":\"Fatal\"}"))))
}

fn bad_request(body: &str) -> Response<Full<Bytes>> {
    build_response(StatusCode::BAD_REQUEST, body)
}

fn payload_too_large() -> Response<Full<Bytes>> {
    build_response(
        StatusCode::PAYLOAD_TOO_LARGE,
        "{\"code\":413,\"error\":\"Payload Too Large\"}",
    )
}

fn internal_error(body: &str) -> Response<Full<Bytes>> {
    build_response(StatusCode::INTERNAL_SERVER_ERROR, body)
}

fn not_found() -> Response<Full<Bytes>> {
    build_response(
        StatusCode::NOT_FOUND,
        "{\"code\":404,\"error\":\"Not Found\"}",
    )
}

/// Binds the `tower::Service` pipeline to an active `TcpListener` to serve traffic synchronously.
///
/// # Examples
///
/// ```no_run
/// use lightx::server::{listen, Router, ContextFactory};
/// use lightx::core::AppError;
/// use std::sync::Arc;
/// use std::future::Future;
/// use std::pin::Pin;
/// use bytes::Bytes;
/// use http_body_util::Full;
/// use hyper::Response;
///
/// struct DummyRouter;
/// impl Router for DummyRouter {
///     type Context = ();
///     fn route<'a>(&'a self, method: &'a str, uri: &'a str, ctx: &'a mut Self::Context) -> Pin<Box<dyn Future<Output = Result<Response<Full<Bytes>>, AppError>> + Send + 'a>> {
///         Box::pin(async { Ok(Response::new(Full::new(Bytes::new()))) })
///     }
/// }
///
/// struct DummyFactory;
/// impl ContextFactory for DummyFactory {
///     type Context = ();
///     fn create_context(&self, _peer: std::net::IpAddr, _map: hyper::HeaderMap, _body: Bytes, _req: Option<hyper::Request<hyper::body::Incoming>>) -> () { () }
///     fn commit_context<'a>(&'a self, _ctx: &'a mut ()) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>> {
///         Box::pin(async { Ok(()) })
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     // listen("127.0.0.1:8080".parse().unwrap(), Arc::new(DummyFactory), Arc::new(DummyRouter)).await.unwrap();
/// }
/// ```
pub async fn listen<C: Send + 'static>(
    addr: std::net::SocketAddr,
    factory: Arc<dyn ContextFactory<Context = C>>,
    router: Arc<dyn Router<Context = C>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    println!(" LightX Server listening on http://{}", addr);

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let tower_svc = build_tower_service(factory.clone(), router.clone(), peer_addr.ip());
        let hyper_svc = TowerToHyperService::new(tower_svc);

        tokio::task::spawn(async move {
            // Anti-Slowloris: enforce connection timeout
            let result = tokio::time::timeout(
                CONNECTION_TIMEOUT,
                http1::Builder::new()
                    .serve_connection(io, hyper_svc)
                    .with_upgrades(),
            )
            .await;
            match result {
                Ok(Err(err)) => eprintln!("Error serving connection: {:?}", err),
                Err(_elapsed) => { /* Connection killed silently after timeout */ }
                Ok(Ok(())) => {}
            }
        });
    }
}

/// Binds the `tower::Service` pipeline enabling HTTPS over TLS 1.3 exclusively.
///
/// # Examples
///
/// ```no_run
/// use lightx::server::{listen_tls, Router, ContextFactory};
/// use lightx::core::AppError;
/// use std::sync::Arc;
/// use std::future::Future;
/// use std::pin::Pin;
/// use bytes::Bytes;
/// use http_body_util::Full;
/// use hyper::Response;
///
/// struct DummyRouter;
/// impl Router for DummyRouter {
///     type Context = ();
///     fn route<'a>(&'a self, method: &'a str, uri: &'a str, ctx: &'a mut Self::Context) -> Pin<Box<dyn Future<Output = Result<Response<Full<Bytes>>, AppError>> + Send + 'a>> {
///         Box::pin(async { Ok(Response::new(Full::new(Bytes::new()))) })
///     }
/// }
///
/// struct DummyFactory;
/// impl ContextFactory for DummyFactory {
///     type Context = ();
///     fn create_context(&self, _peer: std::net::IpAddr, _map: hyper::HeaderMap, _body: Bytes, _req: Option<hyper::Request<hyper::body::Incoming>>) -> () { () }
///     fn commit_context<'a>(&'a self, _ctx: &'a mut ()) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>> {
///         Box::pin(async { Ok(()) })
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     // listen_tls(addr, Arc::new(DummyFactory), Arc::new(DummyRouter), "cert.pem", "key.pem").await.unwrap();
/// }
/// ```
pub async fn listen_tls<C: Send + 'static>(
    addr: std::net::SocketAddr,
    factory: Arc<dyn ContextFactory<Context = C>>,
    router: Arc<dyn Router<Context = C>>,
    cert_path: &str,
    key_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Read the public certificate (no .expect(), proper error propagation)
    let cert_file = File::open(cert_path).map_err(|e| {
        format!(
            "LightX Server: Certificate .pem not found at '{}': {}",
            cert_path, e
        )
    })?;
    let mut cert_reader = BufReader::new(cert_file);
    let certs = rustls_pemfile::certs(&mut cert_reader)
        .collect::<Result<Vec<CertificateDer<'static>>, _>>()?;

    // 2. Read the private key (no .expect(), proper error propagation)
    let key_file = File::open(key_path).map_err(|e| {
        format!(
            "LightX Server: Private key .key not found at '{}': {}",
            key_path, e
        )
    })?;
    let mut key_reader = BufReader::new(key_file);
    let mut keys: Vec<PrivateKeyDer<'static>> = rustls_pemfile::pkcs8_private_keys(&mut key_reader)
        .map(|key| key.map(PrivateKeyDer::Pkcs8))
        .collect::<Result<Vec<_>, _>>()?;

    if keys.is_empty() {
        return Err("LightX Server: No private key found in the key file".into());
    }
    let key = keys.remove(0);

    // 3. Configure Rustls — TLS 1.3 ONLY (military-grade security)
    let mut config = ServerConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    // ALPN: only advertise http/1.1 (h2 removed — we only serve HTTP/1.1)
    config.alpn_protocols = vec![b"http/1.1".to_vec()];

    let tls_acceptor = TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind(addr).await?;

    println!(" LightX TLS Server listening securely on https://{}", addr);

    loop {
        let (tcp_stream, peer_addr) = listener.accept().await?;
        let tls_acceptor = tls_acceptor.clone();
        let tower_svc = build_tower_service(factory.clone(), router.clone(), peer_addr.ip());
        let hyper_svc = TowerToHyperService::new(tower_svc);

        tokio::task::spawn(async move {
            let handshake_result =
                tokio::time::timeout(CONNECTION_TIMEOUT, tls_acceptor.accept(tcp_stream)).await;

            match handshake_result {
                Ok(Ok(tls_stream)) => {
                    let io = TokioIo::new(tls_stream);
                    // Anti-Slowloris: enforce connection timeout on HTTP Layer

                    let result = tokio::time::timeout(
                        CONNECTION_TIMEOUT,
                        http1::Builder::new()
                            .serve_connection(io, hyper_svc)
                            .with_upgrades(),
                    )
                    .await;
                    match result {
                        Ok(Err(err)) => eprintln!("Error serving TLS connection: {:?}", err),
                        Err(_elapsed) => { /* Connection killed silently after timeout */ }
                        Ok(Ok(())) => {}
                    }
                }
                Ok(Err(err)) => {
                    eprintln!("TLS Handshake failed: {:?}", err);
                }
                Err(_elapsed) => { /* Handshake killed silently after timeout */ }
            }
        });
    }
}
