use std::ops::RangeInclusive;

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `ProfileStats`: a profile's statistics, kept with its progress.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProfileStats {
    /// The version of `player_stats`' format. The game reads them as [`PlayerStats::VERSION`]
    /// when this is missing.
    pub player_stats_version: Option<i32>,
    pub player_stats: PlayerStats,
    pub total_play_time: Option<f64>,
    pub level_editor_work_time: Option<f64>,
    pub level_editor_play_time: Option<f64>,
    /// Indexed by `GameModeID`.
    pub solo_mode_play_times: Vec<Option<f64>>,
    /// Indexed by `GameModeID`.
    pub splitscreen_mode_play_times: Vec<Option<f64>>,
    /// Indexed by `GameModeID`.
    pub online_mode_play_times: Vec<Option<f64>>,
    /// Version 1 and later.
    pub adventure_start_time: Option<f64>,
    /// Version 1 and later.
    pub trackmogrify_modifiers: Vec<Option<DString>>,
}

/// `PlayerStats`: totals of a player's actions, in [`ProfileStats`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlayerStats {
    pub total_deaths_count: Option<i64>,
    pub deaths_by_laser_count: Option<i64>,
    pub deaths_by_reset_count: Option<i64>,
    pub deaths_by_impact_count: Option<i64>,
    pub deaths_by_overheat_count: Option<i64>,
    pub deaths_by_kill_grid_count: Option<i64>,
    pub car_as_gibs_time: Option<f64>,
    pub meters_driven: Option<f64>,
    pub meters_driven_forward: Option<f64>,
    pub meters_driven_reverse: Option<f64>,
    pub meters_airborne_flying: Option<f64>,
    pub meters_airborne_not_flying: Option<f64>,
    pub meters_wall_riding: Option<f64>,
    pub meters_ceiling_riding: Option<f64>,
    pub meters_grinding: Option<f64>,
    pub boost_held_down_time: Option<f64>,
    pub grip_held_down_time: Option<f64>,
    pub split_count: Option<i64>,
    pub impact_count: Option<i64>,
    pub checkpoints_hit_count: Option<i64>,
    pub jump_count: Option<i64>,
    pub wings_open_count: Option<i64>,
    pub wings_close_count: Option<i64>,
    pub horn_count: Option<i64>,
    pub trick_count: Option<i64>,
    pub total_points: Option<i64>,
    pub broken_lamp_count: Option<i64>,
    pub broken_pumpkin_count: Option<i64>,
    pub broken_egg_count: Option<i64>,
    /// Version 1 and later.
    pub top_speed_meters_per_second: Option<f64>,
    /// Version 1 and later.
    pub top_forward_speed_meters_per_second: Option<f64>,
    /// Version 1 and later.
    pub top_reverse_speed_meters_per_second: Option<f64>,
    /// Version 1 and later.
    pub cooldown_trigger_hit_count: Option<i64>,
}

impl PlayerStats {
    /// The format version the game writes.
    pub const VERSION: i32 = 1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value("TotalDeathsCount", &mut self.total_deaths_count)?;
            v.value("DeathsByLaserCount", &mut self.deaths_by_laser_count)?;
            v.value("DeathsByResetCount", &mut self.deaths_by_reset_count)?;
            v.value("DeathsByImpactCount", &mut self.deaths_by_impact_count)?;
            v.value("DeathsByOverheatCount", &mut self.deaths_by_overheat_count)?;
            v.value("DeathsByKillGridCount", &mut self.deaths_by_kill_grid_count)?;
            v.value("CarAsGibsTime", &mut self.car_as_gibs_time)?;
            v.value("MetersDriven", &mut self.meters_driven)?;
            v.value("MetersDrivenForward", &mut self.meters_driven_forward)?;
            v.value("MetersDrivenReverse", &mut self.meters_driven_reverse)?;
            v.value("MetersAirborneFlying", &mut self.meters_airborne_flying)?;
            v.value(
                "MetersAirborneNotFlying",
                &mut self.meters_airborne_not_flying,
            )?;
            v.value("MetersWallRiding", &mut self.meters_wall_riding)?;
            v.value("MetersCeilingRiding", &mut self.meters_ceiling_riding)?;
            v.value("MetersGrinding", &mut self.meters_grinding)?;
            v.value("BoostHeldDownTime", &mut self.boost_held_down_time)?;
            v.value("GripHeldDownTime", &mut self.grip_held_down_time)?;
            v.value("SplitCount", &mut self.split_count)?;
            v.value("ImpactCount", &mut self.impact_count)?;
            v.value("CheckpointsHitCount", &mut self.checkpoints_hit_count)?;
            v.value("JumpCount", &mut self.jump_count)?;
            v.value("WingsOpenCount", &mut self.wings_open_count)?;
            v.value("WingsCloseCount", &mut self.wings_close_count)?;
            v.value("HornCount", &mut self.horn_count)?;
            v.value("TrickCount", &mut self.trick_count)?;
            v.value("TotalPoints", &mut self.total_points)?;
            v.value("BrokenLampCount", &mut self.broken_lamp_count)?;
            v.value("BrokenPumpkinCount", &mut self.broken_pumpkin_count)?;
            v.value("BrokenEggCount", &mut self.broken_egg_count)?;
        }
        if version >= 1 {
            v.value(
                "TopSpeedMetersPerSecond",
                &mut self.top_speed_meters_per_second,
            )?;
            v.value(
                "TopForwardSpeedMetersPerSecond",
                &mut self.top_forward_speed_meters_per_second,
            )?;
            v.value(
                "TopReverseSpeedMetersPerSecond",
                &mut self.top_reverse_speed_meters_per_second,
            )?;
            v.value(
                "CooldownTriggerHitCount",
                &mut self.cooldown_trigger_hit_count,
            )?;
        }
        Ok(())
    }
}

impl Visit for ProfileStats {
    const VERSIONS: RangeInclusive<i32> = 0..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        v.value("PlayerStatsVersion", &mut self.player_stats_version)?;
        let stats_version = self.player_stats_version.unwrap_or(PlayerStats::VERSION);
        self.player_stats.visit(v, stats_version)?;
        v.value("TotalPlayTime", &mut self.total_play_time)?;
        v.value("LevelEditorWorkTime", &mut self.level_editor_work_time)?;
        v.value("LevelEditorPlayTime", &mut self.level_editor_play_time)?;
        v.array(
            "SoloModePlayTimes",
            &mut self.solo_mode_play_times,
            |v, t| v.value("SoloModePlayTimes", t),
        )?;
        v.array(
            "SplitscreenModePlayTimes",
            &mut self.splitscreen_mode_play_times,
            |v, t| v.value("SplitscreenModePlayTimes", t),
        )?;
        v.array(
            "OnlineModePlayTimes",
            &mut self.online_mode_play_times,
            |v, t| v.value("OnlineModePlayTimes", t),
        )?;
        if version >= 1 {
            v.value("AdventureStartTime", &mut self.adventure_start_time)?;
            v.array(
                "TrackmogrifyModifier",
                &mut self.trackmogrify_modifiers,
                |v, m| v.value("TrackmogrifyModifier", m),
            )?;
        }
        Ok(())
    }
}
