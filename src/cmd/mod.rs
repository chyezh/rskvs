pub use super::parser::Parser;
pub use super::resp::Resp;
pub use super::{ErrorKind, Result};

mod get;
pub use get::Get;

mod ping;
pub use ping::Ping;

mod set;
pub use set::Set;

mod del;
pub use del::Del;

pub enum Cmd {
    Get(Get),
    Ping(Ping),
    Set(Set),
    Del(Del),
}

impl Cmd {
    // Create new command from resp item
    pub fn from_resp(resp: Resp) -> Result<Cmd> {
        let mut parser = Parser::new(resp)?;

        let command_name = parser.next_string()?.to_lowercase();

        let cmd = match command_name.as_str() {
            "get" => Cmd::Get(Get::from_parser(&mut parser)?),
            "ping" => Cmd::Ping(Ping::from_parser(&mut parser)?),
            "set" => Cmd::Set(Set::from_parser(&mut parser)?),
            "del" => Cmd::Del(Del::from_parser(&mut parser)?),
            _ => return Err(ErrorKind::UnexpectedCommandType.into()),
        };
        parser.check_finish()?;

        Ok(cmd)
    }
}
