use crate::{
    domain::{
        component::{Component, ComponentData, RawComponentData, Transform},
        ComponentId,
    },
    serialization::{
        game_object, quaternion, scope_with_scope_mark_satisfying, vector_3, Error, Parser,
    },
    util,
};
use combine::{
    count_min_max, error,
    error::StreamError,
    parser::{
        byte::num::{le_i32, le_u32},
        combinator::no_partial,
        range::take_while,
    },
    unexpected, value, EasyParser, Parser as _,
};
use mint::{Quaternion, Vector3};
use num_traits::FromPrimitive;
use std::convert::TryFrom;

pub(crate) fn component<'a>() -> impl Parser<'a, Component> {
    let start =
        scope_with_scope_mark_satisfying(|x| [33333333, 22222222, 32323232, 23232323].contains(&x));

    let id = || {
        le_i32().and_then(|n| {
            ComponentId::from_i32(n)
                .ok_or_else(|| Error::message_format(format!("unknown component id {}", n)))
        })
    };
    let version = le_i32;
    let guid = le_u32;

    start
        .flat_map(move |(scope_mark, data)| {
            if scope_mark == 23232323 {
                todo!("components with scope mark \"23232323\" are not implemented yet")
            }

            (id(), version(), guid()).easy_parse(data)
        })
        .flat_map(|((id, version, guid), data)| {
            (value(guid), component_data(id, version, guid))
                .easy_parse(data)
                .map(|x| x.0)
        })
        .map(|(guid, component_data)| Component {
            guid,
            data: component_data,
        })
}

