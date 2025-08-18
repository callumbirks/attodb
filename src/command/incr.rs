use std::io::Cursor;

use tokio::io::AsyncWriteExt;

use crate::{command, message};

#[derive(Debug)]
pub struct Incr {
    pub key: String,
}

impl Incr {
    pub async fn parse(src: &mut Cursor<&[u8]>) -> crate::Result<Incr> {
        let count = command::read_count(src).await?;
        if count != 1 {
            return Err(crate::Error::ParseCommand(
                command::Error::WrongNumberArguments,
            ));
        }
        let key = message::read_string(src).await?;
        Ok(Incr { key })
    }

    pub async fn write<W: AsyncWriteExt + Unpin>(&self, buf: &mut W) -> crate::Result<()> {
        buf.write_u8(1).await?;
        message::write_string(buf, &self.key).await?;
        Ok(())
    }
}
