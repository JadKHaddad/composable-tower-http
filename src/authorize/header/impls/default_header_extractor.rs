use std::borrow::Cow;

use http::{header::ToStrError, HeaderMap};

use crate::authorize::header::header_extractor::HeaderExtractor;

#[derive(Debug)]
pub struct DefaultHeaderExtractor {
    header_name: Cow<'static, str>,
}

impl DefaultHeaderExtractor {
    pub fn new(header_name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            header_name: header_name.into(),
        }
    }
}

impl HeaderExtractor for DefaultHeaderExtractor {
    type Error = DefaultHeaderError;

    #[tracing::instrument(skip_all, fields(header_name = %self.header_name))]
    fn extract_header<'a>(&self, headers: &'a HeaderMap) -> Result<&'a str, Self::Error> {
        let header = headers
            .get(self.header_name.as_ref())
            .ok_or(DefaultHeaderError::Missing)?
            .to_str()
            .map_err(DefaultHeaderError::Ascii)?;

        Ok(header)
    }

    #[tracing::instrument(skip_all)]
    fn header_name(&self) -> &str {
        &self.header_name
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DefaultHeaderError {
    #[error("Header not found")]
    Missing,
    #[error("Header ascii error: {0}")]
    Ascii(ToStrError),
}
