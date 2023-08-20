#[derive(Debug)]
pub struct Command {
    pub keyword: Keyword,
    pub args: Vec<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

        // reset array and keyword settings is case of another command
        // in pipeline
        matched_keyword = false;
        arr_len = 1;
        let command = Command { keyword, args };
        commands.push(command);
    }
    commands
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_message_no_args() {
        let input = "*1\r\n$4\r\nPING\r\n".to_string();
        let commands = parse_message(input);

        assert_eq!(commands.len(), 1);
        assert!(matches!(
            commands.first().unwrap(),
            Command {
                keyword: Keyword::Ping,
                args: a
            } if a.is_empty()
        ));
    }

    #[test]
    fn parse_message_with_args() {
        let input = "*3\r\n$4\r\nECHO\r\n$3\r\nfoo\r\n$3\r\nbar\r\n".to_string();
        let commands = parse_message(input);

        assert_eq!(commands.len(), 1);
        assert!(matches!(
            commands.first().unwrap(),
            Command {
                keyword: Keyword::Echo,
                args: a
            } if a.len() == 2
        ));
    }

    #[test]
    fn parse_multiple_messages() {
        // two PING commands
        let input = "*1\r\n$4\r\nPING\r\n*1\r\n$4\r\nPING\r\n".to_string();
        let commands = parse_message(input);

        println!("{commands:?}");
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].keyword, Keyword::Ping);
        assert_eq!(commands[1].keyword, Keyword::Ping);
    }
}
