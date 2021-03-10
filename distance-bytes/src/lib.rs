#![warn(
    rust_2018_idioms,
    deprecated_in_future,
    macro_use_extern_crate,
    missing_debug_implementations,
    unused_qualifications
)]

pub use crate::internal::component::{Component, ComponentData, RawComponentData};
pub use crate::internal::{Color, GameObject, MaterialColorInfo, MaterialInfo};

pub mod component;

mod internal;
