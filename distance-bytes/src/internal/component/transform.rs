use crate::internal::{
    Quaternion, Serializable, Vector3, VisitDirection, Visitor, DEFAULT_QUATERNION, ONES_VECTOR_3,
    ZEROS_VECTOR_3,
};
use crate::GameObject;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
    pub children: Vec<GameObject>,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: ZEROS_VECTOR_3,
            rotation: DEFAULT_QUATERNION,
            scale: ONES_VECTOR_3,
            children: Vec::new(),
        }
    }
}

impl Serializable for Transform {
    const VERSION: i32 = 0;

    fn accept<V: Visitor>(&mut self, mut visitor: V, _version: i32) -> Result<()> {
        visitor.visit_vector_3("Position", &mut self.position)?;
        visitor.visit_quaternion("Rotation", &mut self.rotation)?;
        visitor.visit_vector_3("Scale", &mut self.scale)?;

        if V::VISIT_DIRECTION == VisitDirection::In {
            // Position
            {
                let is_valid = self.position.x.is_finite()
                    && self.position.y.is_finite()
                    && self.position.z.is_finite();
                if !is_valid {
                    self.position = Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    };
                }
            }

            // Rotation
            {
                let is_valid = self.rotation.v.x.is_finite()
                    && self.rotation.v.y.is_finite()
                    && self.rotation.v.z.is_finite()
                    && self.rotation.s.is_finite();
                if !is_valid {
                    self.rotation = Quaternion::from([0.0, 0.0, 0.0, 1.0]);
                }
            }

            // Scale
            {
                let is_valid = self.scale.x.is_finite()
                    && self.scale.y.is_finite()
                    && self.scale.z.is_finite();
                if !is_valid {
                    self.scale = Vector3 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    };
                } else {
                    self.scale.x = self.scale.x.abs().max(1E-5);
                    self.scale.y = self.scale.y.abs().max(1E-5);
                    self.scale.z = self.scale.z.abs().max(1E-5);
                }
            }
        }

        visitor.visit_children(&mut self.children)?;

        Ok(())
    }
}
