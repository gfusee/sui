use http::Extensions;
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

/// Simplified request wrapper. Carries a payload plus stub metadata and extension
/// storage so higher layers can type-check when compiled for wasm.
#[derive(Clone, Debug)]
pub struct Request<T> {
    inner: T,
    metadata: metadata::MetadataMap,
    extensions: Extensions,
}

impl<T> Request<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            metadata: metadata::MetadataMap::new(),
            extensions: Extensions::new(),
        }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn metadata_mut(&mut self) -> &mut metadata::MetadataMap {
        &mut self.metadata
    }

    pub fn metadata(&self) -> &metadata::MetadataMap {
        &self.metadata
    }

    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }

    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }
}

/// Simplified response wrapper. Matches the `Request` behaviour.
#[derive(Clone, Debug)]
pub struct Response<T> {
    inner: T,
    metadata: metadata::MetadataMap,
}

impl<T> Response<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            metadata: metadata::MetadataMap::new(),
        }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn metadata_mut(&mut self) -> &mut metadata::MetadataMap {
        &mut self.metadata
    }
}

/// Placeholder streaming type.
#[derive(Clone, Debug, Default)]
pub struct Streaming<T> {
    _marker: PhantomData<T>,
}

pub mod codec {
    /// Placeholder compression encodings.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum CompressionEncoding {
        Identity,
        Gzip,
        Deflate,
        Zstd,
    }
}

/// Minimal set of status codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    Ok,
    Cancelled,
    Unknown,
    InvalidArgument,
    DeadlineExceeded,
    NotFound,
    AlreadyExists,
    PermissionDenied,
    ResourceExhausted,
    FailedPrecondition,
    Aborted,
    OutOfRange,
    Unimplemented,
    Internal,
    Unavailable,
    DataLoss,
    Unauthenticated,
}

/// Lightweight error container used across the stub.
#[derive(Clone, Debug)]
pub struct Status {
    code: Code,
    message: String,
}

