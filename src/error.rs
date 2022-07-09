use serde::de;
use serde::ser;
use std::error;
use std::fmt;
use std::result;

/// A result of a serde_typename operation
pub type Result<T> = result::Result<T, Error>;

/// An error that occured while serializing or deserializing a type
#[derive(Clone, Debug)]
pub struct Error {
    direction: Direction,
    code: ErrorCode,
}

impl Error {
    pub(crate) fn deserialization(code: ErrorCode) -> Self {
        Self {
            direction: Direction::Deserialization,
            code,
        }
    }

    pub(crate) fn serialization(code: ErrorCode) -> Self {
        Self {
            direction: Direction::Serialization,
            code,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Direction {
    Serialization,
    Deserialization,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::Serialization => f.write_str("serialization"),
            Direction::Deserialization => f.write_str("deserialization"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum ErrorCode {
    Message(String),
    UnsupportedOperation(String),
    InvalidType {
        unexpected: String,
        expected: String,
    },
    InvalidVariantName {
        received: String,
        allowed: Vec<String>,
    },
    TrailingCharacters,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorCode::Message(msg) => f.write_str(msg),
            ErrorCode::UnsupportedOperation(t) => {
                write!(f, "unsupported operation: {}", t)
            }
            ErrorCode::InvalidType {
                unexpected,
                expected,
            } => write!(f, "invalid type: {}, expected {}", unexpected, expected),
            ErrorCode::InvalidVariantName { received, allowed } => {
                write!(
                    f,
                    "invalid variant: {} is not a valid variant name ({:?})",
                    received, allowed
                )
            }
            ErrorCode::TrailingCharacters => write!(
                f,
                "trailing characters: input ends with trailing characters"
            ),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.direction, self.code)
    }
}

impl de::Error for Error {
    #[cold]
    fn custom<T: fmt::Display>(msg: T) -> Error {
        Error {
            direction: Direction::Deserialization,
            code: ErrorCode::Message(format!("{}", msg)),
        }
    }

    #[cold]
    fn unknown_variant(variant: &str, expected: &'static [&'static str]) -> Self {
        Error {
            direction: Direction::Deserialization,
            code: ErrorCode::InvalidVariantName {
                received: variant.to_string(),
                allowed: expected.iter().map(|v| v.to_string()).collect(),
            },
        }
    }

    #[cold]
    fn invalid_type(unexp: de::Unexpected, exp: &dyn de::Expected) -> Self {
        Error {
            direction: Direction::Deserialization,
            code: ErrorCode::InvalidType {
                unexpected: format!("{}", unexp),
                expected: format!("{}", exp),
            },
        }
    }
}

impl ser::Error for Error {
    #[cold]
    fn custom<T: fmt::Display>(msg: T) -> Error {
        Error {
            direction: Direction::Serialization,
            code: ErrorCode::Message(format!("{}", msg)),
        }
    }
}
