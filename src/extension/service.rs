use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use http::Request;
use tower::Service;

use crate::extract::extractor::Extractor;

#[derive(Debug, Clone)]
pub struct ExtensionService<S, Ex> {
    service: S,
    extractor: Ex,
}

impl<S, Ex> ExtensionService<S, Ex> {
    pub fn new(service: S, extractor: Ex) -> Self {
        Self { service, extractor }
    }
}

impl<S, Ex, B> Service<Request<B>> for ExtensionService<S, Ex>
where
    Ex: Extractor + Clone + Send + 'static,
    S: Service<Request<B>> + Clone + Send + 'static,
    S::Future: Send,
    S::Response: From<Ex::Error>,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<S::Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<B>) -> Self::Future {
        let mut service = self.service.clone();
        let extractor = self.extractor.clone();

        Box::pin(async move {
            let headers = request.headers();

            let extracted = match extractor.extract(headers).await {
                Ok(extracted) => extracted,
                Err(err) => return Ok(From::from(err)),
            };

            request.extensions_mut().insert(extracted);

            service.call(request).await
        })
    }
}
