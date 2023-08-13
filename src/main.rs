use std::io::{Read, Write};
use std::net::TcpListener;

mod command;
mod resp;
mod server;

use server::Server;

fn main() {
    let host_port = "127.0.0.1:6379";
    let listener = TcpListener::bind(host_port).unwrap();
    let mut server = Server::new();
    println!("Server started at {host_port}");
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
                    let result = server.handle_command(command);
                    stream.write(&result.as_bytes()).unwrap();
                }
            }
            Err(err) => {
                println!("ERROR IN STREAM: {err:?}");
                break;
            }
        }
    }
}
