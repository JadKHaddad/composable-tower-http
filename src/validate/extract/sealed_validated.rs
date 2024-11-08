use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct SealedValidated<T>(pub(crate) T);

impl<T> SealedValidated<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for SealedValidated<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
