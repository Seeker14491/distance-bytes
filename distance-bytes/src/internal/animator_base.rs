use crate::internal::{VisitDirection, Visitor};
use anyhow::Result;
use enum_primitive_derive::Primitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AnimatorBase {
    pub delay: f32,
    pub duration: f32,
    pub time_offset: f32,
    pub loop_: bool,
    pub extend: bool,
    pub curve_type: AnimatorBaseCurveType,
    pub editor_animation_t: f32,
    pub custom_pong_values: bool,
    pub pong_delay: f32,
    pub pong_duration: f32,
    pub pong_curve_type: AnimatorBaseCurveType,
    pub default_action: AnimatorBaseTriggerAction,
    pub on_action: AnimatorBaseTriggerAction,
    pub on_wait_for_animation_finish: bool,
    pub on_reset: bool,
    pub off_action: AnimatorBaseTriggerAction,
    pub off_wait_for_animation_finish: bool,
    pub off_reset: bool,
}

impl AnimatorBase {
    pub(crate) fn visit_curve<V: Visitor>(&mut self, mut visitor: V) -> Result<()> {
        visitor.visit_f32("delay_", &mut self.delay)?;
        visitor.visit_f32("duration_", &mut self.duration)?;
        visitor.visit_f32("timeOffset_", &mut self.time_offset)?;
        visitor.visit_bool("loop_", &mut self.loop_)?;
        visitor.visit_bool("extend_", &mut self.extend)?;
        visitor.visit_enum("curveType_", &mut self.curve_type)?;
        visitor.visit_f32("editorAnimationT_", &mut self.editor_animation_t)?;
        visitor.visit_bool("customPongValues_", &mut self.custom_pong_values)?;
        visitor.visit_f32("pongDelay_", &mut self.pong_delay)?;
        visitor.visit_f32("pongDuration_", &mut self.pong_duration)?;
        visitor.visit_enum("pongCurveType_", &mut self.pong_curve_type)?;

        if V::VISIT_DIRECTION == VisitDirection::In && !self.custom_pong_values {
            self.pong_delay = self.delay;
            self.pong_duration = self.duration;
            self.pong_curve_type = self.curve_type.opposite();
        }

        Ok(())
    }

    pub(crate) fn visit_curve_old<V: Visitor>(
        &mut self,
        mut visitor: V,
        old_animation_t: bool,
    ) -> Result<bool> {
        visitor.visit_f32("delay_", &mut self.delay)?;
        visitor.visit_f32("duration_", &mut self.duration)?;
        visitor.visit_f32("timeOffset_", &mut self.time_offset)?;
        visitor.visit_bool("loop_", &mut self.loop_)?;

        let mut extrapolation_type = AnimatorBaseExtrapolationTypeObsolete::PingPong;
        visitor.visit_enum("extrapolationType_", &mut extrapolation_type)?;
        self.extend = extrapolation_type == AnimatorBaseExtrapolationTypeObsolete::Extend;

        visitor.visit_enum("curveType_", &mut self.curve_type)?;
        if old_animation_t {
            let mut centered_animation = false;
            visitor.visit_bool("centeredAnimation_", &mut centered_animation)?;
            self.editor_animation_t = match centered_animation {
                true => 0.5,
                false => 0.0,
            }
        } else {
            visitor.visit_f32("editorAnimationT_", &mut self.editor_animation_t)?;
        }
        visitor.visit_bool("customPongValues_", &mut self.custom_pong_values)?;
        visitor.visit_f32("pongDelay_", &mut self.pong_delay)?;
        visitor.visit_f32("pongDuration_", &mut self.pong_duration)?;
        visitor.visit_enum("pongCurveType_", &mut self.pong_curve_type)?;

        self.pong_curve_type = self.pong_curve_type.opposite();
        if !self.custom_pong_values {
            self.pong_delay = self.delay;
            self.pong_duration = self.duration;
            self.pong_curve_type = self.curve_type.opposite();
        }

        Ok(extrapolation_type == AnimatorBaseExtrapolationTypeObsolete::PingPong)
    }

