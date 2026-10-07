//! Reading and writing the `.bytes` files of the game Distance.
//!
//! ```no_run
//! let data = std::fs::read("level.bytes").unwrap();
//! let file = distance_bytes::BytesFile::read(&data).unwrap();
//! assert_eq!(file.write(), data);
//! ```

mod component_id;
pub mod components;
mod error;
mod fields;
mod generated;
pub mod io;
mod legacy_settings;
mod string;
mod tree;
mod visit;

pub use component_id::ComponentId;
pub use components::ComponentData;
pub use error::{Error, ErrorKind, Result};
pub use fields::{Field, FieldType, FieldValue, Value, field_type};
pub use io::{TraceEvent, TraceValue};
pub use legacy_settings::LegacyLevelSettings;
pub use string::DString;
pub use tree::{
    BytesFile, Component, ComponentBody, ComponentKind, GameObject, Layer, LayerFlags,
    LegacySettingsBody, Level, LevelSettingsData, Transform,
};
pub use visit::BodyEnd;
