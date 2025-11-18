use crate::{ClientConfigKey, ClientOptions, Error, Result};
use std::fmt;
use std::str::FromStr;

/// Stub Azure object store builder.
#[derive(Clone, Debug, Default)]
pub struct MicrosoftAzureBuilder;

impl MicrosoftAzureBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn from_env() -> Self {
        Self
    }

    pub fn with_client_options(self, _options: ClientOptions) -> Self {
        self
    }

    pub fn with_container_name(self, _name: impl Into<String>) -> Self {
        self
    }

    pub fn with_account(self, _account: impl Into<String>) -> Self {
        self
    }

    pub fn with_access_key(self, _key: impl Into<String>) -> Self {
        self
    }

    pub fn with_config(self, _key: AzureConfigKey, _value: impl Into<String>) -> Self {
        self
    }

    pub fn build(self) -> Result<MicrosoftAzure> {
        Err(Error::NotImplemented)
    }
}

/// Stub Azure store.
#[derive(Clone, Debug, Default)]
pub struct MicrosoftAzure;

impl fmt::Display for MicrosoftAzure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MicrosoftAzure(unsupported)")
    }
}

/// Configuration keys accepted by [`MicrosoftAzureBuilder::with_config`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AzureConfigKey {
    AccountName,
    AccessKey,
    ContainerName,
    Endpoint,
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
            "endpoint" => Self::Endpoint,
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
