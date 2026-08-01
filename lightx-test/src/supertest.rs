use lightx::ext::bytes::Bytes;
use lightx::ext::http_body_util::combinators::BoxBody;
use lightx::ext::hyper::{Method, Request, Response};
use lightx::ext::tower::{Service, ServiceExt};
use std::convert::Infallible;

/// SuperTest Builder API equivalent for LightX.
/// Allows simulating full HTTP transactions in O(1) mathematically without ever opening a TCP Socket.
/// Bypasses `Incoming` stream restrictions by leveraging `oneshot` testing logic natively.
pub struct SuperTest<S> {
    app: S,
    req: lightx::ext::hyper::http::request::Builder,
    body: String,
}

impl<S> SuperTest<S>
where
    S: Service<
            Request<String>,
            Response = Response<BoxBody<Bytes, Box<dyn std::error::Error + Send + Sync + 'static>>>,
            Error = Infallible,
        > + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    pub fn new(app: S) -> Self {
        Self {
            app,
            req: Request::builder(),
            body: String::new(),
        }
    }

    pub fn get(app: S, uri: &str) -> Self {
        Self::new(app).uri(uri).method(Method::GET)
    }

    pub fn post(app: S, uri: &str) -> Self {
        Self::new(app).uri(uri).method(Method::POST)
    }

    pub fn put(app: S, uri: &str) -> Self {
        Self::new(app).uri(uri).method(Method::PUT)
    }

    pub fn delete(app: S, uri: &str) -> Self {
        Self::new(app).uri(uri).method(Method::DELETE)
    }

    pub fn uri(mut self, uri: &str) -> Self {
        self.req = self.req.uri(uri);
        self
    }

    pub fn method(mut self, method: Method) -> Self {
        self.req = self.req.method(method);
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.req = self.req.header(key, value);
        self
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = body.into();
        self
    }

    pub fn json(mut self, value: &serde_json::Value) -> Self {
        self.req = self.req.header("Content-Type", "application/json");
        self.body = serde_json::to_string(value).unwrap();
        self
    }

    pub async fn send(
        self,
    ) -> Response<BoxBody<Bytes, Box<dyn std::error::Error + Send + Sync + 'static>>> {
        let request = self
            .req
            .body(self.body)
            .expect("Failed to build virtual Request");

        // Formally usurps physical execution using tower oneshot!
        let mut router = self.app.clone();
        router.ready().await.unwrap().call(request).await.unwrap()
    }
}

// SUPERTEST MOCKING API natively implemented in the test framework
impl lightx::ext::tower::Service<lightx::ext::hyper::Request<String>> for crate::AppRouter {
    type Response = lightx::ext::hyper::Response<
        lightx::ext::http_body_util::combinators::BoxBody<
            lightx::ext::bytes::Bytes,
            Box<dyn std::error::Error + Send + Sync + 'static>,
        >,
    >;
    type Error = std::convert::Infallible;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>,
    >;
    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
    fn call(&mut self, req: lightx::ext::hyper::Request<String>) -> Self::Future {
        use lightx::server::{ContextFactory, Router};
        let factory = self.factory.clone();
        let router = std::sync::Arc::new(self.clone());
        Box::pin(async move {
            let (parts, body) = req.into_parts();
            let raw_bytes = lightx::ext::bytes::Bytes::from(body);
            let peer_ip = "127.0.0.1".parse().unwrap();
            let mut ctx = factory.create_context(peer_ip, parts.headers.clone(), raw_bytes, None);
            let response = match router
                .route(parts.method.as_str(), parts.uri.path(), &mut ctx)
                .await
            {
                Ok(resp) => {
                    let _ = factory.commit_context(&mut ctx).await;
                    resp
                }
                Err(e) => lightx::ext::hyper::Response::builder()
                    .status(500)
                    .body(lightx::ext::http_body_util::BodyExt::boxed(
                        lightx::ext::http_body_util::BodyExt::map_err(
                            lightx::ext::http_body_util::Full::new(
                                lightx::ext::bytes::Bytes::from(format!(
                                    r#"{{"error":"Mock Route Error: {}"}}"#,
                                    e
                                )),
                            ),
                            |ei| Box::new(ei) as Box<dyn std::error::Error + Send + Sync + 'static>,
                        ),
                    ))
                    .unwrap(),
            };
            Ok(response)
        })
    }
}