    pub(crate) fn visit_trigger<V: Visitor>(&mut self, mut visitor: V) -> Result<()> {
        visitor.visit_enum("defaultAction_", &mut self.default_action)?;

        visitor.visit_enum("onAction_", &mut self.on_action)?;
        visitor.visit_bool(
            "onWaitForAnimationFinish_",
            &mut self.on_wait_for_animation_finish,
        )?;
        visitor.visit_bool("onReset_", &mut self.on_reset)?;

        visitor.visit_enum("offAction_", &mut self.off_action)?;
        visitor.visit_bool(
            "offWaitForAnimationFinish_",
            &mut self.off_wait_for_animation_finish,
        )?;
        visitor.visit_bool("offReset_", &mut self.off_reset)?;

        Ok(())
    }

    pub(crate) fn upgrade_to_new_ping_pong(&mut self, apply_ping_pong: bool) {
        if !apply_ping_pong {
            return;
        }

        let mut flag = false;
        if let AnimatorBaseTriggerAction::Play | AnimatorBaseTriggerAction::PlayReverse =
            self.on_action
        {
            self.on_action = AnimatorBaseTriggerAction::PingPong;
            flag = true;
        }
        if let AnimatorBaseTriggerAction::Play | AnimatorBaseTriggerAction::PlayReverse =
            self.off_action
        {
            self.off_action = AnimatorBaseTriggerAction::PingPong;
            flag = true;
        }
        if !flag
            || self.default_action == AnimatorBaseTriggerAction::Play
            || self.default_action == AnimatorBaseTriggerAction::PlayReverse
        {
            self.default_action = AnimatorBaseTriggerAction::PingPong;
        }
    }
}

impl Default for AnimatorBase {
    fn default() -> Self {
        AnimatorBase {
            delay: 1.0,
            duration: 1.0,
            time_offset: 0.0,
            loop_: true,
            extend: false,
            curve_type: Default::default(),
            editor_animation_t: 0.0,
            custom_pong_values: false,
            pong_delay: 1.0,
            pong_duration: 1.0,
            pong_curve_type: Default::default(),
            default_action: AnimatorBaseTriggerAction::PingPong,
            on_action: Default::default(),
            on_wait_for_animation_finish: false,
            on_reset: false,
            off_action: Default::default(),
            off_wait_for_animation_finish: false,
            off_reset: false,
        }
    }
}

#[derive(
    Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Primitive,
)]
pub enum AnimatorBaseCurveType {
    Linear = 0,
    EaseIn = 1,
    EaseOut = 2,
    EaseInOut = 3,
    Quadratic = 4,
    InverseQuadratic = 5,
    SinWave = 6,
}

impl AnimatorBaseCurveType {
    pub fn opposite(self) -> Self {
        match self {
            AnimatorBaseCurveType::EaseIn => AnimatorBaseCurveType::EaseOut,
            AnimatorBaseCurveType::EaseOut => AnimatorBaseCurveType::EaseIn,
            AnimatorBaseCurveType::Quadratic => AnimatorBaseCurveType::InverseQuadratic,
            AnimatorBaseCurveType::InverseQuadratic => AnimatorBaseCurveType::Quadratic,
            other => other,
        }
    }
}

impl Default for AnimatorBaseCurveType {
    fn default() -> Self {
        AnimatorBaseCurveType::EaseInOut
    }
}

#[derive(
    Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Primitive,
)]
pub enum AnimatorBaseTriggerAction {
    None = 0,
    Play = 1,
    PlayReverse = 2,
    Stop = 3,
    PingPong = 4,
}

impl Default for AnimatorBaseTriggerAction {
    fn default() -> Self {
        AnimatorBaseTriggerAction::None
    }
}

#[derive(
    Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Primitive,
)]
pub enum AnimatorBaseExtrapolationTypeObsolete {
    Normal = 0,
    PingPong = 1,
    Extend = 2,
}

impl Default for AnimatorBaseExtrapolationTypeObsolete {
    fn default() -> Self {
        AnimatorBaseExtrapolationTypeObsolete::Normal
    }
}
