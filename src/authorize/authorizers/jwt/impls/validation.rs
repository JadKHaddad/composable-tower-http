use std::collections::HashSet;

use jsonwebtoken::{Algorithm, Validation as JsonWebTokenValidation};

/// Refer to the [`Validation`](jsonwebtoken::Validation) struct from the [`jsonwebtoken`] crate for more information.
#[derive(Debug, Clone)]
pub struct Validation {
    required_spec_claims: HashSet<String>,
    leeway: u64,
    reject_tokens_expiring_in_less_than: u64,
    validate_exp: bool,
    validate_nbf: bool,
    validate_aud: bool,
    aud: Option<HashSet<String>>,
    iss: Option<HashSet<String>>,
    sub: Option<String>,
    validate_signature: bool,
}

impl Default for Validation {
    fn default() -> Self {
        Self {
            required_spec_claims: [String::from("exp")].into(),
            leeway: 60,
            reject_tokens_expiring_in_less_than: 0,
            validate_exp: true,
            validate_nbf: false,
            validate_aud: true,
            aud: None,
            iss: None,
            sub: None,
            validate_signature: true,
        }
    }
}

impl Validation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn required_spec_claims(mut self, required_spec_claims: HashSet<String>) -> Self {
        self.required_spec_claims = required_spec_claims;
        self
    }

    pub fn leeway(mut self, leeway: u64) -> Self {
        self.leeway = leeway;
        self
    }

    pub fn reject_tokens_expiring_in_less_than(
        mut self,
        reject_tokens_expiring_in_less_than: u64,
    ) -> Self {
        self.reject_tokens_expiring_in_less_than = reject_tokens_expiring_in_less_than;
        self
    }

    pub fn validate_exp(mut self, validate_exp: bool) -> Self {
        self.validate_exp = validate_exp;
        self
    }

    pub fn validate_nbf(mut self, validate_nbf: bool) -> Self {
        self.validate_nbf = validate_nbf;
        self
    }

    pub fn validate_aud(mut self, validate_aud: bool) -> Self {
        self.validate_aud = validate_aud;
        self
    }

    pub fn aud<T: ToString>(mut self, aud: &[T]) -> Self {
        self.aud = Some(aud.iter().map(|a| a.to_string()).collect());
        self
    }

    pub fn iss<T: ToString>(mut self, iss: &[T]) -> Self {
        self.iss = Some(iss.iter().map(|i| i.to_string()).collect());
        self
    }

    #[allow(clippy::should_implement_trait)]
    pub fn sub<T: ToString>(mut self, sub: T) -> Self {
        self.sub = Some(sub.to_string());
        self
    }

    pub fn insecure_disable_signature_validation(mut self) -> Self {
        self.validate_signature = false;
        self
    }

    pub fn to_jsonwebtoken_validation(&self, algorithm: Algorithm) -> JsonWebTokenValidation {
        let mut validation = JsonWebTokenValidation::new(algorithm);

        validation.leeway = self.leeway;
        validation.reject_tokens_expiring_in_less_than = self.reject_tokens_expiring_in_less_than;

        validation.validate_exp = self.validate_exp;
        validation.validate_nbf = self.validate_nbf;
        validation.validate_aud = self.validate_aud;

        validation.aud = self.aud.clone();
        validation.iss = self.iss.clone();
        validation.sub = self.sub.clone();

        if !self.validate_signature {
            validation.insecure_disable_signature_validation();
        }

        validation
    }
}
