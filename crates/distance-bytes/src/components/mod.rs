//! Typed bodies of components with hand-written serializers, each translated from the game's
//! `Visit` method.

mod ability_sign_button_color_logic;
mod adventure_ability_settings;
mod adventure_finish_trigger;
mod ambient_audio_object;
mod animator;
mod animator_audio;
mod animator_camera_shake;
mod audio_event_trigger;
mod axis_rotation_logic;
mod back_and_forth_saw_logic;
mod blink_in_trigger;
mod brighten_car_headlights;
mod car_replay_data;
mod car_screen_image_trigger;
mod car_screen_text_decode_trigger;
mod checkpoint_logic;
mod cinematic_camera;
mod cinematic_camera_focal_point;
mod colliders;
mod countdown_text_mesh_logic;
mod credits_name_orb_logic;
mod custom_name;
mod cutscene_camera;
mod cutscene_manager_logic;
mod cutscene_text;
mod dead_car_logic;
mod disable_local_car_warnings;
mod discoverable_stunt_area;
mod empire_proximity_door_logic;
mod engage_broken_pieces;
mod environment;
mod fade_out;
mod flock;
mod force_volume;
mod glitch_field_logic;
mod golden_simples;
mod gps_trigger;
mod gravity_toggle;
mod group;
mod ignore_input_trigger;
mod infinite_cooldown_trigger;
mod info_display_logic;
mod interpolate_rtpc_logic;
mod interpolate_to_position_on_trigger;
mod interpolate_to_rotation_on_trigger;
mod lens_flare;
mod level_image_camera;
mod level_infos;
mod level_settings;
mod light;
mod local_leaderboard;
mod look_at_camera;
mod lost_to_echoes_intro_cutscene;
mod mesh_renderer;
mod move_electricity_along_wire;
mod music_trigger;
mod music_zone;
mod on_collision_break_apart_logic;
mod particles_gpu;
mod popup_blocker_logic;
mod power_poster_logic;
mod profile_progress;
mod profile_stats;
mod pulse_all;
mod quarantine_trigger;
mod rigidbody_axis_rotation_logic;
mod rolling_barrel_dropper_logic;
mod rumble_zone;
mod set_abilities_trigger;
mod set_active_after_warp;
mod set_active_on_intro_cutscene_started;
mod shadows_changed_listener;
mod shard_cluster;
mod soccer_goal_logic;
mod spherical_gravity;
mod spline_segment;
mod teleporter_entrance;
mod teleporter_exit;
mod teleporter_exit_checkpoint;
mod text_mesh;
mod track_attachment;
mod track_link;
mod track_segment;
mod traffic;
mod trigger_cooldown_logic;
mod turn_light_on_near_car;
mod ultra_planet;
mod virus_dropper_drone_logic;
mod warning_pulse_light;
mod warp_anchor;
mod wing_corruption_zone;
mod z_event_listener;
mod z_event_trigger;

