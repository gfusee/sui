use std::io::Result as IoResult;
use std::marker::PhantomData;
use std::time::Duration;

/// Minimal placeholder for [`socket2::TcpKeepalive`].
#[derive(Clone, Debug, Default)]
pub struct TcpKeepalive {
    time: Option<Duration>,
}

impl TcpKeepalive {
    pub fn new() -> Self {
        Self { time: None }
    }

    pub fn with_time(mut self, time: Duration) -> Self {
        self.time = Some(time);
        self
    }

    pub fn with_interval(self, _interval: Duration) -> Self {
        self
    }

    pub fn with_retries(self, _retries: u32) -> Self {
        self
    }
}

/// Minimal placeholder for [`socket2::SockRef`].
#[derive(Clone, Copy, Debug)]
pub struct SockRef<'a, T> {
    _inner: &'a T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> SockRef<'a, T> {
    pub fn from(inner: &'a T) -> Self {
        Self {
            _inner: inner,
            _marker: PhantomData,
        }
    }

    pub fn set_tcp_keepalive(&self, _keepalive: &TcpKeepalive) -> IoResult<()> {
        Ok(())
    }
}
