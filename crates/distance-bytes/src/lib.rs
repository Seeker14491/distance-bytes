//! Reading and writing the `.bytes` files of the game Distance.
//!
//! ```no_run
//! let data = std::fs::read("level.bytes").unwrap();
//! let file = distance_bytes::BytesFile::read(&data).unwrap();
//! assert_eq!(file.write(), data);
//! ```

mod component_id;
mod error;
mod fields;
mod generated;
pub mod io;
mod string;
mod tree;

pub use component_id::ComponentId;
pub use error::{Error, ErrorKind, Result};
pub use fields::{Field, FieldType, FieldValue, Value, field_type};
pub use io::{TraceEvent, TraceValue};
pub use string::DString;
pub use tree::{
    BytesFile, Component, ComponentBody, ComponentKind, GameObject, Layer, LayerFlags, Level,
    LevelSettings, Transform,
};
