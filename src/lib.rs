use thiserror::Error;
use tokio::io::{self};

pub mod command;
pub mod connection;
pub mod message;

pub use command::Command;
pub use connection::Connection;
pub use message::Message;

pub const DEFAULT_PORT: u16 = 7676;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error")]
    Io(#[from] io::Error),
    #[error("invalid utf8")]
    InvalidUtf8,
    #[error("connection reset by peer")]
    ConnectionReset,
    #[error("failed to parse message: {0:?}")]
    ParseMessage(message::Error),
    #[error("failed to parse command: {0:?}")]
    ParseCommand(command::Error),
}

pub type Result<T> = core::result::Result<T, Error>;
