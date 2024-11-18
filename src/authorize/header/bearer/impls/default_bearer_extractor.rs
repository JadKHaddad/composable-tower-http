use std::borrow::Cow;

use http::{header::AUTHORIZATION, HeaderMap};

use crate::authorize::header::{
    bearer::bearer_extractor::BearerExtractor,
    header_extractor::HeaderExtractor,
    {DefaultHeaderError, DefaultHeaderExtractor},
};

#[derive(Debug)]
pub struct DefaultBearerExtractor {
    // This is not generic, because we have to make sure that the header name is always "Authorization"
    header_erxtractor: DefaultHeaderExtractor,
}

impl DefaultBearerExtractor {
    pub fn new() -> Self {
        Self {
            header_erxtractor: DefaultHeaderExtractor::new(Cow::from(AUTHORIZATION.as_str())),
        }
    }

    pub fn extract_bearer(authorization: &str) -> Result<&str, DefaultBearerError> {
        let split = authorization.split_once(' ');
        let bearer_token = match split {
            Some(("Bearer", bearer_token)) => bearer_token,
            _ => return Err(DefaultBearerError::Format),
        };

        Ok(bearer_token)
    }
}

impl Default for DefaultBearerExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl BearerExtractor for DefaultBearerExtractor {
    type Error = DefaultBearerError;

    fn extract_bearer<'a>(&self, headers: &'a HeaderMap) -> Result<&'a str, Self::Error> {
        let authorization = self.header_erxtractor.extract_header(headers)?;
        let bearer_token = Self::extract_bearer(authorization)?;

        Ok(bearer_token)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DefaultBearerError {
    #[error("Authorization header extraction error: {0}")]
    Header(
        #[source]
        #[from]
        DefaultHeaderError,
    ),
    #[error("Authorization header is not in the form: `Bearer xyz`")]
    Format,
}
