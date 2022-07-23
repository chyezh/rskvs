use failure::{Context, Fail};
use std::{io, num::ParseIntError, string::FromUtf8Error};

// crate general Result type
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&failure::Backtrace> {
        self.inner.backtrace()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.inner, f)
    }
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.inner.get_context()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(c: Context<ErrorKind>) -> Self {
        Error { inner: c }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::from(ErrorKind::Io(err))
    }
}

//impl From<FromUtf8Error> for Error {
//    fn from(err: FromUtf8Error) -> Self {
//        Error::from(ErrorKind::FromUtf8(err))
//    }
//}
//
//impl From<ParseIntError> for Error {
//    fn from(err: ParseIntError) -> Self {
//        Error::from(ErrorKind::ParseInt(err))
//    }
//}

#[derive(Fail, Debug)]
pub enum ErrorKind {
    // indicate io error
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),

    // indicate key not found error
    #[fail(display = "Key not found")]
    KeyNotFound,

    #[fail(display = "protocol error: {}", _0)]
    Protocol(String),

    // #[fail(display = "{}", _0)]
    // FromUtf8(#[cause] FromUtf8Error),

    // #[fail(display = "{}", _0)]
    // ParseInt(#[cause] ParseIntError),

    // indicate command not found error
    #[fail(display = "Unexpected command type")]
    UnexpectedCommandType,

    // error with string message
    #[fail(display = "{}", _0)]
    StringError(String),

    #[fail(display = "Unexpected resp first bytes: {}", _0)]
    UnexpectedRespMark(u8),

    #[fail(display = "end of stream")]
    EOS,
}
