pub mod component;

pub use component::ComponentId;

use component::Component;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct GameObject {
    pub name: String,
    pub prefab: String,
    pub guid: u32,
    pub components: Vec<Component>,
}

#[repr(i32)]
enum PlayerEvent {
    CarRespawn,
    CarFailedToRespawn,
    CarInstantiate,
    Uninitialize,
    ReverseTagPlayerTagged,
    ReverseTagPlayerUntagged,
    Finished,
    CameraInstantiate,
    ReplayPlayerFinished,
    Split,
    PreSplit,
    Death,
    PreExplode,
    Explode,
    Impact,
    Jump,
    WingsStateChange,
    TrickComplete,
    CheckpointHit,
    BrokeObject,
    ModeSpecial,
    Horn,
    AbilityFailure,
    AbilityStateChanged,
    Teleport,
    PreTeleport,
    GravityToggled,
    Cooldown,
    WarpAnchorHit,
    DropperDroneStateChange,
    ShardClusterStateChange,
    ShardClusterFireShard,
    WheelsSlicedOff,
    StuntPlayerEnteredBubble,
    StuntPlayerExitedBubble,
    StuntPlayerComboChanged,
    StuntPlayerCollectibleComboChanged,
    CheatedDeath,
}

// 	public static string GetScopeMarkString(int scopeMark)
// 	{
// 		if (scopeMark == 11111111)
// 		{
// 			return "Array";
// 		}
// 		if (scopeMark == 12121212)
// 		{
// 			return "Dictionary";
// 		}
// 		if (scopeMark == 22222222)
// 		{
// 			return "SerialComponent";
// 		}
// 		if (scopeMark == 23232323)
// 		{
// 			return "UnknownComponent";
// 		}
// 		if (scopeMark == 33333333)
// 		{
// 			return "BuiltInComponent";
// 		}
// 		if (scopeMark == 44444444)
// 		{
// 			return "General";
// 		}
// 		if (scopeMark == 55555555)
// 		{
// 			return "Children";
// 		}
// 		if (scopeMark == 66666666)
// 		{
// 			return "GameObject";
// 		}
// 		if (scopeMark == 88888888)
// 		{
// 			return "LevelSettings";
// 		}
// 		if (scopeMark == 99999999)
// 		{
// 			return "Level";
// 		}
// 		if (scopeMark != 2147483645)
// 		{
// 			return "INVALID";
// 		}
// 		return "Empty";
// 	}
