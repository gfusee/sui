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

fn unsupported<T>() -> Result<T> {
    Err(Error::NotImplemented)
}

fn unsupported_stream<T>() -> BoxStream<'static, Result<T>> {
    stream::once(async { Err::<T, Error>(Error::NotImplemented) }).boxed()
}

/// Stub HTTP-backed object store.
#[derive(Clone, Debug, Default)]
pub struct HttpStore;

impl fmt::Display for HttpStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HttpStore(unsupported)")
    }
}

#[async_trait]
impl ObjectStore for HttpStore {
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

/// Stub builder for [`HttpStore`].
#[derive(Clone, Debug, Default)]
pub struct HttpBuilder;

impl HttpBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn with_url(self, _url: impl Into<String>) -> Self {
        self
    }

    pub fn with_retry(self, _config: RetryConfig) -> Self {
        self
    }

    pub fn with_config(self, _key: ClientConfigKey, _value: impl Into<String>) -> Self {
        self
    }

    pub fn with_client_options(self, _options: ClientOptions) -> Self {
        self
    }

    pub fn with_http_connector<C: HttpConnector>(self, _connector: C) -> Self {
        self
    }

    pub fn build(self) -> Result<HttpStore> {
        unsupported()
    }
}
