use crate::client::HttpConnector;
use crate::{ListResult, PutMultipartOptions};
use crate::path::Path;
use crate::upload::MultipartUpload;
use crate::{
    ClientConfigKey, ClientOptions, Error, GetOptions, GetResult, ObjectMeta, ObjectStore,
    PutOptions, PutPayload, PutResult, Result, RetryConfig,
};
use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use futures::StreamExt;
use std::fmt;
use std::str::FromStr;

fn unsupported<T>() -> Result<T> {
    Err(Error::NotImplemented)
}

fn unsupported_stream<T>() -> BoxStream<'static, Result<T>> {
    stream::once(async { Err::<T, Error>(Error::NotImplemented) }).boxed()
}

/// Placeholder credential provider type.
pub type GcpCredentialProvider = ();
/// Placeholder signing credential provider type.
pub type GcpSigningCredentialProvider = ();

/// Placeholder GCP credential types.
#[derive(Clone, Debug)]
pub enum GcpCredential {
    Unsupported,
}

#[derive(Clone, Debug)]
pub enum GcpSigningCredential {
    Unsupported,
}

#[derive(Clone, Debug)]
pub struct ServiceAccountKey;

#[derive(Clone, Debug, Default)]
pub struct GCSAuthorizer;

/// Stub GCS object store.
#[derive(Clone, Debug, Default)]
pub struct GoogleCloudStorage;

impl fmt::Display for GoogleCloudStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GoogleCloudStorage(unsupported)")
    }
}

#[async_trait]
impl ObjectStore for GoogleCloudStorage {
    async fn put_opts(
        &self,
        _location: &Path,
        _payload: PutPayload,
        _opts: PutOptions,
    ) -> Result<PutResult> {
        unsupported()
    }

    async fn put_multipart_opts(
        &self,
        _location: &Path,
        _opts: PutMultipartOptions,
    ) -> Result<Box<dyn MultipartUpload>> {
        unsupported()
    }

    async fn get_opts(&self, _location: &Path, _options: GetOptions) -> Result<GetResult> {
        unsupported()
    }

    async fn delete(&self, _location: &Path) -> Result<()> {
        unsupported()
    }

    fn list(&self, _prefix: Option<&Path>) -> BoxStream<'static, Result<ObjectMeta>> {
        unsupported_stream()
    }

    async fn list_with_delimiter(&self, _prefix: Option<&Path>) -> Result<ListResult> {
        unsupported()
    }

    async fn copy(&self, _from: &Path, _to: &Path) -> Result<()> {
        unsupported()
    }

    async fn copy_if_not_exists(&self, _from: &Path, _to: &Path) -> Result<()> {
        unsupported()
    }
}

/// Stub builder for [`GoogleCloudStorage`].
#[derive(Clone, Debug, Default)]
pub struct GoogleCloudStorageBuilder;

impl GoogleCloudStorageBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn from_env() -> Self {
        Self
    }

    pub fn with_url(self, _url: impl Into<String>) -> Self {
        self
    }

    pub fn with_config(self, _key: GoogleConfigKey, _value: impl Into<String>) -> Self {
        self
    }

    pub fn get_config_value(&self, _key: &GoogleConfigKey) -> Option<String> {
        None
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

    pub fn with_application_credentials(self, _path: impl Into<String>) -> Self {
        self
    }

    pub fn with_skip_signature(self, _enabled: bool) -> Self {
        self
    }

    pub fn with_credentials(self, _credentials: GcpCredentialProvider) -> Self {
        self
    }

    pub fn with_retry(self, _config: RetryConfig) -> Self {
        self
    }

    pub fn with_proxy_url(self, _url: impl Into<String>) -> Self {
        self
    }

    pub fn with_proxy_ca_certificate(self, _cert: impl Into<String>) -> Self {
        self
    }

    pub fn with_proxy_excludes(self, _excludes: impl Into<String>) -> Self {
        self
    }

    pub fn with_client_options(self, _options: ClientOptions) -> Self {
        self
    }

    pub fn with_http_connector<C: HttpConnector>(self, _connector: C) -> Self {
        self
    }

    pub fn build(self) -> Result<GoogleCloudStorage> {
        unsupported()
    }
}

/// Configuration keys accepted by [`GoogleCloudStorageBuilder`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum GoogleConfigKey {
    Bucket,
    ServiceAccountPath,
    ServiceAccountKey,
    ApplicationCredentials,
    SkipSignature,
    Client(ClientConfigKey),
    Custom(String),
}

impl FromStr for GoogleConfigKey {
    type Err = ();

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match value.to_ascii_lowercase().as_str() {
            "bucket" | "bucket_name" => Self::Bucket,
            "google_service_account" | "service_account_path" => Self::ServiceAccountPath,
            "service_account_key" => Self::ServiceAccountKey,
            "google_application_credentials" | "application_credentials" => {
                Self::ApplicationCredentials
            }
            "skip_signature" => Self::SkipSignature,
            other => {
                if let Ok(client_key) = other.parse() {
                    Self::Client(client_key)
                } else {
                    Self::Custom(value.to_string())
                }
            }
        })
    }
}
