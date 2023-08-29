use crate::command::Command;
use crate::command::Keyword;

enum ParseState {
    ParsingCommand,
    ParsingArgs(Keyword, Vec<String>),
}

pub(super) struct State {
    state: ParseState,
}

impl State {
    pub fn new() -> Self {
        State {
            state: ParseState::ParsingCommand,
        }
    }

    pub fn add_word(&mut self, word: &str) {
        match &mut self.state {
            ParseState::ParsingCommand => {
                let keyword = Keyword::from(word);
                self.state = ParseState::ParsingArgs(keyword, vec![])
            }
            ParseState::ParsingArgs(_, ref mut args) => {
                args.push(word.to_string());
            }
        }
    }

    pub fn as_command(&self) -> Result<Command, &'static str> {
        match &self.state {
            ParseState::ParsingCommand => Err("No keyword parsed"),
            ParseState::ParsingArgs(keyword, args) => Ok(Command {
                keyword: *keyword,
                args: args.to_vec(),
            }),
        }
    }
}
