use std::io::Cursor;

use tokio::io::AsyncWriteExt;

use crate::{
    Result,
    command::{self, Error},
    message,
};

#[derive(Debug)]
pub struct Get {
    pub key: String,
}

impl Get {
    pub async fn parse(src: &mut Cursor<&[u8]>) -> Result<Get> {
        let count = command::read_count(src).await?;
        if count != 1 {
            return Err(crate::Error::ParseCommand(Error::WrongNumberArguments));
        }
        let key = message::read_string(src).await?;
        Ok(Get { key })
    }

    pub async fn write<W: AsyncWriteExt + Unpin>(&self, buf: &mut W) -> crate::Result<()> {
        buf.write_u16(self.key.len() as u16).await?;
        buf.write_all(self.key.as_bytes()).await?;
        Ok(())
    }
}
