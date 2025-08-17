use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct Connection {
    socket: TcpStream,
}

#[derive(Debug)]
pub enum Error {
    InvalidFrame,
    InvalidUtf8,
    Io(io::Error),
}

#[derive(Debug)]
pub enum Command {
    Get(String),
    Set(String, String),
}

impl Command {
    pub fn parse(bytes: &[u8]) -> Result<Command, Error> {
        let string = match str::from_utf8(bytes) {
            Ok(string) => string,
            Err(_) => return Err(Error::InvalidUtf8),
        };
        let words: Vec<&str> = string.split(|c| c == ' ').collect();
        if words.is_empty() {
            return Err(Error::InvalidFrame);
        }
        let command_type = words[0].to_ascii_lowercase();
        match command_type.as_str() {
            "get" => {
                if words.len() < 2 {
                    return Err(Error::InvalidFrame);
                }
                let key = words[1].trim_end_matches(|c| c == '\n').to_string();
                Ok(Command::Get(key))
            }
            "set" => {
                if words.len() < 3 {
                    return Err(Error::InvalidFrame);
                }
                let key = words[1].to_string();
                let val = words[2].trim_end_matches(|c| c == '\n').to_string();
                Ok(Command::Set(key, val))
            }
            _ => Err(Error::InvalidFrame),
        }
    }
}

impl Connection {
    pub fn new(socket: TcpStream) -> Connection {
        Connection { socket }
    }

    pub async fn read_command(&mut self) -> Result<Command, Error> {
        let mut buf = [0u8; 256];
        let count = match self.socket.read(&mut buf).await {
            Ok(count) => count,
            Err(err) => return Err(Error::Io(err)),
        };
        let bytes = &buf[..count];
        Command::parse(bytes)
    }

    pub async fn respond(&mut self, response: &str) -> io::Result<()> {
        self.socket.write_all(response.as_bytes()).await
    }
}
