use tower::Layer;

use super::service::ExtensionService;

#[derive(Debug, Clone)]
pub struct ExtensionLayer<Ex> {
    extractor: Ex,
}

impl<Ex> ExtensionLayer<Ex> {
    pub fn new(extractor: Ex) -> Self {
        Self { extractor }
    }
}

impl<S, Ex> Layer<S> for ExtensionLayer<Ex>
where
    Ex: Clone,
{
    type Service = ExtensionService<S, Ex>;

    fn layer(&self, service: S) -> Self::Service {
        ExtensionService::new(service, self.extractor.clone())
    }
}
