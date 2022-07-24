use super::{Parser, Resp, Result};

pub struct Get {
    key: String,
}

impl Get {
    // Create a new get command with a key
    pub fn new(key: String) -> Self {
        Get { key }
    }

    // Create a new get command from parser
    pub fn from_parser(parser: &mut Parser) -> Result<Self> {
        Ok(Get {
            key: parser.next_string()?,
        })
    }

    // Get key
    pub fn key(&self) -> &str {
        &self.key
    }

    // transfer command to resp
    pub fn into_resp(self) -> Resp {
        Resp::Array(vec![
            Resp::SimpleString("get".into()),
            Resp::BulkString(self.key),
        ])
    }
}
