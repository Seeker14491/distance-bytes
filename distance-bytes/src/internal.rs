pub use component::ComponentId;

use anyhow::Result;
use auto_impl::auto_impl;
use chrono::TimeZone;
use component::Component;
use enum_primitive_derive::Primitive;
use num_traits::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::path::Path;

pub(crate) mod animator_base;
pub(crate) mod component;
pub(crate) mod deserializer;
pub(crate) mod level_info;
pub(crate) mod player_stats;
pub(crate) mod serializer;

mod string;
mod util;

#[cfg(test)]
mod test_util;

pub const ZEROS_VECTOR_3: Vector3 = Vector3 {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};

pub const ONES_VECTOR_3: Vector3 = Vector3 {
    x: 1.0,
    y: 1.0,
    z: 1.0,
};

pub const DEFAULT_QUATERNION: Quaternion = Quaternion {
    v: Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    },
    s: 1.0,
};

pub(crate) const INVALID_INT: i32 = -127;
pub(crate) const INVALID_FLOAT: f32 = -10_000.0;

pub(crate) const INVALID_VECTOR_3: Vector3 = Vector3 {
    x: INVALID_FLOAT,
    y: INVALID_FLOAT,
    z: INVALID_FLOAT,
};

pub(crate) const INVALID_QUATERNION: Quaternion = Quaternion {
    v: INVALID_VECTOR_3,
    s: INVALID_FLOAT,
};

pub(crate) const EMPTY_MARK: i32 = 0x7FFF_FFFD;

pub type Vector3 = mint::Vector3<f32>;
pub type Quaternion = mint::Quaternion<f32>;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GameObject {
    pub name: String,
    pub guid: u32,
    pub components: Vec<Component>,
}

impl GameObject {
    pub fn read_from_reader(reader: impl Read + Seek) -> Result<GameObject> {
        deserializer::read_game_object(reader)
    }

    pub fn read_from_file(path: impl AsRef<Path>) -> Result<GameObject> {
        let mut file = BufReader::new(File::open(path.as_ref())?);
        deserializer::read_game_object(&mut file)
    }

    pub fn write_to_reader(&mut self, writer: impl Write + Seek) -> Result<()> {
        serializer::write_game_object(writer, self)
    }

    pub fn write_to_file(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let mut file = BufWriter::new(File::create(path.as_ref())?);
        serializer::write_game_object(&mut file, self)
    }
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MaterialInfo {
    pub mat_name: Option<String>,
    pub colors: Vec<MaterialColorInfo>,
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MaterialColorInfo {
    pub name: Option<String>,
    pub color: Color,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }
}

#[derive(
    Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Primitive,
)]
pub enum MedalStatus {
    None = 0,
    DidNotFinish = 1,
    Completed = 2,
    Bronze = 3,
    Silver = 4,
    Gold = 5,
    Diamond = 6,
    Count = 7,
}

impl Default for MedalStatus {
    fn default() -> Self {
        MedalStatus::None
    }
}

/// An instant in time.
///
/// The internal representation is based on the .NET [System.DateTime][1] type, specifically the
/// values returned from its `FromBinary()` and `ToBinary()` methods. However, the timezone is
/// assumed to always be UTC.
///
/// [1]: https://docs.microsoft.com/en-us/dotnet/api/system.datetime?view=netframework-3.5
#[derive(
    Debug, Copy, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize,
)]
pub struct DistanceDateTime(pub i64);

impl DistanceDateTime {
    const FILE_TIME_OFFSET: i64 = 504911232000000000;

    pub fn from_chrono_datetime(chrono_datetime: chrono::DateTime<chrono::Utc>) -> Option<Self> {
        let file_time = epochs::to_windows_file(chrono_datetime.naive_utc());
        let ticks = if file_time >= 0 {
            file_time.checked_add(Self::FILE_TIME_OFFSET)?
        } else {
            return None;
        };

        if !(0..=3155378975999999999).contains(&ticks) {
            return None;
        }

        let utc_flag = 1 << 62;
        Some(DistanceDateTime(ticks | utc_flag))
    }

