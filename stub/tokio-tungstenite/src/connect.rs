//! Connection helper.

use tokio::net::TcpStream;

use tungstenite::{
    error::Error,
    handshake::client::{Request, Response},
    protocol::WebSocketConfig,
};

use crate::{stream::MaybeTlsStream, Connector, IntoClientRequest, WebSocketStream};

/// Connect to a given URL.
///
/// Accepts any request that implements [`IntoClientRequest`], which is often just `&str`, but can
/// be a variety of types such as `httparse::Request` or [`tungstenite::http::Request`] for more
/// complex uses.
///
/// ```no_run
/// # use tungstenite::client::IntoClientRequest;
///
/// # async fn test() {
/// use tungstenite::http::{Method, Request};
/// use tokio_tungstenite::connect_async;
///
/// let mut request = "wss://api.example.com".into_client_request().unwrap();
/// request.headers_mut().insert("api-key", "42".parse().unwrap());
///
/// let (stream, response) = connect_async(request).await.unwrap();
/// # }
/// ```
pub async fn connect_async<R>(
    request: R,
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), Error>
where
    R: IntoClientRequest + Unpin,
{
    connect_async_with_config(request, None, false).await
}

/// The same as `connect_async()` but the one can specify a websocket configuration.
/// Please refer to `connect_async()` for more details. `disable_nagle` specifies if
/// the Nagle's algorithm must be disabled, i.e. `set_nodelay(true)`. If you don't know
/// what the Nagle's algorithm is, better leave it set to `false`.
pub async fn connect_async_with_config<R>(
    request: R,
    config: Option<WebSocketConfig>,
    disable_nagle: bool,
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), Error>
where
    R: IntoClientRequest + Unpin,
{
    connect(request.into_client_request()?, config, disable_nagle, None).await
}

/// The same as `connect_async()` but the one can specify a websocket configuration,
/// and a TLS connector to use. Please refer to `connect_async()` for more details.
/// `disable_nagle` specifies if the Nagle's algorithm must be disabled, i.e.
/// `set_nodelay(true)`. If you don't know what the Nagle's algorithm is, better
/// leave it to `false`.
#[cfg(any(feature = "native-tls", feature = "__rustls-tls"))]
pub async fn connect_async_tls_with_config<R>(
    request: R,
    config: Option<WebSocketConfig>,
    disable_nagle: bool,
    connector: Option<Connector>,
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), Error>
where
    R: IntoClientRequest + Unpin,
{
    connect(request.into_client_request()?, config, disable_nagle, connector).await
}

async fn connect(
    request: Request,
    config: Option<WebSocketConfig>,
    disable_nagle: bool,
    connector: Option<Connector>,
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), Error> {
    Result::Err(Error::Utf8("should not be called".to_string()))
}
