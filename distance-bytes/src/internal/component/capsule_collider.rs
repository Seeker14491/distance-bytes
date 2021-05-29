use crate::internal::{Serializable, Vector3, Visitor, ZEROS_VECTOR_3};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CapsuleCollider {
    pub center: Vector3,
    pub radius: f32,
    pub height: f32,
    pub direction: i32,
}

impl Default for CapsuleCollider {
    fn default() -> Self {
        CapsuleCollider {
            center: ZEROS_VECTOR_3,
            radius: 1.0,
            height: 1.0,
            direction: 0,
        }
    }
}

impl Serializable for CapsuleCollider {
    const VERSION: i32 = 1;

    fn accept<V: Visitor>(&mut self, mut visitor: V, version: i32) -> Result<()> {
        visitor.visit_vector_3("Center", &mut self.center)?;
        visitor.visit_f32("Radius", &mut self.radius)?;
        visitor.visit_f32("Height", &mut self.height)?;
        visitor.visit_i32("Direction", &mut self.direction)?;
        if version < 1 {
            visitor.visit_serial_collider_deprecated("BaseCollider")?;
        }

        Ok(())
    }
}
