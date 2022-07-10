use failure::{Context, Fail};
use std::io;

// crate general Result type
pub type Result<T> = std::result::Result<T, KvsError>;

#[derive(Debug)]
pub struct KvsError {
    inner: Context<KvsErrorKind>,
}

impl Fail for KvsError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&failure::Backtrace> {
        self.inner.backtrace()
    }
}

impl std::fmt::Display for KvsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.inner, f)
    }
}

impl KvsError {
    pub fn kind(&self) -> &KvsErrorKind {
        self.inner.get_context()
    }
}

impl From<KvsErrorKind> for KvsError {
    fn from(kind: KvsErrorKind) -> Self {
        KvsError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<KvsErrorKind>> for KvsError {
    fn from(c: Context<KvsErrorKind>) -> Self {
        KvsError { inner: c }
    }
}

impl From<io::Error> for KvsError {
    fn from(err: io::Error) -> Self {
        KvsError::from(KvsErrorKind::Io(err))
    }
}

impl From<serde_json::Error> for KvsError {
    fn from(err: serde_json::Error) -> Self {
        KvsError::from(KvsErrorKind::Serde(err))
    }
}

#[derive(Fail, Debug)]
pub enum KvsErrorKind {
    // indicate io error
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),

    // indicate serialize processing error
    #[fail(display = "{}", _0)]
    Serde(#[cause] serde_json::Error),

    // indicate key not found error
    #[fail(display = "Key not found")]
    KeyNotFound,

    // indicate command not found error
    #[fail(display = "Unexpected command type")]
    UnexpectedCommandType,

    // error with string message
    #[fail(display = "{}", _0)]
    StringError(String),
}
