use super::{Parser, Resp, Result};

pub struct Set {
    key: String,
    val: String,
}

impl Set {
    // Create a new set command with a key
    pub fn new(key: String, val: String) -> Self {
        Set { key, val }
    }

    // Create a new set command from parser
    pub fn from_parser(parser: &mut Parser) -> Result<Self> {
        Ok(Set {
            key: parser.next_string()?,
            val: parser.next_string()?,
        })
    }

    // Get key
    pub fn key(&self) -> &str {
        &self.key
    }

    // Get val
    pub fn val(&self) -> &str {
        &self.val
    }

    // transfer command to resp
    pub fn into_resp(self) -> Resp {
        Resp::Array(vec![
            Resp::SimpleString("set".into()),
            Resp::BulkString(self.key),
            Resp::BulkString(self.val),
        ])
    }
}
