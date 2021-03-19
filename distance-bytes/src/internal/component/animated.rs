use crate::internal::{Serializable, Visitor};
use crate::{AnimatorBase, Vector3, ZEROS_VECTOR_3};
use anyhow::Error;
use enum_primitive_derive::Primitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Animated {
    pub base: AnimatorBase,
    pub motion: AnimatedMotionType,
    pub scale: bool,
    pub scale_exponent: Vector3,
    pub rotate: bool,
    pub rotate_axis: Vector3,
    pub center_point: Vector3,
    pub rotate_magnitude: f32,
    pub translate_type: AnimatedTranslateType,
    pub translate_vector: Vector3,
    pub follow_distance: f32,
    pub follow_percent_of_track: bool,
    pub rotate_global: bool,
    pub double_pivot_distance: f32,
    pub wrap_around: bool,
    pub projectile_velocity: Vector3,
    pub projectile_gravity: Vector3,
    pub animate_physics: bool,
    pub always_animate: bool,
}

impl Default for Animated {
    fn default() -> Self {
        Animated {
            base: Default::default(),
            motion: Default::default(),
            scale: false,
            scale_exponent: Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            rotate: true,
            rotate_axis: Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            center_point: ZEROS_VECTOR_3,
            rotate_magnitude: 90.0,
            translate_type: Default::default(),
            translate_vector: Vector3 {
                x: 0.0,
                y: 10.0,
                z: 0.0,
            },
            follow_distance: 100.0,
            follow_percent_of_track: true,
            rotate_global: false,
            double_pivot_distance: 0.0,
            wrap_around: true,
            projectile_velocity: Vector3 {
                x: 0.0,
                y: 50.0,
                z: 25.0,
            },
            projectile_gravity: Vector3 {
                x: 0.0,
                y: -25.0,
                z: 0.0,
            },
            animate_physics: true,
            always_animate: false,
        }
    }
}

impl Serializable for Animated {
    const VERSION: i32 = 11;

    fn accept<V: Visitor>(&mut self, mut visitor: V, version: i32) -> Result<(), Error> {
        visitor.visit_enum("motion_", &mut self.motion)?;
        if version >= 1 {
            visitor.visit_bool("scale_", &mut self.scale)?;
            visitor.visit_vector_3("scaleExponent_", &mut self.scale_exponent)?;
        }
        visitor.visit_bool("rotate_", &mut self.rotate)?;
        visitor.visit_vector_3("rotateAxis_", &mut self.rotate_axis)?;
        if version < 4 {
            visitor.visit_vector_3("rotateCenter_", &mut self.center_point)?;
            visitor.visit_f32("rotateMagnitude_", &mut self.rotate_magnitude)?;

            let mut translate = false;
            visitor.visit_bool("translate_", &mut translate)?;
            self.translate_type = match translate {
                true => AnimatedTranslateType::Local,
                false => AnimatedTranslateType::None,
            };

            visitor.visit_vector_3("translateVector_", &mut self.translate_vector)?;
            if version >= 2 {
                let mut move_along_track = false;
                visitor.visit_bool("moveAlongTrack_", &mut move_along_track)?;
                if move_along_track {
                    self.translate_type = AnimatedTranslateType::FollowTrack;
                }

                visitor.visit_f32("moveDistance_", &mut self.follow_distance)?;
            }
            self.follow_percent_of_track = false;
        } else {
            if version >= 5 {
                visitor.visit_bool("rotateGlobal_", &mut self.rotate_global)?;
            }
            visitor.visit_f32("rotateMagnitude_", &mut self.rotate_magnitude)?;
            visitor.visit_vector_3("centerPoint_", &mut self.center_point)?;
            visitor.visit_enum("translateType_", &mut self.translate_type)?;
            visitor.visit_vector_3("translateVector_", &mut self.translate_vector)?;
            visitor.visit_f32("followDistance_", &mut self.follow_distance)?;
            if version >= 11 {
                visitor.visit_f32("doublePivotDistance_", &mut self.double_pivot_distance)?;
            }

            if version >= 9 {
                visitor.visit_bool("followPercentOfTrack_", &mut self.follow_percent_of_track)?;
            } else {
                self.follow_percent_of_track = false;
            }

            if version >= 10 {
                visitor.visit_bool("wrapAround_", &mut self.wrap_around)?;
            }
            if version == 4 {
                visitor.visit_vector_3("projectileVelocity_", &mut self.projectile_velocity)?;
            }
            visitor.visit_vector_3("projectileGravity_", &mut self.projectile_gravity)?;
        }

        let mut apply_ping_pong = false;
        if version >= 8 {
            self.base.visit_curve(&mut visitor)?;
        } else {
            apply_ping_pong = self.base.visit_curve_old(&mut visitor, version < 6)?;
        }

        if version >= 7 {
            visitor.visit_bool("animatePhysics_", &mut self.animate_physics)?;
            visitor.visit_bool("alwaysAnimate_", &mut self.always_animate)?;
        }
        if version >= 3 {
            self.base.visit_trigger(&mut visitor)?;
        }
        if version < 8 {
            self.base.upgrade_to_new_ping_pong(apply_ping_pong);
        }

        Ok(())
    }
}

#[derive(
    Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Primitive,
)]
pub enum AnimatedMotionType {
    Spinning = 0,
    Sliding = 1,
    Hinge = 2,
    Pendulum = 3,
    Bouncing = 4,
    Advanced = 5,
}

impl Default for AnimatedMotionType {
    fn default() -> Self {
        AnimatedMotionType::Hinge
    }
}

#[derive(
    Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Primitive,
)]
pub enum AnimatedTranslateType {
    None = 0,
    Local = 1,
    Global = 2,
    FollowTrack = 3,
    Projectile = 4,
    Absolute = 5,
}

impl Default for AnimatedTranslateType {
    fn default() -> Self {
        AnimatedTranslateType::None
    }
}
