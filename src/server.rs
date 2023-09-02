use std::collections::HashMap;

use crate::command::{Command, Keyword};
use crate::resp::Resp;

pub struct Server {
    memory: HashMap<String, String>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            memory: HashMap::new(),
        }
    }
    fn get(&self, key: &str) -> Option<&String> {
        self.memory.get(key)
    }

    fn set(&mut self, key: &str, val: String) {
        self.memory.insert(key.to_owned(), val);
    }

    pub fn handle_command_mut(&mut self, command: Command) -> Resp {
        match command.keyword {
            Keyword::Set => {
                if let [key, val, ..] = command.args.as_slice() {
                    self.set(key, val.to_owned());
                    Resp::SimpleString("OK".to_string())
                } else {
                    Resp::Error("Not enough arguments to SET command".to_string())
                }
            }
            _ => panic!("keyword not handled by 'handle_command_mut'"),
        }
    }

    pub fn handle_command(&self, command: Command) -> Resp {
        match command.keyword {
            Keyword::Ping => Resp::SimpleString("PONG".to_string()),
            Keyword::Echo => Resp::BulkString(command.args.join(" ")),
            Keyword::Get => {
                if let [key, ..] = command.args.as_slice() {
                    match self.get(key) {
                        Some(val) => Resp::BulkString(val.to_owned()),
                        None => Resp::Nil,
                    }
                } else {
                    Resp::Error("Missing argument to command GET".to_string())
                }
            }
            Keyword::Unknown => Resp::Error("Unknown command".to_string()),
            _ => panic!("keyword not covered by 'handle_command'"),
        }
    }
}
