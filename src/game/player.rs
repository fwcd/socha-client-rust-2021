use crate::util::{SCResult, FromXmlNode, XmlNode};
use super::Team;

/// Metadata about a player.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Player {
    pub team: Team,
    pub display_name: String
}

impl FromXmlNode for Player {
    fn from_node(node: &XmlNode) -> SCResult<Self> {
        Ok(Self {
            team: Team::from_node(node.child_by_name("color")?)?,
            display_name: node.attribute("displayName")?.to_owned()
        })
    }
}
