use core::panic;
use std::io::Write;

pub enum Item {
    SimpleString(String),

    Error(String),

    Integers(i64),

    BulkString(String),

    Array(Vec<Item>),

    Null,
}

impl Item {
    pub fn serialize(&self, data: &mut Vec<u8>) {
        match self {
            Self::SimpleString(s) => {
                data.write_all(b"+");
                data.write_all(s.as_bytes());
                data.write_all(b"\r\n");
            }
            Self::Error(e) => {
                data.write_all(b"-");
                data.write_all(e.as_bytes());
                data.write_all(b"\r\n");
            }
            Self::Integers(i) => {
                data.write_all(b":");
                data.write_all(i.to_string().as_bytes());
                data.write_all(b"\r\n");
            }
            Self::BulkString(s) => {
                data.write_all(b"$");
                data.write_all(s.len().to_string().as_bytes());
                data.write_all(b"\r\n");
                data.write_all(s.as_bytes());
                data.write_all(b"\r\n");
            }
            Self::Array(vs) => {
                data.write_all(b"*");
                data.write_all(vs.len().to_string().as_bytes());
                data.write_all(b"\r\n");
                for i in vs.iter() {
                    i.serialize(data);
                }
            }
            Self::Null => {
                data.write_all(b"$-1\r\n");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resp_item_serialize() {
        let mut data = Vec::new();
        Item::SimpleString(String::from("OK")).serialize(&mut data);
        assert_eq!(data, b"+OK\r\n");
        data.clear();

        Item::Error(String::from(
            "WRONGTYPE Operation against a key holding the wrong kind of value",
        ))
        .serialize(&mut data);
        assert_eq!(
            data,
            b"-WRONGTYPE Operation against a key holding the wrong kind of value\r\n"
        );
        data.clear();

        Item::Integers(1000).serialize(&mut data);
        assert_eq!(data, b":1000\r\n");
        data.clear();

        Item::BulkString(String::from("hello world\r")).serialize(&mut data);
        assert_eq!(data, b"$12\r\nhello world\r\r\n");
        data.clear();

        Item::Null.serialize(&mut data);
        assert_eq!(data, b"$-1\r\n");
        data.clear();

        Item::Array(vec![
            Item::Integers(1),
            Item::Integers(2),
            Item::Integers(3),
            Item::Null,
            Item::BulkString(String::from("hello")),
        ])
        .serialize(&mut data);
        assert_eq!(data, b"*5\r\n:1\r\n:2\r\n:3\r\n$-1\r\n$5\r\nhello\r\n");
        data.clear();
    }
}
