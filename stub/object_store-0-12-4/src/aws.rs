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

/// Placeholder no-op representation of Amazon S3.
#[derive(Clone, Debug, Default)]
pub struct AmazonS3;

impl AmazonS3 {
    /// Construct a new stub instance.
    pub fn new(_endpoint: impl AsRef<str>) -> Result<Self> {
        unsupported()
    }
}

impl fmt::Display for AmazonS3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AmazonS3(unsupported)")
    }
}

#[async_trait]
impl ObjectStore for AmazonS3 {
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

/// Builder for [`AmazonS3`] which accepts the same configuration calls as the
/// real implementation but always returns `Error::NotImplemented` when built.
#[derive(Clone, Debug, Default)]
pub struct AmazonS3Builder;

impl AmazonS3Builder {
    pub fn new() -> Self {
        Self
    }

    pub fn from_env() -> Self {
        Self
    }

    pub fn with_url(self, _url: impl Into<String>) -> Self {
        self
    }

    pub fn with_config(self, _key: AmazonS3ConfigKey, _value: impl Into<String>) -> Self {
        self
    }

    pub fn get_config_value(&self, _key: &AmazonS3ConfigKey) -> Option<String> {
        None
    }

    pub fn with_access_key_id(self, _id: impl Into<String>) -> Self {
        self
    }

    pub fn with_secret_access_key(self, _key: impl Into<String>) -> Self {
        self
    }

    pub fn with_token(self, _token: impl Into<String>) -> Self {
        self
    }

    pub fn with_region(self, _region: impl Into<String>) -> Self {
        self
    }

    pub fn with_bucket_name(self, _bucket: impl Into<String>) -> Self {
        self
    }

    pub fn with_endpoint(self, _endpoint: impl Into<String>) -> Self {
        self
    }

    pub fn with_credentials(self, _credentials: AwsCredentialProvider) -> Self {
        self
    }

    pub fn with_allow_http(self, _allow: bool) -> Self {
        self
    }

    pub fn with_virtual_hosted_style_request(self, _enabled: bool) -> Self {
        self
    }

    pub fn with_s3_express(self, _enabled: bool) -> Self {
        self
    }

    pub fn with_retry(self, _config: RetryConfig) -> Self {
        self
    }

    pub fn with_imdsv1_fallback(self) -> Self {
        self
    }

    pub fn with_unsigned_payload(self, _enabled: bool) -> Self {
        self
    }

    pub fn with_skip_signature(self, _enabled: bool) -> Self {
        self
    }

    pub fn with_checksum_algorithm(self, _checksum: Checksum) -> Self {
        self
    }

    pub fn with_metadata_endpoint(self, _endpoint: impl Into<String>) -> Self {
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

    pub fn with_copy_if_not_exists(self, _config: S3CopyIfNotExists) -> Self {
        self
    }

    pub fn with_conditional_put(self, _config: S3ConditionalPut) -> Self {
        self
    }

    pub fn with_disable_tagging(self, _disabled: bool) -> Self {
        self
    }

    pub fn with_sse_kms_encryption(self, _kms_key_id: impl Into<String>) -> Self {
        self
    }

    pub fn with_dsse_kms_encryption(self, _kms_key_id: impl Into<String>) -> Self {
        self
    }

    pub fn with_ssec_encryption(self, _customer_key_base64: impl Into<String>) -> Self {
        self
    }

    pub fn with_bucket_key(self, _enabled: bool) -> Self {
        self
    }

    pub fn with_request_payer(self, _enabled: bool) -> Self {
        self
    }

    pub fn with_http_connector<C: HttpConnector>(self, _connector: C) -> Self {
        self
    }

    pub fn build(self) -> Result<AmazonS3> {
        unsupported()
    }
}

/// Configurable keys accepted by [`AmazonS3Builder::with_config`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AmazonS3ConfigKey {
    AccessKeyId,
    SecretAccessKey,
    Bucket,
    Region,
    Endpoint,
    Profile,
    VirtualHostedStyleRequest,
    AllowHttp,
    Client(ClientConfigKey),
    Custom(String),
}

impl FromStr for AmazonS3ConfigKey {
    type Err = ();

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        let normalized = value.to_ascii_lowercase();
        Ok(match normalized.as_str() {
            "aws_access_key_id" | "access_key_id" => Self::AccessKeyId,
            "aws_secret_access_key" | "secret_access_key" => Self::SecretAccessKey,
            "bucket" | "bucket_name" => Self::Bucket,
            "region" | "aws_region" => Self::Region,
            "aws_profile" | "profile" => Self::Profile,
            "endpoint" | "aws_endpoint" => Self::Endpoint,
            "aws_virtual_hosted_style_request" => Self::VirtualHostedStyleRequest,
            "aws_allow_http" => Self::AllowHttp,
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

/// Minimal checksum enumeration so the interface matches the real crate.
#[derive(Clone, Copy, Debug)]
pub enum Checksum {
    Sha256,
    Sha256TreeHash,
    Md5,
    Unsupported,
}

/// Placeholder for conditional copy configuration.
#[derive(Clone, Copy, Debug)]
pub enum S3CopyIfNotExists {
    Unsupported,
}

/// Placeholder for conditional put configuration.
#[derive(Clone, Debug)]
pub enum S3ConditionalPut {
    Unsupported,
}

/// Placeholder marker for Dynamo DB assisted commits.
#[derive(Clone, Debug)]
pub struct DynamoCommit;

/// Placeholder AWS credential representation.
#[derive(Clone, Debug, Default)]
pub struct AwsCredential;

/// Placeholder authorizer.
#[derive(Clone, Debug, Default)]
pub struct AwsAuthorizer;

/// Placeholder credential provider type.
pub type AwsCredentialProvider = ();

/// Resolve a bucket region – not available in this stub.
#[cfg(not(target_arch = "wasm32"))]
pub fn resolve_bucket_region(_bucket: &str) -> Result<String> {
    unsupported()
}
