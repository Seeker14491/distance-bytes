use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `ParticlesGPU`: a particle system. Versions 6 and earlier are read by the game's
/// `VisitOlderVersions`, with some fields in a different order.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct ParticlesGPU {
    /// `[r, g, b, a]`.
    pub color_start: Option<[f32; 4]>,
    /// `[r, g, b, a]`.
    pub color_middle: Option<[f32; 4]>,
    /// `[r, g, b, a]`.
    pub color_end: Option<[f32; 4]>,
    /// `ParticlesGPU.DrawModeType`. Version 3 and later.
    pub draw_mode: Option<i32>,
    pub brightness: Option<f32>,
    pub fade_sharpness: Option<f32>,
    pub image_index: Option<i32>,
    /// Version 2 and earlier.
    pub particle_scale: Option<f32>,
    /// Version 3 and later.
    pub scale_start: Option<f32>,
    /// Version 3 and later.
    pub scale_middle: Option<f32>,
    /// Version 3 and later.
    pub scale_end: Option<f32>,
    pub particle_count: Option<i32>,
    /// Version 8 and later.
    pub time_scale: Option<f32>,
    /// Before version 7, negative for reversed time.
    pub lifetime: Option<f32>,
    pub lifetime_varience: Option<f32>,
    /// Version 3 and later. Before version 7, it includes the lifetime.
    pub respawn_time: Option<f32>,
    /// Version 3 and later.
    pub respawn_time_varience: Option<f32>,
    pub spawn_time_varience: Option<f32>,
    /// Version 7 only.
    pub reverse_time: Option<bool>,
    /// `ParticlesGPU.SpawnBoundsType`. Version 1 and later; in version 2, the game maps
    /// `CameraXY` to `CameraXYZ` and `Sphere` to `CameraXY`.
    pub spawn_bounds_type: Option<i32>,
    /// The x of the spawn bounds, for cylinder and sphere bounds. Version 7 and later.
    pub bounds_inner_radius: Option<f32>,
    /// The y of the spawn bounds, for cylinder and sphere bounds. Version 7 and later.
    pub bounds_outer_radius: Option<f32>,
    /// The z of the spawn bounds, for cylinder and sphere bounds. Version 7 and later.
    pub bounds_height: Option<f32>,
    pub spawn_bounds: Option<[f32; 3]>,
    pub gravity: Option<f32>,
    pub particle_velocity: Option<f32>,
    /// Version 4 and later.
    pub gravity_latitude: Option<f32>,
    /// Version 4 and later.
    pub gravity_longitude: Option<f32>,
    /// Version 7 and later.
    pub local_gravity: Option<bool>,
    pub velocity_varience: Option<f32>,
    /// Version 3 and later.
    pub acceleration: Option<f32>,
    /// Version 1 and later.
    pub look_at_camera: Option<bool>,
    pub is_directional: Option<bool>,
    pub directional_length: Option<f32>,
    pub subdivision_count: Option<i32>,
    pub twirl: Option<f32>,
    pub cone_emitter: Option<bool>,
    /// The outer cone angle. Version 0 only.
    pub cone_angle: Option<f32>,
    /// Version 1 and later.
    pub cone_angle_inner: Option<f32>,
    /// Version 1 and later.
    pub cone_angle_outer: Option<f32>,
    /// Version 1 and later.
    pub cone_latitude: Option<f32>,
    /// Version 1 and later.
    pub cone_longitude: Option<f32>,
    pub is_static: Option<bool>,
    /// Version 0 only, ignored by the game.
    pub time: Option<f32>,
    /// Version 1 and later.
    pub time_offset: Option<f32>,
    /// Version 5 and later.
    pub one_shot: Option<bool>,
    /// Version 5 and later.
    pub trigger_distance: Option<f32>,
    pub seed: Option<i32>,
    /// Versions 0 and 1.
    pub camera_range: Option<f32>,
    /// Version 2 and later.
    pub fade_near: Option<f32>,
    /// Version 2 and later.
    pub fade_far: Option<f32>,
    /// Version 0 only.
    pub wrap_in_camera_xy: Option<bool>,
    /// Version 0 only.
    pub wrap_in_camera_xyz: Option<bool>,
    /// The quantize and repeater spawn patterns, version 1 and later.
    pub x_quantize: Option<i32>,
    pub x_repeater: Option<f32>,
    pub y_quantize: Option<i32>,
    pub y_repeater: Option<f32>,
    pub z_quantize: Option<i32>,
    pub z_repeater: Option<f32>,
    pub velocity_quantize: Option<i32>,
    pub velocity_repeater: Option<f32>,
    pub angle1_quantize: Option<i32>,
    pub angle1_repeater: Option<f32>,
    pub angle2_quantize: Option<i32>,
    pub angle2_repeater: Option<f32>,
    pub time_quantize: Option<i32>,
    pub time_repeater: Option<f32>,
}

