use crate::{ClientConfigKey, ClientOptions, Error, Result, RetryConfig};
use std::fmt;
use std::str::FromStr;

/// Placeholder no-op representation of Amazon S3.
#[derive(Clone, Debug, Default)]
pub struct AmazonS3;

impl AmazonS3 {
    /// Construct a new stub instance.
    pub fn new(_endpoint: impl AsRef<str>) -> Result<Self> {
        Err(Error::NotImplemented)
    }
}

impl fmt::Display for AmazonS3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AmazonS3(unsupported)")
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

    pub fn with_client_options(self, _options: ClientOptions) -> Self {
        self
    }

    pub fn with_imdsv1_fallback(self) -> Self {
        self
    }

    pub fn with_virtual_hosted_style_request(self, _enabled: bool) -> Self {
        self
    }

    pub fn with_allow_http(self, _allow: bool) -> Self {
        self
    }

    pub fn with_region(self, _region: impl Into<String>) -> Self {
        self
    }

    pub fn with_bucket_name(self, _bucket: impl Into<String>) -> Self {
        self
    }

    pub fn with_access_key_id(self, _id: impl Into<String>) -> Self {
        self
    }

    pub fn with_secret_access_key(self, _secret: impl Into<String>) -> Self {
        self
    }

    pub fn with_endpoint(self, _endpoint: impl Into<String>) -> Self {
        self
    }

    pub fn with_url(self, _url: impl AsRef<str>) -> Self {
        self
    }

    pub fn with_retry(self, _config: RetryConfig) -> Self {
        self
    }

    pub fn with_config(self, _key: AmazonS3ConfigKey, _value: impl Into<String>) -> Self {
        self
    }

    pub fn with_unsigned_payload(self, _enabled: bool) -> Self {
        self
    }

    pub fn with_checksum_algorithm(self, _checksum: Checksum) -> Self {
        self
    }

    pub fn build(self) -> Result<AmazonS3> {
        Err(Error::NotImplemented)
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
        let key = match normalized.as_str() {
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
                    return Ok(Self::Client(client_key));
                }
                return Ok(Self::Custom(value.to_string()));
            }
        };
        Ok(key)
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
pub fn resolve_bucket_region(_bucket: &str) -> Result<String> {
    Err(Error::NotImplemented)
}