pub use ability_sign_button_color_logic::AbilitySignButtonColorLogic;
pub use adventure_ability_settings::AdventureAbilitySettings;
pub use adventure_finish_trigger::AdventureFinishTrigger;
pub use ambient_audio_object::AmbientAudioObject;
pub use animator::{Animated, AnimatorCurve, AnimatorTrigger, GoldenAnimator};
pub use animator_audio::AnimatorAudio;
pub use animator_camera_shake::AnimatorCameraShake;
pub use audio_event_trigger::AudioEventTrigger;
pub use axis_rotation_logic::AxisRotationLogic;
pub use back_and_forth_saw_logic::BackAndForthSawLogic;
pub use blink_in_trigger::BlinkInTrigger;
pub use brighten_car_headlights::BrightenCarHeadlights;
pub use car_replay_data::{CarColors, CarData, CarReplayData};
pub use car_screen_image_trigger::CarScreenImageTrigger;
pub use car_screen_text_decode_trigger::CarScreenTextDecodeTrigger;
pub use checkpoint_logic::CheckpointLogic;
pub use cinematic_camera::CinematicCamera;
pub use cinematic_camera_focal_point::CinematicCameraFocalPoint;
pub use colliders::{BoxCollider, CapsuleCollider, SerialCollider, SphereCollider};
pub use countdown_text_mesh_logic::CountdownTextMeshLogic;
pub use credits_name_orb_logic::CreditsNameOrbLogic;
pub use custom_name::CustomName;
pub use cutscene_camera::CutsceneCamera;
pub use cutscene_manager_logic::CutsceneManagerLogic;
pub use cutscene_text::CutsceneText;
pub use dead_car_logic::DeadCarLogic;
pub use disable_local_car_warnings::DisableLocalCarWarnings;
pub use discoverable_stunt_area::DiscoverableStuntArea;
pub use empire_proximity_door_logic::EmpireProximityDoorLogic;
pub use engage_broken_pieces::EngageBrokenPieces;
pub use environment::{AmbientColors, Biodome, DirectionLight, Fog, Skybox};
pub use fade_out::FadeOut;
pub use flock::Flock;
pub use force_volume::ForceVolume;
pub use glitch_field_logic::GlitchFieldLogic;
pub use golden_simples::GoldenSimples;
pub use gps_trigger::GPSTrigger;
pub use gravity_toggle::GravityToggle;
pub use group::Group;
pub use ignore_input_trigger::IgnoreInputTrigger;
pub use infinite_cooldown_trigger::InfiniteCooldownTrigger;
pub use info_display_logic::InfoDisplayLogic;
pub use interpolate_rtpc_logic::InterpolateRTPCLogic;
pub use interpolate_to_position_on_trigger::InterpolateToPositionOnTrigger;
pub use interpolate_to_rotation_on_trigger::InterpolateToRotationOnTrigger;
pub use lens_flare::LensFlare;
pub use level_image_camera::LevelImageCamera;
pub use level_infos::{LevelInfo, LevelInfos};
pub use level_settings::{LevelEditorPrivateSettings, LevelSettings};
pub use light::Light;
pub use local_leaderboard::{LocalLeaderboard, ResultInfo};
pub use look_at_camera::LookAtCamera;
pub use lost_to_echoes_intro_cutscene::LostToEchoesIntroCutscene;
pub use mesh_renderer::{MaterialColor, MaterialColors, MeshRenderer};
pub use move_electricity_along_wire::MoveElectricityAlongWire;
pub use music_trigger::MusicTrigger;
pub use music_zone::MusicZone;
pub use on_collision_break_apart_logic::OnCollisionBreakApartLogic;
pub use particles_gpu::ParticlesGPU;
pub use popup_blocker_logic::PopupBlockerLogic;
pub use power_poster_logic::PowerPosterLogic;
pub use profile_progress::{LevelProgress, ProfileProgress};
pub use profile_stats::{PlayerStats, ProfileStats};
pub use pulse_all::PulseAll;
pub use quarantine_trigger::QuarantineTrigger;
pub use rigidbody_axis_rotation_logic::RigidbodyAxisRotationLogic;
pub use rolling_barrel_dropper_logic::RollingBarrelDropperLogic;
pub use rumble_zone::RumbleZone;
pub use set_abilities_trigger::SetAbilitiesTrigger;
pub use set_active_after_warp::SetActiveAfterWarp;
pub use set_active_on_intro_cutscene_started::SetActiveOnIntroCutsceneStarted;
pub use shadows_changed_listener::ShadowsChangedListener;
pub use shard_cluster::ShardCluster;
pub use soccer_goal_logic::SoccerGoalLogic;
pub use spherical_gravity::SphericalGravity;
pub use spline_segment::SplineSegment;
pub use teleporter_entrance::TeleporterEntrance;
pub use teleporter_exit::TeleporterExit;
pub use teleporter_exit_checkpoint::TeleporterExitCheckpoint;
pub use text_mesh::TextMesh;
pub use track_attachment::TrackAttachment;
pub use track_link::TrackLink;
pub use track_segment::TrackSegment;
pub use traffic::Traffic;
pub use trigger_cooldown_logic::TriggerCooldownLogic;
pub use turn_light_on_near_car::TurnLightOnNearCar;
pub use ultra_planet::UltraPlanet;
pub use virus_dropper_drone_logic::VirusDropperDroneLogic;
pub use warning_pulse_light::WarningPulseLight;
pub use warp_anchor::WarpAnchor;
pub use wing_corruption_zone::WingCorruptionZone;
pub use z_event_listener::ZEventListener;
pub use z_event_trigger::ZEventTrigger;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::component_id::ComponentId;
use crate::error::{ErrorKind, Result};
use crate::io::{Reader, Writer};
use crate::visit::{BodyEnd, Visit};

