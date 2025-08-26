use std::io::Cursor;

use tokio::io::AsyncReadExt;

use crate::Result;

mod del;
mod get;
mod incr;
mod set;

pub use del::Del;
pub use get::Get;
pub use incr::Incr;
pub use set::Set;

#[derive(Debug)]
pub enum Command {
    Get(Get),
    Set(Set),
    Incr(Incr),
    Del(Del),
}

#[repr(u8)]
pub enum Variant {
    Get = 0,
    Set = 1,
    Incr = 2,
    Del = 3,
}

#[derive(Debug)]
pub enum Error {
    UnknownCommandType(u8),
    WrongNumberArguments,
}

impl TryFrom<u8> for Variant {
    type Error = Error;

    fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Variant::Get),
            1 => Ok(Variant::Set),
            2 => Ok(Variant::Incr),
            3 => Ok(Variant::Del),
            _ => Err(Error::UnknownCommandType(value)),
        }
    }
}

impl Command {
    pub async fn parse(src: &mut Cursor<&[u8]>) -> Result<Command> {
        let variant_byte = src.read_u8().await?;
        let variant = match Variant::try_from(variant_byte) {
            Ok(v) => v,
            Err(e) => return Err(crate::Error::ParseCommand(e)),
        };
        match variant {
            Variant::Get => Get::parse(src).await.map(Command::Get),
            Variant::Set => Set::parse(src).await.map(Command::Set),
            Variant::Incr => Incr::parse(src).await.map(Command::Incr),
            Variant::Del => Del::parse(src).await.map(Command::Del),
        }
    }

    pub async fn write<W: tokio::io::AsyncWriteExt + std::marker::Unpin>(
        &self,
        buf: &mut W,
    ) -> Result<()> {
        match self {
            Command::Get(get) => {
                buf.write_u8(Variant::Get as u8).await?;
                get.write(buf).await?;
                Ok(())
            }
            Command::Set(set) => {
                buf.write_u8(Variant::Set as u8).await?;
                set.write(buf).await?;
                Ok(())
            }
            Command::Incr(incr) => {
                buf.write_u8(Variant::Incr as u8).await?;
                incr.write(buf).await?;
                Ok(())
            }
            Command::Del(del) => {
                buf.write_u8(Variant::Del as u8).await?;
                del.write(buf).await?;
                Ok(())
            }
        }
    }
}

pub async fn read_count(src: &mut Cursor<&[u8]>) -> Result<u8> {
    match src.read_u8().await {
        Ok(n) => Ok(n),
        Err(e) => Err(e.into()),
    }
}
