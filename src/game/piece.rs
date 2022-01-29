use crate::util::{SCResult, FromXmlNode, XmlNode};
use super::{Color, Vec2, PieceShape, Rotation};

/// A game piece with color, position and transformed form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Piece {
    /// The piece's untransformed shape
    pub kind: PieceShape,
    /// How far the piece has been rotated
    pub rotation: Rotation,
    /// Whether the piece has been mirrored along the y-axis
    pub is_flipped: bool,
    /// The piece's color
    pub color: Color,
    /// The top left corner of the piece's rectangular bounding box
    pub position: Vec2
}

impl Piece {
    /// Fetches the piece's actual (transformed) shape
    pub fn shape(&self) -> PieceShape {
        self.kind.transform(self.rotation, self.is_flipped)
    }

    /// Fetches the piece's actual coordinates.
    pub fn coordinates(&self) -> impl Iterator<Item=Vec2> {
        let position = self.position;
        self.shape().coordinates().map(move |c| c + position)
    }
}

impl FromXmlNode for Piece {
    fn from_node(node: &XmlNode) -> SCResult<Self> {
        Ok(Self {
            color: node.attribute("color")?.parse()?,
            kind: node.attribute("kind")?.parse()?,
            rotation: node.attribute("rotation")?.parse()?,
            is_flipped: node.attribute("isFlipped")?.parse()?,
            position: Vec2::from_node(node.child_by_name("position")?)?
        })
    }
}

impl From<Piece> for XmlNode {
    fn from(piece: Piece) -> Self {
        XmlNode::new("piece")
            .attribute("color", piece.color.to_string())
            .attribute("kind", piece.kind.to_string())
            .attribute("rotation", piece.rotation.to_string())
            .attribute("isFlipped", piece.is_flipped.to_string())
            .child(XmlNode::new("position")
                .attribute("x", piece.position.x.to_string())
                .attribute("y", piece.position.y.to_string())
                .build())
            .build()
    }
}
