use std::{borrow::Cow, string::FromUtf8Error};

use base64::Engine;
use http::{header::AUTHORIZATION, HeaderMap};

use crate::authorize::header::{
    basic_auth::basic_auth_extractor::BasicAuthExtractor,
    header_extractor::HeaderExtractor,
    impls::default_header_extractor::{DefaultHeaderError, DefaultHeaderExtractor},
};

#[derive(Debug)]
pub struct DefaultBasicAuthExtractor {
    // This is not generic, because we have to make sure that the header name is always "Authorization"
    header_extractor: DefaultHeaderExtractor,
}

impl DefaultBasicAuthExtractor {
    pub fn new() -> Self {
        Self {
            header_extractor: DefaultHeaderExtractor::new(Cow::from(AUTHORIZATION.as_str())),
        }
    }

    fn extract_encoded_basic(authorization: &str) -> Result<&str, DefaultBasicAuthError> {
        let split = authorization.split_once(' ');
        let encoded_basic = match split {
            Some(("Basic", encoded_basic)) => encoded_basic,
            _ => return Err(DefaultBasicAuthError::Format),
        };

        Ok(encoded_basic)
    }

    fn decode(encoded_basic: &str) -> Result<String, DefaultBasicAuthError> {
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(encoded_basic)
            .map_err(DefaultBasicAuthError::Decode)?;

        let decoded = String::from_utf8(decoded).map_err(DefaultBasicAuthError::Utf8)?;

        Ok(decoded)
    }

    fn split(basic_auth: String) -> Result<(String, String), DefaultBasicAuthError> {
        match basic_auth.split_once(':') {
            Some((username, password)) => Ok((username.to_string(), password.to_string())),
            None => Err(DefaultBasicAuthError::Colon),
        }
    }
}

impl Default for DefaultBasicAuthExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl BasicAuthExtractor for DefaultBasicAuthExtractor {
    type Error = DefaultBasicAuthError;

    fn extract_basic_auth(&self, headers: &HeaderMap) -> Result<(String, String), Self::Error> {
        let authorization = self.header_extractor.extract_header(headers)?;
        let (username, password) = Self::extract_encoded_basic(authorization)
            .map(Self::decode)?
            .map(Self::split)??;

        Ok((username, password))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DefaultBasicAuthError {
    #[error("Authorization header extraction error: {0}")]
    Header(
        #[source]
        #[from]
        DefaultHeaderError,
    ),
    #[error("Authorization header is not in the form: `Basic xyz`")]
    Format,
    #[error("Authorization header base64 decode error: {0}")]
    Decode(base64::DecodeError),
    #[error("Authorization header utf-8 error: {0}")]
    Utf8(FromUtf8Error),
    #[error("Authorization header does not contain a colon")]
    Colon,
}