    pub fn to_chrono_datetime(self) -> Option<chrono::DateTime<chrono::Utc>> {
        let ticks_mask = (1 << 62) - 1;
        let ticks = self.0 & ticks_mask;

        let file_time = ticks.checked_sub(Self::FILE_TIME_OFFSET)?;
        let naive_date_time = epochs::windows_file(file_time)?;

        Some(chrono::Utc.from_utc_datetime(&naive_date_time))
    }
}

#[derive(
    Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Primitive,
)]
pub enum LevelDifficulty {
    Casual = 0,
    Normal = 1,
    Advanced = 2,
    Expert = 3,
    Nightmare = 4,
    None = 5,
}

impl Default for LevelDifficulty {
    fn default() -> Self {
        LevelDifficulty::Casual
    }
}

#[derive(
    Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Primitive,
)]
pub enum LevelType {
    None = 0,
    Example = 1,
    Official = 2,
    Workshop = 3,
    My = 4,
    Levels = 5,
    Community = 6,
    Count = 7,
}

impl Default for LevelType {
    fn default() -> Self {
        LevelType::None
    }
}

#[allow(non_camel_case_types)]
#[derive(
    Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Primitive,
)]
pub enum MusicCueId {
    None = 0,
    Grime = 1,
    Engage = 2,
    KaBamSon = 3,
    Momenta = 4,
    TheAlliance = 5,
    Continuum = 6,
    Drive = 7,
    GroundZero = 8,
    ColdWind = 9,
    Halloween = 10,
    Corruption = 11,
    Storm = 12,
    The_Tunnel = 15,
    Darkness_Intense = 16,
    WereLivingInASociety = 17,
    RandomChordSomething = 18,
    Aftermath = 19,
    TheMonolith = 20,
    FadedAway = 26,
    DarknessSlight = 27,
    Spectrum = 28,
    GZLight = 29,
    GZGate = 30,
    GZInsane = 31,
    CarolOfTheBells = 32,
    DigitalApathy = 33,
    TheManor = 34,
    DevValue_ = 1000,
    FFTTestLoop = 1001,
    IntroCutscene = 1002,
    AftermathLoop = 1003,
    Continuum2 = 1004,
    GZInsaneSlow = 1005,
    TunnelHorror = 1006,
    SpectrumSlow = 1007,
    IntroCutsceneV2 = 1008,
    NewBSAmbient = 1009,
    Convolution = 1010,
    Forge = 1011,
    DarknessIntenseV2 = 1012,
    NSFinalBoot = 1013,
    Scratch = 1014,
    Flip = 1015,
    NegativeSpace = 1016,
    Abstraction = 1017,
    ForgeLoop = 1018,
    GrimeLoop = 1019,
    AbstractionLoop = 1020,
    AbstractionStrings = 1021,
    AbstractionStringsDU = 1022,
    DUEnding = 1023,
    AbandonedUtopia = 1024,
    DeeperIntoTheVoid = 1025,
    Low = 1026,
    Ambient2 = 1027,
    EchoesIntro = 1028,
    TepidPants = 1029,
    Violent = 1030,
    Belly = 1031,
    Perfect = 1032,
    Sentinel = 1033,
    EchoesStorm = 1034,
    Void = 1035,
    Isolation = 1036,
    IsolationReturn = 1037,
    SecretTeleport = 1038,
    Echoes = 1040,
    EnemyIntro = 1041,
    DarkWind = 1042,
    Machine17 = 1043,
    Machine17Remix = 1044,
    Observatory = 1045,
    Resonance = 1046,
    Deterence = 1047,
    Terminus = 1048,
    Mobilization = 1049,
    Collapse = 1050,
    TheOtherSide = 1051,
    twoHOTtwoPANTS = 1052,
    TotalCount_ = 1053,
}

