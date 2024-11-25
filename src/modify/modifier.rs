use std::future::Future;

pub trait Modifier<T> {
    type Modified: Clone + Send + Sync;

    type Error;

    fn modify(&self, value: T) -> impl Future<Output = Result<Self::Modified, Self::Error>> + Send;
}
