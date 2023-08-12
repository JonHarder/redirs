use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    let host_port = "127.0.0.1:6379";
    let listener = TcpListener::bind(host_port).unwrap();
    println!("Server started at {host_port}");
    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                let mut buf = vec![0; 512];
                let bytes_read = s.read(&mut buf).unwrap();
                if bytes_read == 0 {
                    break;
                }
                let message = String::from_utf8(buf).unwrap();
                let commands = parse_message(message);
                for command in commands {
                    handle_command(command, &mut s);
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
    Unknown,
}

#[derive(Debug)]
struct Command {
    keyword: Keyword,
    args: Vec<String>,
}

fn handle_command(command: Command, stream: &mut TcpStream) {
    let result = match command.keyword {
        Keyword::Ping => stream.write("+PONG\r\n".as_bytes()),
        Keyword::Echo => {
            let response = command.args.join(" ");
            let len = response.len();
            stream.write(format!("${len}\r\n{response}\r\n").as_bytes())
        }
        Keyword::Unknown => stream.write("-Unknown command\r\n".as_bytes()),
    };
    println!("RESULT: {result:?}");
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
