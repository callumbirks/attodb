use attodb::{DEFAULT_PORT, connection::Connection, message::Message, value::Value};
use clap::{Parser, Subcommand};
use tokio::net::TcpStream;

#[derive(Parser, Debug)]
struct Cli {
    #[clap(subcommand)]
    command: Command,

    #[arg(id = "hostname", long, default_value = "127.0.0.1")]
    host: String,

    #[arg(long, default_value_t = DEFAULT_PORT)]
    port: u16,
}

#[derive(Subcommand, Debug)]
enum Command {
    Ping,
    Get { key: String },
    Set { key: String, value: String },
    Incr { key: String },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> attodb::Result<()> {
    let cli = Cli::parse();
    let addr = format!("{}:{}", cli.host, cli.port);
    let socket = TcpStream::connect(addr).await?;
    let mut connection = Connection::new(socket);
    match cli.command {
        Command::Ping => connection.write_message(Message::Ping).await?,
        Command::Get { key } => {
            connection
                .write_message(Message::Command(attodb::Command::Get(
                    attodb::command::Get { key },
                )))
                .await?
        }
        Command::Set { key, value } => {
            connection
                .write_message(Message::Command(attodb::Command::Set(
                    attodb::command::Set {
                        key,
                        value: Value::String(&value).into_vec(),
                    },
                )))
                .await?
        }
        Command::Incr { key } => {
            connection
                .write_message(Message::Command(attodb::Command::Incr(
                    attodb::command::Incr { key },
                )))
                .await?
        }
    }
    match connection.read_message().await? {
        Some(message) => {
            println!("{message:?}");
        }
        None => {}
    }
    Ok(())
}
