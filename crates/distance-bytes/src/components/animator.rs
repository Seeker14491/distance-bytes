use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// The timing of an animation (`AnimatorBase.VisitCurve` and, in older versions,
/// `AnimatorBase.VisitCurveOld`), shared by `Animated` and `GoldenAnimator`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct AnimatorCurve {
    pub delay: Option<f32>,
    pub duration: Option<f32>,
    pub time_offset: Option<f32>,
    pub r#loop: Option<bool>,
    /// `AnimatorBase.ExtrapolationTypeObsolete`. Old format only.
    pub extrapolation_type: Option<i32>,
    /// New format only.
    pub extend: Option<bool>,
    /// `AnimatorBase.CurveType`.
    pub curve_type: Option<i32>,
    /// Old format of `Animated` before version 6 only, in place of `editor_animation_t`.
    pub centered_animation: Option<bool>,
    pub editor_animation_t: Option<f32>,
    /// Whether the pong values are used. When they aren't, the game sets them when it reads the
    /// curve: the delay and duration to the curve's, and the curve type to the opposite of the
    /// curve's.
    pub custom_pong_values: Option<bool>,
    /// See `custom_pong_values`.
    pub pong_delay: Option<f32>,
    /// See `custom_pong_values`.
    pub pong_duration: Option<f32>,
    /// `AnimatorBase.CurveType`. See `custom_pong_values`.
    pub pong_curve_type: Option<i32>,
}

impl AnimatorCurve {
    fn visit(
        &mut self,
        v: &mut impl Visitor,
        new_format: bool,
        old_animation_t: bool,
    ) -> Result<()> {
        v.value("delay_", &mut self.delay)?;
        v.value("duration_", &mut self.duration)?;
        v.value("timeOffset_", &mut self.time_offset)?;
        v.value("loop_", &mut self.r#loop)?;
        if new_format {
            v.value("extend_", &mut self.extend)?;
            v.value("curveType_", &mut self.curve_type)?;
            v.value("editorAnimationT_", &mut self.editor_animation_t)?;
        } else {
            v.value("extrapolationType_", &mut self.extrapolation_type)?;
            v.value("curveType_", &mut self.curve_type)?;
            if old_animation_t {
                v.value("centeredAnimation_", &mut self.centered_animation)?;
            } else {
                v.value("editorAnimationT_", &mut self.editor_animation_t)?;
            }
        }
        v.value("customPongValues_", &mut self.custom_pong_values)?;
        v.value("pongDelay_", &mut self.pong_delay)?;
        v.value("pongDuration_", &mut self.pong_duration)?;
        v.value("pongCurveType_", &mut self.pong_curve_type)
    }
}

/// What an animation does on events (`AnimatorBase.VisitTrigger`). The actions are
/// `AnimatorBase.TriggerAction`s.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct AnimatorTrigger {
    pub default_action: Option<i32>,
    pub on_action: Option<i32>,
    pub on_wait_for_animation_finish: Option<bool>,
    pub on_reset: Option<bool>,
    pub off_action: Option<i32>,
    pub off_wait_for_animation_finish: Option<bool>,
    pub off_reset: Option<bool>,
}

impl AnimatorTrigger {
    fn visit(&mut self, v: &mut impl Visitor) -> Result<()> {
        v.value("defaultAction_", &mut self.default_action)?;
        v.value("onAction_", &mut self.on_action)?;
        v.value(
            "onWaitForAnimationFinish_",
            &mut self.on_wait_for_animation_finish,
        )?;
        v.value("onReset_", &mut self.on_reset)?;
        v.value("offAction_", &mut self.off_action)?;
        v.value(
            "offWaitForAnimationFinish_",
            &mut self.off_wait_for_animation_finish,
        )?;
        v.value("offReset_", &mut self.off_reset)
    }
}

