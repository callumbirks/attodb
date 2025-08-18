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
    pub value: Vec<u8>,
}

impl Set {
    pub async fn parse(src: &mut Cursor<&[u8]>) -> Result<Set> {
        let count = command::read_count(src).await?;
        if count != 2 {
            return Err(crate::Error::ParseCommand(Error::WrongNumberArguments));
        }
        let key = message::read_string(src).await?;
        let value = message::read_bytes(src).await?;
        Ok(Set { key, value })
    }

    pub async fn write<W: AsyncWriteExt + Unpin>(&self, buf: &mut W) -> crate::Result<()> {
        buf.write_u8(2).await?;
        message::write_string(buf, &self.key).await?;
        message::write_bytes(buf, &self.value).await?;
        Ok(())
    }
}
