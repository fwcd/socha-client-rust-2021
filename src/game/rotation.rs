use std::{convert::TryFrom, fmt, str::FromStr};
use crate::util::{SCError, SCResult};

pub const ROTATIONS: [Rotation; 4] = [Rotation::None, Rotation::Left, Rotation::Right, Rotation::Mirror];

/// Describes how a piece shape is rotated.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Rotation {
    None,
    Right,
    Mirror,
    Left
}

impl TryFrom<i32> for Rotation {
    type Error = SCError;

    fn try_from(n: i32) -> SCResult<Self> {
        match n {
            0 => Ok(Self::None),
            1 => Ok(Self::Right),
            2 => Ok(Self::Mirror),
            3 => Ok(Self::Left),
            _ => Err(format!("Could not parse rotation {}", n).into())
        }
    }
}

impl From<Rotation> for i32 {
    fn from(rotation: Rotation) -> Self {
        match rotation {
            Rotation::None => 0,
            Rotation::Right => 1,
            Rotation::Mirror => 2,
            Rotation::Left => 3
        }
    }
}

impl FromStr for Rotation {
    type Err = SCError;

    fn from_str(raw: &str) -> SCResult<Self> {
        match raw.to_uppercase().as_str() {
            "NONE" => Ok(Rotation::None),
            "RIGHT" => Ok(Rotation::Right),
            "MIRROR" => Ok(Rotation::Mirror),
            "LEFT" => Ok(Rotation::Left),
            _ => Err(format!("Could not parse rotation {}", raw).into())
        }
    }
}

impl fmt::Display for Rotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rotation::None => write!(f, "NONE"),
            Rotation::Right => write!(f, "RIGHT"),
            Rotation::Mirror => write!(f, "MIRROR"),
            Rotation::Left => write!(f, "LEFT")
        }
    }
}
