use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `CinematicCamera`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct CinematicCamera {
    pub start_time: Option<f32>,
    /// Version 7 only.
    pub time_before_returning: Option<f32>,
    /// Version 7 only.
    pub transition_time_to: Option<f32>,
    /// Version 7 only.
    pub transition_time_from: Option<f32>,
    /// Version 1 and later.
    pub use_focal_point: Option<bool>,
    /// Version 1 and later.
    pub focal_point_id: Option<i32>,
    /// Version 2 and later.
    pub fov: Option<f32>,
    /// Version 3 and later.
    pub animate_on_start: Option<bool>,
    /// Version 3 and later.
    pub animate_focal_point_on_start: Option<bool>,
    /// Version 4 and later.
    pub black_fade_in_out: Option<bool>,
    /// Version 4 and later.
    pub black_fade_time: Option<f32>,
    /// Version 5 and later.
    pub is_menu_camera: Option<bool>,
    /// Version 4 and later.
    pub menu_index: Option<i32>,
    /// Version 4 and later.
    pub menu_duration: Option<f32>,
    /// Version 6 and later.
    pub track_car_from_front: Option<bool>,
}

impl Visit for CinematicCamera {
    const VERSIONS: RangeInclusive<i32> = 0..=7;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value("StartTime", &mut self.start_time)?;
        }
        if version >= 7 {
            v.value("Time Before Returning", &mut self.time_before_returning)?;
            v.value("Transition Time To", &mut self.transition_time_to)?;
            v.value("Transition Time From", &mut self.transition_time_from)?;
        }
        if version >= 1 {
            v.value("UseFocalPoint", &mut self.use_focal_point)?;
            v.value("FocalPointID", &mut self.focal_point_id)?;
        }
        if version >= 2 {
            v.value("FOV", &mut self.fov)?;
        }
        if version >= 3 {
            v.value("AnimateOnStart", &mut self.animate_on_start)?;
            v.value(
                "AnimateFocalPointOnStart",
                &mut self.animate_focal_point_on_start,
            )?;
        }
        if version >= 4 {
            v.value("BlackFadeInOut", &mut self.black_fade_in_out)?;
            v.value("BlackFadeTime", &mut self.black_fade_time)?;
            if version >= 5 {
                v.value("isMenuCamera_", &mut self.is_menu_camera)?;
            }
            v.value("MenuIndex", &mut self.menu_index)?;
            v.value("MenuDuration", &mut self.menu_duration)?;
        }
        if version >= 6 {
            v.value("Track Car From Front", &mut self.track_car_from_front)?;
        }
        Ok(())
    }
}
