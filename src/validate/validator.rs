use std::future::Future;

pub trait Validator<T> {
    type Error;

    fn validate(&self, value: &T) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
