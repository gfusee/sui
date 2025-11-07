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

/// List of known Azure authority hosts.
pub mod authority_hosts {
    /// China-based Azure Authority Host
    pub const AZURE_CHINA: &str = "https://login.chinacloudapi.cn";
    /// Germany-based Azure Authority Host
    pub const AZURE_GERMANY: &str = "https://login.microsoftonline.de";
    /// US Government Azure Authority Host
    pub const AZURE_GOVERNMENT: &str = "https://login.microsoftonline.us";
    /// Public Cloud Azure Authority Host
    pub const AZURE_PUBLIC_CLOUD: &str = "https://login.microsoftonline.com";
}

/// Placeholder Azure access key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AzureAccessKey;

/// Placeholder Azure credential.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AzureCredential {
    AccessKey(AzureAccessKey),
    SASToken(Vec<(String, String)>),
    BearerToken(String),
    Unsupported,
}

/// Placeholder Azure authorizer.
#[derive(Clone, Debug, Default)]
pub struct AzureAuthorizer;

/// Placeholder credential provider type.
pub type AzureCredentialProvider = ();

/// Stub Azure object store.
#[derive(Clone, Debug, Default)]
pub struct MicrosoftAzure;

impl fmt::Display for MicrosoftAzure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MicrosoftAzure(unsupported)")
    }
}

#[async_trait]
impl ObjectStore for MicrosoftAzure {
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

/// Stub builder for [`MicrosoftAzure`].
#[derive(Clone, Debug, Default)]
pub struct MicrosoftAzureBuilder;

impl MicrosoftAzureBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn from_env() -> Self {
        Self
    }

    pub fn with_url(self, _url: impl Into<String>) -> Self {
        self
    }

    pub fn with_config(self, _key: AzureConfigKey, _value: impl Into<String>) -> Self {
        self
    }

    pub fn get_config_value(&self, _key: &AzureConfigKey) -> Option<String> {
        None
    }

    pub fn with_account(self, _account: impl Into<String>) -> Self {
        self
    }

    pub fn with_container_name(self, _name: impl Into<String>) -> Self {
        self
    }

    pub fn with_access_key(self, _key: impl Into<String>) -> Self {
        self
    }

    pub fn with_bearer_token_authorization(self, _token: impl Into<String>) -> Self {
        self
    }

    pub fn with_client_secret_authorization(
        self,
        _client_id: impl Into<String>,
        _client_secret: impl Into<String>,
        _tenant_id: impl Into<String>,
    ) -> Self {
        self
    }

    pub fn with_client_id(self, _client_id: impl Into<String>) -> Self {
        self
    }

    pub fn with_client_secret(self, _client_secret: impl Into<String>) -> Self {
        self
    }

    pub fn with_tenant_id(self, _tenant_id: impl Into<String>) -> Self {
        self
    }

    pub fn with_sas_authorization(self, _pairs: impl Into<Vec<(String, String)>>) -> Self {
        self
    }

    pub fn with_credentials(self, _credentials: AzureCredentialProvider) -> Self {
        self
    }

    pub fn with_use_emulator(self, _enabled: bool) -> Self {
        self
    }

    pub fn with_endpoint(self, _endpoint: impl Into<String>) -> Self {
        self
    }

    pub fn with_use_fabric_endpoint(self, _enabled: bool) -> Self {
        self
    }

    pub fn with_allow_http(self, _allow: bool) -> Self {
        self
    }

    pub fn with_authority_host(self, _host: impl Into<String>) -> Self {
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

    pub fn with_msi_endpoint(self, _endpoint: impl Into<String>) -> Self {
        self
    }

    pub fn with_federated_token_file(self, _path: impl Into<String>) -> Self {
        self
    }

    pub fn with_use_azure_cli(self, _enabled: bool) -> Self {
        self
    }

    pub fn with_skip_signature(self, _enabled: bool) -> Self {
        self
    }

    pub fn with_disable_tagging(self, _disabled: bool) -> Self {
        self
    }

    pub fn with_http_connector<C: HttpConnector>(self, _connector: C) -> Self {
        self
    }

    pub fn build(self) -> Result<MicrosoftAzure> {
        unsupported()
    }
}

/// Configuration keys accepted by [`MicrosoftAzureBuilder`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AzureConfigKey {
    AccountName,
    AccessKey,
    ContainerName,
    Token,
    ClientId,
    ClientSecret,
    AuthorityId,
    AuthorityHost,
    SasKey,
    MsiEndpoint,
    ObjectId,
    MsiResourceId,
    FederatedTokenFile,
    UseAzureCli,
    SkipSignature,
    UseEmulator,
    Endpoint,
    UseFabricEndpoint,
    DisableTagging,
    FabricTokenServiceUrl,
    FabricWorkloadHost,
    FabricSessionToken,
    FabricClusterIdentifier,
    ProxyUrl,
    ProxyCaCertificate,
    ProxyExcludes,
    Client(ClientConfigKey),
    Custom(String),
}

impl FromStr for AzureConfigKey {
    type Err = ();

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match value.to_ascii_lowercase().as_str() {
            "azure_storage_account" | "account_name" => Self::AccountName,
            "azure_storage_access_key" | "access_key" => Self::AccessKey,
            "container" | "container_name" | "bucket" => Self::ContainerName,
            "token" | "bearer_token" => Self::Token,
            "client_id" => Self::ClientId,
            "client_secret" => Self::ClientSecret,
            "authority_id" | "tenant_id" => Self::AuthorityId,
            "authority_host" => Self::AuthorityHost,
            "sas_key" => Self::SasKey,
            "msi_endpoint" => Self::MsiEndpoint,
            "object_id" => Self::ObjectId,
            "msi_resource_id" => Self::MsiResourceId,
            "federated_token_file" => Self::FederatedTokenFile,
            "use_azure_cli" => Self::UseAzureCli,
            "skip_signature" => Self::SkipSignature,
            "use_emulator" => Self::UseEmulator,
            "endpoint" => Self::Endpoint,
            "use_fabric_endpoint" => Self::UseFabricEndpoint,
            "disable_tagging" => Self::DisableTagging,
            "fabric_token_service_url" => Self::FabricTokenServiceUrl,
            "fabric_workload_host" => Self::FabricWorkloadHost,
            "fabric_session_token" => Self::FabricSessionToken,
            "fabric_cluster_identifier" => Self::FabricClusterIdentifier,
            "proxy_url" => Self::ProxyUrl,
            "proxy_ca_certificate" => Self::ProxyCaCertificate,
            "proxy_excludes" => Self::ProxyExcludes,
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
