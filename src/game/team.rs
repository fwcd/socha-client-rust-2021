use std::{fmt, str::FromStr};
use crate::util::{SCError, SCResult, FromXmlNode, XmlNode};

/// A player's team.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Team {
    None,
    One,
    Two
}

impl Team {
    /// Unwraps an Option, mapping None to Team::None.
    pub fn from_option(option: Option<Self>) -> Self {
        option.unwrap_or_default()
    }

    /// Converts the team into an Option, mapping Team::None to None.
    pub fn to_option(self) -> Option<Self> {
        match self {
            Self::None => None,
            c => Some(c)
        }
    }

    /// Fetches the team's opponent team.
    pub fn opponent(self) -> Self {
        match self {
            Self::None => Self::None,
            Self::One => Self::Two,
            Self::Two => Self::One
        }
    }
}

impl Default for Team {
    fn default() -> Self {
        Self::None
    }
}

impl FromStr for Team {
    type Err = SCError;

    fn from_str(raw: &str) -> SCResult<Self> {
        match raw.to_uppercase().as_str() {
            "NONE" => Ok(Self::None),
            "ONE" => Ok(Self::One),
            "TWO" => Ok(Self::Two),
            _ => Err(format!("Could not parse team {}", raw).into())
        }
    }
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Team::None => write!(f, "NONE"),
            Team::One => write!(f, "ONE"),
            Team::Two => write!(f, "TWO")
        }
    }
}

impl FromXmlNode for Team {
    fn from_node(node: &XmlNode) -> SCResult<Self> {
        node.content().parse()
    }
}
