//! Reading and writing the `.bytes` files of the game Distance.
//!
//! ```no_run
//! let data = std::fs::read("level.bytes").unwrap();
//! let file = distance_bytes::BytesFile::read(&data).unwrap();
//! assert_eq!(file.write(), data);
//! ```

mod build;
mod component_id;
pub mod components;
mod error;
mod fields;
mod generated;
pub mod io;
pub mod json;
mod legacy_settings;
mod prefabs;
pub mod replay;
mod stack;
mod string;
mod tree;
mod visit;

pub use build::{Problem, Source, Target};
pub use component_id::ComponentId;
pub use components::ComponentData;
pub use error::{Error, ErrorKind, Result};
pub use fields::{Field, FieldType, FieldValue, Value, field_type};
pub use io::{TraceEvent, TraceValue};
pub use legacy_settings::LegacyLevelSettings;
pub use prefabs::{Prefab, PrefabDefaults, Unresolved};
pub use string::DString;
pub use tree::{
    BytesFile, Component, ComponentBody, ComponentKind, GameObject, Layer, LayerFlags,
    LegacySettingsBody, Level, LevelSettingsData, Transform,
};
pub use visit::BodyEnd;
