// TODO
// pub mod then_authorizer;

use std::future::Future;

// TODO: make then global to crate and impl for authorizer and extrctor like validator and remove validator
pub trait Then<T> {
    type Out;

    type Error;

    fn then(&self, value: T) -> impl Future<Output = Result<Self::Out, Self::Error>> + Send;
}
