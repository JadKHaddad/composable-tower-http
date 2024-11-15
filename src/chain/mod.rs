use std::future::Future;

pub mod chain_extractor;

pub trait Chain<T> {
    type Extracted: Clone + Send + Sync;

    type Error;

    fn chain(&self, value: T) -> impl Future<Output = Result<Self::Extracted, Self::Error>> + Send;
}
