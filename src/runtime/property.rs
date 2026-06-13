use std::fmt;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PropertyKey {
    String(String),
    Symbol(u64),
}

impl PropertyKey {
    pub fn string(name: impl Into<String>) -> Self {
        Self::String(name.into())
    }

    pub fn array_index(index: u64) -> Self {
        Self::String(index.to_string())
    }
}

impl From<&str> for PropertyKey {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<String> for PropertyKey {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl fmt::Display for PropertyKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyKey::String(value) => write!(f, "{value}"),
            PropertyKey::Symbol(id) => write!(f, "Symbol({id})"),
        }
    }
}
