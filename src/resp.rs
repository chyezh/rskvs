use std::{
    array,
    io::{self, BufRead, Write},
};

#[derive(Debug, std::cmp::PartialEq)]
pub enum Item {
    SimpleString(String),

    Error(String),

    Integers(i64),

    BulkString(String),

    Array(Vec<Item>),

    Null,
}

pub fn serialize<I, W>(input: I, data: &mut W)
where
    I: IntoIterator<Item = Item>,
    W: Write,
{
    for item in input {
        serialize_one(&item, data);
    }
}

pub fn unserilize<R>(reader: &mut R) -> Result<Vec<Item>, std::io::Error>
where
    R: BufRead,
{
    let mut result: Vec<Item> = Vec::new();
    let mut mark_buf = [0 as u8; 1];

    loop {
        match unserilize_one(reader) {
            Ok(item) => {
                result.push(item);
            }
            Err(err) => break,
        }
    }

    Ok(result)
}

fn unserilize_one<R>(reader: &mut R) -> Result<Item, std::io::Error>
where
    R: BufRead,
{
    let mut mark_buf = [0 as u8; 1];

    reader.read(&mut mark_buf[..])?;
    match mark_buf[0] {
        b'+' => {
            let mut buf = Vec::new();
            reader.read_until(b'\n', &mut buf)?;
            Ok(Item::SimpleString(
                String::from_utf8(buf[..buf.len() - 2].to_vec()).unwrap(),
            ))
        }
        b'-' => {
            let mut buf = Vec::new();
            reader.read_until(b'\n', &mut buf)?;
            Ok(Item::Error(
                String::from_utf8(buf[..buf.len() - 2].to_vec()).unwrap(),
            ))
        }
        b':' => {
            let mut buf = Vec::new();
            reader.read_until(b'\n', &mut buf)?;
            let num: i64 = String::from_utf8(buf[..buf.len() - 2].to_vec())
                .unwrap()
                .parse()
                .unwrap();

            Ok(Item::Integers(num))
        }
        b'$' => {
            let mut buf = Vec::new();
            reader.read_until(b'\n', &mut buf)?;
            let num: i64 = String::from_utf8(buf[..buf.len() - 2].to_vec())
                .unwrap()
                .parse()
                .unwrap();

            if num < 0 {
                Ok(Item::Null)
            } else {
                let mut buf = Vec::new();
                buf.resize((num + 2) as usize, 0);
                reader.read(&mut buf[..])?;

                Ok(Item::BulkString(
                    String::from_utf8(buf[..buf.len() - 2].to_vec()).unwrap(),
                ))
            }
        }
        b'*' => {
            let mut buf = Vec::new();
            reader.read_until(b'\n', &mut buf)?;
            let num: i64 = String::from_utf8(buf[..buf.len() - 2].to_vec())
                .unwrap()
                .parse()
                .unwrap();

            let mut items = Vec::with_capacity(num as usize);
            for _ in 0..num {
                items.push(unserilize_one::<R>(reader).unwrap());
            }

            Ok(Item::Array(items))
        }
        b'\0' => Err(std::io::Error::new(std::io::ErrorKind::Other, "")),
        _ => panic!("err"),
    }
}

fn serialize_one<W>(item: &Item, writer: &mut W)
where
    W: Write,
{
    match item {
        Item::SimpleString(s) => {
            writer.write_all(b"+");
            writer.write_all(s.as_bytes());
            writer.write_all(b"\r\n");
        }
        Item::Error(e) => {
            writer.write_all(b"-");
            writer.write_all(e.as_bytes());
            writer.write_all(b"\r\n");
        }
        Item::Integers(i) => {
            writer.write_all(b":");
            writer.write_all(i.to_string().as_bytes());
            writer.write_all(b"\r\n");
        }
        Item::BulkString(s) => {
            writer.write_all(b"$");
            writer.write_all(s.len().to_string().as_bytes());
            writer.write_all(b"\r\n");
            writer.write_all(s.as_bytes());
            writer.write_all(b"\r\n");
        }
        Item::Array(vs) => {
            writer.write_all(b"*");
            writer.write_all(vs.len().to_string().as_bytes());
            writer.write_all(b"\r\n");
            for i in vs.iter() {
                serialize_one(i, writer);
            }
        }
        Item::Null => {
            writer.write_all(b"$-1\r\n");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resp_item_serialize() {
        let mut data = Vec::new();
        serialize(vec![Item::SimpleString(String::from("OK"))], &mut data);
        assert_eq!(data, b"+OK\r\n");
        data.clear();

        serialize(
            vec![Item::Error(String::from(
                "WRONGTYPE Operation against a key holding the wrong kind of value",
            ))],
            &mut data,
        );
        assert_eq!(
            data,
            b"-WRONGTYPE Operation against a key holding the wrong kind of value\r\n"
        );
        data.clear();

        serialize(vec![Item::Integers(1000)], &mut data);
        assert_eq!(data, b":1000\r\n");
        data.clear();

        serialize(
            vec![Item::BulkString(String::from("hello world\r"))],
            &mut data,
        );
        assert_eq!(data, b"$12\r\nhello world\r\r\n");
        data.clear();

        serialize(vec![Item::Null], &mut data);
        assert_eq!(data, b"$-1\r\n");
        data.clear();

        serialize(
            vec![Item::Array(vec![
                Item::Integers(1),
                Item::Integers(2),
                Item::Integers(3),
                Item::Null,
                Item::BulkString(String::from("hello")),
            ])],
            &mut data,
        );
        assert_eq!(data, b"*5\r\n:1\r\n:2\r\n:3\r\n$-1\r\n$5\r\nhello\r\n");
        data.clear();
    }

    #[test]
    fn resp_item_unserilize() {
        let data: Vec<u8> = b"*5\r\n:1\r\n:2\r\n:3\r\n$-1\r\n$5\r\nhello\r\n*6\r\n:1\r\n:2\r\n:3\r\n$-1\r\n$5\r\nhello\r\n$-1\r\n"
            .iter()
            .map(|x| *x)
            .collect();
        let mut bufreader = io::BufReader::new(&*data);
        let v = unserilize(&mut bufreader).unwrap();
        assert_eq!(2, v.len());
        assert_eq!(
            v[0],
            Item::Array(vec![
                Item::Integers(1),
                Item::Integers(2),
                Item::Integers(3),
                Item::Null,
                Item::BulkString(String::from("hello")),
            ])
        );
        assert_eq!(
            v[1],
            Item::Array(vec![
                Item::Integers(1),
                Item::Integers(2),
                Item::Integers(3),
                Item::Null,
                Item::BulkString(String::from("hello")),
                Item::Null,
            ])
        );
    }
}
