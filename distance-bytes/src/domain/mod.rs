pub mod component;

pub use component::ComponentId;

use component::Component;
use serde::{Deserialize, Serialize};

pub type Vector3 = mint::Vector3<f32>;
pub type Quaternion = mint::Quaternion<f32>;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct GameObject {
    pub name: String,
    pub guid: u32,
    pub components: Vec<Component>,
}
