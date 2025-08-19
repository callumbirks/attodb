use std::sync::Arc;

use attodb::{command::Command, connection::Connection, message::Message};
use dashmap::DashMap;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:7676").await.unwrap();
    let db: Arc<DashMap<String, Vec<u8>>> = Arc::new(DashMap::new());

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db = db.clone();
        tokio::spawn(async move {
            process(db, socket).await.unwrap();
        });
    }
}

async fn process(db: Arc<DashMap<String, Vec<u8>>>, socket: TcpStream) -> attodb::Result<()> {
    let mut connection = Connection::new(socket);
    let message = connection.read_message().await;
    println!("Received message: {:?}", &message);
    match message {
        Ok(Some(Message::Ping)) => {
            connection.write_message(Message::Ok).await?;
        }
        Ok(Some(Message::Command(Command::Get(get)))) => {
            let message = get.perform(db)?;
            connection.write_message(message).await?;
        }
        Ok(Some(Message::Command(Command::Set(set)))) => {
            let message = set.perform(db)?;
            connection.write_message(message).await?;
        }
        Ok(Some(Message::Command(Command::Incr(incr)))) => {
            let message = incr.perform(db)?;
            connection.write_message(message).await?;
        }
        // None means the connection closed gracefully
        Ok(None) => {}
        Err(err) => match err {
            attodb::Error::InvalidUtf8
            | attodb::Error::ParseMessage(_)
            | attodb::Error::ParseCommand(_) => {
                connection
                    .write_message(Message::Err(err.to_string()))
                    .await?
            }
            _ => return Err(err),
        },
        _ => {}
    };
    Ok(())
}
