use crate::internal::{Serializable, Visitor};
use anyhow::Error;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Copy, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize,
)]
pub struct TrackLink {
    /// Reference to a `TrackLinkParent`
    parent: u32,

    /// Reference to a `TrackLink`
    link: u32,

    /// Reference to a `TrackManipulatorNode`
    manipulator_node: u32,

    owned_node_between_connected_links: bool,
}

impl Serializable for TrackLink {
    const VERSION: i32 = 2;

    fn accept<V: Visitor>(&mut self, mut visitor: V, version: i32) -> Result<(), Error> {
        match version {
            1 => {
                visitor.visit_reference("SegRef", &mut self.parent)?;
                visitor.visit_reference("LinkRef", &mut self.link)?;
                visitor.visit_reference("TrackNodeRef", &mut self.manipulator_node)?;
            }
            2 => {
                visitor.visit_reference("SegRef", &mut self.parent)?;
                visitor.visit_reference("LinkRef", &mut self.link)?;
                visitor.visit_bool(
                    "ManipulatorNodeShouldBeCreated",
                    &mut self.owned_node_between_connected_links,
                )?;
            }
            _ => {}
        }

        Ok(())
    }
}
