use std::{io::Cursor, sync::Arc};

use dashmap::DashMap;
use tokio::io::AsyncWriteExt;

use crate::{Message, command, message, value::Value};

#[derive(Debug)]
pub struct Incr {
    pub key: String,
}

impl Incr {
    pub fn perform(self, db: Arc<DashMap<String, Vec<u8>>>) -> crate::Result<Message> {
        let e = db
            .entry(self.key)
            .and_modify(|e| {
                if let Ok(Value::Int(int)) = Value::parse(&e) {
                    Value::Int(int + 1).write(e);
                }
            })
            .or_insert_with(|| Value::Int(1).into_vec());
        if let Ok(Value::Int(int)) = Value::parse(&e) {
            Ok(Message::Int(int))
        } else {
            Ok(Message::Err("not a number".to_string()))
        }
    }

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
