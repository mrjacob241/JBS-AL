use std::fmt;

use super::Value;

pub type Completion<T> = Result<T, JsError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    Type,
    Range,
    Reference,
    Syntax,
    URI,
    Throw,
    Internal,
}

#[derive(Clone, Debug, PartialEq)]
pub struct JsError {
    pub kind: ErrorKind,
    pub message: String,
    pub thrown: Option<Value>,
}

impl JsError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            thrown: None,
        }
    }

    pub fn throw_value(value: Value) -> Self {
        Self {
            kind: ErrorKind::Throw,
            message: "thrown value".to_owned(),
            thrown: Some(value),
        }
    }

    pub fn type_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Type, message)
    }

    pub fn range_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Range, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Internal, message)
    }

    pub fn syntax(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Syntax, message)
    }

    pub fn reference(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Reference, message)
    }

    pub fn uri_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::URI, message)
    }
}

impl fmt::Display for JsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}Error: {}", self.kind, self.message)
    }
}

impl std::error::Error for JsError {}
