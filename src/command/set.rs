use std::io::Cursor;

use tokio::io::AsyncWriteExt;

use crate::{
    Result,
    command::{self, Error},
    message,
};

#[derive(Debug)]
pub struct Set {
    pub key: String,
    pub val: String,
}

impl Set {
    pub async fn parse(src: &mut Cursor<&[u8]>) -> Result<Set> {
        let count = command::read_count(src).await?;
        if count != 2 {
            return Err(crate::Error::ParseCommand(Error::WrongNumberArguments));
        }
        let key = message::read_string(src).await?;
        let val = message::read_string(src).await?;
        Ok(Set { key, val })
    }

    pub async fn write<W: AsyncWriteExt + Unpin>(&self, buf: &mut W) -> crate::Result<()> {
        buf.write_u16(self.key.len() as u16).await?;
        buf.write_all(self.key.as_bytes()).await?;
        Ok(())
    }
}
