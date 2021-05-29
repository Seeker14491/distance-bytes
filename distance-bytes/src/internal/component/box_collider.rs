use crate::internal::{Serializable, Vector3, Visitor, ONES_VECTOR_3, ZEROS_VECTOR_3};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct BoxCollider {
    pub center: Vector3,
    pub size: Vector3,
}

impl Default for BoxCollider {
    fn default() -> Self {
        BoxCollider {
            center: ZEROS_VECTOR_3,
            size: ONES_VECTOR_3,
        }
    }
}

impl Serializable for BoxCollider {
    const VERSION: i32 = 2;

    fn accept<V: Visitor>(&mut self, mut visitor: V, version: i32) -> Result<()> {
        visitor.visit_vector_3("Center", &mut self.center)?;
        if version == 0 {
            visitor.visit_vector_3("Extents", &mut { ZEROS_VECTOR_3 })?;
        }
        visitor.visit_vector_3("Size", &mut self.size)?;
        if version < 2 {
            visitor.visit_serial_collider_deprecated("Collider")?;
        }

        Ok(())
    }
}
