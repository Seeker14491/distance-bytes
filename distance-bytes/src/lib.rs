#![warn(
    rust_2018_idioms,
    deprecated_in_future,
    macro_use_extern_crate,
    missing_debug_implementations,
    unused_qualifications
)]

pub use crate::internal::animator_base::*;
pub use crate::internal::component::{Component, ComponentData, RawComponentData};
pub use crate::internal::player_stats::*;
pub use crate::internal::{
    Color, GameObject, MaterialColorInfo, MaterialInfo, Quaternion, Vector3, DEFAULT_QUATERNION,
    ONES_VECTOR_3, ZEROS_VECTOR_3,
};

pub mod component;

mod internal;
