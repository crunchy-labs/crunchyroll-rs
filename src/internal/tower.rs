use crate::error::Error;
use crate::middleware::MiddlewareContext;
use reqwest::Response;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};
use tower_service::Service;

pub(crate) type ServiceResponse = Response;
pub(crate) type ServiceFutureOutput = Result<Response, Error>;

pub(crate) type MiddlewareType = Box<
    dyn for<'a> Service<
            MiddlewareContext<'a>,
            Response = ServiceResponse,
            Error = Error,
            Future = Pin<Box<dyn Future<Output = ServiceFutureOutput> + Send>>,
        > + Send,
>;

pub(crate) struct Middleware(MiddlewareType);

struct ServiceDynProxy<F, S>
where
    F: Future<Output = ServiceFutureOutput> + Send + 'static,
    S: for<'a> Service<
            MiddlewareContext<'a>,
            Response = ServiceResponse,
            Error = Error,
            Future = F,
        > + Send
        + 'static,
{
    inner: S,
}

impl<F, S> Service<MiddlewareContext<'_>> for ServiceDynProxy<F, S>
where
    F: Future<Output = ServiceFutureOutput> + Send + 'static,
    S: for<'a> Service<
            MiddlewareContext<'a>,
            Response = ServiceResponse,
            Error = Error,
            Future = F,
        > + Send
        + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: MiddlewareContext<'_>) -> Self::Future {
        Box::pin(self.inner.call(req))
    }
}

impl Middleware {
    pub(crate) fn new<F, S>(service: S) -> Self
    where
        F: Future<Output = ServiceFutureOutput> + Send + 'static,
        S: for<'a> Service<
                MiddlewareContext<'a>,
                Response = ServiceResponse,
                Error = Error,
                Future = F,
            > + Send
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
