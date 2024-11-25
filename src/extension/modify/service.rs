use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use http::{Request, StatusCode};
use tower::Service;

use crate::{extract::SealedExtracted, modify::Modifier};

#[derive(Debug, Clone)]
pub struct ModifyService<S, M, T> {
    service: S,
    modifier: M,
    _phantom: PhantomData<T>,
}

impl<S, M, T> ModifyService<S, M, T> {
    pub const fn new(service: S, modifier: M) -> Self {
        Self {
            service,
            modifier,
            _phantom: PhantomData,
        }
    }
}

impl<S, M, B, T> Service<Request<B>> for ModifyService<S, M, T>
where
    M: Modifier<T> + Clone + Send + 'static,
    T: Send + Sync + 'static,
    S: Service<Request<B>> + Clone + Send + 'static,
    S::Future: Send,
    S::Response: From<M::Error> + From<StatusCode>,
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
        let modifier = self.modifier.clone();

        Box::pin(async move {
            match request.extensions_mut().remove::<SealedExtracted<T>>() {
                Some(SealedExtracted(extracted)) => {
                    match modifier.modify(extracted).await {
                        Ok(modified) => request.extensions_mut().insert(SealedExtracted(modified)),
                        Err(err) => return Ok(From::from(err)),
                    };
                }
                None => {
                    tracing::error!(
                        "Requested extracted extension was not found. Did you use `Extractor` with `ExtensionLayer`?"
                    );

                    return Ok(S::Response::from(StatusCode::INTERNAL_SERVER_ERROR));
                }
            }

            service.call(request).await
        })
    }
}