/// `Animated`: animates the object's transform.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct Animated {
    /// `Animated.MotionType`.
    pub motion: Option<i32>,
    /// Version 1 and later.
    pub scale: Option<bool>,
    /// Version 1 and later.
    pub scale_exponent: Option<[f32; 3]>,
    pub rotate: Option<bool>,
    pub rotate_axis: Option<[f32; 3]>,
    /// Version 5 and later.
    pub rotate_global: Option<bool>,
    pub rotate_magnitude: Option<f32>,
    /// Named `rotateCenter_` in version 3 and earlier.
    pub center_point: Option<[f32; 3]>,
    /// Version 3 and earlier.
    pub translate: Option<bool>,
    /// `Animated.TranslateType`. Version 4 and later.
    pub translate_type: Option<i32>,
    pub translate_vector: Option<[f32; 3]>,
    /// Versions 2 and 3.
    pub move_along_track: Option<bool>,
    /// Version 2 and later. Named `moveDistance_` in versions 2 and 3.
    pub follow_distance: Option<f32>,
    /// Version 11 and later.
    pub double_pivot_distance: Option<f32>,
    /// Version 9 and later.
    pub follow_percent_of_track: Option<bool>,
    /// Version 10 and later.
    pub wrap_around: Option<bool>,
    /// Version 4 only.
    pub projectile_velocity: Option<[f32; 3]>,
    /// Version 4 and later.
    pub projectile_gravity: Option<[f32; 3]>,
    /// In the new format from version 8.
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub curve: AnimatorCurve,
    /// Version 7 and later.
    pub animate_physics: Option<bool>,
    /// Version 7 and later.
    pub always_animate: Option<bool>,
    /// Version 3 and later.
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub trigger: AnimatorTrigger,
}

impl Visit for Animated {
    const VERSIONS: RangeInclusive<i32> = 0..=11;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        v.value("motion_", &mut self.motion)?;
        if version >= 1 {
            v.value("scale_", &mut self.scale)?;
            v.value("scaleExponent_", &mut self.scale_exponent)?;
        }
        v.value("rotate_", &mut self.rotate)?;
        v.value("rotateAxis_", &mut self.rotate_axis)?;
        if version < 4 {
            v.value("rotateCenter_", &mut self.center_point)?;
            v.value("rotateMagnitude_", &mut self.rotate_magnitude)?;
            v.value("translate_", &mut self.translate)?;
            v.value("translateVector_", &mut self.translate_vector)?;
            if version >= 2 {
                v.value("moveAlongTrack_", &mut self.move_along_track)?;
                v.value("moveDistance_", &mut self.follow_distance)?;
            }
        } else {
            if version >= 5 {
                v.value("rotateGlobal_", &mut self.rotate_global)?;
            }
            v.value("rotateMagnitude_", &mut self.rotate_magnitude)?;
            v.value("centerPoint_", &mut self.center_point)?;
            v.value("translateType_", &mut self.translate_type)?;
            v.value("translateVector_", &mut self.translate_vector)?;
            v.value("followDistance_", &mut self.follow_distance)?;
            if version >= 11 {
                v.value("doublePivotDistance_", &mut self.double_pivot_distance)?;
            }
            if version >= 9 {
                v.value("followPercentOfTrack_", &mut self.follow_percent_of_track)?;
            }
            if version >= 10 {
                v.value("wrapAround_", &mut self.wrap_around)?;
            }
            if version == 4 {
                v.value("projectileVelocity_", &mut self.projectile_velocity)?;
            }
            v.value("projectileGravity_", &mut self.projectile_gravity)?;
        }
        self.curve.visit(v, version >= 8, version < 6)?;
        if version >= 7 {
            v.value("animatePhysics_", &mut self.animate_physics)?;
            v.value("alwaysAnimate_", &mut self.always_animate)?;
        }
        if version >= 3 {
            self.trigger.visit(v)?;
        }
        Ok(())
    }
}

/// `GoldenAnimator`: animates the scale and position of the object.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct GoldenAnimator {
    pub scale: Option<bool>,
    pub scale_exponent: Option<[f32; 3]>,
    pub translate: Option<bool>,
    pub translate_vector: Option<[f32; 3]>,
    /// In the new format from version 1.
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub curve: AnimatorCurve,
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub trigger: AnimatorTrigger,
}

impl Visit for GoldenAnimator {
    const VERSIONS: RangeInclusive<i32> = 0..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        v.value("scale_", &mut self.scale)?;
        v.value("scaleExponent_", &mut self.scale_exponent)?;
        v.value("translate_", &mut self.translate)?;
        v.value("translateVector_", &mut self.translate_vector)?;
        self.curve.visit(v, version >= 1, false)?;
        self.trigger.visit(v)
    }
}
