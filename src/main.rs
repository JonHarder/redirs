use chrono::Utc;
use std::fmt::Display;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::time::Instant;

mod command;
mod resp;
mod server;

use server::Server;

#[derive(Clone, Copy)]
enum LogLevel {
    Debug,
    Info,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LogLevel::Info => "INFO",
            LogLevel::Error => "ERROR",
            LogLevel::Debug => "DEBUG",
        };
        write!(f, "{s}")
    }
}

fn log(level: LogLevel, msg: &str) {
    let now = Utc::now();
    let level = format!("[{level}]");
    eprintln!("{now} {level} - {msg}", now = now.to_rfc3339());
}

fn handle_stream(stream: &mut TcpStream, server: &RwLock<Server>) {
    let mut buf = vec![0; 512];
    let bytes_read = stream.read(&mut buf).unwrap();
    if bytes_read == 0 {
        return;
    }
    let message = String::from_utf8(buf).unwrap();
    let commands = command::parse_message(message).unwrap();
    for command in commands {
        log(
            LogLevel::Info,
            format!(
                "Processing command: {:?}, args: {:?}",
                command.keyword, command.args
            )
            .as_str(),
        );
        let now = Instant::now();
        let result = if command.mutable {
            log(LogLevel::Debug, "locking server for mutable command");
            let result = server.write().unwrap().handle_command_mut(command);
            log(LogLevel::Debug, "unlocking server");
            result
        } else {
            log(LogLevel::Debug, "non blocking command");
            server.read().unwrap().handle_command(command)
        };
        log(
            LogLevel::Info,
            format!("Result computed in {:?}", now.elapsed()).as_str(),
        );
        match &result {
            resp::Resp::Error(e) => log(LogLevel::Error, e.as_str()),
            r => log(LogLevel::Info, format!("Result: {r:?}").as_str()),
        }
        stream.write_all(&result.as_bytes()).unwrap();
    }
}

fn main() {
    let binding = SocketAddr::from(([127, 0, 0, 1], 6379));
    let listener = TcpListener::bind(binding).unwrap();
    let server = Arc::new(RwLock::new(Server::new()));
    let mut threads = vec![];
    log(
        LogLevel::Info,
        format!("Server started at {binding}").as_str(),
    );
    for stream in listener.incoming() {
        log(LogLevel::Debug, "new incoming request");
        match stream {
            Ok(mut stream) => {
                let server = Arc::clone(&server);
                let handle = std::thread::spawn(move || loop {
                    handle_stream(&mut stream, &server);
                });
                log(LogLevel::Debug, "spawned new thread to handle request");
                threads.push(handle);
            }
            Err(err) => {
                println!("ERROR IN STREAM: {err:?}");
                break;
            }
        }
        log(LogLevel::Debug, "client closed the connection");
    }
    for thread in threads {
        thread.join().unwrap();
    }
    log(LogLevel::Info, "exiting");
}
