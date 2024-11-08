use std::borrow::Cow;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ApiKey {
    pub value: Cow<'static, str>,
}

impl core::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiKey").field("value", &"...").finish()
    }
}

impl ApiKey {
    pub fn new(value: impl Into<Cow<'static, str>>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl<T> From<T> for ApiKey
where
    T: Into<Cow<'static, str>>,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}
