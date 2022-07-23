use super::resp::Resp;
use super::{ErrorKind, Result};

pub struct Parser {
    resp_iter: std::vec::IntoIter<Resp>,
}

// a iterator for parsing resp sequence
// consume a Resp::Array item
impl Parser {
    // Create a new `Parser` to parse resp data
    pub fn new(data: Resp) -> Result<Parser> {
        let d = match data {
            Resp::Array(array) => array,
            _ => return Err(ErrorKind::StringError(String::from("expected array")).into()),
        };

        Ok(Parser {
            resp_iter: d.into_iter(),
        })
    }

    // get next resp item, return end of stream error if iterating completed
    fn next(&mut self) -> Result<Resp> {
        self.resp_iter.next().ok_or(ErrorKind::EOS.into())
    }

    // get next string type content.
    // return protocol error if next node is not string.
    pub fn next_string(&mut self) -> Result<String> {
        match self.next()? {
            Resp::BulkString(str) => Ok(str),
            Resp::SimpleString(str) => Ok(str),
            _ => Err(ErrorKind::Protocol(
                "protocol error: unexpected resp item received, expected a string".to_string(),
            )
            .into()),
        }
    }

    // get next integer type content.
    // return protocol error if next node is not integer.
    pub fn next_integer(&mut self) -> Result<i64> {
        match self.next()? {
            Resp::Integer(i) => Ok(i),
            _ => Err(ErrorKind::Protocol(
                "protocol error: unexpected resp item received, expected a integer".to_string(),
            )
            .into()),
        }
    }

    // check that parsing is finish
    pub fn check_finish(&mut self) -> Result<()> {
        if self.resp_iter.next().is_none() {
            Ok(())
        } else {
            Err(ErrorKind::Protocol(
                "protocol error: unexpected resp item received, expected end of stream".to_string(),
            )
            .into())
        }
    }
}
