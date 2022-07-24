use failure::Fail;

use super::{ErrorKind, Result};
use std::{
    io::{self, BufRead, Write},
    num::ParseIntError,
};

#[derive(Debug, PartialEq)]
pub enum Resp {
    SimpleString(String),

    Error(String),

    Integer(i64),

    BulkString(String),

    Array(Vec<Resp>),

    Null,
}

// Serialize multi Resp item into writer sequentially
pub fn serialize<I, W>(input: I, data: &mut W) -> Result<()>
where
    I: IntoIterator<Item = Resp>,
    W: Write,
{
    for item in input {
        serialize_one(&item, data)?;
    }
    Ok(())
}

// Unserialize multi Resp item into a Vec of Resp
pub fn unserialize<R>(reader: &mut R) -> Result<Vec<Resp>>
where
    R: BufRead,
{
    let mut result: Vec<Resp> = Vec::new();
    loop {
        match unserialize_one(reader) {
            Ok(item) => {
                result.push(item);
            }
            Err(err) => {
                // if touch the end of stream, return the unserialize result
                if matches!(err.kind(), ErrorKind::EOS) {
                    break;
                }
                return Err(err);
            }
        }
    }

    Ok(result)
}

// Unserialize single string and handle utf8 convert error
fn unserialize_utf8_string(v: Vec<u8>) -> Result<String> {
    let str = String::from_utf8(v).map_err(|err| ErrorKind::Protocol(err.to_string()))?;

    Ok(str)
}

// Unserialize single integer and handle parsing error
fn unserialize_integer(v: Vec<u8>) -> Result<i64> {
    let num: i64 = unserialize_utf8_string(v)?
        .parse()
        .map_err(|err: ParseIntError| ErrorKind::Protocol(err.to_string()))?;

    Ok(num)
}

// Unserlize single Resp item
fn unserialize_one<R>(reader: &mut R) -> Result<Resp>
where
    R: BufRead,
{
    let mut mark_buf = [0 as u8; 1];

    if reader.read(&mut mark_buf[..])? == 0 {
        return Err(ErrorKind::EOS.into());
    }
    match mark_buf[0] {
        b'+' => {
            let mut buf = Vec::new();
            reader.read_until(b'\n', &mut buf)?;
            Ok(Resp::SimpleString(unserialize_utf8_string(
                buf[..buf.len() - 2].to_vec(),
            )?))
        }
        b'-' => {
            let mut buf = Vec::new();
            reader.read_until(b'\n', &mut buf)?;
            Ok(Resp::Error(unserialize_utf8_string(
                buf[..buf.len() - 2].to_vec(),
            )?))
        }
        b':' => {
            let mut buf = Vec::new();
            reader.read_until(b'\n', &mut buf)?;
            Ok(Resp::Integer(unserialize_integer(
                buf[..buf.len() - 2].to_vec(),
            )?))
        }
        b'$' => {
            let mut buf = Vec::new();
            reader.read_until(b'\n', &mut buf)?;
            let num: i64 = unserialize_integer(buf[..buf.len() - 2].to_vec())?;

            if num < 0 {
                Ok(Resp::Null)
            } else {
                let mut buf = Vec::new();
                buf.resize((num + 2) as usize, 0);
                reader.read(&mut buf[..])?;

                Ok(Resp::BulkString(unserialize_utf8_string(
                    buf[..buf.len() - 2].to_vec(),
                )?))
            }
        }
        b'*' => {
            let mut buf = Vec::new();
            reader.read_until(b'\n', &mut buf)?;
            let num: i64 = unserialize_integer(buf[..buf.len() - 2].to_vec())?;

            let mut items = Vec::with_capacity(num as usize);
            for _ in 0..num {
                items.push(unserialize_one::<R>(reader)?);
            }

            Ok(Resp::Array(items))
        }
        _others => Err(ErrorKind::UnexpectedRespMark(_others))?,
    }
}

