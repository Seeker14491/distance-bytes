use crate::GameObject;
use enum_primitive_derive::Primitive;
use mint::{Quaternion, Vector3};
use serde::{Deserialize, Serialize};

// TODO: Handle components that have a name string instead of id and version.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Component {
    pub guid: u32,

    #[serde(flatten)]
    pub data: ComponentData,
}

#[repr(i32)]
#[derive(
    Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Primitive, Serialize, Deserialize,
)]
pub enum ComponentId {
    Invalid_ = -1_i32,
    None = 0,
    Transform = 1,
    MeshFilter = 2,
    MeshRenderer = 3,
    SkinnedMeshRenderer = 4,
    LineRenderer = 5,
    TrailRenderer = 6,
    TextMesh = 7,
    Animation = 8,
    Light = 9,
    LensFlare = 10,
    ParticleSystem = 11,
    Projector = 12,
    MeshCollider = 13,
    SphereCollider = 14,
    BoxCollider = 15,
    CapsuleCollider = 16,
    Rigidbody = 17,
    AudioSource = 18,
    ConstantForce = 19,
    BezierSplineTrack = 20,
    TrackSegment = 21,
    TrackLink = 22,
    RigidbodyAxisRotationLogic = 23,
    BackAndForthSawLogic = 24,
    CheckpointLogic = 25,
    LaserLogic = 26,
    LightFlickerLogic = 27,
    SceneryCameraLogic = 28,
    Group = 29,
    SkyboxAdder = 30,
    LevelCubeMapRenderer = 31,
    LevelGodRayCaster = 32,
    TutorialBoxText = 33,
    BoostPadLogic = 34,
    CloudCreator = 35,
    FlyingRingLogic = 36,
    PopupBlockerLogic = 37,
    PulseLight = 38,
    PulseMaterial = 39,
    SmoothRandomPosition = 40,
    SoccerGoalLogic = 41,
    VirusMineLogic = 42,
    AnimateUVs = 43,
    BrightenCarHeadlights = 44,
    TrackManipulationNode = 45,
    SpawnLaserLogic = 46,
    GameData = 47,
    GraphicsSettings = 48,
    AudioSettings = 49,
    ControlsSettings = 50,
    Profile = 51,
    LevelSet = 52,
    ToolInputCombos = 53,
    ColorPreset = 54,
    LocalLeaderboard = 55,
    AxisRotationLogic = 56,
    ParticleEmitLogic = 57,
    VirusSpiritSpawner = 58,
    GlitchTrigger = 59,
    Teleporter = 60,
    PulseRotateOnTrigger = 61,
    TeleporterEntrance = 62,
    TeleporterExit = 63,
    ControlScheme = 64,
    DeviceToSchemeLinks = 65,
    ObjectSpawnCircle = 66,
    InterpolateToPositionOnTrigger = 67,
    EngageBrokenPieces = 68,
    GravityToggle = 69,
    CarSpawner = 70,
    RaceStartCarSpawner = 71,
    LevelEditorCarSpawner = 72,
    OnlyActiveInLevelEditor = 73,
    InfoDisplayLogic = 74,
    MusicTrigger = 75,
    TabPopulator = 76,
    AdventureAbilitySettings = 77,
    IndicatorDisplayLogic = 78,
    PulseCoreLogic = 79,
    PulseAll = 80,
    TeleporterExitCheckpoint = 81,
    LevelSettings = 82,
    WingCorruptionZone = 83,
    GenerateCreditsNames = 84,
    IntroCutsceneLightFadeIn = 85,
    QuarantineTrigger = 86,
    CarScreenTextDecodeTrigger = 87,
    GlitchFieldLogic = 88,
    FogSkyboxAmbientChangeTrigger = 89,
    FinalCountdownLogic = 90,
    SetActiveOnIntroCutsceneStarted = 91,
    SphericalGravityTrigger = 92,
    RaceEndLogic = 93,
    EnableAbilitiesTrigger = 94,
    SphericalGravity = 95,
    GlobalFogLogic = 96,
    CreditsNameOrbLogic = 97,
    DisableLocalCarWarnings = 98,
    CustomName = 99,
    SplineSegment = 100,
    WarningPulseLight = 101,
    RumbleZone = 102,
    HideOnVirusSpiritEvent = 103,
    TrackAttachment = 104,
    LevelPlaylist = 105,
    ProfileProgress = 106,
    GeneralSettings = 107,
    ReplayAllPurposeTrigger = 108,
    WorkshopPublishedFileInfos = 109,
    WarpAnchor = 110,
    SetActiveOnMIDIEvent = 111,
    TurnLightOnNearCar = 112,
    Traffic = 113,
    TrackManipulatorNode = 114,
    TurnLightOnNearCarTrigger = 115,
    AudioEventTrigger = 116,
    LevelEditorSettings = 117,
    EmpireProximityDoorLogic = 118,
    Biodome = 119,
    TunnelHorrorLogic = 120,
    LogicTrigger = 121,
    ChangeEmissiveColorLogicTriggerListener = 122,
    MoveLogicTriggerListener = 123,
    RotateLogicTriggerListener = 124,
    ScaleLogicTriggerListener = 125,
    VirusSpiritWarpTeaserLogic = 126,
    CarReplayData = 127,
    LevelImageCamera = 128,
    ParticlesGPU = 129,
    KillGridBox = 130,
    GoldenSimples = 131,
    SetActiveAfterWarp = 132,
    AmbientAudioObject = 133,
    BiodomeAudioInterpolator = 134,
    MoveElectricityAlongWire = 135,
    ActivationRampLogic = 136,
    ZEventTrigger = 137,
    ZEventListener = 138,
    BlackPortalLogic = 139,
    VRSettings = 140,
    CutsceneCamera = 141,
    ProfileStats = 142,
    InterpolateToRotationOnTrigger = 143,
    MoveAlongAttachedTrack = 144,
    ShowDuringGlitch = 145,
    AddCameraNoise = 146,
    CarVoiceTrigger = 147,
    HoverScreenSpecialObjectTrigger = 148,
    ReplaySettings = 149,
    CutsceneCamForTrailer = 150,
    LevelInfos = 151,
    AchievementTrigger = 152,
    ArenaCarSpawner = 153,
    Animated = 154,
    BlinkInTrigger = 155,
    CarScreenImageTrigger = 156,
    ExcludeFromEMP = 157,
    InfiniteCooldownTrigger = 158,
    DiscoverableStuntArea = 159,
    ForceVolume = 160,
    AdventureModeCompleteTrigger = 161,
    CountdownTextMeshLogic = 162,
    AbilitySignButtonColorLogic = 163,
    GoldenAnimator = 164,
    StuntCollectibleSpawner = 165,
    AnimatorAudio = 166,
    AnimatorCameraShake = 167,
    ShardCluster = 168,
    AdventureSpecialIntro = 169,
    AudioEffectZone = 170,
    CinematicCamera = 171,
    CinematicCameraFocalPoint = 172,
    SetAbilitiesTrigger = 173,
    LostToEchoesIntroCutscene = 174,
    CutsceneText = 175,
    UltraPlanet = 176,
    DeadCarLogic = 177,
    RollingBarrelDropperLogic = 178,
    AdventureFinishTrigger = 179,
    AchievementSettings = 180,
    InterpolateRTPCLogic = 181,
    TriggerCooldownLogic = 182,
    ShadowsChangedListener = 183,
    LookAtCamera = 184,
    InterceptorCollectable = 185,
    CubeMapRenderer = 186,
    RealtimeReflectionRenderer = 187,
    VirusDropperDroneLogic = 188,
    OnCollisionBreakApartLogic = 189,
    CheatSettings = 190,
    IgnoreInCullGroups = 191,
    IgnoreInputTrigger = 192,
    PowerPosterLogic = 193,
    MusicZone = 194,
    LightsFlickerLogic = 195,
    CutsceneManagerLogic = 196,
    FadeOut = 197,
    Flock = 198,
    GPSTrigger = 199,
    ResetOnCarDeath = 200,
    SprintMode = 201,
    StuntMode = 202,
    SoccerMode = 203,
    FreeRoamMode = 204,
    ReverseTagMode = 205,
    LevelEditorPlayMode = 206,
    CoopSprintMode = 207,
    ChallengeMode = 208,
    AdventureMode = 209,
    SpeedAndStyleMode = 210,
    TrackmogrifyMode = 211,
    DemoMode = 212,
    MainMenuMode = 213,
    LostToEchoesMode = 214,
    NexusMode = 215,
    TheOtherSideMode = 216,
}

