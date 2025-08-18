use std::sync::Arc;

use attodb::{command::Command, connection::Connection, message::Message, value::Value};
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
        Ok(Some(Message::Command(Command::Get(get)))) => match db.get(&get.key) {
            Some(val) => {
                let value = Value::parse(val.as_ref());
                match value {
                    Ok(Value::Int(int)) => connection.write_message(Message::Int(int)).await?,
                    Ok(Value::String(string)) => {
                        connection
                            .write_message(Message::Text(string.to_string()))
                            .await?
                    }
                    Err(e) => {
                        connection
                            .write_message(Message::Err("internal encoding error".to_string()))
                            .await?
                    }
                }
            }
            None => connection.write_message(Message::Null).await?,
        },
        Ok(Some(Message::Command(Command::Set(set)))) => {
            db.insert(set.key, set.value);
            connection.write_message(Message::Ok).await?;
        }
        Ok(Some(Message::Command(Command::Incr(incr)))) => {
            let e = db
                .entry(incr.key)
                .and_modify(|e| {
                    if let Ok(Value::Int(int)) = Value::parse(&e) {
                        Value::Int(int + 1).write(e);
                    }
                })
                .or_insert_with(|| Value::Int(1).into_vec());
            if let Ok(Value::Int(int)) = Value::parse(&e) {
                connection.write_message(Message::Int(int)).await?;
            } else {
                connection
                    .write_message(Message::Err("not a number".to_string()))
                    .await?;
            }
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