impl Status {
    pub fn new(code: Code, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn code(&self) -> Code {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn unknown(message: impl Into<String>) -> Self {
        Self::new(Code::Unknown, message)
    }

    pub fn unimplemented(message: impl Into<String>) -> Self {
        Self::new(Code::Unimplemented, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(Code::NotFound, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(Code::Internal, message)
    }

    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::new(Code::Unavailable, message)
    }

    pub fn from_error<E>(err: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self::new(Code::Unknown, err.into().to_string())
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for Status {}

/// Minimal conversion trait for request builders.
pub trait IntoRequest<T> {
    fn into_request(self) -> Request<T>;
}

impl<T> IntoRequest<T> for Request<T> {
    fn into_request(self) -> Request<T> {
        self
    }
}

impl<T> IntoRequest<T> for T {
    fn into_request(self) -> Request<T> {
        Request::new(self)
    }
}

pub mod metadata {
    use super::Status;
    use std::marker::PhantomData;

    /// Empty marker used by some tonic APIs.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct Ascii;

    /// Simplified metadata container.
    #[derive(Clone, Debug, Default)]
    pub struct MetadataMap;

    impl MetadataMap {
        pub fn new() -> Self {
            Self
        }

        pub fn insert<K, V>(&mut self, _key: K, _value: V) {}

        pub fn insert_bin<K, V>(&mut self, _key: K, _value: V) {}

        pub fn iter(&self) -> MetadataIter {
            MetadataIter
        }
    }

    /// No-op iterator used by `for_each`.
    #[derive(Clone, Debug)]
    pub struct MetadataIter;

    impl MetadataIter {
        pub fn for_each<F>(self, mut _f: F)
        where
            F: FnMut(KeyAndValueRef<'_>),
        {
            // no-op
        }
    }

    /// Simplified metadata value type.
    #[derive(Clone, Debug, Default)]
    pub struct MetadataValue<P = ()> {
        _marker: PhantomData<P>,
    }

    impl<P> MetadataValue<P> {
        pub fn from_static(_value: &'static str) -> Self {
            Self {
                _marker: PhantomData,
            }
        }
    }

    impl<P> std::str::FromStr for MetadataValue<P> {
        type Err = Status;

        fn from_str(_s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                _marker: PhantomData,
            })
        }
    }

    pub type AsciiMetadataValue = MetadataValue<Ascii>;

    /// Simplified key/value iterator item.
    pub enum KeyAndValueRef<'a> {
        Ascii(&'a str, AsciiMetadataValue),
        Binary(&'a str, MetadataValue<()>),
    }

    pub const GRPC_CONTENT_TYPE: &str = "application/grpc";
}

pub mod body {
    use bytes::Bytes;
    use http::HeaderMap;
    use std::convert::Infallible;
    use std::task::{Context, Poll};

    /// Placeholder body type used by generated code.
    #[derive(Clone, Debug, Default)]
    pub struct Body;

    impl http_body::Body for Body {
        type Data = Bytes;
        type Error = Infallible;

        fn poll_data(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
            Poll::Ready(None)
        }

        fn poll_trailers(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
            Poll::Ready(Ok(None))
        }
    }

    pub type BoxBody = Body;
}

pub mod service {
    use super::{body::Body, Request, Status};

    pub trait Interceptor: Send + Sync + 'static {
        fn call(&mut self, request: Request<Body>) -> Result<Request<Body>, Status>;
    }
}

pub mod client {
    use super::codec::CompressionEncoding;
    use super::{Request, Response, Status};
    use http::uri::PathAndQuery;
    use http::Uri;

    #[derive(Clone)]
    pub struct Grpc<T> {
        inner: T,
    }

    impl<T> Grpc<T> {
        pub fn new(inner: T) -> Self {
            Self { inner }
        }

        pub fn with_origin(inner: T, _origin: Uri) -> Self {
            Self { inner }
        }

        pub fn send_compressed(self, _encoding: CompressionEncoding) -> Self {
            self
        }

        pub fn accept_compressed(self, _encoding: CompressionEncoding) -> Self {
            self
        }

        pub fn max_decoding_message_size(self, _limit: usize) -> Self {
            self
        }

        pub fn max_encoding_message_size(self, _limit: usize) -> Self {
            self
        }

        pub async fn ready(&mut self) -> Result<(), Status> {
            Ok(())
        }

        pub async fn unary<Req, Res, Codec>(
            &mut self,
            _request: Request<Req>,
            _path: PathAndQuery,
            _codec: Codec,
        ) -> Result<Response<Res>, Status>
        where
            Codec: Default,
        {
            Err(Status::unimplemented(
                "tonic stubs do not perform network operations",
            ))
        }
    }

    pub trait GrpcService<B>: tower_service::Service<http::Request<B>> {}

    impl<T, B> GrpcService<B> for T where T: tower_service::Service<http::Request<B>> {}
}

pub mod server {
    use super::codec::CompressionEncoding;
    use super::{Request, Response, Status};
    use http::Request as HttpRequest;
    use std::future::Future;
    use std::task::{Context, Poll};

    #[derive(Clone, Default)]
    pub struct Grpc<T> {
        _marker: std::marker::PhantomData<T>,
    }

    impl<T> Grpc<T> {
        pub fn new(_codec: T) -> Self {
            Self {
                _marker: std::marker::PhantomData,
            }
        }

        pub fn apply_compression_config(
            self,
            _accept: CompressionEncoding,
            _send: CompressionEncoding,
        ) -> Self {
            self
        }

        pub fn apply_max_message_size_config(
            self,
            _decode: Option<usize>,
            _encode: Option<usize>,
        ) -> Self {
            self
        }
    }

    pub trait NamedService {
        const NAME: &'static str;
    }

    pub trait UnaryService<T> {
        type Response;
        type Future: Future<Output = Result<Response<Self::Response>, Status>>;

        fn call(&mut self, request: Request<T>) -> Self::Future;
    }

    pub trait StreamingService<T> {
        type Response;
        type ResponseStream;
        type Future: Future<Output = Result<Response<Self::ResponseStream>, Status>>;

        fn call(&mut self, request: Request<T>) -> Self::Future;
    }

    pub trait Service<RequestBody>: tower_service::Service<HttpRequest<RequestBody>> {}

    impl<T, RequestBody> Service<RequestBody> for T where
        T: tower_service::Service<HttpRequest<RequestBody>>
    {
    }

    #[derive(Clone, Debug)]
    pub struct EnabledCompressionEncodings;

    impl Default for EnabledCompressionEncodings {
        fn default() -> Self {
            Self
        }
    }

    impl EnabledCompressionEncodings {
        pub fn enable(&mut self, _encoding: CompressionEncoding) {}
    }

    #[derive(Clone, Debug)]
    pub struct GrpcMethod(pub &'static str, pub &'static str);

    impl GrpcMethod {
        pub fn new(service: &'static str, method: &'static str) -> Self {
            Self(service, method)
        }
    }

    pub trait Interceptor: Send + Sync + 'static {
        fn call(
            &mut self,
            request: http::Request<super::body::Body>,
        ) -> Result<http::Request<super::body::Body>, Status>;
    }
}

pub mod transport {
    use super::Status;
    use http::Uri;
    use std::convert::TryInto;
    use std::fmt;
    use std::net::SocketAddr;

    #[derive(Clone, Debug)]
    pub struct Error {
        message: String,
    }

    impl Error {
        pub fn new(message: impl Into<String>) -> Self {
            Self {
                message: message.into(),
            }
        }
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl std::error::Error for Error {}

    #[derive(Clone, Debug, Default)]
    pub struct Channel;

    impl Channel {
        pub async fn connect<D>(_dst: D) -> Result<Self, Error>
        where
            D: TryInto<Endpoint>,
            D::Error: Into<crate::codegen::StdError>,
        {
            Err(Error::new(
                "tonic stubs cannot establish transport channels",
            ))
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct Endpoint;

    impl Endpoint {
        pub fn new<D>(_dst: D) -> Result<Self, Error>
        where
            D: TryInto<Uri>,
            D::Error: Into<crate::codegen::StdError>,
        {
            Ok(Self)
        }

        pub fn connect_lazy(self) -> Channel {
            Channel
        }

        pub async fn connect(self) -> Result<Channel, Error> {
            Channel::connect(self)
        }

        pub fn tls_config(self, _config: ClientTlsConfig) -> Result<Self, Error> {
            Ok(self)
        }
    }

    impl From<Uri> for Endpoint {
        fn from(_value: Uri) -> Self {
            Self
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct ClientTlsConfig;

    impl ClientTlsConfig {
        pub fn new() -> Self {
            Self
        }

        pub fn domain_name(self, _name: impl Into<String>) -> Self {
            self
        }

        pub fn ca_certificate(self, _cert: Certificate) -> Self {
            self
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct Certificate;

    impl Certificate {
        pub fn from_pem(_pem: impl AsRef<[u8]>) -> Result<Self, Error> {
            Ok(Self)
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct Identity;

    impl Identity {
        pub fn from_pem(
            _cert: impl AsRef<[u8]>,
            _key: impl AsRef<[u8]>,
        ) -> Identity {
            Identity
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct Server;

    impl Server {
        pub fn builder() -> Self {
            Self
        }

        pub fn tls_config(self, _config: ServerTlsConfig) -> Result<Self, Error> {
            Ok(self)
        }

        pub fn add_service<T>(self, _service: T) -> Self {
            self
        }

        pub async fn serve(self, _addr: SocketAddr) -> Result<(), Error> {
            Err(Error::new(
                "tonic stubs cannot listen for incoming connections",
            ))
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct ServerTlsConfig;

    impl ServerTlsConfig {
        pub fn new() -> Self {
            Self
        }

        pub fn identity(self, _identity: Identity) -> Self {
            self
        }
    }

    pub mod server {
        #[derive(Clone, Debug, Default)]
        pub struct TcpConnectInfo;
    }
}

pub mod codegen {
    pub use super::body::Body;
    pub use super::codec::CompressionEncoding;
    pub use super::metadata::{Ascii, AsciiMetadataValue, KeyAndValueRef, MetadataMap};
    pub use super::service::Interceptor;
    pub use super::{Code, Request, Response, Status};
    pub use bytes::Bytes;
    pub use http;
    pub use http::uri::PathAndQuery;
    pub use http::Uri;
    pub use std::future::Future;
    pub use std::pin::Pin;
    pub use std::sync::Arc;
    pub use std::task::{Context, Poll};
    pub use tower_service::Service;

    pub type StdError = Box<dyn std::error::Error + Send + Sync + 'static>;

    pub type BoxFuture<T, E = super::Status> =
        Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'static>>;

    #[derive(Clone)]
    pub struct InterceptedService<S, I> {
        inner: S,
        interceptor: I,
    }

    impl<S, I> InterceptedService<S, I> {
        pub fn new(inner: S, interceptor: I) -> Self {
            Self { inner, interceptor }
        }

        pub fn into_inner(self) -> (S, I) {
            (self.inner, self.interceptor)
        }
    }
}

pub use codegen::InterceptedService;
pub use server::GrpcMethod;
