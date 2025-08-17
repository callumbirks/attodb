use std::{collections::BTreeMap, sync::Arc};

use attodb::{command::Command, connection::Connection, message::Message};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::RwLock,
};

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

async fn process(
    db: Arc<RwLock<BTreeMap<String, String>>>,
    socket: TcpStream,
) -> attodb::Result<()> {
    let mut connection = Connection::new(socket);
    let message = connection.read_message().await;
    println!("Received message: {:?}", &message);
    match message {
        Ok(Some(Message::Ping)) => {
            connection.write_message(Message::Ok).await?;
        }
        Ok(Some(Message::Command(Command::Get(get)))) => {
            let db = db.read().await;
            match db.get(&get.key) {
                Some(val) => connection.write_message(Message::Text(val.clone())).await?,
                None => connection.write_message(Message::Null).await?,
            }
        }
        Ok(Some(Message::Command(Command::Set(set)))) => {
            let mut db = db.write().await;
            db.insert(set.key, set.value);
            connection.write_message(Message::Ok).await?;
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
