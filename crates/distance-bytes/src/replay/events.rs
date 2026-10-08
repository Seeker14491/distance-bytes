//! The player events of a replay (`event_buffer`): what happened to the car besides its
//! movement, such as respawns, jumps, deaths, tricks and checkpoints, each at a frame.
//!
//! The game records every event of the types it sends to other players in multiplayer, as the
//! event's index in its list of player events ([`PLAYER_EVENTS`]) and its data. Each type has a
//! version, and `player_event_versions` holds those of the game that recorded the replay.
//! [`CarReplayData::events`] decodes each event in the version of its type there, as recorded.
//!
//! The game plays the events of a replay in the versions of its own types. When it loads a
//! replay of an older component version whose event versions differ from its own, it first
//! upgrades the events: a `TrickComplete` of version 0 gets zeros for the values version 1
//! added, and the events without data (`Jump` and `ModeSpecial`) are dropped. A replay of the
//! current component version is never upgraded, so the game reads its events in its own
//! versions whatever the replay records.

use serde::{Deserialize, Serialize};

use super::packets;
use crate::components::CarReplayData;
use crate::error::Result;
use crate::io::Reader;

/// The game's player events, by index (`PlayerEvents`). Only some are recorded in replays: those
/// [`PlayerEvent`] has a variant for.
pub const PLAYER_EVENTS: [&str; 38] = [
    "CarRespawn",
    "CarFailedToRespawn",
    "CarInstantiate",
    "Uninitialize",
    "ReverseTagPlayerTagged",
    "ReverseTagPlayerUntagged",
    "Finished",
    "CameraInstantiate",
    "ReplayPlayerFinished",
    "Split",
    "PreSplit",
    "Death",
    "PreExplode",
    "Explode",
    "Impact",
    "Jump",
    "WingsStateChange",
    "TrickComplete",
    "CheckpointHit",
    "BrokeObject",
    "ModeSpecial",
    "Horn",
    "AbilityFailure",
    "AbilityStateChanged",
    "Teleport",
    "PreTeleport",
    "GravityToggled",
    "Cooldown",
    "WarpAnchorHit",
    "DropperDroneStateChange",
    "ShardClusterStateChange",
    "ShardClusterFireShard",
    "WheelsSlicedOff",
    "StuntPlayerEnteredBubble",
    "StuntPlayerExitedBubble",
    "StuntPlayerComboChanged",
    "StuntPlayerCollectibleComboChanged",
    "CheatedDeath",
];

/// The version of the event types without data (`EmptyInstancedTransceivedEvent`).
const NO_DATA: i32 = -311;

/// A player event of a replay, at a frame.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayEvent {
    /// The frame, at 100 frames a second, as for the states. The car's spawn at the start is
    /// at frame -300.
    pub frame: i32,
    #[serde(flatten)]
    pub event: PlayerEvent,
}

/// The data of a player event, by its type. The fields are the game's. Objects of the level are
/// identified by a `handle`: the ID that the game gives them, by type, as it loads the level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PlayerEvent {
    /// The car spawned or respawned there.
    CarRespawn {
        position: [f32; 3],
        /// Quaternion as `[x, y, z, w]`.
        rotation: [f32; 4],
        fast_respawn: bool,
    },
    /// The player finished (`Events.Player.Finished`).
    Finished {
        /// The result, as the mode counts it.
        finish_data: i32,
        /// `FinishType`: 0 None, 1 Normal, 2 DNF, 3 Spectate, 4 JoinedLate, 5 ViewingReplay,
        /// 6 LeavingLevel.
        finish_type: i32,
    },
    /// The car was cut in two.
    Split {
        /// The cutting plane in the car's space, as `[x, y, z, w]`.
        plane_ls: [f32; 4],
        penetration: f32,
        separation_speed: f32,
    },
    /// The car was destroyed.
    Death {
        /// `Death.Cause`: 0 None, 1 KillGrid, 2 SelfTermination, 3 LaserOverheated, 4 Impact,
        /// 5 Overheated, 6 ImpactSquish, 7 AntiTunnelSquish.
        cause_of_death: i32,
    },
    Jump,
    /// The wings opened or closed.
    WingsStateChange {
        open: bool,
    },
    /// A trick ended, with its points, the heat it cooled, and how far or long each stunt went.
    TrickComplete {
        points: i32,
        cooldown_amount: f32,
        wall_ride_meters: f32,
        ceiling_ride_meters: f32,
        grind_meters: f32,
        /// Version 1 and later.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        wheelie_meters: Option<f32>,
        /// Version 1 and later.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        side_wheelie_meters: Option<f32>,
        /// Version 1 and later.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        nose_stand_secs: Option<f32>,
    },
    /// The car passed a checkpoint (`CheckpointLogicBase`).
    CheckpointHit {
        track_t: f32,
        handle: i32,
    },
    /// The car broke an object (`OnCollisionBreakApartLogic`).
    BrokeObject {
        handle: i32,
    },
    ModeSpecial,
    /// The car honked.
    Horn {
        horn_percent: f32,
        position: [f32; 3],
    },
    /// The car entered a teleporter (`TeleporterEntrance`).
    PreTeleport {
        handle: i32,
    },
    /// The car's gravity turned off or on, with the drag it has then.
    GravityToggled {
        gravity_disabled: bool,
        drag: f32,
        ang_drag: f32,
    },
    /// The car's heat cooled.
    Cooldown {
        cooldown_amount: f32,
    },
    /// The car reached a warp anchor (`WarpAnchor`).
    WarpAnchorHit {
        handle: i32,
    },
    /// A dropper drone (`VirusDropperDroneLogic`) changed state.
    DropperDroneStateChange {
        handle: i32,
        /// `VirusDropperDroneLogic.State`: 0 Search, 1 Chase, 2 StompWarning, 3 Stomp,
        /// 4 WaitForCarResetOrCheckpoint, 5 Finished.
        state: i32,
    },
    /// A shard cluster (`ShardCluster`) changed state.
    ShardClusterStateChange {
        handle: i32,
        /// `ShardCluster.State`: 0 Wait, 1 Prepare, 2 Attack, 3 Finished.
        state: i32,
    },
    /// A shard cluster (`ShardCluster`) fired a shard.
    ShardClusterFireShard {
        handle: i32,
        pos: [f32; 3],
        up: [f32; 3],
        fire_time: f32,
        shard_index: i32,
    },
    /// An event of a type the game doesn't record, or of a version that isn't known, or whose
    /// data is too short for its version, with its data.
    Unknown {
        /// The type's index in [`PLAYER_EVENTS`].
        index: i32,
        #[serde(with = "crate::json::hex")]
        data: Vec<u8>,
    },
}

