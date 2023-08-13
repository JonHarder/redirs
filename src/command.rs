#[derive(Debug)]
pub struct Command {
    pub keyword: Keyword,
    pub args: Vec<String>,
}

#[derive(Copy, Clone, Debug)]
pub enum Keyword {
    Ping,
    Echo,
    Get,
    Set,
    Unknown,
}

pub fn parse_message(message: String) -> Vec<Command> {
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