fn serialize_one<W>(item: &Resp, writer: &mut W) -> Result<()>
where
    W: Write,
{
    match item {
        Resp::SimpleString(s) => {
            writer.write_all(b"+")?;
            writer.write_all(s.as_bytes())?;
            writer.write_all(b"\r\n")?;
        }
        Resp::Error(e) => {
            writer.write_all(b"-")?;
            writer.write_all(e.as_bytes())?;
            writer.write_all(b"\r\n")?;
        }
        Resp::Integer(i) => {
            writer.write_all(b":")?;
            writer.write_all(i.to_string().as_bytes())?;
            writer.write_all(b"\r\n")?;
        }
        Resp::BulkString(s) => {
            writer.write_all(b"$")?;
            writer.write_all(s.len().to_string().as_bytes())?;
            writer.write_all(b"\r\n")?;
            writer.write_all(s.as_bytes())?;
            writer.write_all(b"\r\n")?;
        }
        Resp::Array(vs) => {
            writer.write_all(b"*")?;
            writer.write_all(vs.len().to_string().as_bytes())?;
            writer.write_all(b"\r\n")?;
            for i in vs.iter() {
                serialize_one(i, writer)?;
            }
        }
        Resp::Null => {
            writer.write_all(b"$-1\r\n")?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resp_item_serialize() {
        let mut data = Vec::new();
        serialize(vec![Resp::SimpleString(String::from("OK"))], &mut data).unwrap();
        assert_eq!(data, b"+OK\r\n");
        data.clear();

        serialize(
            vec![Resp::Error(String::from(
                "WRONGTYPE Operation against a key holding the wrong kind of value",
            ))],
            &mut data,
        )
        .unwrap();
        assert_eq!(
            data,
            b"-WRONGTYPE Operation against a key holding the wrong kind of value\r\n"
        );
        data.clear();

        serialize(vec![Resp::Integer(1000)], &mut data).unwrap();
        assert_eq!(data, b":1000\r\n");
        data.clear();

        serialize(
            vec![Resp::BulkString(String::from("hello world\r"))],
            &mut data,
        )
        .unwrap();
        assert_eq!(data, b"$12\r\nhello world\r\r\n");
        data.clear();

        serialize(vec![Resp::Null], &mut data).unwrap();
        assert_eq!(data, b"$-1\r\n");
        data.clear();

        serialize(
            vec![Resp::Array(vec![
                Resp::Integer(1),
                Resp::Integer(2),
                Resp::Integer(3),
                Resp::Null,
                Resp::BulkString(String::from("hello")),
            ])],
            &mut data,
        )
        .unwrap();
        assert_eq!(data, b"*5\r\n:1\r\n:2\r\n:3\r\n$-1\r\n$5\r\nhello\r\n");
        data.clear();
    }

    #[test]
    fn resp_item_unserialize() {
        let data: Vec<u8> = b"*5\r\n:1\r\n:2\r\n:3\r\n$-1\r\n$5\r\nhello\r\n*6\r\n:1\r\n:2\r\n:3\r\n$-1\r\n$5\r\nhello\r\n$-1\r\n"
            .iter()
            .map(|x| *x)
            .collect();
        let mut bufreader = io::BufReader::new(&*data);
        let v = unserialize(&mut bufreader).unwrap();
        assert_eq!(2, v.len());
        assert_eq!(
            v[0],
            Resp::Array(vec![
                Resp::Integer(1),
                Resp::Integer(2),
                Resp::Integer(3),
                Resp::Null,
                Resp::BulkString(String::from("hello")),
            ])
        );
        assert_eq!(
            v[1],
            Resp::Array(vec![
                Resp::Integer(1),
                Resp::Integer(2),
                Resp::Integer(3),
                Resp::Null,
                Resp::BulkString(String::from("hello")),
                Resp::Null,
            ])
        );
    }
}
