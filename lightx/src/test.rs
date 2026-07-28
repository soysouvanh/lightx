use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::{Method, Request, Response};
use std::convert::Infallible;
use tower::{Service, ServiceExt};

/// SuperTest Builder API equivalent for LightX.
/// Allows simulating full HTTP transactions in O(1) mathematically without ever opening a TCP Socket.
/// Bypasses `Incoming` stream restrictions by leveraging `oneshot` testing logic natively.
pub struct SuperTest<S> {
    app: S,
    req: hyper::http::request::Builder,
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
