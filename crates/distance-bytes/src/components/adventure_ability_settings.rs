use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `AdventureAbilitySettings`.
///
/// Version 0 uses the generic field format.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AdventureAbilitySettings {
    pub disable_flying: Option<bool>,
    pub disable_jumping: Option<bool>,
    pub disable_boosting: Option<bool>,
    pub disable_jet_rotating: Option<bool>,
    pub disable_show_scores: Option<bool>,
}

impl Visit for AdventureAbilitySettings {
    const VERSIONS: RangeInclusive<i32> = 1..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 1 {
            v.value("Disable Flying", &mut self.disable_flying)?;
            v.value("Disable Jumping", &mut self.disable_jumping)?;
            v.value("Disable Boosting", &mut self.disable_boosting)?;
            v.value("Disable Jet Rotating", &mut self.disable_jet_rotating)?;
            v.value("Disable Show Scores", &mut self.disable_show_scores)?;
        }
        Ok(())
    }
}
