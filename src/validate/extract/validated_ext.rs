use crate::extract::extractor::Extractor;

use super::validation_extractor::ValidationExtractor;

pub trait ValidationExt: Sized + Extractor {
    fn validated<V>(self, validator: V) -> ValidationExtractor<Self, V>;
}

impl<T> ValidationExt for T
where
    T: Sized + Extractor,
{
    fn validated<V>(self, validator: V) -> ValidationExtractor<Self, V> {
        ValidationExtractor::new(self, validator)
    }
}
