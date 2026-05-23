#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Context {
    Cg,
    Chg,
    Chh,
    Cxx,
    Other(String),
}

impl Context {
    pub fn as_str(&self) -> &str {
        match self {
            Context::Cg => "CG",
            Context::Chg => "CHG",
            Context::Chh => "CHH",
            Context::Cxx => "CXX",
            Context::Other(value) => value.as_str(),
        }
    }
}

impl From<&str> for Context {
    fn from(value: &str) -> Self {
        match value {
            "CG" => Context::Cg,
            "CHG" => Context::Chg,
            "CHH" => Context::Chh,
            "CXX" => Context::Cxx,
            other => Context::Other(other.to_string()),
        }
    }
}
