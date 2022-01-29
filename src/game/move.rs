use crate::util::XmlNode;
use super::{Color, Piece};

/// A move in the game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Move {
    /// A move that skips a round.
    Skip { color: Color },
    /// A move that places an own, not yet placed piece.
    Set { piece: Piece }
}

impl Move {
    pub fn color(&self) -> Color {
        match self {
            Self::Skip { color } => *color,
            Self::Set { piece } => piece.color
        }
    }
}

impl From<Move> for XmlNode {
    fn from(game_move: Move) -> Self {
        match game_move {
            Move::Set { piece } => XmlNode::new("data")
                .attribute("class", "sc.plugin2021.SetMove")
                .child(piece)
                .build(),
            Move::Skip { color } => XmlNode::new("data")
                .attribute("class", "sc.plugin2021.SkipMove")
                .child(XmlNode::new("color")
                    .content(color.to_string().as_str())
                    .build())
                .build()
        }
    }
}
