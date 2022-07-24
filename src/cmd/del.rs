use super::{Parser, Resp, Result};

pub struct Del {
    key: String,
}

impl Del {
    // Create a new del command with a key
    pub fn new(key: String) -> Self {
        Del { key }
    }

    // Create a new del command from parser
    pub fn from_parser(parser: &mut Parser) -> Result<Self> {
        Ok(Del {
            key: parser.next_string()?,
        })
    }

    // Get key
    pub fn key(&self) -> &str {
        &self.key
    }

    // Transfer command to resp
    pub fn into_resp(self) -> Resp {
        Resp::Array(vec![
            Resp::SimpleString("del".into()),
            Resp::BulkString(self.key),
        ])
    }
}
