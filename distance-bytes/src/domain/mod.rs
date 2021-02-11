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

// #[repr(i32)]
// enum PlayerEvent {
//     CarRespawn,
//     CarFailedToRespawn,
//     CarInstantiate,
//     Uninitialize,
//     ReverseTagPlayerTagged,
//     ReverseTagPlayerUntagged,
//     Finished,
//     CameraInstantiate,
//     ReplayPlayerFinished,
//     Split,
//     PreSplit,
//     Death,
//     PreExplode,
//     Explode,
//     Impact,
//     Jump,
//     WingsStateChange,
//     TrickComplete,
//     CheckpointHit,
//     BrokeObject,
//     ModeSpecial,
//     Horn,
//     AbilityFailure,
//     AbilityStateChanged,
//     Teleport,
//     PreTeleport,
//     GravityToggled,
//     Cooldown,
//     WarpAnchorHit,
//     DropperDroneStateChange,
//     ShardClusterStateChange,
//     ShardClusterFireShard,
//     WheelsSlicedOff,
//     StuntPlayerEnteredBubble,
//     StuntPlayerExitedBubble,
//     StuntPlayerComboChanged,
//     StuntPlayerCollectibleComboChanged,
//     CheatedDeath,
// }