impl Default for ComponentId {
    fn default() -> Self {
        ComponentId::None
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ComponentData {
    Transform(Transform),
    MeshRenderer(RawComponentData),
    TextMesh(RawComponentData),
    Light(RawComponentData),
    LensFlare(RawComponentData),
    Projector(RawComponentData),
    SphereCollider(RawComponentData),
    BoxCollider(RawComponentData),
    CapsuleCollider(RawComponentData),
    BezierSplineTrack(RawComponentData),
    TrackSegment(RawComponentData),
    TrackLink(RawComponentData),
    RigidbodyAxisRotationLogic(RawComponentData),
    BackAndForthSawLogic(RawComponentData),
    CheckpointLogic(RawComponentData),
    LightFlickerLogic(RawComponentData),
    Group(RawComponentData),
    TutorialBoxText(RawComponentData),
    FlyingRingLogic(RawComponentData),
    PopupBlockerLogic(RawComponentData),
    PulseLight(RawComponentData),
    PulseMaterial(RawComponentData),
    SmoothRandomPosition(RawComponentData),
    SoccerGoalLogic(RawComponentData),
    VirusMineLogic(RawComponentData),
    BrightenCarHeadlights(RawComponentData),
    GameData(RawComponentData),
    GraphicsSettings(RawComponentData),
    AudioSettings(RawComponentData),
    ControlsSettings(RawComponentData),
    Profile(RawComponentData),
    ToolInputCombos(RawComponentData),
    ColorPreset(RawComponentData),
    LocalLeaderboard(RawComponentData),
    AxisRotationLogic(RawComponentData),
    ParticleEmitLogic(RawComponentData),
    VirusSpiritSpawner(RawComponentData),
    PulseRotateOnTrigger(RawComponentData),
    TeleporterEntrance(RawComponentData),
    TeleporterExit(RawComponentData),
    ControlScheme(RawComponentData),
    DeviceToSchemeLinks(RawComponentData),
    ObjectSpawnCircle(RawComponentData),
    InterpolateToPositionOnTrigger(RawComponentData),
    EngageBrokenPieces(RawComponentData),
    GravityToggle(RawComponentData),
    CarSpawner(RawComponentData),
    RaceStartCarSpawner(RawComponentData),
    LevelEditorCarSpawner(RawComponentData),
    InfoDisplayLogic(RawComponentData),
    MusicTrigger(RawComponentData),
    TabPopulator(RawComponentData),
    AdventureAbilitySettings(RawComponentData),
    IndicatorDisplayLogic(RawComponentData),
    PulseCoreLogic(RawComponentData),
    PulseAll(RawComponentData),
    TeleporterExitCheckpoint(RawComponentData),
    LevelSettings(RawComponentData),
    WingCorruptionZone(RawComponentData),
    GenerateCreditsNames(RawComponentData),
    IntroCutsceneLightFadeIn(RawComponentData),
    QuarantineTrigger(RawComponentData),
    CarScreenTextDecodeTrigger(RawComponentData),
    GlitchFieldLogic(RawComponentData),
    FogSkyboxAmbientChangeTrigger(RawComponentData),
    FinalCountdownLogic(RawComponentData),
    SetActiveOnIntroCutsceneStarted(RawComponentData),
    RaceEndLogic(RawComponentData),
    EnableAbilitiesTrigger(RawComponentData),
    SphericalGravity(RawComponentData),
    CreditsNameOrbLogic(RawComponentData),
    DisableLocalCarWarnings(RawComponentData),
    CustomName(RawComponentData),
    SplineSegment(RawComponentData),
    WarningPulseLight(RawComponentData),
    RumbleZone(RawComponentData),
    HideOnVirusSpiritEvent(RawComponentData),
    TrackAttachment(RawComponentData),
    LevelPlaylist(RawComponentData),
    ProfileProgress(RawComponentData),
    GeneralSettings(RawComponentData),
    WorkshopPublishedFileInfos(RawComponentData),
    WarpAnchor(RawComponentData),
    SetActiveOnMIDIEvent(RawComponentData),
    TurnLightOnNearCar(RawComponentData),
    Traffic(RawComponentData),
    TrackManipulatorNode(RawComponentData),
    AudioEventTrigger(RawComponentData),
    LevelEditorSettings(RawComponentData),
    EmpireProximityDoorLogic(RawComponentData),
    Biodome(RawComponentData),
    TunnelHorrorLogic(RawComponentData),
    VirusSpiritWarpTeaserLogic(RawComponentData),
    CarReplayData(RawComponentData),
    LevelImageCamera(RawComponentData),
    ParticlesGPU(RawComponentData),
    KillGridBox(RawComponentData),
    GoldenSimples(RawComponentData),
    SetActiveAfterWarp(RawComponentData),
    AmbientAudioObject(RawComponentData),
    BiodomeAudioInterpolator(RawComponentData),
    MoveElectricityAlongWire(RawComponentData),
    ActivationRampLogic(RawComponentData),
    ZEventTrigger(RawComponentData),
    ZEventListener(RawComponentData),
    BlackPortalLogic(RawComponentData),
    VRSettings(RawComponentData),
    CutsceneCamera(RawComponentData),
    ProfileStats(RawComponentData),
    InterpolateToRotationOnTrigger(RawComponentData),
    MoveAlongAttachedTrack(RawComponentData),
    ShowDuringGlitch(RawComponentData),
    AddCameraNoise(RawComponentData),
    CarVoiceTrigger(RawComponentData),
    HoverScreenSpecialObjectTrigger(RawComponentData),
    ReplaySettings(RawComponentData),
    CutsceneCamForTrailer(RawComponentData),
    LevelInfos(RawComponentData),
    AchievementTrigger(RawComponentData),
    ArenaCarSpawner(RawComponentData),
    Animated(RawComponentData),
    BlinkInTrigger(RawComponentData),
    CarScreenImageTrigger(RawComponentData),
    ExcludeFromEMP(RawComponentData),
    InfiniteCooldownTrigger(RawComponentData),
    DiscoverableStuntArea(RawComponentData),
    ForceVolume(RawComponentData),
    AdventureModeCompleteTrigger(RawComponentData),
    CountdownTextMeshLogic(RawComponentData),
    AbilitySignButtonColorLogic(RawComponentData),
    GoldenAnimator(RawComponentData),
    AnimatorAudio(RawComponentData),
    AnimatorCameraShake(RawComponentData),
    ShardCluster(RawComponentData),
    AdventureSpecialIntro(RawComponentData),
    AudioEffectZone(RawComponentData),
    CinematicCamera(RawComponentData),
    CinematicCameraFocalPoint(RawComponentData),
    SetAbilitiesTrigger(RawComponentData),
    LostToEchoesIntroCutscene(RawComponentData),
    CutsceneText(RawComponentData),
    UltraPlanet(RawComponentData),
    DeadCarLogic(RawComponentData),
    RollingBarrelDropperLogic(RawComponentData),
    AdventureFinishTrigger(RawComponentData),
    AchievementSettings(RawComponentData),
    InterpolateRTPCLogic(RawComponentData),
    TriggerCooldownLogic(RawComponentData),
    ShadowsChangedListener(RawComponentData),
    LookAtCamera(RawComponentData),
    CubeMapRenderer(RawComponentData),
    RealtimeReflectionRenderer(RawComponentData),
    VirusDropperDroneLogic(RawComponentData),
    OnCollisionBreakApartLogic(RawComponentData),
    CheatSettings(RawComponentData),
    IgnoreInCullGroups(RawComponentData),
    IgnoreInputTrigger(RawComponentData),
    PowerPosterLogic(RawComponentData),
    MusicZone(RawComponentData),
    LightsFlickerLogic(RawComponentData),
    CutsceneManagerLogic(RawComponentData),
    FadeOut(RawComponentData),
    Flock(RawComponentData),
    GPSTrigger(RawComponentData),
    SprintMode(RawComponentData),
    StuntMode(RawComponentData),
    SoccerMode(RawComponentData),
    FreeRoamMode(RawComponentData),
    ReverseTagMode(RawComponentData),
    LevelEditorPlayMode(RawComponentData),
    CoopSprintMode(RawComponentData),
    ChallengeMode(RawComponentData),
    AdventureMode(RawComponentData),
    SpeedAndStyleMode(RawComponentData),
    TrackmogrifyMode(RawComponentData),
    DemoMode(RawComponentData),
    MainMenuMode(RawComponentData),
    LostToEchoesMode(RawComponentData),
    NexusMode(RawComponentData),
    TheOtherSideMode(RawComponentData),
}

#[derive(Debug, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct RawComponentData(pub Vec<u8>);

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Transform {
    pub position: Option<Vector3<f32>>,
    pub rotation: Option<Quaternion<f32>>,
    pub scale: Option<Vector3<f32>>,
    pub children: Vec<GameObject>,
}

// Serializable components:

// GoldenSimples
// BoxCollider
// CapsuleCollider
// LensFlare
// Light
// MeshRenderer
// Projector
// SphereCollider
// TextMesh
// Transform
// ZEventListener
// ZEventTrigger
// MoveElectricityAlongWire
// ParticlesGPU
// Traffic
// AbilitySignButtonColorLogic
// AchievementTrigger
// ActivationRampLogic
// AddCameraNoise
// Animated
// AnimatorAudio
// AnimatorCameraShake
// GoldenAnimator
// LookAtCamera
// IgnoreInputTrigger
// AdventureAbilitySettings
// AdventureFinishTrigger
// AdventureModeCompleteTrigger
// AdventureSpecialIntro
// AmbientAudioObject
// InterpolateRTPCLogic
// MusicZone
// AudioEffectZone
// AudioEventTrigger
// AxisRotationLogic
// BackAndForthSawLogic
// Biodome
// BiodomeAudioInterpolator
// BlackPortalLogic
// BlinkInTrigger
// BrightenCarHeadlights
// CinematicCamera
// CinematicCameraFocalPoint
// CutsceneManagerLogic
// LostToEchoesIntroCutscene
// CarSpawner
// LightsFlickerLogic
// CarReplayData
// CarScreenImageTrigger
// CarScreenTextDecodeTrigger
// CarVoiceTrigger
// ChallengeMode
// CheckpointLogic
// ColorPreset
// CountdownTextMeshLogic
// CreditsNameOrbLogic
// CubeMapRenderer
// RealtimeReflectionRenderer
// CustomName
// CutsceneCamera
// CutsceneText
// DeadCarLogic
// DisableLocalCarWarnings
// DiscoverableStuntArea
// EmpireProximityDoorLogic
// EnableAbilitiesTrigger
// EngageBrokenPieces
// ExcludeFromEMP
// FadeOut
// FinalCountdownLogic
// Flock
// FlyingRingLogic
// FogSkyboxAmbientChangeTrigger
// ForceVolume
// FreeRoamMode
// GPSTrigger
// GenerateCreditsNames
// GlitchFieldLogic
// GravityToggle
// HideOnVirusSpiritEvent
// HoverScreenSpecialObjectTrigger
// IgnoreInCullGroups
// IndicatorDisplayLogic
// InfiniteCooldownTrigger
// InfoDisplayLogic
// InterpolateToPositionOnTrigger
// InterpolateToRotationOnTrigger
// IntroCutsceneLightFadeIn
// KillGridBox
// ArenaCarSpawner
// LevelEditorCarSpawner
// LevelEditorPlayMode
// LightFlickerLogic
// LevelPlaylist
// MusicTrigger
// ObjectSpawnCircle
// OnCollisionBreakApartLogic
// ParticleEmitLogic
// PopupBlockerLogic
// PowerPosterLogic
// PulseAll
// PulseCoreLogic
// PulseLight
// PulseMaterial
// PulseRotateOnTrigger
// QuarantineTrigger
// CoopSprintMode
// RaceEndLogic
// RaceStartCarSpawner
// SprintMode
// TrackmogrifyMode
// RigidbodyAxisRotationLogic
// RollingBarrelDropperLogic
// RumbleZone
// SetAbilitiesTrigger
// SetActiveAfterWarp
// SetActiveOnIntroCutsceneStarted
// SetActiveOnMIDIEvent
// ShardCluster
// ShowDuringGlitch
// SmoothRandomPosition
// SoccerGoalLogic
// SoccerMode
// SpeedAndStyleMode
// SphericalGravity
// AdventureMode
// DemoMode
// LostToEchoesMode
// MainMenuMode
// NexusMode
// TheOtherSideMode
// StuntMode
// ReverseTagMode
// TeleporterEntrance
// TeleporterExit
// TeleporterExitCheckpoint
// TriggerCooldownLogic
// TunnelHorrorLogic
// TurnLightOnNearCar
// TutorialBoxText
// VirusDropperDroneLogic
// VirusMineLogic
// VirusSpiritWarpTeaserLogic
// WarpAnchor
// VirusSpiritSpawner
// WarningPulseLight
// WingCorruptionZone
// ControlScheme
// DeviceToSchemeLinks
// TabPopulator
// GameData
// LocalLeaderboard
// Group
// LevelImageCamera
// LevelSettings
// ToolInputCombos
// LevelInfos
// ShadowsChangedListener
// AchievementSettings
// AudioSettings
// CheatSettings
// ControlsSettings
// GeneralSettings
// GraphicsSettings
// LevelEditorSettings
// ReplaySettings
// VRSettings
// Profile
// ProfileProgress
// ProfileStats
// WorkshopPublishedFileInfos
// BezierSplineTrack
// MoveAlongAttachedTrack
// SplineSegment
// TrackAttachment
// TrackLink
// TrackManipulatorNode
// TrackSegment
// Invalid_
// CutsceneCamForTrailer
// UltraPlanet