impl CarReplayData {
    /// The player events of the replay, each decoded in the version of its type that
    /// `player_event_versions` gives. An error's offset is into the buffer, not the file.
    pub fn events(&self) -> Result<Vec<ReplayEvent>> {
        packets(&self.event_buffer)?
            .into_iter()
            .map(|(frame, mut r)| {
                let index = r.i32()?;
                let data = r.rest();
                let version = usize::try_from(index)
                    .ok()
                    .and_then(|i| self.player_event_versions.get(i).copied().flatten());
                // Like the game, data after an event's values is skipped.
                let event = version
                    .and_then(|v| decode(index, v, &mut Reader::new(data)).ok().flatten())
                    .unwrap_or_else(|| PlayerEvent::Unknown {
                        index,
                        data: data.to_vec(),
                    });
                Ok(ReplayEvent { frame, event })
            })
            .collect()
    }
}

/// The event of type `index` and `version` from its data, if the type is one the game records
/// and the version is known.
fn decode(index: i32, version: i32, r: &mut Reader) -> Result<Option<PlayerEvent>> {
    use PlayerEvent::*;
    // `BinaryReader.ReadBoolean`.
    let bool = |r: &mut Reader| -> Result<bool> { Ok(r.bytes(1)?[0] != 0) };
    let v1 = |r: &mut Reader| -> Result<Option<f32>> {
        Ok(if version >= 1 { Some(r.f32()?) } else { None })
    };
    // Fields are read in the order they're written.
    Ok(Some(match (index, version) {
        (0, 0) => CarRespawn {
            position: r.f32s()?,
            rotation: r.f32s()?,
            fast_respawn: bool(r)?,
        },
        (6, 0) => Finished {
            finish_data: r.i32()?,
            finish_type: r.i32()?,
        },
        (9, 0) => Split {
            plane_ls: r.f32s()?,
            penetration: r.f32()?,
            separation_speed: r.f32()?,
        },
        (11, 0) => Death {
            cause_of_death: r.i32()?,
        },
        (15, NO_DATA) => Jump,
        (16, 0) => WingsStateChange { open: bool(r)? },
        (17, 0 | 1) => TrickComplete {
            points: r.i32()?,
            cooldown_amount: r.f32()?,
            wall_ride_meters: r.f32()?,
            ceiling_ride_meters: r.f32()?,
            grind_meters: r.f32()?,
            wheelie_meters: v1(r)?,
            side_wheelie_meters: v1(r)?,
            nose_stand_secs: v1(r)?,
        },
        (18, 0) => CheckpointHit {
            track_t: r.f32()?,
            handle: r.i32()?,
        },
        (19, 0) => BrokeObject { handle: r.i32()? },
        (20, NO_DATA) => ModeSpecial,
        (21, 0) => Horn {
            horn_percent: r.f32()?,
            position: r.f32s()?,
        },
        (25, 0) => PreTeleport { handle: r.i32()? },
        (26, 0) => GravityToggled {
            gravity_disabled: bool(r)?,
            drag: r.f32()?,
            ang_drag: r.f32()?,
        },
        (27, 0) => Cooldown {
            cooldown_amount: r.f32()?,
        },
        (28, 0) => WarpAnchorHit { handle: r.i32()? },
        (29, 0) => DropperDroneStateChange {
            handle: r.i32()?,
            state: r.i32()?,
        },
        (30, 0) => ShardClusterStateChange {
            handle: r.i32()?,
            state: r.i32()?,
        },
        (31, 0) => ShardClusterFireShard {
            handle: r.i32()?,
            pos: r.f32s()?,
            up: r.f32s()?,
            fire_time: r.f32()?,
            shard_index: r.i32()?,
        },
        _ => return Ok(None),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::Writer;

    /// The game's event versions, as replays record them.
    const VERSIONS: [i32; 38] = [
        0, -1, -1, -1, -1, -1, 0, -1, -1, 0, -1, 0, -1, -1, -1, -311, 0, 1, 0, 0, -311, 0, -1, -1,
        -1, 0, 0, 0, 0, 0, 0, 0, -1, -1, -1, -1, -1, -1,
    ];

    fn replay(events: &[(i32, i32, Vec<u8>)], versions: [i32; 38]) -> CarReplayData {
        let mut w = Writer::new();
        for (frame, index, data) in events {
            w.i32((w.pos() + 12 + data.len()) as i32);
            w.i32(*frame);
            w.i32(*index);
            w.bytes(data);
        }
        CarReplayData {
            player_event_versions: versions.map(Some).to_vec(),
            event_buffer: w.finish(),
            ..CarReplayData::default()
        }
    }

    fn data(f: impl FnOnce(&mut Writer)) -> Vec<u8> {
        let mut w = Writer::new();
        f(&mut w);
        w.finish()
    }

    #[test]
    fn events() {
        let respawn = data(|w| {
            w.f32s(&[1.0, 2.0, 3.0, 0.0, 0.0, 0.0, 1.0]);
            w.u8(1);
        });
        let trick = data(|w| {
            w.i32(125);
            w.f32s(&[0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        });
        let checkpoint = data(|w| {
            w.f32(0.25);
            w.i32(3);
            // Skipped, as the game does.
            w.i32(-1);
        });
        let replay = replay(
            &[
                (-300, 0, respawn),
                (10, 15, vec![]),
                (20, 17, trick),
                (30, 18, checkpoint),
                // Not recorded by the game.
                (40, 1, vec![7]),
                // Too short.
                (50, 11, vec![1, 0]),
            ],
            VERSIONS,
        );
        let events: Vec<_> = replay.events().unwrap();
        let expected = [
            (
                -300,
                PlayerEvent::CarRespawn {
                    position: [1.0, 2.0, 3.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    fast_respawn: true,
                },
            ),
            (10, PlayerEvent::Jump),
            (
                20,
                PlayerEvent::TrickComplete {
                    points: 125,
                    cooldown_amount: 0.5,
                    wall_ride_meters: 1.0,
                    ceiling_ride_meters: 2.0,
                    grind_meters: 3.0,
                    wheelie_meters: Some(4.0),
                    side_wheelie_meters: Some(5.0),
                    nose_stand_secs: Some(6.0),
                },
            ),
            (
                30,
                PlayerEvent::CheckpointHit {
                    track_t: 0.25,
                    handle: 3,
                },
            ),
            (
                40,
                PlayerEvent::Unknown {
                    index: 1,
                    data: vec![7],
                },
            ),
            (
                50,
                PlayerEvent::Unknown {
                    index: 11,
                    data: vec![1, 0],
                },
            ),
        ]
        .map(|(frame, event)| ReplayEvent { frame, event });
        assert_eq!(events, expected);
    }

    #[test]
    fn events_in_recorded_versions() {
        let trick = data(|w| {
            w.i32(50);
            w.f32s(&[0.5, 1.0, 2.0, 3.0]);
        });
        let mut versions = VERSIONS;
        versions[17] = 0;
        let events = replay(&[(20, 17, trick.clone())], versions)
            .events()
            .unwrap();
        assert_eq!(
            events[0].event,
            PlayerEvent::TrickComplete {
                points: 50,
                cooldown_amount: 0.5,
                wall_ride_meters: 1.0,
                ceiling_ride_meters: 2.0,
                grind_meters: 3.0,
                wheelie_meters: None,
                side_wheelie_meters: None,
                nose_stand_secs: None,
            }
        );
        // In version 1, the same data is too short.
        let events = replay(&[(20, 17, trick.clone())], VERSIONS)
            .events()
            .unwrap();
        assert_eq!(
            events[0].event,
            PlayerEvent::Unknown {
                index: 17,
                data: trick
            }
        );
    }

    #[test]
    fn events_as_json() {
        let replay = replay(&[(10, 15, vec![]), (20, 11, vec![1, 0, 0, 0])], VERSIONS);
        let text = serde_json::to_string(&replay.events().unwrap()).unwrap();
        assert_eq!(
            text,
            r#"[{"frame":10,"type":"Jump"},{"frame":20,"type":"Death","cause_of_death":1}]"#
        );
        let back: Vec<ReplayEvent> = serde_json::from_str(&text).unwrap();
        assert_eq!(back, replay.events().unwrap());
    }
}
