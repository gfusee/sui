use crate::{ClientOptions, Error, Result, RetryConfig};
use std::fmt;
use url::Url;

/// Stub HTTP-backed object store builder.
#[derive(Clone, Debug, Default)]
pub struct HttpBuilder;

impl HttpBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn with_url(self, _url: Url) -> Self {
        self
    }

    pub fn with_client_options(self, _options: ClientOptions) -> Self {
        self
    }

    pub fn with_retry(self, _config: RetryConfig) -> Self {
        self
    }

    pub fn build(self) -> Result<HttpStore> {
        Err(Error::NotImplemented)
    }
}

/// Stub HTTP store.
#[derive(Clone, Debug, Default)]
pub struct HttpStore;

impl fmt::Display for HttpStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HttpStore(unsupported)")
    }
}
