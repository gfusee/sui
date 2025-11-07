use crate::{ClientOptions, Error, Result, RetryConfig};
use std::fmt;
use std::str::FromStr;

/// Stub Google Cloud Storage builder.
#[derive(Clone, Debug, Default)]
pub struct GoogleCloudStorageBuilder;

impl GoogleCloudStorageBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn from_env() -> Self {
        Self
    }

    pub fn with_bucket_name(self, _bucket: impl Into<String>) -> Self {
        self
    }

    pub fn with_service_account_path(self, _path: impl Into<String>) -> Self {
        self
    }

    pub fn with_service_account_key(self, _key: impl Into<String>) -> Self {
        self
    }

    pub fn with_client_options(self, _options: ClientOptions) -> Self {
        self
    }

    pub fn with_retry(self, _config: RetryConfig) -> Self {
        self
    }

    pub fn with_url(self, _url: impl AsRef<str>) -> Self {
        self
    }

    pub fn with_config(self, _key: GoogleConfigKey, _value: impl Into<String>) -> Self {
        self
    }

    pub fn build(self) -> Result<GoogleCloudStorage> {
        Err(Error::NotImplemented)
    }
}

/// Stub Google storage.
#[derive(Clone, Debug, Default)]
pub struct GoogleCloudStorage;

impl fmt::Display for GoogleCloudStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GoogleCloudStorage(unsupported)")
    }
}

/// Keys accepted by [`GoogleCloudStorageBuilder::with_config`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum GoogleConfigKey {
    Bucket,
    ServiceAccountPath,
    ServiceAccountKey,
    Custom(String),
}

impl FromStr for GoogleConfigKey {
    type Err = ();

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match value.to_ascii_lowercase().as_str() {
            "bucket" | "bucket_name" => Self::Bucket,
            "google_service_account" | "service_account_path" => Self::ServiceAccountPath,
            "service_account_key" => Self::ServiceAccountKey,
            _ => Self::Custom(value.to_string()),
        })
    }
}
