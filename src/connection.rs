use std::io::Cursor;

use bytes::{Buf, BytesMut};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

use crate::{
    Result,
    message::{self, Message},
};

pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

#[derive(Debug)]
pub enum Error {
    InvalidFrame,
    Io(io::Error),
}

impl Connection {
    pub fn new(socket: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub async fn read_message(&mut self) -> Result<Option<Message>> {
        loop {
            if let Some(message) = self.parse_message().await? {
                return Ok(Some(message));
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(crate::Error::ConnectionReset);
                }
            }
        }
    }

    async fn parse_message(&mut self) -> Result<Option<Message>> {
        let mut buf = Cursor::new(&self.buffer[..]);

        if message::is_complete(&mut buf) {
            let len = buf.position() as usize;
            buf.set_position(0);
            let message = Message::parse(&mut buf).await?;
            self.buffer.advance(len);
            Ok(Some(message))
        } else {
            Ok(None)
        }
    }

    pub async fn write_message(&mut self, message: Message) -> crate::Result<()> {
        message.write(&mut self.stream).await?;
        self.stream.flush().await?;
        Ok(())
    }
}
