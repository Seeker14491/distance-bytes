mod deserializer;
mod serializer;
mod string;

pub(crate) use deserializer::Deserializer;
pub(crate) use serializer::Serializer;

use crate::{
    domain::{
        component::{GoldenSimples, GoldenSimplesPresets, Transform},
        Quaternion, Vector3,
    },
    util, GameObject,
};
use anyhow::Error;
use auto_impl::auto_impl;
use num_traits::{FromPrimitive, ToPrimitive};

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

    fn visit_bool(&mut self, name: &str, val: &mut bool) -> Result<(), Error>;
    fn visit_i32(&mut self, name: &str, val: &mut i32) -> Result<(), Error>;
    fn visit_u32(&mut self, name: &str, val: &mut u32) -> Result<(), Error>;
    fn visit_i64(&mut self, name: &str, val: &mut i64) -> Result<(), Error>;
    fn visit_f32(&mut self, name: &str, val: &mut f32) -> Result<(), Error>;
    fn visit_vector_3(&mut self, name: &str, val: &mut Vector3) -> Result<(), Error>;
    fn visit_quaternion(&mut self, name: &str, val: &mut Quaternion) -> Result<(), Error>;

    fn visit_children(&mut self, val: &mut Vec<GameObject>) -> Result<(), Error>;

    fn visit_enum<T: FromPrimitive + ToPrimitive>(
        &mut self,
        name: &str,
        enum_: &mut T,
    ) -> Result<(), Error> {
        let mut n = enum_.to_i32().unwrap();
        self.visit_i32(name, &mut n)?;

        if Self::VISIT_DIRECTION == VisitDirection::In {
            *enum_ = T::from_i32(n).unwrap();
        }

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
enum VisitDirection {
    In,
    Out,
}

#[auto_impl(&mut)]
trait Serializable: Default {
    const VERSION: i32;

    fn accept<V: Visitor>(&mut self, visitor: V, version: i32) -> Result<(), Error>;
}

impl Serializable for Transform {
    const VERSION: i32 = 0;

    fn accept<V: Visitor>(&mut self, mut visitor: V, _version: i32) -> Result<(), Error> {
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

impl Serializable for GoldenSimples {
    const VERSION: i32 = 4;

    fn accept<V: Visitor>(&mut self, mut visitor: V, version: i32) -> Result<(), Error> {
        const TEXTURE_COUNT: i32 = 72;
        const TEXTURE_COUNT_VERSION_1: i32 = 35;
        const TEXTURE_COUNT_VERSION_2: i32 = 46;
        const TEXTURE_COUNT_VERSION_3: i32 = 55;

        visitor.visit_i32("ImageIndex", &mut self.image_index)?;

        match version {
            n if n < 1 => {
                self.emit_index = self.image_index;
            }
            1 => {
                visitor.visit_i32("EmitIndex", &mut self.emit_index)?;
                if self.image_index >= TEXTURE_COUNT_VERSION_1 {
                    self.image_index += TEXTURE_COUNT - TEXTURE_COUNT_VERSION_1;
                }
            }
            2 => {
                visitor.visit_i32("EmitIndex", &mut self.emit_index)?;
                if self.image_index >= TEXTURE_COUNT_VERSION_2 {
                    self.image_index += TEXTURE_COUNT - TEXTURE_COUNT_VERSION_2;
                }
            }
            3 => {
                visitor.visit_i32("EmitIndex", &mut self.emit_index)?;
                if self.image_index >= TEXTURE_COUNT_VERSION_3 {
                    self.image_index += TEXTURE_COUNT - TEXTURE_COUNT_VERSION_3;
                }
            }
            _ => {
                visitor.visit_i32("EmitIndex", &mut self.emit_index)?;
            }
        }

        if version >= 1 {
            visitor.visit_enum("Preset", &mut self.preset)?;
        } else {
            self.preset = GoldenSimplesPresets::Custom;
        }

        visitor.visit_vector_3("textureScale", &mut self.texture_scale)?;
        visitor.visit_vector_3("textureOffset", &mut self.texture_offset)?;
        visitor.visit_bool("flipTextureUV", &mut self.flip_texture_uv)?;
        visitor.visit_bool("worldMapped", &mut self.world_mapped)?;
        visitor.visit_bool("disableDiffuse", &mut self.disable_diffuse)?;
        visitor.visit_bool("disableBump", &mut self.disable_bump)?;
        if version >= 3 {
            visitor.visit_f32("bumpStrength", &mut self.bump_strength)?;
        }
        visitor.visit_bool("disableReflect", &mut self.disable_reflect)?;
        if version >= 1 {
            visitor.visit_bool("disableCollision", &mut self.disable_collision)?;
            visitor.visit_bool("additiveTransparency", &mut self.additive_transparency)?;
        }
        if version >= 2 {
            visitor.visit_bool(
                "multiplicativeTransparency",
                &mut self.multiplicative_transparency,
            )?;
            visitor.visit_bool("invertEmit", &mut self.invert_emit)?;
        }

        Ok(())
    }
}
