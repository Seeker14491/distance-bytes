pub mod component;

pub use component::ComponentId;

use component::Component;
use serde::{Deserialize, Serialize};

pub type Vector3 = mint::Vector3<f32>;
pub type Quaternion = mint::Quaternion<f32>;

pub(crate) const ZEROS_VECTOR_3: Vector3 = Vector3 {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};

pub(crate) const ONES_VECTOR_3: Vector3 = Vector3 {
    x: 1.0,
    y: 1.0,
    z: 1.0,
};

pub(crate) const DEFAULT_QUATERNION: Quaternion = Quaternion {
    v: Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    },
    s: 1.0,
};

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct GameObject {
    pub name: String,
    pub guid: u32,
    pub components: Vec<Component>,
}
