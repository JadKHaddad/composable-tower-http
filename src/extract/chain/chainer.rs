use std::future::Future;

pub trait Chainer<T> {
    type Chained: Clone + Send + Sync;

    type Error;

    fn chain(&self, value: T) -> impl Future<Output = Result<Self::Chained, Self::Error>> + Send;
}
