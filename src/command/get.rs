use std::{io::Cursor, sync::Arc};

use dashmap::DashMap;
use tokio::io::AsyncWriteExt;

use crate::{
    Message,
    command::{self, Error},
    message,
    value::Value,
};

#[derive(Debug)]
pub struct Get {
    pub key: String,
}

impl Get {
    pub fn perform(self, db: Arc<DashMap<String, Vec<u8>>>) -> crate::Result<Message> {
        match db.get(&self.key) {
            Some(val) => {
                let value = Value::parse(val.as_ref())?;
                match value {
                    Value::Int(int) => Ok(Message::Int(int)),
                    Value::String(string) => Ok(Message::Text(string.to_string())),
                }
            }
            None => Ok(Message::Null),
        }
    }

    pub async fn parse(src: &mut Cursor<&[u8]>) -> crate::Result<Get> {
        let count = command::read_count(src).await?;
        if count != 1 {
            return Err(crate::Error::ParseCommand(Error::WrongNumberArguments));
        }
        let key = message::read_string(src).await?;
        Ok(Get { key })
    }

    pub async fn write<W: AsyncWriteExt + Unpin>(&self, buf: &mut W) -> crate::Result<()> {
        buf.write_u8(1).await?;
        buf.write_u16(self.key.len() as u16).await?;
        buf.write_all(self.key.as_bytes()).await?;
        Ok(())
    }
}
