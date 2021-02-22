mod deserializer;
mod serializer;

pub(crate) use deserializer::Deserializer;

use crate::{
    domain::{component::Transform, Quaternion, Vector3},
    util, GameObject,
};
use anyhow::Error;
use auto_impl::auto_impl;

pub(crate) const INVALID_INT: i32 = -127;
pub(crate) const INVALID_FLOAT: f32 = -10_000.0;

pub(crate) const INVALID_VECTOR_3: Vector3 = Vector3 {
    x: INVALID_FLOAT,
    y: INVALID_FLOAT,
    z: INVALID_FLOAT,
};

pub(crate) const INVALID_QUATERNION: Quaternion = Quaternion {
    v: INVALID_VECTOR_3,
    s: INVALID_FLOAT,
};

pub(crate) const EMPTY_MARK: i32 = 0x7FFF_FFFD;

#[auto_impl(&mut)]
trait Visitor {
    const VISIT_DIRECTION: VisitDirection;

    fn visit_i32(&mut self, name: &str, val: &mut i32) -> Result<(), Error>;
    fn visit_u32(&mut self, name: &str, val: &mut u32) -> Result<(), Error>;
    fn visit_i64(&mut self, name: &str, val: &mut i64) -> Result<(), Error>;
    fn visit_f32(&mut self, name: &str, val: &mut f32) -> Result<(), Error>;
    fn visit_vector_3(&mut self, name: &str, val: &mut Vector3) -> Result<(), Error>;
    fn visit_quaternion(&mut self, name: &str, val: &mut Quaternion) -> Result<(), Error>;

    fn visit_children(&mut self, val: &mut Vec<GameObject>) -> Result<(), Error>;
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
enum VisitDirection {
    In,
    Out,
}

#[auto_impl(&mut)]
trait Serializable: Default {
    fn accept<V: Visitor>(&mut self, visitor: V) -> Result<(), Error>;
}

impl Serializable for Transform {
    fn accept<V: Visitor>(&mut self, mut visitor: V) -> Result<(), Error> {
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
                    self.scale.x = util::f32_max(self.scale.x.abs(), 1E-5);
                    self.scale.y = util::f32_max(self.scale.y.abs(), 1E-5);
                    self.scale.z = util::f32_max(self.scale.z.abs(), 1E-5);
                }
            }
        }

        visitor.visit_children(&mut self.children)?;

        Ok(())
    }
}
