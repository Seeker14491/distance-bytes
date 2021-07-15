use crate::internal::{Serializable, Visitor};
use crate::Enum;
use anyhow::Result;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Group {
    /// References to `TrackLink` components
    pub links: Vec<u32>,

    pub inspect_children: Enum<GroupInspectChildrenType>,
}

impl Serializable for Group {
    const VERSION: i32 = 1;

    fn accept<V: Visitor>(&mut self, mut visitor: V, version: i32) -> Result<()> {
        visitor.visit_reference_array("Links", "LinkRef", &mut self.links)?;

        if version >= 1 {
            visitor.visit_enum("inspectChildren_", &mut self.inspect_children)?;
        }

        Ok(())
    }
}

#[derive(
    Debug,
    Copy,
    Clone,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Serialize,
    Deserialize,
    IntoPrimitive,
    TryFromPrimitive,
)]
#[repr(i32)]
pub enum GroupInspectChildrenType {
    None = 0,
    Combined = 1,
    All = 2,
}

impl Default for GroupInspectChildrenType {
    fn default() -> Self {
        GroupInspectChildrenType::None
    }
}
