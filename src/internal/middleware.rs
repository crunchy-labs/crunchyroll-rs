use crate::Error;
use crate::error::ErrorKind;
use crate::middleware::MiddlewareContext;
use futures_util::TryFutureExt;
use reqwest::Response;
use std::error::Error as StdError;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};
use tower_service::Service;

pub(crate) type MiddlewareType = Box<
    dyn for<'a> Service<
            MiddlewareContext<'a>,
            Response = Response,
            Error = Box<dyn StdError + Send + Sync + 'static>,
            Future = Pin<
                Box<
                    dyn Future<Output = Result<Response, Box<dyn StdError + Send + Sync + 'static>>>
                        + Send,
                >,
            >,
        > + Send,
>;

pub(crate) struct Middleware(MiddlewareType);

impl Middleware {
    pub(crate) fn new<E, F, S>(service: S) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync + 'static>> + 'static,
        F: Future<Output = Result<Response, E>> + Send + 'static,
        S: for<'a> Service<MiddlewareContext<'a>, Response = Response, Error = E, Future = F>
            + Send
            + 'static,
    {
        Self(Box::new(ServiceDynProxy { inner: service }))
    }
}

impl Debug for Middleware {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<middleware>")
    }
}

impl Deref for Middleware {
    type Target = MiddlewareType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Middleware {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

struct ServiceDynProxy<E, F, S>
where
    E: Into<Box<dyn std::error::Error + Send + Sync + 'static>> + 'static,
    F: Future<Output = Result<Response, E>> + Send + 'static,
    S: for<'a> Service<MiddlewareContext<'a>, Response = Response, Error = E, Future = F>
        + Send
        + 'static,
{
    inner: S,
}

impl<E, F, S> Service<MiddlewareContext<'_>> for ServiceDynProxy<E, F, S>
where
    E: Into<Box<dyn std::error::Error + Send + Sync + 'static>> + 'static,
    F: Future<Output = Result<Response, E>> + Send + 'static,
    S: for<'a> Service<MiddlewareContext<'a>, Response = Response, Error = E, Future = F>
        + Send
        + 'static,
{
    type Response = Response;
    type Error = Box<dyn StdError + Send + Sync + 'static>;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: MiddlewareContext<'_>) -> Self::Future {
        Box::pin(self.inner.call(req).map_err(Into::into))
    }
}

pub(crate) fn middleware_error_to_error(
    err: Box<dyn StdError + Send + Sync + 'static>,
    url: String,
) -> Error {
    Error::error_from_other_error_and_url(err, ErrorKind::Request { status: None }, url)
}
