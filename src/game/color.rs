use std::{fmt, str::FromStr};
use crate::util::{SCResult, SCError, FromXmlNode, XmlNode};
use super::Team;

pub const COLOR_COUNT: usize = 4;

/// A color in the game.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Color {
    None,
    Blue,
    Yellow,
    Red,
    Green
}

impl Color {
    /// Unwraps an Option, mapping None to Color::None.
    pub fn from_option(option: Option<Self>) -> Self {
        option.unwrap_or_default()
    }

    /// The color's associated team.
    pub fn team(self) -> Team {
        match self {
            Color::Red | Color::Blue => Team::One,
            Color::Yellow | Color::Green => Team::Two,
            Color::None => Team::None
        }
    }

    /// Converts the color into an Option, mapping Color::None to None.
    pub fn to_option(self) -> Option<Self> {
        match self {
            Self::None => None,
            c => Some(c)
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::None
    }
}

impl FromStr for Color {
    type Err = SCError;

    fn from_str(raw: &str) -> SCResult<Self> {
        match raw.to_uppercase().as_str() {
            "BLUE" => Ok(Self::Blue),
            "YELLOW" => Ok(Self::Yellow),
            "RED" => Ok(Self::Red),
            "GREEN" => Ok(Self::Green),
            _ => Err(format!("Color not parse color {}", raw).into())
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blue => write!(f, "BLUE"),
            Self::Yellow => write!(f, "YELLOW"),
            Self::Red => write!(f, "RED"),
            Self::Green => write!(f, "GREEN"),
            Self::None => write!(f, "NONE")
        }
    }
}

impl FromXmlNode for Color {
    fn from_node(node: &XmlNode) -> SCResult<Self> {
        node.content().parse()
    }
}
