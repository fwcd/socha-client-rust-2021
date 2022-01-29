use crate::util::{SCResult, FromXmlNode, XmlNode};
use super::{Color, Vec2};

/// A field on the board holding a color.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub position: Vec2,
    pub content: Color
}

impl FromXmlNode for Field {
    fn from_node(node: &XmlNode) -> SCResult<Self> {
        Ok(Self {
            position: Vec2::new(
                node.attribute("x")?.parse()?,
                node.attribute("y")?.parse()?
            ),
            content: node.attribute("content")?.parse()?
        })
    }
}