#[rustfmt::skip]
fn component_data<'a>(
    component_id: ComponentId,
    _version: i32,
    _guid: u32,
) -> impl Parser<'a, ComponentData> {
    let make = |f: fn(&[u8]) -> ComponentData| {
        no_partial(take_while(|_| true).map(f)).boxed()
    };

    match component_id {
        ComponentId::Transform => {
            no_partial(transform().map(ComponentData::Transform)).boxed()
        }
        ComponentId::MeshRenderer => {
            make(|data| ComponentData::MeshRenderer(RawComponentData(data.to_vec())))
        }
        ComponentId::TextMesh => {
            make(|data| ComponentData::TextMesh(RawComponentData(data.to_vec())))
        }
        ComponentId::Light => {
            make(|data| ComponentData::Light(RawComponentData(data.to_vec())))
        }
        ComponentId::LensFlare => {
            make(|data| ComponentData::LensFlare(RawComponentData(data.to_vec())))
        }
        ComponentId::Projector => {
            make(|data| ComponentData::Projector(RawComponentData(data.to_vec())))
        }
        ComponentId::SphereCollider => {
            make(|data| ComponentData::SphereCollider(RawComponentData(data.to_vec())))
        }
        ComponentId::BoxCollider => {
            make(|data| ComponentData::BoxCollider(RawComponentData(data.to_vec())))
        }
        ComponentId::CapsuleCollider => {
            make(|data| ComponentData::CapsuleCollider(RawComponentData(data.to_vec())))
        }
        ComponentId::BezierSplineTrack => {
            make(|data| ComponentData::BezierSplineTrack(RawComponentData(data.to_vec())))
        }
        ComponentId::TrackSegment => {
            make(|data| ComponentData::TrackSegment(RawComponentData(data.to_vec())))
        }
        ComponentId::TrackLink => {
            make(|data| ComponentData::TrackLink(RawComponentData(data.to_vec())))
        }
        ComponentId::RigidbodyAxisRotationLogic => {
            make(|data| ComponentData::RigidbodyAxisRotationLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::BackAndForthSawLogic => {
            make(|data| ComponentData::BackAndForthSawLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::CheckpointLogic => {
            make(|data| ComponentData::CheckpointLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::LightFlickerLogic => {
            make(|data| ComponentData::LightFlickerLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::Group => {
            make(|data| ComponentData::Group(RawComponentData(data.to_vec())))
        }
        ComponentId::TutorialBoxText => {
            make(|data| ComponentData::TutorialBoxText(RawComponentData(data.to_vec())))
        }
        ComponentId::FlyingRingLogic => {
            make(|data| ComponentData::FlyingRingLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::PopupBlockerLogic => {
            make(|data| ComponentData::PopupBlockerLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::PulseLight => {
            make(|data| ComponentData::PulseLight(RawComponentData(data.to_vec())))
        }
        ComponentId::PulseMaterial => {
            make(|data| ComponentData::PulseMaterial(RawComponentData(data.to_vec())))
        }
        ComponentId::SmoothRandomPosition => {
            make(|data| ComponentData::SmoothRandomPosition(RawComponentData(data.to_vec())))
        }
        ComponentId::SoccerGoalLogic => {
            make(|data| ComponentData::SoccerGoalLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::VirusMineLogic => {
            make(|data| ComponentData::VirusMineLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::BrightenCarHeadlights => {
            make(|data| ComponentData::BrightenCarHeadlights(RawComponentData(data.to_vec())))
        }
        ComponentId::GameData => {
            make(|data| ComponentData::GameData(RawComponentData(data.to_vec())))
        }
        ComponentId::GraphicsSettings => {
            make(|data| ComponentData::GraphicsSettings(RawComponentData(data.to_vec())))
        }
        ComponentId::AudioSettings => {
            make(|data| ComponentData::AudioSettings(RawComponentData(data.to_vec())))
        }
        ComponentId::ControlsSettings => {
            make(|data| ComponentData::ControlsSettings(RawComponentData(data.to_vec())))
        }
        ComponentId::Profile => {
            make(|data| ComponentData::Profile(RawComponentData(data.to_vec())))
        }
        ComponentId::ToolInputCombos => {
            make(|data| ComponentData::ToolInputCombos(RawComponentData(data.to_vec())))
        }
        ComponentId::ColorPreset => {
            make(|data| ComponentData::ColorPreset(RawComponentData(data.to_vec())))
        }
        ComponentId::LocalLeaderboard => {
            make(|data| ComponentData::LocalLeaderboard(RawComponentData(data.to_vec())))
        }
        ComponentId::AxisRotationLogic => {
            make(|data| ComponentData::AxisRotationLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::ParticleEmitLogic => {
            make(|data| ComponentData::ParticleEmitLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::VirusSpiritSpawner => {
            make(|data| ComponentData::VirusSpiritSpawner(RawComponentData(data.to_vec())))
        }
        ComponentId::PulseRotateOnTrigger => {
            make(|data| ComponentData::PulseRotateOnTrigger(RawComponentData(data.to_vec())))
        }
        ComponentId::TeleporterEntrance => {
            make(|data| ComponentData::TeleporterEntrance(RawComponentData(data.to_vec())))
        }
        ComponentId::TeleporterExit => {
            make(|data| ComponentData::TeleporterExit(RawComponentData(data.to_vec())))
        }
        ComponentId::ControlScheme => {
            make(|data| ComponentData::ControlScheme(RawComponentData(data.to_vec())))
        }
        ComponentId::DeviceToSchemeLinks => {
            make(|data| ComponentData::DeviceToSchemeLinks(RawComponentData(data.to_vec())))
        }
        ComponentId::ObjectSpawnCircle => {
            make(|data| ComponentData::ObjectSpawnCircle(RawComponentData(data.to_vec())))
        }
        ComponentId::InterpolateToPositionOnTrigger => {
            make(|data| ComponentData::InterpolateToPositionOnTrigger(RawComponentData(data.to_vec())))
        }
        ComponentId::EngageBrokenPieces => {
            make(|data| ComponentData::EngageBrokenPieces(RawComponentData(data.to_vec())))
        }
        ComponentId::GravityToggle => {
            make(|data| ComponentData::GravityToggle(RawComponentData(data.to_vec())))
        }
        ComponentId::CarSpawner => {
            make(|data| ComponentData::CarSpawner(RawComponentData(data.to_vec())))
        }
        ComponentId::RaceStartCarSpawner => {
            make(|data| ComponentData::RaceStartCarSpawner(RawComponentData(data.to_vec())))
        }
        ComponentId::LevelEditorCarSpawner => {
            make(|data| ComponentData::LevelEditorCarSpawner(RawComponentData(data.to_vec())))
        }
        ComponentId::InfoDisplayLogic => {
            make(|data| ComponentData::InfoDisplayLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::MusicTrigger => {
            make(|data| ComponentData::MusicTrigger(RawComponentData(data.to_vec())))
        }
        ComponentId::TabPopulator => {
            make(|data| ComponentData::TabPopulator(RawComponentData(data.to_vec())))
        }
        ComponentId::AdventureAbilitySettings => {
            make(|data| ComponentData::AdventureAbilitySettings(RawComponentData(data.to_vec())))
        }
        ComponentId::IndicatorDisplayLogic => {
            make(|data| ComponentData::IndicatorDisplayLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::PulseCoreLogic => {
            make(|data| ComponentData::PulseCoreLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::PulseAll => {
            make(|data| ComponentData::PulseAll(RawComponentData(data.to_vec())))
        }
        ComponentId::TeleporterExitCheckpoint => {
            make(|data| ComponentData::TeleporterExitCheckpoint(RawComponentData(data.to_vec())))
        }
        ComponentId::LevelSettings => {
            make(|data| ComponentData::LevelSettings(RawComponentData(data.to_vec())))
        }
        ComponentId::WingCorruptionZone => {
            make(|data| ComponentData::WingCorruptionZone(RawComponentData(data.to_vec())))
        }
        ComponentId::GenerateCreditsNames => {
            make(|data| ComponentData::GenerateCreditsNames(RawComponentData(data.to_vec())))
        }
        ComponentId::IntroCutsceneLightFadeIn => {
            make(|data| ComponentData::IntroCutsceneLightFadeIn(RawComponentData(data.to_vec())))
        }
        ComponentId::QuarantineTrigger => {
            make(|data| ComponentData::QuarantineTrigger(RawComponentData(data.to_vec())))
        }
        ComponentId::CarScreenTextDecodeTrigger => {
            make(|data| ComponentData::CarScreenTextDecodeTrigger(RawComponentData(data.to_vec())))
        }
        ComponentId::GlitchFieldLogic => {
            make(|data| ComponentData::GlitchFieldLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::FogSkyboxAmbientChangeTrigger => {
            make(|data| ComponentData::FogSkyboxAmbientChangeTrigger(RawComponentData(data.to_vec())))
        }
        ComponentId::FinalCountdownLogic => {
            make(|data| ComponentData::FinalCountdownLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::SetActiveOnIntroCutsceneStarted => {
            make(|data| ComponentData::SetActiveOnIntroCutsceneStarted(RawComponentData(data.to_vec())))
        }
        ComponentId::RaceEndLogic => {
            make(|data| ComponentData::RaceEndLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::EnableAbilitiesTrigger => {
            make(|data| ComponentData::EnableAbilitiesTrigger(RawComponentData(data.to_vec())))
        }
        ComponentId::SphericalGravity => {
            make(|data| ComponentData::SphericalGravity(RawComponentData(data.to_vec())))
        }
        ComponentId::CreditsNameOrbLogic => {
            make(|data| ComponentData::CreditsNameOrbLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::DisableLocalCarWarnings => {
            make(|data| ComponentData::DisableLocalCarWarnings(RawComponentData(data.to_vec())))
        }
        ComponentId::CustomName => {
            make(|data| ComponentData::CustomName(RawComponentData(data.to_vec())))
        }
        ComponentId::SplineSegment => {
            make(|data| ComponentData::SplineSegment(RawComponentData(data.to_vec())))
        }
        ComponentId::WarningPulseLight => {
            make(|data| ComponentData::WarningPulseLight(RawComponentData(data.to_vec())))
        }
        ComponentId::RumbleZone => {
            make(|data| ComponentData::RumbleZone(RawComponentData(data.to_vec())))
        }
        ComponentId::HideOnVirusSpiritEvent => {
            make(|data| ComponentData::HideOnVirusSpiritEvent(RawComponentData(data.to_vec())))
        }
        ComponentId::TrackAttachment => {
            make(|data| ComponentData::TrackAttachment(RawComponentData(data.to_vec())))
        }
        ComponentId::LevelPlaylist => {
            make(|data| ComponentData::LevelPlaylist(RawComponentData(data.to_vec())))
        }
        ComponentId::ProfileProgress => {
            make(|data| ComponentData::ProfileProgress(RawComponentData(data.to_vec())))
        }
        ComponentId::GeneralSettings => {
            make(|data| ComponentData::GeneralSettings(RawComponentData(data.to_vec())))
        }
        ComponentId::WorkshopPublishedFileInfos => {
            make(|data| ComponentData::WorkshopPublishedFileInfos(RawComponentData(data.to_vec())))
        }
        ComponentId::WarpAnchor => {
            make(|data| ComponentData::WarpAnchor(RawComponentData(data.to_vec())))
        }
        ComponentId::SetActiveOnMIDIEvent => {
            make(|data| ComponentData::SetActiveOnMIDIEvent(RawComponentData(data.to_vec())))
        }
        ComponentId::TurnLightOnNearCar => {
            make(|data| ComponentData::TurnLightOnNearCar(RawComponentData(data.to_vec())))
        }
        ComponentId::Traffic => {
            make(|data| ComponentData::Traffic(RawComponentData(data.to_vec())))
        }
        ComponentId::TrackManipulatorNode => {
            make(|data| ComponentData::TrackManipulatorNode(RawComponentData(data.to_vec())))
        }
        ComponentId::AudioEventTrigger => {
            make(|data| ComponentData::AudioEventTrigger(RawComponentData(data.to_vec())))
        }
        ComponentId::LevelEditorSettings => {
            make(|data| ComponentData::LevelEditorSettings(RawComponentData(data.to_vec())))
        }
        ComponentId::EmpireProximityDoorLogic => {
            make(|data| ComponentData::EmpireProximityDoorLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::Biodome => {
            make(|data| ComponentData::Biodome(RawComponentData(data.to_vec())))
        }
        ComponentId::TunnelHorrorLogic => {
            make(|data| ComponentData::TunnelHorrorLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::VirusSpiritWarpTeaserLogic => {
            make(|data| ComponentData::VirusSpiritWarpTeaserLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::CarReplayData => {
            make(|data| ComponentData::CarReplayData(RawComponentData(data.to_vec())))
        }
        ComponentId::LevelImageCamera => {
            make(|data| ComponentData::LevelImageCamera(RawComponentData(data.to_vec())))
        }
        ComponentId::ParticlesGPU => {
            make(|data| ComponentData::ParticlesGPU(RawComponentData(data.to_vec())))
        }
        ComponentId::KillGridBox => {
            make(|data| ComponentData::KillGridBox(RawComponentData(data.to_vec())))
        }
        ComponentId::GoldenSimples => {
            make(|data| ComponentData::GoldenSimples(RawComponentData(data.to_vec())))
        }
        ComponentId::SetActiveAfterWarp => {
            make(|data| ComponentData::SetActiveAfterWarp(RawComponentData(data.to_vec())))
        }
        ComponentId::AmbientAudioObject => {
            make(|data| ComponentData::AmbientAudioObject(RawComponentData(data.to_vec())))
        }
        ComponentId::BiodomeAudioInterpolator => {
            make(|data| ComponentData::BiodomeAudioInterpolator(RawComponentData(data.to_vec())))
        }
        ComponentId::MoveElectricityAlongWire => {
            make(|data| ComponentData::MoveElectricityAlongWire(RawComponentData(data.to_vec())))
        }
        ComponentId::ActivationRampLogic => {
            make(|data| ComponentData::ActivationRampLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::ZEventTrigger => {
            make(|data| ComponentData::ZEventTrigger(RawComponentData(data.to_vec())))
        }
        ComponentId::ZEventListener => {
            make(|data| ComponentData::ZEventListener(RawComponentData(data.to_vec())))
        }
        ComponentId::BlackPortalLogic => {
            make(|data| ComponentData::BlackPortalLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::VRSettings => {
            make(|data| ComponentData::VRSettings(RawComponentData(data.to_vec())))
        }
        ComponentId::CutsceneCamera => {
            make(|data| ComponentData::CutsceneCamera(RawComponentData(data.to_vec())))
        }
        ComponentId::ProfileStats => {
            make(|data| ComponentData::ProfileStats(RawComponentData(data.to_vec())))
        }
        ComponentId::InterpolateToRotationOnTrigger => {
            make(|data| ComponentData::InterpolateToRotationOnTrigger(RawComponentData(data.to_vec())))
        }
        ComponentId::MoveAlongAttachedTrack => {
            make(|data| ComponentData::MoveAlongAttachedTrack(RawComponentData(data.to_vec())))
        }
        ComponentId::ShowDuringGlitch => {
            make(|data| ComponentData::ShowDuringGlitch(RawComponentData(data.to_vec())))
        }
        ComponentId::AddCameraNoise => {
            make(|data| ComponentData::AddCameraNoise(RawComponentData(data.to_vec())))
        }
        ComponentId::CarVoiceTrigger => {
            make(|data| ComponentData::CarVoiceTrigger(RawComponentData(data.to_vec())))
        }
        ComponentId::HoverScreenSpecialObjectTrigger => {
            make(|data| ComponentData::HoverScreenSpecialObjectTrigger(RawComponentData(data.to_vec())))
        }
        ComponentId::ReplaySettings => {
            make(|data| ComponentData::ReplaySettings(RawComponentData(data.to_vec())))
        }
        ComponentId::CutsceneCamForTrailer => {
            make(|data| ComponentData::CutsceneCamForTrailer(RawComponentData(data.to_vec())))
        }
        ComponentId::LevelInfos => {
            make(|data| ComponentData::LevelInfos(RawComponentData(data.to_vec())))
        }
        ComponentId::AchievementTrigger => {
            make(|data| ComponentData::AchievementTrigger(RawComponentData(data.to_vec())))
        }
        ComponentId::ArenaCarSpawner => {
            make(|data| ComponentData::ArenaCarSpawner(RawComponentData(data.to_vec())))
        }
        ComponentId::Animated => {
            make(|data| ComponentData::Animated(RawComponentData(data.to_vec())))
        }
        ComponentId::BlinkInTrigger => {
            make(|data| ComponentData::BlinkInTrigger(RawComponentData(data.to_vec())))
        }
        ComponentId::CarScreenImageTrigger => {
            make(|data| ComponentData::CarScreenImageTrigger(RawComponentData(data.to_vec())))
        }
        ComponentId::ExcludeFromEMP => {
            make(|data| ComponentData::ExcludeFromEMP(RawComponentData(data.to_vec())))
        }
        ComponentId::InfiniteCooldownTrigger => {
            make(|data| ComponentData::InfiniteCooldownTrigger(RawComponentData(data.to_vec())))
        }
        ComponentId::DiscoverableStuntArea => {
            make(|data| ComponentData::DiscoverableStuntArea(RawComponentData(data.to_vec())))
        }
        ComponentId::ForceVolume => {
            make(|data| ComponentData::ForceVolume(RawComponentData(data.to_vec())))
        }
        ComponentId::AdventureModeCompleteTrigger => {
            make(|data| ComponentData::AdventureModeCompleteTrigger(RawComponentData(data.to_vec())))
        }
        ComponentId::CountdownTextMeshLogic => {
            make(|data| ComponentData::CountdownTextMeshLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::AbilitySignButtonColorLogic => {
            make(|data| ComponentData::AbilitySignButtonColorLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::GoldenAnimator => {
            make(|data| ComponentData::GoldenAnimator(RawComponentData(data.to_vec())))
        }
        ComponentId::AnimatorAudio => {
            make(|data| ComponentData::AnimatorAudio(RawComponentData(data.to_vec())))
        }
        ComponentId::AnimatorCameraShake => {
            make(|data| ComponentData::AnimatorCameraShake(RawComponentData(data.to_vec())))
        }
        ComponentId::ShardCluster => {
            make(|data| ComponentData::ShardCluster(RawComponentData(data.to_vec())))
        }
        ComponentId::AdventureSpecialIntro => {
            make(|data| ComponentData::AdventureSpecialIntro(RawComponentData(data.to_vec())))
        }
        ComponentId::AudioEffectZone => {
            make(|data| ComponentData::AudioEffectZone(RawComponentData(data.to_vec())))
        }
        ComponentId::CinematicCamera => {
            make(|data| ComponentData::CinematicCamera(RawComponentData(data.to_vec())))
        }
        ComponentId::CinematicCameraFocalPoint => {
            make(|data| ComponentData::CinematicCameraFocalPoint(RawComponentData(data.to_vec())))
        }
        ComponentId::SetAbilitiesTrigger => {
            make(|data| ComponentData::SetAbilitiesTrigger(RawComponentData(data.to_vec())))
        }
        ComponentId::LostToEchoesIntroCutscene => {
            make(|data| ComponentData::LostToEchoesIntroCutscene(RawComponentData(data.to_vec())))
        }
        ComponentId::CutsceneText => {
            make(|data| ComponentData::CutsceneText(RawComponentData(data.to_vec())))
        }
        ComponentId::UltraPlanet => {
            make(|data| ComponentData::UltraPlanet(RawComponentData(data.to_vec())))
        }
        ComponentId::DeadCarLogic => {
            make(|data| ComponentData::DeadCarLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::RollingBarrelDropperLogic => {
            make(|data| ComponentData::RollingBarrelDropperLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::AdventureFinishTrigger => {
            make(|data| ComponentData::AdventureFinishTrigger(RawComponentData(data.to_vec())))
        }
        ComponentId::AchievementSettings => {
            make(|data| ComponentData::AchievementSettings(RawComponentData(data.to_vec())))
        }
        ComponentId::InterpolateRTPCLogic => {
            make(|data| ComponentData::InterpolateRTPCLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::TriggerCooldownLogic => {
            make(|data| ComponentData::TriggerCooldownLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::ShadowsChangedListener => {
            make(|data| ComponentData::ShadowsChangedListener(RawComponentData(data.to_vec())))
        }
        ComponentId::LookAtCamera => {
            make(|data| ComponentData::LookAtCamera(RawComponentData(data.to_vec())))
        }
        ComponentId::CubeMapRenderer => {
            make(|data| ComponentData::CubeMapRenderer(RawComponentData(data.to_vec())))
        }
        ComponentId::RealtimeReflectionRenderer => {
            make(|data| ComponentData::RealtimeReflectionRenderer(RawComponentData(data.to_vec())))
        }
        ComponentId::VirusDropperDroneLogic => {
            make(|data| ComponentData::VirusDropperDroneLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::OnCollisionBreakApartLogic => {
            make(|data| ComponentData::OnCollisionBreakApartLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::CheatSettings => {
            make(|data| ComponentData::CheatSettings(RawComponentData(data.to_vec())))
        }
        ComponentId::IgnoreInCullGroups => {
            make(|data| ComponentData::IgnoreInCullGroups(RawComponentData(data.to_vec())))
        }
        ComponentId::IgnoreInputTrigger => {
            make(|data| ComponentData::IgnoreInputTrigger(RawComponentData(data.to_vec())))
        }
        ComponentId::PowerPosterLogic => {
            make(|data| ComponentData::PowerPosterLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::MusicZone => {
            make(|data| ComponentData::MusicZone(RawComponentData(data.to_vec())))
        }
        ComponentId::LightsFlickerLogic => {
            make(|data| ComponentData::LightsFlickerLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::CutsceneManagerLogic => {
            make(|data| ComponentData::CutsceneManagerLogic(RawComponentData(data.to_vec())))
        }
        ComponentId::FadeOut => {
            make(|data| ComponentData::FadeOut(RawComponentData(data.to_vec())))
        }
        ComponentId::Flock => {
            make(|data| ComponentData::Flock(RawComponentData(data.to_vec())))
        }
        ComponentId::GPSTrigger => {
            make(|data| ComponentData::GPSTrigger(RawComponentData(data.to_vec())))
        }
        ComponentId::SprintMode => {
            make(|data| ComponentData::SprintMode(RawComponentData(data.to_vec())))
        }
        ComponentId::StuntMode => {
            make(|data| ComponentData::StuntMode(RawComponentData(data.to_vec())))
        }
        ComponentId::SoccerMode => {
            make(|data| ComponentData::SoccerMode(RawComponentData(data.to_vec())))
        }
        ComponentId::FreeRoamMode => {
            make(|data| ComponentData::FreeRoamMode(RawComponentData(data.to_vec())))
        }
        ComponentId::ReverseTagMode => {
            make(|data| ComponentData::ReverseTagMode(RawComponentData(data.to_vec())))
        }
        ComponentId::LevelEditorPlayMode => {
            make(|data| ComponentData::LevelEditorPlayMode(RawComponentData(data.to_vec())))
        }
        ComponentId::CoopSprintMode => {
            make(|data| ComponentData::CoopSprintMode(RawComponentData(data.to_vec())))
        }
        ComponentId::ChallengeMode => {
            make(|data| ComponentData::ChallengeMode(RawComponentData(data.to_vec())))
        }
        ComponentId::AdventureMode => {
            make(|data| ComponentData::AdventureMode(RawComponentData(data.to_vec())))
        }
        ComponentId::SpeedAndStyleMode => {
            make(|data| ComponentData::SpeedAndStyleMode(RawComponentData(data.to_vec())))
        }
        ComponentId::TrackmogrifyMode => {
            make(|data| ComponentData::TrackmogrifyMode(RawComponentData(data.to_vec())))
        }
        ComponentId::DemoMode => {
            make(|data| ComponentData::DemoMode(RawComponentData(data.to_vec())))
        }
        ComponentId::MainMenuMode => {
            make(|data| ComponentData::MainMenuMode(RawComponentData(data.to_vec())))
        }
        ComponentId::LostToEchoesMode => {
            make(|data| ComponentData::LostToEchoesMode(RawComponentData(data.to_vec())))
        }
        ComponentId::NexusMode => {
            make(|data| ComponentData::NexusMode(RawComponentData(data.to_vec())))
        }
        ComponentId::TheOtherSideMode => {
            make(|data| ComponentData::TheOtherSideMode(RawComponentData(data.to_vec())))
        }
        id => {
            unexpected(error::Format(format!(
                "unserializable component \"{:?}\"",
                id
            ))).map(|_| unreachable!()).boxed()
        }
    }
}

fn transform<'a>() -> impl Parser<'a, Transform> {
    let position = vector_3().map(|mut position| {
        if let Some(position) = &mut position {
            let is_valid =
                position.x.is_finite() && position.y.is_finite() && position.z.is_finite();
            if !is_valid {
                *position = Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                };
            }
        }

        position
    });

    let rotation = quaternion().map(|mut rotation| {
        if let Some(rotation) = &mut rotation {
            let is_valid = rotation.v.x.is_finite()
                && rotation.v.y.is_finite()
                && rotation.v.z.is_finite()
                && rotation.s.is_finite();
            if !is_valid {
                *rotation = Quaternion::from([0.0, 0.0, 0.0, 1.0]);
            }
        }

        rotation
    });

    let scale = vector_3().map(|mut scale| {
        if let Some(scale) = &mut scale {
            let is_valid = scale.x.is_finite() && scale.y.is_finite() && scale.z.is_finite();
            if !is_valid {
                *scale = Vector3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                };
            } else {
                scale.x = util::f32_max(scale.x.abs(), 1E-5);
                scale.y = util::f32_max(scale.y.abs(), 1E-5);
                scale.z = util::f32_max(scale.z.abs(), 1E-5);
            }
        }

        scale
    });

    let children_scope = scope_with_scope_mark_satisfying(|x| x == 55555555);

    (position, rotation, scale, children_scope).flat_map(
        |(position, rotation, scale, (_mark, children_scope))| {
            let num_children = le_i32().and_then(|n| {
                usize::try_from(n)
                    .map_err(|_| Error::unexpected_format(format!("number of children {}", n)))
            });
            let children = num_children
                .then(|n| count_min_max(n, n, game_object()))
                .easy_parse(children_scope)
                .map(|x| x.0)?;

            let transform = Transform {
                position,
                rotation,
                scale,
                children,
            };

            Ok(transform)
        },
    )
}
