pub enum Resp {
    SimpleString(String),
    BulkString(String),
    Error(String),
    Integer(i64),
    Nil,
}

impl Resp {
    pub fn as_bytes(&self) -> Vec<u8> {
        let msg = match self {
            Resp::SimpleString(s) => format!("+{}\r\n", s),
            Resp::BulkString(s) => format!("${len}\r\n{s}\r\n", len = s.len()),
            Resp::Error(s) => format!("-{s}\r\n"),
            Resp::Integer(i) => format!(":{i}\r\n"),
            Resp::Nil => "$-1\r\n".to_string(),
        };
        msg.into_bytes()
    }
}