macro_rules! component_data {
    ($($ty:ident = $id:ident),* $(,)?) => {
        /// A decoded component body. The variant is named after the component type, and boxed
        /// to keep the enum small.
        #[derive(Debug, Clone, PartialEq)]
        pub enum ComponentData {
            $($ty(Box<$ty>),)*
        }

        impl ComponentData {
            /// Whether a body of this component type and version is decoded.
            pub(crate) fn decodes(id: ComponentId, version: i32) -> bool {
                match id {
                    $(ComponentId::$id => <$ty as Visit>::VERSIONS.contains(&version),)*
                    _ => false,
                }
            }

            #[inline(never)]
            pub(crate) fn read(r: &mut Reader, id: ComponentId, version: i32) -> Result<(Self, BodyEnd)> {
                match id {
                    $(ComponentId::$id => {
                        let (data, end) = <$ty>::read_body(r, version)?;
                        Ok((ComponentData::$ty(data), end))
                    })*
                    _ => Err(r.error(ErrorKind::UnsupportedVersion(version))),
                }
            }

            /// A body of this component type and version with every value set to a distinct
            /// sample and one element in every list, or `None` if the version isn't decoded. For
            /// testing serializers against the game on versions that no file has.
            #[doc(hidden)]
            pub fn sample(id: ComponentId, version: i32) -> Option<Self> {
                match id {
                    $(ComponentId::$id if <$ty as Visit>::VERSIONS.contains(&version) => {
                        Some(ComponentData::$ty(<$ty>::sample(version)))
                    })*
                    _ => None,
                }
            }

            /// The component type of this body.
            pub fn id(&self) -> ComponentId {
                match self {
                    $(ComponentData::$ty(_) => ComponentId::$id,)*
                }
            }

            /// The version the game writes for this component type.
            pub fn latest_version(&self) -> i32 {
                match self {
                    $(ComponentData::$ty(_) => *<$ty as Visit>::VERSIONS.end(),)*
                }
            }

            /// Serializes the body without its type.
            pub(crate) fn serialize_body<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
                match self {
                    $(ComponentData::$ty(data) => data.serialize(s),)*
                }
            }

            /// Deserializes a body of the given component type.
            pub(crate) fn deserialize_body<'de, D: Deserializer<'de>>(
                id: ComponentId,
                d: D,
            ) -> std::result::Result<Self, D::Error> {
                match id {
                    $(ComponentId::$id => <$ty>::deserialize(d).map(|data| ComponentData::$ty(Box::new(data))),)*
                    _ => Err(D::Error::custom(format_args!(
                        "{id} has no typed body; use \"fields\" or \"raw\""
                    ))),
                }
            }

            /// Writes the body for the given component version, which should be one of the
            /// versions the type decodes.
            #[inline(never)]
            pub(crate) fn write(&self, w: &mut Writer, version: i32, end: &BodyEnd) {
                match self {
                    $(ComponentData::$ty(data) => data.write_body(w, version, end),)*
                }
            }
        }
    };
}

