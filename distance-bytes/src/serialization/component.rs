use crate::{
    domain::{
        component::{Component, ComponentData, RawComponentData, Transform},
        ComponentId,
    },
    serialization,
    serialization::{
        error::{BytesDeserializeErrorKind, SliceWithOffset},
        failure, BytesParseResult, Input,
    },
    util,
};
use mint::{Quaternion, Vector3};
use nom::{
    bytes::complete::take,
    combinator::all_consuming,
    multi::many_m_n,
    number::complete::{le_i32, le_i64, le_u32},
};
use num_traits::FromPrimitive;
use std::convert::TryInto;

pub fn component(input: Input<'_>) -> BytesParseResult<'_, Component> {
    let (after_component, (scope_mark, scope)) = serialization::read_scope(input)?;
    if ![33333333, 22222222, 32323232, 23232323].contains(&scope_mark) {
        return Err(failure(input, BytesDeserializeErrorKind::Component(None)));
    }

    if scope_mark == 23232323 {
        todo!()
    }

    let err_input = scope;
    let (scope, id) = le_i32(scope)?;
    let id = ComponentId::from_i32(id)
        .ok_or_else(|| failure(err_input, BytesDeserializeErrorKind::Component(None)))?;

    let (scope, version) = le_i32(scope)?;
    let (scope, guid) = le_u32(scope)?;

    let (_, component) = all_consuming(|input| make_component(id, version, guid, input))(scope)?;

    Ok((after_component, component))
}

fn make_component(
    component_id: ComponentId,
    _version: i32,
    guid: u32,
    data: Input<'_>,
) -> BytesParseResult<'_, Component> {
    let input_returned_on_success = SliceWithOffset {
        slice: &[],
        offset: data.offset + data.slice.len(),
    };

    #[rustfmt::skip]
    let (input, component_data) = match component_id {
        ComponentId::Transform => {
            let (input, transform) = transform(data)?;
            (input, ComponentData::Transform(transform))
        }
        ComponentId::MeshRenderer => (input_returned_on_success, ComponentData::MeshRenderer(RawComponentData(data.slice.to_vec()))),
        ComponentId::TextMesh => (input_returned_on_success, ComponentData::TextMesh(RawComponentData(data.slice.to_vec()))),
        ComponentId::Light => (input_returned_on_success, ComponentData::Light(RawComponentData(data.slice.to_vec()))),
        ComponentId::LensFlare => (input_returned_on_success, ComponentData::LensFlare(RawComponentData(data.slice.to_vec()))),
        ComponentId::Projector => (input_returned_on_success, ComponentData::Projector(RawComponentData(data.slice.to_vec()))),
        ComponentId::SphereCollider => (input_returned_on_success, ComponentData::SphereCollider(RawComponentData(data.slice.to_vec()))),
        ComponentId::BoxCollider => (input_returned_on_success, ComponentData::BoxCollider(RawComponentData(data.slice.to_vec()))),
        ComponentId::CapsuleCollider => (input_returned_on_success, ComponentData::CapsuleCollider(RawComponentData(data.slice.to_vec()))),
        ComponentId::BezierSplineTrack => (input_returned_on_success, ComponentData::BezierSplineTrack(RawComponentData(data.slice.to_vec()))),
        ComponentId::TrackSegment => (input_returned_on_success, ComponentData::TrackSegment(RawComponentData(data.slice.to_vec()))),
        ComponentId::TrackLink => (input_returned_on_success, ComponentData::TrackLink(RawComponentData(data.slice.to_vec()))),
        ComponentId::RigidbodyAxisRotationLogic => (input_returned_on_success, ComponentData::RigidbodyAxisRotationLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::BackAndForthSawLogic => (input_returned_on_success, ComponentData::BackAndForthSawLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::CheckpointLogic => (input_returned_on_success, ComponentData::CheckpointLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::LightFlickerLogic => (input_returned_on_success, ComponentData::LightFlickerLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::Group => (input_returned_on_success, ComponentData::Group(RawComponentData(data.slice.to_vec()))),
        ComponentId::TutorialBoxText => (input_returned_on_success, ComponentData::TutorialBoxText(RawComponentData(data.slice.to_vec()))),
        ComponentId::FlyingRingLogic => (input_returned_on_success, ComponentData::FlyingRingLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::PopupBlockerLogic => (input_returned_on_success, ComponentData::PopupBlockerLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::PulseLight => (input_returned_on_success, ComponentData::PulseLight(RawComponentData(data.slice.to_vec()))),
        ComponentId::PulseMaterial => (input_returned_on_success, ComponentData::PulseMaterial(RawComponentData(data.slice.to_vec()))),
        ComponentId::SmoothRandomPosition => (input_returned_on_success, ComponentData::SmoothRandomPosition(RawComponentData(data.slice.to_vec()))),
        ComponentId::SoccerGoalLogic => (input_returned_on_success, ComponentData::SoccerGoalLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::VirusMineLogic => (input_returned_on_success, ComponentData::VirusMineLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::BrightenCarHeadlights => (input_returned_on_success, ComponentData::BrightenCarHeadlights(RawComponentData(data.slice.to_vec()))),
        ComponentId::GameData => (input_returned_on_success, ComponentData::GameData(RawComponentData(data.slice.to_vec()))),
        ComponentId::GraphicsSettings => (input_returned_on_success, ComponentData::GraphicsSettings(RawComponentData(data.slice.to_vec()))),
        ComponentId::AudioSettings => (input_returned_on_success, ComponentData::AudioSettings(RawComponentData(data.slice.to_vec()))),
        ComponentId::ControlsSettings => (input_returned_on_success, ComponentData::ControlsSettings(RawComponentData(data.slice.to_vec()))),
        ComponentId::Profile => (input_returned_on_success, ComponentData::Profile(RawComponentData(data.slice.to_vec()))),
        ComponentId::ToolInputCombos => (input_returned_on_success, ComponentData::ToolInputCombos(RawComponentData(data.slice.to_vec()))),
        ComponentId::ColorPreset => (input_returned_on_success, ComponentData::ColorPreset(RawComponentData(data.slice.to_vec()))),
        ComponentId::LocalLeaderboard => (input_returned_on_success, ComponentData::LocalLeaderboard(RawComponentData(data.slice.to_vec()))),
        ComponentId::AxisRotationLogic => (input_returned_on_success, ComponentData::AxisRotationLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::ParticleEmitLogic => (input_returned_on_success, ComponentData::ParticleEmitLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::VirusSpiritSpawner => (input_returned_on_success, ComponentData::VirusSpiritSpawner(RawComponentData(data.slice.to_vec()))),
        ComponentId::PulseRotateOnTrigger => (input_returned_on_success, ComponentData::PulseRotateOnTrigger(RawComponentData(data.slice.to_vec()))),
        ComponentId::TeleporterEntrance => (input_returned_on_success, ComponentData::TeleporterEntrance(RawComponentData(data.slice.to_vec()))),
        ComponentId::TeleporterExit => (input_returned_on_success, ComponentData::TeleporterExit(RawComponentData(data.slice.to_vec()))),
        ComponentId::ControlScheme => (input_returned_on_success, ComponentData::ControlScheme(RawComponentData(data.slice.to_vec()))),
        ComponentId::DeviceToSchemeLinks => (input_returned_on_success, ComponentData::DeviceToSchemeLinks(RawComponentData(data.slice.to_vec()))),
        ComponentId::ObjectSpawnCircle => (input_returned_on_success, ComponentData::ObjectSpawnCircle(RawComponentData(data.slice.to_vec()))),
        ComponentId::InterpolateToPositionOnTrigger => (input_returned_on_success, ComponentData::InterpolateToPositionOnTrigger(RawComponentData(data.slice.to_vec()))),
        ComponentId::EngageBrokenPieces => (input_returned_on_success, ComponentData::EngageBrokenPieces(RawComponentData(data.slice.to_vec()))),
        ComponentId::GravityToggle => (input_returned_on_success, ComponentData::GravityToggle(RawComponentData(data.slice.to_vec()))),
        ComponentId::CarSpawner => (input_returned_on_success, ComponentData::CarSpawner(RawComponentData(data.slice.to_vec()))),
        ComponentId::RaceStartCarSpawner => (input_returned_on_success, ComponentData::RaceStartCarSpawner(RawComponentData(data.slice.to_vec()))),
        ComponentId::LevelEditorCarSpawner => (input_returned_on_success, ComponentData::LevelEditorCarSpawner(RawComponentData(data.slice.to_vec()))),
        ComponentId::InfoDisplayLogic => (input_returned_on_success, ComponentData::InfoDisplayLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::MusicTrigger => (input_returned_on_success, ComponentData::MusicTrigger(RawComponentData(data.slice.to_vec()))),
        ComponentId::TabPopulator => (input_returned_on_success, ComponentData::TabPopulator(RawComponentData(data.slice.to_vec()))),
        ComponentId::AdventureAbilitySettings => (input_returned_on_success, ComponentData::AdventureAbilitySettings(RawComponentData(data.slice.to_vec()))),
        ComponentId::IndicatorDisplayLogic => (input_returned_on_success, ComponentData::IndicatorDisplayLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::PulseCoreLogic => (input_returned_on_success, ComponentData::PulseCoreLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::PulseAll => (input_returned_on_success, ComponentData::PulseAll(RawComponentData(data.slice.to_vec()))),
        ComponentId::TeleporterExitCheckpoint => (input_returned_on_success, ComponentData::TeleporterExitCheckpoint(RawComponentData(data.slice.to_vec()))),
        ComponentId::LevelSettings => (input_returned_on_success, ComponentData::LevelSettings(RawComponentData(data.slice.to_vec()))),
        ComponentId::WingCorruptionZone => (input_returned_on_success, ComponentData::WingCorruptionZone(RawComponentData(data.slice.to_vec()))),
        ComponentId::GenerateCreditsNames => (input_returned_on_success, ComponentData::GenerateCreditsNames(RawComponentData(data.slice.to_vec()))),
        ComponentId::IntroCutsceneLightFadeIn => (input_returned_on_success, ComponentData::IntroCutsceneLightFadeIn(RawComponentData(data.slice.to_vec()))),
        ComponentId::QuarantineTrigger => (input_returned_on_success, ComponentData::QuarantineTrigger(RawComponentData(data.slice.to_vec()))),
        ComponentId::CarScreenTextDecodeTrigger => (input_returned_on_success, ComponentData::CarScreenTextDecodeTrigger(RawComponentData(data.slice.to_vec()))),
        ComponentId::GlitchFieldLogic => (input_returned_on_success, ComponentData::GlitchFieldLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::FogSkyboxAmbientChangeTrigger => (input_returned_on_success, ComponentData::FogSkyboxAmbientChangeTrigger(RawComponentData(data.slice.to_vec()))),
        ComponentId::FinalCountdownLogic => (input_returned_on_success, ComponentData::FinalCountdownLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::SetActiveOnIntroCutsceneStarted => (input_returned_on_success, ComponentData::SetActiveOnIntroCutsceneStarted(RawComponentData(data.slice.to_vec()))),
        ComponentId::RaceEndLogic => (input_returned_on_success, ComponentData::RaceEndLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::EnableAbilitiesTrigger => (input_returned_on_success, ComponentData::EnableAbilitiesTrigger(RawComponentData(data.slice.to_vec()))),
        ComponentId::SphericalGravity => (input_returned_on_success, ComponentData::SphericalGravity(RawComponentData(data.slice.to_vec()))),
        ComponentId::CreditsNameOrbLogic => (input_returned_on_success, ComponentData::CreditsNameOrbLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::DisableLocalCarWarnings => (input_returned_on_success, ComponentData::DisableLocalCarWarnings(RawComponentData(data.slice.to_vec()))),
        ComponentId::CustomName => (input_returned_on_success, ComponentData::CustomName(RawComponentData(data.slice.to_vec()))),
        ComponentId::SplineSegment => (input_returned_on_success, ComponentData::SplineSegment(RawComponentData(data.slice.to_vec()))),
        ComponentId::WarningPulseLight => (input_returned_on_success, ComponentData::WarningPulseLight(RawComponentData(data.slice.to_vec()))),
        ComponentId::RumbleZone => (input_returned_on_success, ComponentData::RumbleZone(RawComponentData(data.slice.to_vec()))),
        ComponentId::HideOnVirusSpiritEvent => (input_returned_on_success, ComponentData::HideOnVirusSpiritEvent(RawComponentData(data.slice.to_vec()))),
        ComponentId::TrackAttachment => (input_returned_on_success, ComponentData::TrackAttachment(RawComponentData(data.slice.to_vec()))),
        ComponentId::LevelPlaylist => (input_returned_on_success, ComponentData::LevelPlaylist(RawComponentData(data.slice.to_vec()))),
        ComponentId::ProfileProgress => (input_returned_on_success, ComponentData::ProfileProgress(RawComponentData(data.slice.to_vec()))),
        ComponentId::GeneralSettings => (input_returned_on_success, ComponentData::GeneralSettings(RawComponentData(data.slice.to_vec()))),
        ComponentId::WorkshopPublishedFileInfos => (input_returned_on_success, ComponentData::WorkshopPublishedFileInfos(RawComponentData(data.slice.to_vec()))),
        ComponentId::WarpAnchor => (input_returned_on_success, ComponentData::WarpAnchor(RawComponentData(data.slice.to_vec()))),
        ComponentId::SetActiveOnMIDIEvent => (input_returned_on_success, ComponentData::SetActiveOnMIDIEvent(RawComponentData(data.slice.to_vec()))),
        ComponentId::TurnLightOnNearCar => (input_returned_on_success, ComponentData::TurnLightOnNearCar(RawComponentData(data.slice.to_vec()))),
        ComponentId::Traffic => (input_returned_on_success, ComponentData::Traffic(RawComponentData(data.slice.to_vec()))),
        ComponentId::TrackManipulatorNode => (input_returned_on_success, ComponentData::TrackManipulatorNode(RawComponentData(data.slice.to_vec()))),
        ComponentId::AudioEventTrigger => (input_returned_on_success, ComponentData::AudioEventTrigger(RawComponentData(data.slice.to_vec()))),
        ComponentId::LevelEditorSettings => (input_returned_on_success, ComponentData::LevelEditorSettings(RawComponentData(data.slice.to_vec()))),
        ComponentId::EmpireProximityDoorLogic => (input_returned_on_success, ComponentData::EmpireProximityDoorLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::Biodome => (input_returned_on_success, ComponentData::Biodome(RawComponentData(data.slice.to_vec()))),
        ComponentId::TunnelHorrorLogic => (input_returned_on_success, ComponentData::TunnelHorrorLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::VirusSpiritWarpTeaserLogic => (input_returned_on_success, ComponentData::VirusSpiritWarpTeaserLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::CarReplayData => (input_returned_on_success, ComponentData::CarReplayData(RawComponentData(data.slice.to_vec()))),
        ComponentId::LevelImageCamera => (input_returned_on_success, ComponentData::LevelImageCamera(RawComponentData(data.slice.to_vec()))),
        ComponentId::ParticlesGPU => (input_returned_on_success, ComponentData::ParticlesGPU(RawComponentData(data.slice.to_vec()))),
        ComponentId::KillGridBox => (input_returned_on_success, ComponentData::KillGridBox(RawComponentData(data.slice.to_vec()))),
        ComponentId::GoldenSimples => (input_returned_on_success, ComponentData::GoldenSimples(RawComponentData(data.slice.to_vec()))),
        ComponentId::SetActiveAfterWarp => (input_returned_on_success, ComponentData::SetActiveAfterWarp(RawComponentData(data.slice.to_vec()))),
        ComponentId::AmbientAudioObject => (input_returned_on_success, ComponentData::AmbientAudioObject(RawComponentData(data.slice.to_vec()))),
        ComponentId::BiodomeAudioInterpolator => (input_returned_on_success, ComponentData::BiodomeAudioInterpolator(RawComponentData(data.slice.to_vec()))),
        ComponentId::MoveElectricityAlongWire => (input_returned_on_success, ComponentData::MoveElectricityAlongWire(RawComponentData(data.slice.to_vec()))),
        ComponentId::ActivationRampLogic => (input_returned_on_success, ComponentData::ActivationRampLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::ZEventTrigger => (input_returned_on_success, ComponentData::ZEventTrigger(RawComponentData(data.slice.to_vec()))),
        ComponentId::ZEventListener => (input_returned_on_success, ComponentData::ZEventListener(RawComponentData(data.slice.to_vec()))),
        ComponentId::BlackPortalLogic => (input_returned_on_success, ComponentData::BlackPortalLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::VRSettings => (input_returned_on_success, ComponentData::VRSettings(RawComponentData(data.slice.to_vec()))),
        ComponentId::CutsceneCamera => (input_returned_on_success, ComponentData::CutsceneCamera(RawComponentData(data.slice.to_vec()))),
        ComponentId::ProfileStats => (input_returned_on_success, ComponentData::ProfileStats(RawComponentData(data.slice.to_vec()))),
        ComponentId::InterpolateToRotationOnTrigger => (input_returned_on_success, ComponentData::InterpolateToRotationOnTrigger(RawComponentData(data.slice.to_vec()))),
        ComponentId::MoveAlongAttachedTrack => (input_returned_on_success, ComponentData::MoveAlongAttachedTrack(RawComponentData(data.slice.to_vec()))),
        ComponentId::ShowDuringGlitch => (input_returned_on_success, ComponentData::ShowDuringGlitch(RawComponentData(data.slice.to_vec()))),
        ComponentId::AddCameraNoise => (input_returned_on_success, ComponentData::AddCameraNoise(RawComponentData(data.slice.to_vec()))),
        ComponentId::CarVoiceTrigger => (input_returned_on_success, ComponentData::CarVoiceTrigger(RawComponentData(data.slice.to_vec()))),
        ComponentId::HoverScreenSpecialObjectTrigger => (input_returned_on_success, ComponentData::HoverScreenSpecialObjectTrigger(RawComponentData(data.slice.to_vec()))),
        ComponentId::ReplaySettings => (input_returned_on_success, ComponentData::ReplaySettings(RawComponentData(data.slice.to_vec()))),
        ComponentId::CutsceneCamForTrailer => (input_returned_on_success, ComponentData::CutsceneCamForTrailer(RawComponentData(data.slice.to_vec()))),
        ComponentId::LevelInfos => (input_returned_on_success, ComponentData::LevelInfos(RawComponentData(data.slice.to_vec()))),
        ComponentId::AchievementTrigger => (input_returned_on_success, ComponentData::AchievementTrigger(RawComponentData(data.slice.to_vec()))),
        ComponentId::ArenaCarSpawner => (input_returned_on_success, ComponentData::ArenaCarSpawner(RawComponentData(data.slice.to_vec()))),
        ComponentId::Animated => (input_returned_on_success, ComponentData::Animated(RawComponentData(data.slice.to_vec()))),
        ComponentId::BlinkInTrigger => (input_returned_on_success, ComponentData::BlinkInTrigger(RawComponentData(data.slice.to_vec()))),
        ComponentId::CarScreenImageTrigger => (input_returned_on_success, ComponentData::CarScreenImageTrigger(RawComponentData(data.slice.to_vec()))),
        ComponentId::ExcludeFromEMP => (input_returned_on_success, ComponentData::ExcludeFromEMP(RawComponentData(data.slice.to_vec()))),
        ComponentId::InfiniteCooldownTrigger => (input_returned_on_success, ComponentData::InfiniteCooldownTrigger(RawComponentData(data.slice.to_vec()))),
        ComponentId::DiscoverableStuntArea => (input_returned_on_success, ComponentData::DiscoverableStuntArea(RawComponentData(data.slice.to_vec()))),
        ComponentId::ForceVolume => (input_returned_on_success, ComponentData::ForceVolume(RawComponentData(data.slice.to_vec()))),
        ComponentId::AdventureModeCompleteTrigger => (input_returned_on_success, ComponentData::AdventureModeCompleteTrigger(RawComponentData(data.slice.to_vec()))),
        ComponentId::CountdownTextMeshLogic => (input_returned_on_success, ComponentData::CountdownTextMeshLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::AbilitySignButtonColorLogic => (input_returned_on_success, ComponentData::AbilitySignButtonColorLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::GoldenAnimator => (input_returned_on_success, ComponentData::GoldenAnimator(RawComponentData(data.slice.to_vec()))),
        ComponentId::AnimatorAudio => (input_returned_on_success, ComponentData::AnimatorAudio(RawComponentData(data.slice.to_vec()))),
        ComponentId::AnimatorCameraShake => (input_returned_on_success, ComponentData::AnimatorCameraShake(RawComponentData(data.slice.to_vec()))),
        ComponentId::ShardCluster => (input_returned_on_success, ComponentData::ShardCluster(RawComponentData(data.slice.to_vec()))),
        ComponentId::AdventureSpecialIntro => (input_returned_on_success, ComponentData::AdventureSpecialIntro(RawComponentData(data.slice.to_vec()))),
        ComponentId::AudioEffectZone => (input_returned_on_success, ComponentData::AudioEffectZone(RawComponentData(data.slice.to_vec()))),
        ComponentId::CinematicCamera => (input_returned_on_success, ComponentData::CinematicCamera(RawComponentData(data.slice.to_vec()))),
        ComponentId::CinematicCameraFocalPoint => (input_returned_on_success, ComponentData::CinematicCameraFocalPoint(RawComponentData(data.slice.to_vec()))),
        ComponentId::SetAbilitiesTrigger => (input_returned_on_success, ComponentData::SetAbilitiesTrigger(RawComponentData(data.slice.to_vec()))),
        ComponentId::LostToEchoesIntroCutscene => (input_returned_on_success, ComponentData::LostToEchoesIntroCutscene(RawComponentData(data.slice.to_vec()))),
        ComponentId::CutsceneText => (input_returned_on_success, ComponentData::CutsceneText(RawComponentData(data.slice.to_vec()))),
        ComponentId::UltraPlanet => (input_returned_on_success, ComponentData::UltraPlanet(RawComponentData(data.slice.to_vec()))),
        ComponentId::DeadCarLogic => (input_returned_on_success, ComponentData::DeadCarLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::RollingBarrelDropperLogic => (input_returned_on_success, ComponentData::RollingBarrelDropperLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::AdventureFinishTrigger => (input_returned_on_success, ComponentData::AdventureFinishTrigger(RawComponentData(data.slice.to_vec()))),
        ComponentId::AchievementSettings => (input_returned_on_success, ComponentData::AchievementSettings(RawComponentData(data.slice.to_vec()))),
        ComponentId::InterpolateRTPCLogic => (input_returned_on_success, ComponentData::InterpolateRTPCLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::TriggerCooldownLogic => (input_returned_on_success, ComponentData::TriggerCooldownLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::ShadowsChangedListener => (input_returned_on_success, ComponentData::ShadowsChangedListener(RawComponentData(data.slice.to_vec()))),
        ComponentId::LookAtCamera => (input_returned_on_success, ComponentData::LookAtCamera(RawComponentData(data.slice.to_vec()))),
        ComponentId::CubeMapRenderer => (input_returned_on_success, ComponentData::CubeMapRenderer(RawComponentData(data.slice.to_vec()))),
        ComponentId::RealtimeReflectionRenderer => (input_returned_on_success, ComponentData::RealtimeReflectionRenderer(RawComponentData(data.slice.to_vec()))),
        ComponentId::VirusDropperDroneLogic => (input_returned_on_success, ComponentData::VirusDropperDroneLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::OnCollisionBreakApartLogic => (input_returned_on_success, ComponentData::OnCollisionBreakApartLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::CheatSettings => (input_returned_on_success, ComponentData::CheatSettings(RawComponentData(data.slice.to_vec()))),
        ComponentId::IgnoreInCullGroups => (input_returned_on_success, ComponentData::IgnoreInCullGroups(RawComponentData(data.slice.to_vec()))),
        ComponentId::IgnoreInputTrigger => (input_returned_on_success, ComponentData::IgnoreInputTrigger(RawComponentData(data.slice.to_vec()))),
        ComponentId::PowerPosterLogic => (input_returned_on_success, ComponentData::PowerPosterLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::MusicZone => (input_returned_on_success, ComponentData::MusicZone(RawComponentData(data.slice.to_vec()))),
        ComponentId::LightsFlickerLogic => (input_returned_on_success, ComponentData::LightsFlickerLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::CutsceneManagerLogic => (input_returned_on_success, ComponentData::CutsceneManagerLogic(RawComponentData(data.slice.to_vec()))),
        ComponentId::FadeOut => (input_returned_on_success, ComponentData::FadeOut(RawComponentData(data.slice.to_vec()))),
        ComponentId::Flock => (input_returned_on_success, ComponentData::Flock(RawComponentData(data.slice.to_vec()))),
        ComponentId::GPSTrigger => (input_returned_on_success, ComponentData::GPSTrigger(RawComponentData(data.slice.to_vec()))),
        ComponentId::SprintMode => (input_returned_on_success, ComponentData::SprintMode(RawComponentData(data.slice.to_vec()))),
        ComponentId::StuntMode => (input_returned_on_success, ComponentData::StuntMode(RawComponentData(data.slice.to_vec()))),
        ComponentId::SoccerMode => (input_returned_on_success, ComponentData::SoccerMode(RawComponentData(data.slice.to_vec()))),
        ComponentId::FreeRoamMode => (input_returned_on_success, ComponentData::FreeRoamMode(RawComponentData(data.slice.to_vec()))),
        ComponentId::ReverseTagMode => (input_returned_on_success, ComponentData::ReverseTagMode(RawComponentData(data.slice.to_vec()))),
        ComponentId::LevelEditorPlayMode => (input_returned_on_success, ComponentData::LevelEditorPlayMode(RawComponentData(data.slice.to_vec()))),
        ComponentId::CoopSprintMode => (input_returned_on_success, ComponentData::CoopSprintMode(RawComponentData(data.slice.to_vec()))),
        ComponentId::ChallengeMode => (input_returned_on_success, ComponentData::ChallengeMode(RawComponentData(data.slice.to_vec()))),
        ComponentId::AdventureMode => (input_returned_on_success, ComponentData::AdventureMode(RawComponentData(data.slice.to_vec()))),
        ComponentId::SpeedAndStyleMode => (input_returned_on_success, ComponentData::SpeedAndStyleMode(RawComponentData(data.slice.to_vec()))),
        ComponentId::TrackmogrifyMode => (input_returned_on_success, ComponentData::TrackmogrifyMode(RawComponentData(data.slice.to_vec()))),
        ComponentId::DemoMode => (input_returned_on_success, ComponentData::DemoMode(RawComponentData(data.slice.to_vec()))),
        ComponentId::MainMenuMode => (input_returned_on_success, ComponentData::MainMenuMode(RawComponentData(data.slice.to_vec()))),
        ComponentId::LostToEchoesMode => (input_returned_on_success, ComponentData::LostToEchoesMode(RawComponentData(data.slice.to_vec()))),
        ComponentId::NexusMode => (input_returned_on_success, ComponentData::NexusMode(RawComponentData(data.slice.to_vec()))),
        ComponentId::TheOtherSideMode => (input_returned_on_success, ComponentData::TheOtherSideMode(RawComponentData(data.slice.to_vec()))),
        id => {
            return Err(failure(data,BytesDeserializeErrorKind::Component(Some(id))));
        }
    };

    let component = Component {
        guid,
        data: component_data,
    };
    Ok((input, component))
}

fn transform(input: Input<'_>) -> BytesParseResult<'_, Transform> {
    let (input, mut position) = serialization::read_vector_3(input)?;
    let (input, mut rotation) = serialization::read_quaternion(input)?;
    let (input, mut scale) = serialization::read_vector_3(input)?;

    if let Some(position) = &mut position {
        let is_valid = position.x.is_finite() && position.y.is_finite() && position.z.is_finite();
        if !is_valid {
            *position = Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
        }
    }

    if let Some(rotation) = &mut rotation {
        let is_valid = rotation.v.x.is_finite()
            && rotation.v.y.is_finite()
            && rotation.v.z.is_finite()
            && rotation.s.is_finite();
        if !is_valid {
            *rotation = Quaternion::from([0.0, 0.0, 0.0, 1.0]);
        }
    }

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

    let err_input = input;
    let (input, children_scope_mark) = serialization::read_scope_mark(input)?;
    if children_scope_mark != 55555555 {
        return Err(failure(
            err_input,
            BytesDeserializeErrorKind::Component(Some(ComponentId::Transform)),
        ));
    }

    let err_input = input;
    let (input, children_scope_len) = le_i64(input)?;
    let children_scope_len: usize = children_scope_len.try_into().map_err(|_| {
        failure(
            err_input,
            BytesDeserializeErrorKind::Component(Some(ComponentId::Transform)),
        )
    })?;

    let (after_children, children_scope) = take(children_scope_len)(input)?;

    let err_input = input;
    let (children_scope, num_children) = le_i32(children_scope)?;
    let num_children: usize = num_children.try_into().map_err(|_| {
        failure(
            err_input,
            BytesDeserializeErrorKind::Component(Some(ComponentId::Transform)),
        )
    })?;
    let (children_scope, children) =
        many_m_n(num_children, num_children, serialization::read_game_object)(children_scope)?;

    if !children_scope.slice.is_empty() {
        return Err(failure(
            children_scope,
            BytesDeserializeErrorKind::Component(Some(ComponentId::Transform)),
        ));
    }

    let transform = Transform {
        position,
        rotation,
        scale,
        children,
    };

    Ok((after_children, transform))
}
