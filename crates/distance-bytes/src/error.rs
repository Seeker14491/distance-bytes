use std::fmt;

/// An error encountered while reading a `.bytes` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    /// Absolute byte offset in the file where the error was detected.
    pub offset: usize,
    pub kind: ErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// Tried to read `needed` bytes but the enclosing scope (or file) ended first.
    UnexpectedEnd { needed: usize },
    /// A scope started with a different mark than expected.
    UnexpectedMark { expected: &'static [i32], found: i32 },
    /// A scope's length is negative or extends past its parent scope.
    InvalidScopeLength(i64),
    /// A 7-bit encoded length prefix was malformed or too large.
    InvalidLength,
    /// An element count was negative.
    NegativeCount(i32),
    /// A boolean byte other than 0 or 1.
    InvalidBool(u8),
    /// A scope had bytes left over after its contents were read.
    TrailingBytes { len: usize },
    /// The file doesn't start with a level or game object scope.
    UnknownRoot(i32),
}

impl Error {
    pub(crate) fn new(offset: usize, kind: ErrorKind) -> Self {
        Error { offset, kind }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "at offset {:#x}: ", self.offset)?;
        match &self.kind {
            ErrorKind::UnexpectedEnd { needed } => {
                write!(f, "unexpected end of scope (needed {needed} bytes)")
            }
            ErrorKind::UnexpectedMark { expected, found } => {
                write!(f, "expected scope mark {expected:?}, found {found}")
            }
            ErrorKind::InvalidScopeLength(len) => write!(f, "invalid scope length {len}"),
            ErrorKind::InvalidLength => write!(f, "invalid length prefix"),
            ErrorKind::NegativeCount(n) => write!(f, "negative element count {n}"),
            ErrorKind::InvalidBool(b) => write!(f, "invalid boolean byte {b:#04x}"),
            ErrorKind::TrailingBytes { len } => write!(f, "{len} unread bytes at end of scope"),
            ErrorKind::UnknownRoot(mark) => write!(f, "unknown root mark {mark}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T, E = Error> = std::result::Result<T, E>;
