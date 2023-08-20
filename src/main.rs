use chrono::Utc;
use std::fmt::Display;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::time::Instant;

mod command;
mod resp;
mod server;

use server::Server;

#[derive(Clone, Copy)]
enum LogLevel {
    Info,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LogLevel::Info => "INFO",
            LogLevel::Error => "ERROR",
        };
        write!(f, "{s}")
    }
}

fn log(level: LogLevel, msg: String) {
    let now = Utc::now();
    eprintln!("{now} [{level}] - {msg}", now = now.to_rfc3339());
}

fn main() {
    let binding = SocketAddr::from(([127, 0, 0, 1], 6379));
    let listener = TcpListener::bind(binding).unwrap();
    let mut server = Server::new();
    println!("Server started at {binding}");
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf = vec![0; 512];
                let bytes_read = stream.read(&mut buf).unwrap();
                if bytes_read == 0 {
                    break;
                }
                let message = String::from_utf8(buf).unwrap();
                let commands = command::parse_message(message);
                for command in commands {
                    log(
                        LogLevel::Info,
                        format!(
                            "Processing command: {:?}, args: {:?}",
                            command.keyword, command.args
                        ),
                    );
                    let now = Instant::now();
                    let result = server.handle_command(command);
                    log(
                        LogLevel::Info,
                        format!("Result computed in {:?}", now.elapsed()),
                    );
                    match &result {
                        resp::Resp::Error(e) => log(LogLevel::Error, e.to_string()),
                        r => log(LogLevel::Info, format!("{r:?}")),
                    }
                    stream.write_all(&result.as_bytes()).unwrap();
                }
            }
            Err(err) => {
                println!("ERROR IN STREAM: {err:?}");
                break;
            }
        }
    }
}