impl Visit for ParticlesGPU {
    const VERSIONS: RangeInclusive<i32> = 0..=8;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        let new = version >= 7;
        v.value("ColorStart", &mut self.color_start)?;
        v.value("ColorMiddle", &mut self.color_middle)?;
        v.value("ColorEnd", &mut self.color_end)?;
        if version >= 3 {
            v.value("DrawMode", &mut self.draw_mode)?;
        }
        v.value("Brightness", &mut self.brightness)?;
        v.value("FadeSharpness", &mut self.fade_sharpness)?;
        v.value("ImageIndex", &mut self.image_index)?;
        if version >= 3 {
            v.value("ScaleStart", &mut self.scale_start)?;
            v.value("ScaleMiddle", &mut self.scale_middle)?;
            v.value("ScaleEnd", &mut self.scale_end)?;
        } else {
            v.value("ParticleScale", &mut self.particle_scale)?;
        }
        v.value("ParticleCount", &mut self.particle_count)?;
        if version >= 8 {
            v.value("Time Scale", &mut self.time_scale)?;
        }
        v.value("Lifetime", &mut self.lifetime)?;
        v.value("LifetimeVarience", &mut self.lifetime_varience)?;
        if version >= 3 {
            v.value("RespawnTime", &mut self.respawn_time)?;
            v.value("RespawnTimeVarience", &mut self.respawn_time_varience)?;
        }
        v.value("SpawnTimeVarience", &mut self.spawn_time_varience)?;
        if version == 7 {
            v.value("ReverseTime", &mut self.reverse_time)?;
        }
        if version >= 1 {
            v.value("SpawnBoundsType", &mut self.spawn_bounds_type)?;
        }
        if new {
            v.value("Bounds Inner Radius", &mut self.bounds_inner_radius)?;
            v.value("Bounds Outer Radius", &mut self.bounds_outer_radius)?;
            v.value("Bounds Height", &mut self.bounds_height)?;
        }
        v.value("SpawnBounds", &mut self.spawn_bounds)?;
        v.value("Gravity", &mut self.gravity)?;
        if new {
            v.value("ParticleVelocity", &mut self.particle_velocity)?;
        }
        if version >= 4 {
            v.value("Gravity Latitude", &mut self.gravity_latitude)?;
            v.value("Gravity Longitude", &mut self.gravity_longitude)?;
        }
        if new {
            v.value("LocalGravity", &mut self.local_gravity)?;
        } else {
            v.value("ParticleVelocity", &mut self.particle_velocity)?;
        }
        v.value("VelocityVarience", &mut self.velocity_varience)?;
        if version >= 3 {
            v.value("Acceleration", &mut self.acceleration)?;
        }
        if version >= 1 {
            v.value("LookAtCamera", &mut self.look_at_camera)?;
        }
        v.value("IsDirectional", &mut self.is_directional)?;
        v.value("DirectionalLength", &mut self.directional_length)?;
        v.value("SubdivisionCount", &mut self.subdivision_count)?;
        v.value("Twirl", &mut self.twirl)?;
        v.value("ConeEmitter", &mut self.cone_emitter)?;
        if version == 0 {
            v.value("ConeAngle", &mut self.cone_angle)?;
        } else {
            v.value("ConeAngleInner", &mut self.cone_angle_inner)?;
            v.value("ConeAngleOuter", &mut self.cone_angle_outer)?;
            v.value("ConeLatitude", &mut self.cone_latitude)?;
            v.value("ConeLongitude", &mut self.cone_longitude)?;
        }
        v.value("IsStatic", &mut self.is_static)?;
        if version == 0 {
            v.value("Time", &mut self.time)?;
        } else {
            v.value("TimeOffset", &mut self.time_offset)?;
        }
        if version >= 5 {
            v.value("OneShot", &mut self.one_shot)?;
            v.value("TriggerDistance", &mut self.trigger_distance)?;
        }
        v.value("Seed", &mut self.seed)?;
        if version == 1 {
            v.value("CameraRange", &mut self.camera_range)?;
        } else if version >= 2 {
            v.value("FadeNear", &mut self.fade_near)?;
            v.value("FadeFar", &mut self.fade_far)?;
        }
        if version == 0 {
            v.value("WrapInCameraXY", &mut self.wrap_in_camera_xy)?;
            v.value("WrapInCameraXYZ", &mut self.wrap_in_camera_xyz)?;
            v.value("CameraRange", &mut self.camera_range)?;
        } else {
            v.value("X Quantize", &mut self.x_quantize)?;
            v.value("X Repeater", &mut self.x_repeater)?;
            v.value("Y Quantize", &mut self.y_quantize)?;
            v.value("Y Repeater", &mut self.y_repeater)?;
            v.value("Z Quantize", &mut self.z_quantize)?;
            v.value("Z Repeater", &mut self.z_repeater)?;
            v.value("Velocity Quantize", &mut self.velocity_quantize)?;
            v.value("Velocity Repeater", &mut self.velocity_repeater)?;
            v.value("Angle1 Quantize", &mut self.angle1_quantize)?;
            v.value("Angle1 Repeater", &mut self.angle1_repeater)?;
            v.value("Angle2 Quantize", &mut self.angle2_quantize)?;
            v.value("Angle2 Repeater", &mut self.angle2_repeater)?;
            v.value("Time Quantize", &mut self.time_quantize)?;
            v.value("Time Repeater", &mut self.time_repeater)?;
        }
        Ok(())
    }
}
