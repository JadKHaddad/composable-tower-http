use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct SealedExtracted<T>(pub(crate) T);

impl<T> SealedExtracted<T> {
    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn map<Fn, U>(self, map: Fn) -> SealedExtracted<U>
    where
        Fn: FnOnce(T) -> U,
    {
        SealedExtracted(map(self.0))
    }
}

impl<T> Deref for SealedExtracted<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