component_data! {
    AbilitySignButtonColorLogic = ABILITY_SIGN_BUTTON_COLOR_LOGIC,
    AdventureAbilitySettings = ADVENTURE_ABILITY_SETTINGS,
    AdventureFinishTrigger = ADVENTURE_FINISH_TRIGGER,
    AmbientAudioObject = AMBIENT_AUDIO_OBJECT,
    Animated = ANIMATED,
    GoldenAnimator = GOLDEN_ANIMATOR,
    AnimatorAudio = ANIMATOR_AUDIO,
    AnimatorCameraShake = ANIMATOR_CAMERA_SHAKE,
    AudioEventTrigger = AUDIO_EVENT_TRIGGER,
    AxisRotationLogic = AXIS_ROTATION_LOGIC,
    BackAndForthSawLogic = BACK_AND_FORTH_SAW_LOGIC,
    BlinkInTrigger = BLINK_IN_TRIGGER,
    BrightenCarHeadlights = BRIGHTEN_CAR_HEADLIGHTS,
    CarReplayData = CAR_REPLAY_DATA,
    CarScreenImageTrigger = CAR_SCREEN_IMAGE_TRIGGER,
    CarScreenTextDecodeTrigger = CAR_SCREEN_TEXT_DECODE_TRIGGER,
    CheckpointLogic = CHECKPOINT_LOGIC,
    CinematicCamera = CINEMATIC_CAMERA,
    CinematicCameraFocalPoint = CINEMATIC_CAMERA_FOCAL_POINT,
    BoxCollider = BOX_COLLIDER,
    SphereCollider = SPHERE_COLLIDER,
    CapsuleCollider = CAPSULE_COLLIDER,
    CountdownTextMeshLogic = COUNTDOWN_TEXT_MESH_LOGIC,
    CreditsNameOrbLogic = CREDITS_NAME_ORB_LOGIC,
    CustomName = CUSTOM_NAME,
    CutsceneCamera = CUTSCENE_CAMERA,
    CutsceneManagerLogic = CUTSCENE_MANAGER_LOGIC,
    CutsceneText = CUTSCENE_TEXT,
    DeadCarLogic = DEAD_CAR_LOGIC,
    DisableLocalCarWarnings = DISABLE_LOCAL_CAR_WARNINGS,
    DiscoverableStuntArea = DISCOVERABLE_STUNT_AREA,
    EmpireProximityDoorLogic = EMPIRE_PROXIMITY_DOOR_LOGIC,
    EngageBrokenPieces = ENGAGE_BROKEN_PIECES,
    Biodome = BIODOME,
    FadeOut = FADE_OUT,
    Flock = FLOCK,
    ForceVolume = FORCE_VOLUME,
    GlitchFieldLogic = GLITCH_FIELD_LOGIC,
    GoldenSimples = GOLDEN_SIMPLES,
    GPSTrigger = GPS_TRIGGER,
    GravityToggle = GRAVITY_TOGGLE,
    Group = GROUP,
    IgnoreInputTrigger = IGNORE_INPUT_TRIGGER,
    InfiniteCooldownTrigger = INFINITE_COOLDOWN_TRIGGER,
    InfoDisplayLogic = INFO_DISPLAY_LOGIC,
    InterpolateRTPCLogic = INTERPOLATE_RTPC_LOGIC,
    InterpolateToPositionOnTrigger = INTERPOLATE_TO_POSITION_ON_TRIGGER,
    InterpolateToRotationOnTrigger = INTERPOLATE_TO_ROTATION_ON_TRIGGER,
    LensFlare = LENS_FLARE,
    LevelImageCamera = LEVEL_IMAGE_CAMERA,
    LevelInfos = LEVEL_INFOS,
    LevelSettings = LEVEL_SETTINGS,
    Light = LIGHT,
    LocalLeaderboard = LOCAL_LEADERBOARD,
    LookAtCamera = LOOK_AT_CAMERA,
    LostToEchoesIntroCutscene = LOST_TO_ECHOES_INTRO_CUTSCENE,
    MeshRenderer = MESH_RENDERER,
    MoveElectricityAlongWire = MOVE_ELECTRICITY_ALONG_WIRE,
    MusicTrigger = MUSIC_TRIGGER,
    MusicZone = MUSIC_ZONE,
    OnCollisionBreakApartLogic = ON_COLLISION_BREAK_APART_LOGIC,
    ParticlesGPU = PARTICLES_GPU,
    PopupBlockerLogic = POPUP_BLOCKER_LOGIC,
    PowerPosterLogic = POWER_POSTER_LOGIC,
    ProfileProgress = PROFILE_PROGRESS,
    ProfileStats = PROFILE_STATS,
    PulseAll = PULSE_ALL,
    QuarantineTrigger = QUARANTINE_TRIGGER,
    RigidbodyAxisRotationLogic = RIGIDBODY_AXIS_ROTATION_LOGIC,
    RollingBarrelDropperLogic = ROLLING_BARREL_DROPPER_LOGIC,
    RumbleZone = RUMBLE_ZONE,
    SetAbilitiesTrigger = SET_ABILITIES_TRIGGER,
    SetActiveAfterWarp = SET_ACTIVE_AFTER_WARP,
    SetActiveOnIntroCutsceneStarted = SET_ACTIVE_ON_INTRO_CUTSCENE_STARTED,
    ShadowsChangedListener = SHADOWS_CHANGED_LISTENER,
    ShardCluster = SHARD_CLUSTER,
    SoccerGoalLogic = SOCCER_GOAL_LOGIC,
    SphericalGravity = SPHERICAL_GRAVITY,
    SplineSegment = SPLINE_SEGMENT,
    TeleporterEntrance = TELEPORTER_ENTRANCE,
    TeleporterExit = TELEPORTER_EXIT,
    TeleporterExitCheckpoint = TELEPORTER_EXIT_CHECKPOINT,
    TextMesh = TEXT_MESH,
    TrackAttachment = TRACK_ATTACHMENT,
    TrackLink = TRACK_LINK,
    TrackSegment = TRACK_SEGMENT,
    Traffic = TRAFFIC,
    TriggerCooldownLogic = TRIGGER_COOLDOWN_LOGIC,
    TurnLightOnNearCar = TURN_LIGHT_ON_NEAR_CAR,
    UltraPlanet = ULTRA_PLANET,
    VirusDropperDroneLogic = VIRUS_DROPPER_DRONE_LOGIC,
    WarningPulseLight = WARNING_PULSE_LIGHT,
    WarpAnchor = WARP_ANCHOR,
    WingCorruptionZone = WING_CORRUPTION_ZONE,
    ZEventListener = Z_EVENT_LISTENER,
    ZEventTrigger = Z_EVENT_TRIGGER,
}
