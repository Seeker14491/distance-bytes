use crate::internal::{Serializable, Visitor};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PlayerStats {
    pub total_deaths_count: i64,
    pub deaths_by_laser_count: i64,
    pub deaths_by_reset_count: i64,
    pub deaths_by_impact_count: i64,
    pub deaths_by_overheat_count: i64,
    pub deaths_by_kill_grid_count: i64,
    pub car_as_gibs_time: f64,
    pub meters_driven: f64,
    pub meters_driven_forward: f64,
    pub meters_driven_reverse: f64,
    pub meters_airborne_flying: f64,
    pub meters_airborne_not_flying: f64,
    pub meters_wall_riding: f64,
    pub meters_ceiling_riding: f64,
    pub meters_grinding: f64,
    pub boost_held_down_time: f64,
    pub grip_held_down_time: f64,
    pub split_count: i64,
    pub impact_count: i64,
    pub checkpoints_hit_count: i64,
    pub jump_count: i64,
    pub wings_open_count: i64,
    pub wings_close_count: i64,
    pub horn_count: i64,
    pub trick_count: i64,
    pub total_points: i64,
    pub broken_lamp_count: i64,
    pub broken_pumpkin_count: i64,
    pub broken_egg_count: i64,
    pub top_speed_meters_per_second: f64,
    pub top_forward_speed_meters_per_second: f64,
    pub top_reverse_speed_meters_per_second: f64,
    pub cooldown_trigger_hit_count: i64,
}

impl Serializable for PlayerStats {
    const VERSION: i32 = 1;

    fn accept<V: Visitor>(&mut self, mut visitor: V, version: i32) -> Result<()> {
        if version >= 0 {
            visitor.visit_i64("TotalDeathsCount", &mut self.total_deaths_count)?;
            visitor.visit_i64("DeathsByLaserCount", &mut self.deaths_by_laser_count)?;
            visitor.visit_i64("DeathsByResetCount", &mut self.deaths_by_reset_count)?;
            visitor.visit_i64("DeathsByImpactCount", &mut self.deaths_by_impact_count)?;
            visitor.visit_i64("DeathsByOverheatCount", &mut self.deaths_by_overheat_count)?;
            visitor.visit_i64("DeathsByKillGridCount", &mut self.deaths_by_kill_grid_count)?;
            visitor.visit_f64("CarAsGibsTime", &mut self.car_as_gibs_time)?;
            visitor.visit_f64("MetersDriven", &mut self.meters_driven)?;
            visitor.visit_f64("MetersDrivenForward", &mut self.meters_driven_forward)?;
            visitor.visit_f64("MetersDrivenReverse", &mut self.meters_driven_reverse)?;
            visitor.visit_f64("MetersAirborneFlying", &mut self.meters_airborne_flying)?;
            visitor.visit_f64(
                "MetersAirborneNotFlying",
                &mut self.meters_airborne_not_flying,
            )?;
            visitor.visit_f64("MetersWallRiding", &mut self.meters_wall_riding)?;
            visitor.visit_f64("MetersCeilingRiding", &mut self.meters_ceiling_riding)?;
            visitor.visit_f64("MetersGrinding", &mut self.meters_grinding)?;
            visitor.visit_f64("BoostHeldDownTime", &mut self.boost_held_down_time)?;
            visitor.visit_f64("GripHeldDownTime", &mut self.grip_held_down_time)?;
            visitor.visit_i64("SplitCount", &mut self.split_count)?;
            visitor.visit_i64("ImpactCount", &mut self.impact_count)?;
            visitor.visit_i64("CheckpointsHitCount", &mut self.checkpoints_hit_count)?;
            visitor.visit_i64("JumpCount", &mut self.jump_count)?;
            visitor.visit_i64("WingsOpenCount", &mut self.wings_open_count)?;
            visitor.visit_i64("WingsCloseCount", &mut self.wings_close_count)?;
            visitor.visit_i64("HornCount", &mut self.horn_count)?;
            visitor.visit_i64("TrickCount", &mut self.trick_count)?;
            visitor.visit_i64("TotalPoints", &mut self.total_points)?;
            visitor.visit_i64("BrokenLampCount", &mut self.broken_lamp_count)?;
            visitor.visit_i64("BrokenPumpkinCount", &mut self.broken_pumpkin_count)?;
            visitor.visit_i64("BrokenEggCount", &mut self.broken_egg_count)?;
        }

        if version >= 1 {
            visitor.visit_f64(
                "TopSpeedMetersPerSecond",
                &mut self.top_speed_meters_per_second,
            )?;
            visitor.visit_f64(
                "TopForwardSpeedMetersPerSecond",
                &mut self.top_forward_speed_meters_per_second,
            )?;
            visitor.visit_f64(
                "TopReverseSpeedMetersPerSecond",
                &mut self.top_reverse_speed_meters_per_second,
            )?;
            visitor.visit_i64(
                "CooldownTriggerHitCount",
                &mut self.cooldown_trigger_hit_count,
            )?;
        }

        Ok(())
    }
}
