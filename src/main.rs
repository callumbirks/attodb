use std::{collections::BTreeMap, sync::Arc};

use thiserror::Error;
use tokio::{
    io::{self},
    net::{TcpListener, TcpStream},
    sync::RwLock,
};

use crate::connection::Connection;
use crate::{command::Command, message::Message};

mod command;
mod connection;
mod message;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error")]
    Io(#[from] io::Error),
    #[error("invalid utf8")]
    InvalidUtf8,
    #[error("connection reset by peer")]
    ConnectionReset,
    #[error("failed to parse message: {0:?}")]
    ParseMessage(message::Error),
    #[error("failed to parse command: {0:?}")]
    ParseCommand(command::Error),
}

pub type Result<T> = core::result::Result<T, Error>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:7676").await.unwrap();
    let db = Arc::new(RwLock::new(BTreeMap::<String, String>::new()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db = db.clone();
        tokio::spawn(async move {
            process(db, socket).await.unwrap();
        });
    }
}

async fn process(db: Arc<RwLock<BTreeMap<String, String>>>, socket: TcpStream) -> Result<()> {
    let mut connection = Connection::new(socket);
    let message = connection.read_message().await;
    println!("Received message: {:?}", &message);
    match message {
        Ok(Some(Message::Ping)) => {
            connection.respond(Message::Ok).await?;
        }
        Ok(Some(Message::Command(Command::Get(get)))) => {
            let db = db.read().await;
            match db.get(&get.key) {
                Some(val) => connection.respond(Message::Text(val.clone())).await?,
                None => connection.respond(Message::Null).await?,
            }
        }
        Ok(Some(Message::Command(Command::Set(set)))) => {
            let mut db = db.write().await;
            db.insert(set.key, set.val);
            connection.respond(Message::Ok).await?;
        }
        // None means the connection closed gracefully
        Ok(None) => {}
        Err(err) => match err {
            Error::InvalidUtf8 | Error::ParseMessage(_) | Error::ParseCommand(_) => {
                connection.respond(Message::Err(err.to_string())).await?
            }
            _ => return Err(err),
        },
        _ => {}
    };
    Ok(())
}
