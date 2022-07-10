use std::cmp::PartialEq;

/// # Examples
/// ```rust
///    use rskvs::Cmder;
///
///    let cmder_get = Cmder::new("get key");
///    let cmder_set = Cmder::new("set key val");
///    let cmder_del = Cmder::new("del key");
/// ```
pub struct Cmder {
    cmd: Cmd,
    args: Vec<String>,
}

impl Cmder {
    pub fn new(s: &str) -> Option<Self> {
        let mut split = s.trim().split(' ');
        let mut cmd = Cmd::Unimplement;
        if let Some(cmd_string) = split.next() {
            cmd = Cmd::from(cmd_string);
            if cmd == Cmd::Unimplement {
                return None;
            }
        } else {
            return None;
        }
        let args: Vec<String> = split
            .map(|s: &str| String::from(s.trim()))
            .filter(|x| !x.is_empty())
            .collect();

        Some(Cmder { cmd, args })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmder_new() {
        let cmder = Cmder::new("get key").unwrap();
        assert_eq!(cmder.cmd, Cmd::Get);
        assert_eq!(cmder.args, vec!["key".to_string()]);

        let cmder = Cmder::new("set key val").unwrap();
        assert_eq!(cmder.cmd, Cmd::Set);
        assert_eq!(cmder.args, vec!["key".to_string(), "val".to_string()]);

        let cmder = Cmder::new(" get    key  ").unwrap();
        assert_eq!(cmder.cmd, Cmd::Get);
        assert_eq!(cmder.args, vec!["key".to_string()]);

        let cmder = Cmder::new(" set key    val    ").unwrap();
        assert_eq!(cmder.cmd, Cmd::Set);
        assert_eq!(cmder.args, vec!["key".to_string(), "val".to_string()]);

        let cmder = Cmder::new("set key val    ");
        assert!(cmder.is_none());
    }
}

#[derive(Debug, PartialEq)]
pub enum Cmd {
    Get,

    Set,

    Del,

    Unimplement,
}

impl From<&str> for Cmd {
    fn from(s: &str) -> Self {
        match s {
            "get" => Cmd::Get,
            "set" => Cmd::Set,
            "del" => Cmd::Del,
            _ => Cmd::Unimplement,
        }
    }
}
