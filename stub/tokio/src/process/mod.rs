use std::io;
use std::io::ErrorKind;
use std::pin::Pin;
use std::process::Output;
use std::task::{Context, Poll};
use crate::io::{AsyncRead, AsyncWrite, ReadBuf};

#[derive(Debug)]
pub struct Command;

pub struct Child {
    pub stdin: Option<Stdio>,
    pub stdout: Option<Stdio>,
}

pub struct Stdio;

impl Stdio {
    pub fn piped() -> Stdio {
        Stdio
    }
}

impl AsyncRead for Stdio {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Err(io::Error::new(ErrorKind::Unsupported, "should not be called")))
    }
}

impl AsyncWrite for Stdio {
    fn poll_write(self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, io::Error>> {
        Poll::Ready(Err(io::Error::new(ErrorKind::Unsupported, "should not be called")))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Err(io::Error::new(ErrorKind::Unsupported, "should not be called")))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Err(io::Error::new(ErrorKind::Unsupported, "should not be called")))
    }
}

impl Command {
    pub fn new(_command: &str) -> Self {
        Command
    }

    pub fn arg(self, _arg: &str) -> Command {
        self
    }

    pub fn stdin(self, _stdio: std::process::Stdio) -> Command {
        self
    }

    pub fn stdout(self, _stdio: std::process::Stdio) -> Command {
        self
    }

    pub fn spawn(&mut self) -> io::Result<Child> {
        Err(io::Error::new(ErrorKind::Unsupported, "should not be called"))
    }
}

impl Child {
    pub async fn wait_with_output(mut self) -> io::Result<Output> {
        Err(io::Error::new(ErrorKind::Unsupported, "should not be called"))
    }
}