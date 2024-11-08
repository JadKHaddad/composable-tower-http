use std::borrow::Cow;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BasicAuthUser {
    pub username: Cow<'static, str>,
    pub password: Cow<'static, str>,
}

impl core::fmt::Debug for BasicAuthUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BasicAuthUser")
            .field("username", &self.username)
            .field("password", &"...")
            .finish()
    }
}

impl BasicAuthUser {
    pub fn new(
        username: impl Into<Cow<'static, str>>,
        password: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
}

impl<U, P> From<(U, P)> for BasicAuthUser
where
    U: Into<Cow<'static, str>>,
    P: Into<Cow<'static, str>>,
{
    fn from(value: (U, P)) -> Self {
        Self::new(value.0, value.1)
    }
}
