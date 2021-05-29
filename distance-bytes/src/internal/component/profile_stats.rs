use crate::internal::{Serializable, VisitDirection, Visitor};
use crate::PlayerStats;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ProfileStats {
    pub accumulated_player_stats: PlayerStats,
    pub total_play_time: f64,
    pub level_editor_work_time: f64,
    pub level_editor_play_time: f64,
    pub solo_mode_play_times: Vec<f64>,
    pub splitscreen_mode_play_times: Vec<f64>,
    pub online_mode_play_times: Vec<f64>,
    pub adventure_start_mode_time: f64,
    pub trackmogrify_modifiers: Vec<Option<String>>,
}

impl Serializable for ProfileStats {
    const VERSION: i32 = 1;

    fn accept<V: Visitor>(&mut self, mut visitor: V, version: i32) -> Result<()> {
        if version >= 0 {
            let mut player_stats_version = 0;
            visitor.visit_i32("PlayerStatsVersion", &mut player_stats_version)?;
            self.accumulated_player_stats
                .accept(&mut visitor, player_stats_version)?;

            visitor.visit_f64("TotalPlayTime", &mut self.total_play_time)?;
            visitor.visit_f64("LevelEditorWorkTime", &mut self.level_editor_work_time)?;
            visitor.visit_f64("LevelEditorPlayTime", &mut self.level_editor_play_time)?;

            visitor.visit_array(
                "SoloModePlayTimes",
                &mut self.solo_mode_play_times,
                |visitor, item| visitor.visit_f64("SoloModePlayTimes", item),
            )?;
            visitor.visit_array(
                "SplitscreenModePlayTimes",
                &mut self.splitscreen_mode_play_times,
                |visitor, item| visitor.visit_f64("SplitscreenModePlayTimes", item),
            )?;
            visitor.visit_array(
                "OnlineModePlayTimes",
                &mut self.online_mode_play_times,
                |visitor, item| visitor.visit_f64("OnlineModePlayTimes", item),
            )?;
            if V::VISIT_DIRECTION == VisitDirection::In {
                initialize_mode_times_array(&mut self.solo_mode_play_times);
                initialize_mode_times_array(&mut self.splitscreen_mode_play_times);
                initialize_mode_times_array(&mut self.online_mode_play_times);
            }
        }

        if version >= 1 {
            visitor.visit_f64("AdventureStartTime", &mut self.adventure_start_mode_time)?;
            visitor.visit_array(
                "TrackmogrifyModifier",
                &mut self.trackmogrify_modifiers,
                |visitor, item| visitor.visit_string("TrackmogrifyModifier", item),
            )?;
        }

        Ok(())
    }
}

fn initialize_mode_times_array(mode_play_times: &mut Vec<f64>) {
    mode_play_times.resize(17, 0.0);
}
