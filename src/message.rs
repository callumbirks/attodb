// VARIANT COMMAND [LENGTH BYTES]...
//   00      1a     ffff    ...

// Variants
// COMMAND = 0
// OK = 1
// NULL = 2
// INT = 3
// TEXT = 4
// ERR = 5

// Message
// COMMAND = CMD(8) COUNT(8) [LENGTH(16) BYTES]...
// OK = _
// NULL = _
// INT = INT(32)
// TEXT = LENGTH(16) BYTES
// ERR = LENGTH(16) BYTES

use bytes::Buf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::command::Command;
use std::io::Cursor;

#[repr(u8)]
pub enum Variant {
    Ping = 0,
    Command = 1,
    Ok = 2,
    Null = 3,
    Err = 4,
    Int = 5,
    Text = 6,
}

#[derive(Debug)]
pub enum Error {
    Incomplete,
    UnknownMessageType(u8),
    StringTooLarge,
}

impl TryFrom<u8> for Variant {
    type Error = Error;

    fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Variant::Ping),
            1 => Ok(Variant::Command),
            2 => Ok(Variant::Ok),
            3 => Ok(Variant::Null),
            4 => Ok(Variant::Err),
            5 => Ok(Variant::Int),
            6 => Ok(Variant::Text),
            _ => Err(Error::UnknownMessageType(value)),
        }
    }
}

#[derive(Debug)]
pub enum Message {
    Ping,
    Command(Command),
    Ok,
    Null,
    Err(String),
    Int(i32),
    Text(String),
}

pub async fn read_string(src: &mut Cursor<&[u8]>) -> crate::Result<String> {
    let count = src.read_u16().await?;
    let mut buf = vec![0_u8; count as usize];
    src.read_exact(buf.as_mut_slice()).await?;
    match String::from_utf8(buf) {
        Ok(s) => Ok(s),
        Err(_) => Err(crate::Error::InvalidUtf8),
    }
}

pub async fn write_string<W: AsyncWriteExt + Unpin>(buf: &mut W, value: &str) -> crate::Result<()> {
    // TODO: Validate length
    buf.write_u16(value.len() as u16).await?;
    buf.write_all(value.as_bytes()).await?;
    Ok(())
}

pub async fn read_int(src: &mut Cursor<&[u8]>) -> crate::Result<i32> {
    Ok(src.read_i32().await?)
}

impl Message {
    pub async fn parse(src: &mut Cursor<&[u8]>) -> crate::Result<Message> {
        let mut line = Cursor::new(match read_line(src) {
            Ok(l) => l,
            Err(e) => return Err(crate::Error::ParseMessage(e)),
        });
        let variant_byte = line.read_u8().await?;
        let variant = match Variant::try_from(variant_byte) {
            Ok(v) => v,
            Err(e) => return Err(crate::Error::ParseMessage(e)),
        };
        match variant {
            Variant::Ping => Ok(Message::Ping),
            Variant::Command => Command::parse(&mut line).await.map(Message::Command),
            Variant::Ok => Ok(Message::Ok),
            Variant::Null => Ok(Message::Null),
            Variant::Err => read_string(&mut line).await.map(Message::Err),
            Variant::Int => read_int(&mut line).await.map(Message::Int),
            Variant::Text => read_string(&mut line).await.map(Message::Text),
        }
    }

    pub async fn write<W: tokio::io::AsyncWriteExt + std::marker::Unpin>(
        &self,
        buf: &mut W,
    ) -> crate::Result<()> {
        match self {
            Message::Ping => {
                buf.write_u8(Variant::Ping as u8).await?;
            }
            Message::Command(command) => {
                buf.write_u8(Variant::Command as u8).await?;
                command.write(buf).await?;
            }
            Message::Ok => {
                buf.write_u8(Variant::Ok as u8).await?;
            }
            Message::Null => {
                buf.write_u8(Variant::Null as u8).await?;
            }
            Message::Err(text) => {
                buf.write_u8(Variant::Err as u8).await?;
                write_string(buf, text).await?;
            }
            Message::Int(int) => {
                buf.write_u8(Variant::Int as u8).await?;
                buf.write_i32(*int).await?;
            }
            Message::Text(text) => {
                buf.write_u8(Variant::Text as u8).await?;
                write_string(buf, text).await?;
            }
        }
        buf.write_u8(b'\r').await?;
        buf.write_u8(b'\n').await?;
        Ok(())
    }
}

pub fn is_complete(src: &mut Cursor<&[u8]>) -> bool {
    read_line(src).is_ok()
}

fn read_line<'a>(src: &mut Cursor<&'a [u8]>) -> core::result::Result<&'a [u8], Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            src.set_position((i + 2) as u64);
            return Ok(&src.get_ref()[start..i]);
        }
    }
    Err(Error::Incomplete)
}
