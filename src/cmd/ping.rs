use super::{Parser, Resp, Result};

pub struct Ping {}

impl Ping {
    // Create a new ping command
    pub fn new() -> Self {
        Ping {}
    }

    // Create a new ping command from parser
    pub fn from_parser(parser: &mut Parser) -> Result<Self> {
        Ok(Ping {})
    }

    // Transfer command to resp
    pub fn into_resp(self) -> Resp {
        Resp::Array(vec![Resp::SimpleString("ping".into())])
    }
}
