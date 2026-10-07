use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `PulseAll`: Pulse materials, lights, and/or particles to a beat or interval.
///
/// Version 0 uses the generic field format.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PulseAll {
    /// `PulseAll.SupportedColors`. Named `PulseProperty` in version 1 only.
    pub material_property: Option<i32>,
    /// `PulseAll.PulseToMode`. Named `PulseTo` in version 1 only.
    pub pulse_to: Option<i32>,
    /// Named `MinBrightness` in version 1 only.
    pub minimum_material_brightness: Option<f32>,
    /// Named `DimTime` in version 1 only.
    pub beat_dim_speed: Option<f32>,
    /// Named `MIDIChannel` in version 1 only.
    pub midi_channel: Option<i32>,
    /// Named `PulseCoreChannel` in version 1 only.
    pub pulse_core_channel: Option<i32>,
    /// Version 2 and later.
    pub interval_pulse_frequency: Option<f32>,
    /// Version 3 and later.
    pub interval_offset: Option<f32>,
    /// `AnimatorBase.CurveType`. Version 4 only.
    pub interval_curve_type: Option<i32>,
    /// Version 2 and later.
    pub use_pulse_core: Option<bool>,
    /// Version 2 and later.
    pub pulse_materials: Option<bool>,
    /// Version 2 and later.
    pub pulse_lights: Option<bool>,
    /// Version 2 and later.
    pub minimum_light_brightness: Option<f32>,
    /// Version 2 and later.
    pub pulse_particles: Option<bool>,
    /// Version 2 and later.
    pub minimum_particles_brightness: Option<f32>,
}

impl Visit for PulseAll {
    const VERSIONS: RangeInclusive<i32> = 1..=4;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 1 {
            v.value("PulseProperty", &mut self.material_property)?;
            v.value("PulseTo", &mut self.pulse_to)?;
            v.value("MinBrightness", &mut self.minimum_material_brightness)?;
            v.value("DimTime", &mut self.beat_dim_speed)?;
            v.value("MIDIChannel", &mut self.midi_channel)?;
            v.value("PulseCoreChannel", &mut self.pulse_core_channel)?;
        }
        if version >= 2 {
            v.value("Pulse to", &mut self.pulse_to)?;
            v.value("MIDI Channel", &mut self.midi_channel)?;
            v.value("Beat Dim Speed", &mut self.beat_dim_speed)?;
            v.value("Interval Pulse Frequency", &mut self.interval_pulse_frequency)?;
            if version >= 3 {
                v.value("Interval Offset", &mut self.interval_offset)?;
            }
            if version >= 4 {
                v.value("Interval Curve Type", &mut self.interval_curve_type)?;
            }
            v.value("Use PulseCore", &mut self.use_pulse_core)?;
            v.value("PulseCore Channel", &mut self.pulse_core_channel)?;
            v.value("Pulse Materials", &mut self.pulse_materials)?;
            v.value("Material Property", &mut self.material_property)?;
            v.value("Minimum Material Brightness", &mut self.minimum_material_brightness)?;
            v.value("Pulse Lights", &mut self.pulse_lights)?;
            v.value("Minimum Light Brightness", &mut self.minimum_light_brightness)?;
            v.value("Pulse Particles", &mut self.pulse_particles)?;
            v.value("Minimum Particles Brightness", &mut self.minimum_particles_brightness)?;
        }
        Ok(())
    }
}
