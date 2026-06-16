use crunchyroll_rs::crunchyroll::DeviceIdentifier;
use crunchyroll_rs::middleware::MiddlewareContext;
use crunchyroll_rs::{Crunchyroll, Error};
use reqwest::Response;
use std::env;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower_service::Service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let email = env::var("EMAIL").expect("'EMAIL' environment variable not found");
    let password = env::var("PASSWORD").expect("'PASSWORD' environment variable not found");

    let _crunchyroll = Crunchyroll::builder()
        .middleware(RequestPrinterMiddleware)
        .login_with_credentials(email, password, DeviceIdentifier::default())
        .await?;

    Ok(())
}

struct RequestPrinterMiddleware;

impl<'a> Service<MiddlewareContext<'a>> for RequestPrinterMiddleware {
    type Response = Response;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, ctx: MiddlewareContext<'a>) -> Self::Future {
        let method = ctx.request.method().clone();
        let url = ctx.request.url().clone();

        let body_str = if let Some(body) = ctx.request.body() {
            let Some(bytes) = body.as_bytes() else {
                unreachable!();
            };

            let raw_body = String::from_utf8_lossy(bytes);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw_body) {
                serde_json::to_string_pretty(&json).unwrap_or_else(|_| raw_body.to_string())
            } else {
                raw_body.to_string()
            }
        } else {
            "".to_string()
        };

        println!("{} {}: {}", method, url, body_str);

        let client = ctx.client.clone();
        Box::pin(async move { client.execute(ctx.request).await.map_err(Into::into) })
    }
}
