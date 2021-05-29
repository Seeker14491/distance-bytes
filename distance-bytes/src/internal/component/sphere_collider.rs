use crate::internal::{Serializable, Vector3, Visitor, ZEROS_VECTOR_3};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SphereCollider {
    pub center: Vector3,
    pub radius: f32,
}

impl Default for SphereCollider {
    fn default() -> Self {
        SphereCollider {
            center: ZEROS_VECTOR_3,
            radius: 1.0,
        }
    }
}

impl Serializable for SphereCollider {
    const VERSION: i32 = 1;

    fn accept<V: Visitor>(&mut self, mut visitor: V, version: i32) -> Result<()> {
        visitor.visit_vector_3("Center", &mut self.center)?;
        visitor.visit_f32("Radius", &mut self.radius)?;
        if version < 1 {
            visitor.visit_serial_collider_deprecated("BaseCollider")?;
        }

        Ok(())
    }
}
