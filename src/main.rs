use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};

mod resp;

use resp::Resp;

struct Memory {
    storage: HashMap<String, String>,
}

impl Memory {
    fn new() -> Self {
        Memory {
            storage: HashMap::new(),
        }
    }

    fn get(&self, key: &String) -> Option<&String> {
        self.storage.get(key)
    }

    fn set(&mut self, key: String, val: String) {
        self.storage.insert(key, val);
    }
}

fn main() {
    let host_port = "127.0.0.1:6379";
    let listener = TcpListener::bind(host_port).unwrap();
    let mut storage = Memory::new();
    storage.set("foo".to_string(), "bar".to_string());
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
                let commands = parse_message(message);
                for command in commands {
                    let result = handle_command(command, &mut storage, &mut stream);
                    println!("{result:?}");
                }
            }
            Err(err) => {
                println!("ERROR IN STREAM: {err:?}");
                break;
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Keyword {
    Ping,
    Echo,
    Get,
    Set,
    Unknown,
}

#[derive(Debug)]
struct Command {
    keyword: Keyword,
    args: Vec<String>,
}

fn handle_command(
    command: Command,
    memory: &mut Memory,
    stream: &mut TcpStream,
) -> io::Result<usize> {
    let result = match command.keyword {
        Keyword::Ping => Resp::SimpleString("PONG".to_string()),
        Keyword::Echo => Resp::BulkString(command.args.join(" ")),
        Keyword::Get => {
            if let [key, ..] = command.args.as_slice() {
                match memory.get(key) {
                    Some(val) => Resp::BulkString(val.to_owned()),
                    None => Resp::Nil,
                }
            } else {
                Resp::Error("Missing argument to command GET".to_string())
            }
        }
        Keyword::Unknown => Resp::Error("Unknown command".to_string()),
        Keyword::Set => {
            if let [key, val, ..] = command.args.as_slice() {
                memory.set(key.to_owned(), val.to_owned());
                Resp::SimpleString("OK".to_string())
            } else {
                Resp::Error("Not enough arguments to SET command".to_string())
            }
        }
    };
    stream.write(&result.as_bytes())
}

fn parse_message(message: String) -> Vec<Command> {
    let mut chunks = message.split("\r\n");
    let mut commands: Vec<Command> = vec![];
    let mut matched_keyword = false;
    let mut arr_len = 1;
    loop {
        let mut word: Option<&str> = None;
        let mut args = vec![];
        let mut keyword = Keyword::Unknown;
        let mut chunk = match chunks.next() {
            Some(chunk) => chunk,
            None => break,
        };
        while arr_len > 0 {
            let first_char = match chunk.chars().next() {
                Some(c) => c,
                None => break,
            };

            match first_char {
                '*' => {
                    arr_len = chunk.chars().nth(1).unwrap().to_digit(10).unwrap() + 1;
                }
                '$' => {
                    word = chunks.next();
                }
                _ => {
                    word = Some(chunk);
                }
            }

            if let Some(word) = word {
                if !matched_keyword {
                    keyword = match word.to_uppercase().as_str() {
                        "PING" => Keyword::Ping,
                        "ECHO" => Keyword::Echo,
                        "GET" => Keyword::Get,
                        "SET" => Keyword::Set,
                        _ => Keyword::Unknown,
                    };
                    matched_keyword = true;
                } else {
                    args.push(word.to_string());
                }
            }
            chunk = match chunks.next() {
                Some(c) => c,
                None => break,
            };

            arr_len -= 1;
        }
        let command = Command { keyword, args };
        commands.push(command);
    }
    commands
}