impl Default for MusicCueId {
    fn default() -> Self {
        MusicCueId::None
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum VisitDirection {
    In,
    Out,
}

#[auto_impl(&mut)]
pub(crate) trait Visitor
where
    Self: Sized,
{
    // Workaround for a problem with referencing `Self` while using the `auto_impl` macro
    type Self_: Visitor;

    const VISIT_DIRECTION: VisitDirection;

    fn visit_bool(&mut self, name: &str, value: &mut bool) -> Result<()>;
    fn visit_i32(&mut self, name: &str, value: &mut i32) -> Result<()>;
    fn visit_u32(&mut self, name: &str, value: &mut u32) -> Result<()>;
    fn visit_i64(&mut self, name: &str, value: &mut i64) -> Result<()>;
    fn visit_f32(&mut self, name: &str, value: &mut f32) -> Result<()>;
    fn visit_f64(&mut self, name: &str, value: &mut f64) -> Result<()>;
    fn visit_string(&mut self, name: &str, value: &mut Option<String>) -> Result<()>;
    fn visit_datetime(&mut self, name: &str, value: &mut DistanceDateTime) -> Result<()>;
    fn visit_vector_3(&mut self, name: &str, value: &mut Vector3) -> Result<()>;
    fn visit_quaternion(&mut self, name: &str, value: &mut Quaternion) -> Result<()>;
    fn visit_reference(&mut self, name: &str, value: &mut u32) -> Result<()>;
    fn visit_reference_array(
        &mut self,
        array_name: &str,
        element_name: &str,
        value: &mut Vec<u32>,
    ) -> Result<()>;

    fn visit_material_info(&mut self, _name: &str, value: &mut MaterialInfo) -> Result<()> {
        self.visit_string("MatName", &mut value.mat_name)?;
        self.visit_array("", &mut value.colors, |visitor, element| {
            visitor.visit_material_color_info("", element)
        })?;

        Ok(())
    }

    fn visit_material_color_info(
        &mut self,
        _name: &str,
        value: &mut MaterialColorInfo,
    ) -> Result<()> {
        self.visit_string("Name", &mut value.name)?;

        if Self::VISIT_DIRECTION == VisitDirection::In {
            value.color = Color::new(1.0, 1.0, 1.0, 1.0);
        }
        self.visit_color("Color", &mut value.color)?;

        Ok(())
    }

    fn visit_color(&mut self, _name: &str, value: &mut Color) -> Result<()> {
        self.visit_f32("r", &mut value.r)?;
        self.visit_f32("g", &mut value.g)?;
        self.visit_f32("b", &mut value.b)?;
        self.visit_f32("a", &mut value.a)?;

        Ok(())
    }

    fn visit_array<T, F>(
        &mut self,
        element_name: &str,
        array: &mut Vec<T>,
        visit_t_fn: F,
    ) -> Result<()>
    where
        T: Default,
        F: FnMut(&mut Self::Self_, &mut T) -> Result<()>;

    fn visit_dictionary_generic<Key, Value, KeyAcceptor, ValueAcceptor>(
        &mut self,
        name: &str,
        value: &mut Option<HashMap<Key, Value>>,
        key_acceptor: KeyAcceptor,
        value_acceptor: ValueAcceptor,
        default_key: Key,
        default_value: Value,
    ) -> Result<()>
    where
        Key: Clone + Hash + Eq,
        Value: Clone,
        KeyAcceptor: FnMut(&mut Self::Self_, &mut Key) -> Result<()>,
        ValueAcceptor: FnMut(&mut Self::Self_, &mut Value) -> Result<()>;

    fn visit_dictionary_i32_to_bool(
        &mut self,
        name: &str,
        value: &mut Option<HashMap<i32, bool>>,
    ) -> Result<()> {
        self.visit_dictionary_generic(
            name,
            value,
            |visitor, key| visitor.visit_i32("Key", key),
            |visitor, value| visitor.visit_bool("Val", value),
            -1,
            false,
        )
    }

    fn visit_children(&mut self, value: &mut Vec<GameObject>) -> Result<()>;

    fn visit_enum<T: FromPrimitive + ToPrimitive>(
        &mut self,
        name: &str,
        value: &mut T,
    ) -> Result<()> {
        let mut n = value.to_i32().unwrap();
        self.visit_i32(name, &mut n)?;

        if Self::VISIT_DIRECTION == VisitDirection::In {
            *value = T::from_i32(n).unwrap();
        }

        Ok(())
    }

    fn visit_serial_collider_deprecated(&mut self, _name: &str) -> Result<()> {
        self.visit_bool("IsTrigger", &mut false)?;
        self.visit_string("PhysicMaterialName", &mut None)?;

        Ok(())
    }
}

#[auto_impl(&mut, Box)]
pub(crate) trait Serializable: Default {
    const VERSION: i32;

    fn accept<V: Visitor>(&mut self, visitor: V, version: i32) -> Result<()>;
}
