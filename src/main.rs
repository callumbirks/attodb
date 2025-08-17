use std::{collections::BTreeMap, sync::Arc};

use tokio::{
    io::{self},
    net::{TcpListener, TcpStream},
    sync::RwLock,
};

use crate::connection::{Command, Connection};

mod connection;

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

async fn process(db: Arc<RwLock<BTreeMap<String, String>>>, socket: TcpStream) -> io::Result<()> {
    let mut connection = Connection::new(socket);
    let command = connection.read_command().await.unwrap();
    println!("Received command: {:?}", &command);
    let response = match command {
        Command::Get(key) => {
            let db = db.read().await;
            match db.get(&key) {
                Some(val) => format!("{}\n", &val),
                None => format!("No such key '{}'\n", &key),
            }
        }
        Command::Set(key, val) => {
            let mut db = db.write().await;
            db.insert(key, val);
            "OK\n".to_string()
        }
    };
    connection.respond(&response).await.unwrap();
    Ok(())
}
