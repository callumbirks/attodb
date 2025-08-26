use std::{io::Cursor, sync::Arc};

use dashmap::DashMap;
use tokio::io::AsyncWriteExt;

use crate::{
    Message,
    command::{self, Error},
    message,
};

#[derive(Debug)]
pub struct Del {
    pub key: String,
}

impl Del {
    pub fn perform(self, db: Arc<DashMap<String, Vec<u8>>>) -> crate::Result<Message> {
        match db.remove(&self.key) {
            Some(_) => Ok(Message::Ok),
            None => Ok(Message::Null),
        }
    }

    pub async fn parse(src: &mut Cursor<&[u8]>) -> crate::Result<Del> {
        let count = command::read_count(src).await?;
        if count != 1 {
            return Err(crate::Error::ParseCommand(Error::WrongNumberArguments));
        }
        let key = message::read_string(src).await?;
        Ok(Del { key })
    }

    pub async fn write<W: AsyncWriteExt + Unpin>(&self, buf: &mut W) -> crate::Result<()> {
        buf.write_u8(1).await?;
        message::write_string(buf, &self.key).await?;
        Ok(())
    }
}
