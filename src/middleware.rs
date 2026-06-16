use reqwest::{Client, Request};

pub struct MiddlewareContext<'a> {
    pub client: &'a Client,
    pub request: Request,
}

impl<'a> MiddlewareContext<'a> {
    pub fn new(client: &'a Client, request: Request) -> Self {
        Self { client, request }
    }
}
