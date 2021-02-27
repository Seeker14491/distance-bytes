use crate::{
    domain::{
        component::{GoldenSimples, Transform},
        Quaternion, Vector3,
    },
    serialization::{
        string, Serializable, VisitDirection, Visitor, EMPTY_MARK, INVALID_FLOAT, INVALID_INT,
        INVALID_QUATERNION, INVALID_VECTOR_3,
    },
    util, Component, ComponentData, GameObject, RawComponentData,
};
use anyhow::Error;
use byteorder::{WriteBytesExt, LE};
use std::{
    convert::TryInto,
    io::{Seek, SeekFrom, Write},
};

#[derive(Debug, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Serializer<W: Write + Seek> {
    writer: W,
    scope_stack: Vec<u64>,
}

impl<W: Write + Seek> Serializer<W> {
    pub fn new(writer: W) -> Self {
        Serializer {
            writer,
            scope_stack: Vec::new(),
        }
    }

    pub fn write_game_object(&mut self, game_object: &mut GameObject) -> Result<(), Error> {
        self.write_start_scope(66666666)?;
        self.write_string(&game_object.name)?;
        self.write_string("")?;
        self.writer.write_u32::<LE>(game_object.guid)?;

        self.write_components(&mut game_object.components)?;

        self.write_end_scope(-1)?;

        Ok(())
    }

    fn write_components(&mut self, components: &mut [Component]) -> Result<(), Error> {
        self.writer.write_i32::<LE>(components.len().try_into()?)?;
        for component in components {
            self.write_component(component)?;
        }

        Ok(())
    }

    fn write_component(&mut self, component: &mut Component) -> Result<(), Error> {
        // The game would actually write either `22222222` or `33333333` here, but any of these
        // values work the same when read.
        self.write_component_start(component, 32323232)?;

        self.write_component_helper(component)?;
        self.write_end_scope(-1)?;

        Ok(())
    }

    #[rustfmt::skip]
    fn write_component_helper(&mut self, component: &mut Component) -> Result<(), Error> {
        let mut unimplemented_component =
            |data: &mut RawComponentData| self.writer.write_all(&data.0);

        match &mut component.data {
            ComponentData::Transform(x) => x.accept(self, Transform::VERSION)?,
            ComponentData::MeshRenderer(x) => unimplemented_component(x)?,
            ComponentData::TextMesh(x) => unimplemented_component(x)?,
            ComponentData::Light(x) => unimplemented_component(x)?,
            ComponentData::LensFlare(x) => unimplemented_component(x)?,
            ComponentData::Projector(x) => unimplemented_component(x)?,
            ComponentData::SphereCollider(x) => unimplemented_component(x)?,
            ComponentData::BoxCollider(x) => unimplemented_component(x)?,
            ComponentData::CapsuleCollider(x) => unimplemented_component(x)?,
            ComponentData::BezierSplineTrack(x) => unimplemented_component(x)?,
            ComponentData::TrackSegment(x) => unimplemented_component(x)?,
            ComponentData::TrackLink(x) => unimplemented_component(x)?,
            ComponentData::RigidbodyAxisRotationLogic(x) => unimplemented_component(x)?,
            ComponentData::BackAndForthSawLogic(x) => unimplemented_component(x)?,
            ComponentData::CheckpointLogic(x) => unimplemented_component(x)?,
            ComponentData::LightFlickerLogic(x) => unimplemented_component(x)?,
            ComponentData::Group(x) => unimplemented_component(x)?,
            ComponentData::TutorialBoxText(x) => unimplemented_component(x)?,
            ComponentData::FlyingRingLogic(x) => unimplemented_component(x)?,
            ComponentData::PopupBlockerLogic(x) => unimplemented_component(x)?,
            ComponentData::PulseLight(x) => unimplemented_component(x)?,
            ComponentData::PulseMaterial(x) => unimplemented_component(x)?,
            ComponentData::SmoothRandomPosition(x) => unimplemented_component(x)?,
            ComponentData::SoccerGoalLogic(x) => unimplemented_component(x)?,
            ComponentData::VirusMineLogic(x) => unimplemented_component(x)?,
            ComponentData::BrightenCarHeadlights(x) => unimplemented_component(x)?,
            ComponentData::GameData(x) => unimplemented_component(x)?,
            ComponentData::GraphicsSettings(x) => unimplemented_component(x)?,
            ComponentData::AudioSettings(x) => unimplemented_component(x)?,
            ComponentData::ControlsSettings(x) => unimplemented_component(x)?,
            ComponentData::Profile(x) => unimplemented_component(x)?,
            ComponentData::ToolInputCombos(x) => unimplemented_component(x)?,
            ComponentData::ColorPreset(x) => unimplemented_component(x)?,
            ComponentData::LocalLeaderboard(x) => unimplemented_component(x)?,
            ComponentData::AxisRotationLogic(x) => unimplemented_component(x)?,
            ComponentData::ParticleEmitLogic(x) => unimplemented_component(x)?,
            ComponentData::VirusSpiritSpawner(x) => unimplemented_component(x)?,
            ComponentData::PulseRotateOnTrigger(x) => unimplemented_component(x)?,
            ComponentData::TeleporterEntrance(x) => unimplemented_component(x)?,
            ComponentData::TeleporterExit(x) => unimplemented_component(x)?,
            ComponentData::ControlScheme(x) => unimplemented_component(x)?,
            ComponentData::DeviceToSchemeLinks(x) => unimplemented_component(x)?,
            ComponentData::ObjectSpawnCircle(x) => unimplemented_component(x)?,
            ComponentData::InterpolateToPositionOnTrigger(x) => unimplemented_component(x)?,
            ComponentData::EngageBrokenPieces(x) => unimplemented_component(x)?,
            ComponentData::GravityToggle(x) => unimplemented_component(x)?,
            ComponentData::CarSpawner(x) => unimplemented_component(x)?,
            ComponentData::RaceStartCarSpawner(x) => unimplemented_component(x)?,
            ComponentData::LevelEditorCarSpawner(x) => unimplemented_component(x)?,
            ComponentData::InfoDisplayLogic(x) => unimplemented_component(x)?,
            ComponentData::MusicTrigger(x) => unimplemented_component(x)?,
            ComponentData::TabPopulator(x) => unimplemented_component(x)?,
            ComponentData::AdventureAbilitySettings(x) => unimplemented_component(x)?,
            ComponentData::IndicatorDisplayLogic(x) => unimplemented_component(x)?,
            ComponentData::PulseCoreLogic(x) => unimplemented_component(x)?,
            ComponentData::PulseAll(x) => unimplemented_component(x)?,
            ComponentData::TeleporterExitCheckpoint(x) => unimplemented_component(x)?,
            ComponentData::LevelSettings(x) => unimplemented_component(x)?,
            ComponentData::WingCorruptionZone(x) => unimplemented_component(x)?,
            ComponentData::GenerateCreditsNames(x) => unimplemented_component(x)?,
            ComponentData::IntroCutsceneLightFadeIn(x) => unimplemented_component(x)?,
            ComponentData::QuarantineTrigger(x) => unimplemented_component(x)?,
            ComponentData::CarScreenTextDecodeTrigger(x) => unimplemented_component(x)?,
            ComponentData::GlitchFieldLogic(x) => unimplemented_component(x)?,
            ComponentData::FogSkyboxAmbientChangeTrigger(x) => unimplemented_component(x)?,
            ComponentData::FinalCountdownLogic(x) => unimplemented_component(x)?,
            ComponentData::SetActiveOnIntroCutsceneStarted(x) => unimplemented_component(x)?,
            ComponentData::RaceEndLogic(x) => unimplemented_component(x)?,
            ComponentData::EnableAbilitiesTrigger(x) => unimplemented_component(x)?,
            ComponentData::SphericalGravity(x) => unimplemented_component(x)?,
            ComponentData::CreditsNameOrbLogic(x) => unimplemented_component(x)?,
            ComponentData::DisableLocalCarWarnings(x) => unimplemented_component(x)?,
            ComponentData::CustomName(x) => unimplemented_component(x)?,
            ComponentData::SplineSegment(x) => unimplemented_component(x)?,
            ComponentData::WarningPulseLight(x) => unimplemented_component(x)?,
            ComponentData::RumbleZone(x) => unimplemented_component(x)?,
            ComponentData::HideOnVirusSpiritEvent(x) => unimplemented_component(x)?,
            ComponentData::TrackAttachment(x) => unimplemented_component(x)?,
            ComponentData::LevelPlaylist(x) => unimplemented_component(x)?,
            ComponentData::ProfileProgress(x) => unimplemented_component(x)?,
            ComponentData::GeneralSettings(x) => unimplemented_component(x)?,
            ComponentData::WorkshopPublishedFileInfos(x) => unimplemented_component(x)?,
            ComponentData::WarpAnchor(x) => unimplemented_component(x)?,
            ComponentData::SetActiveOnMIDIEvent(x) => unimplemented_component(x)?,
            ComponentData::TurnLightOnNearCar(x) => unimplemented_component(x)?,
            ComponentData::Traffic(x) => unimplemented_component(x)?,
            ComponentData::TrackManipulatorNode(x) => unimplemented_component(x)?,
            ComponentData::AudioEventTrigger(x) => unimplemented_component(x)?,
            ComponentData::LevelEditorSettings(x) => unimplemented_component(x)?,
            ComponentData::EmpireProximityDoorLogic(x) => unimplemented_component(x)?,
            ComponentData::Biodome(x) => unimplemented_component(x)?,
            ComponentData::TunnelHorrorLogic(x) => unimplemented_component(x)?,
            ComponentData::VirusSpiritWarpTeaserLogic(x) => unimplemented_component(x)?,
            ComponentData::CarReplayData(x) => unimplemented_component(x)?,
            ComponentData::LevelImageCamera(x) => unimplemented_component(x)?,
            ComponentData::ParticlesGPU(x) => unimplemented_component(x)?,
            ComponentData::KillGridBox(x) => unimplemented_component(x)?,
            ComponentData::GoldenSimples(x) => x.accept(self, GoldenSimples::VERSION)?,
            ComponentData::SetActiveAfterWarp(x) => unimplemented_component(x)?,
            ComponentData::AmbientAudioObject(x) => unimplemented_component(x)?,
            ComponentData::BiodomeAudioInterpolator(x) => unimplemented_component(x)?,
            ComponentData::MoveElectricityAlongWire(x) => unimplemented_component(x)?,
            ComponentData::ActivationRampLogic(x) => unimplemented_component(x)?,
            ComponentData::ZEventTrigger(x) => unimplemented_component(x)?,
            ComponentData::ZEventListener(x) => unimplemented_component(x)?,
            ComponentData::BlackPortalLogic(x) => unimplemented_component(x)?,
            ComponentData::VRSettings(x) => unimplemented_component(x)?,
            ComponentData::CutsceneCamera(x) => unimplemented_component(x)?,
            ComponentData::ProfileStats(x) => unimplemented_component(x)?,
            ComponentData::InterpolateToRotationOnTrigger(x) => unimplemented_component(x)?,
            ComponentData::MoveAlongAttachedTrack(x) => unimplemented_component(x)?,
            ComponentData::ShowDuringGlitch(x) => unimplemented_component(x)?,
            ComponentData::AddCameraNoise(x) => unimplemented_component(x)?,
            ComponentData::CarVoiceTrigger(x) => unimplemented_component(x)?,
            ComponentData::HoverScreenSpecialObjectTrigger(x) => unimplemented_component(x)?,
            ComponentData::ReplaySettings(x) => unimplemented_component(x)?,
            ComponentData::CutsceneCamForTrailer(x) => unimplemented_component(x)?,
            ComponentData::LevelInfos(x) => unimplemented_component(x)?,
            ComponentData::AchievementTrigger(x) => unimplemented_component(x)?,
            ComponentData::ArenaCarSpawner(x) => unimplemented_component(x)?,
            ComponentData::Animated(x) => unimplemented_component(x)?,
            ComponentData::BlinkInTrigger(x) => unimplemented_component(x)?,
            ComponentData::CarScreenImageTrigger(x) => unimplemented_component(x)?,
            ComponentData::ExcludeFromEMP(x) => unimplemented_component(x)?,
            ComponentData::InfiniteCooldownTrigger(x) => unimplemented_component(x)?,
            ComponentData::DiscoverableStuntArea(x) => unimplemented_component(x)?,
            ComponentData::ForceVolume(x) => unimplemented_component(x)?,
            ComponentData::AdventureModeCompleteTrigger(x) => unimplemented_component(x)?,
            ComponentData::CountdownTextMeshLogic(x) => unimplemented_component(x)?,
            ComponentData::AbilitySignButtonColorLogic(x) => unimplemented_component(x)?,
            ComponentData::GoldenAnimator(x) => unimplemented_component(x)?,
            ComponentData::AnimatorAudio(x) => unimplemented_component(x)?,
            ComponentData::AnimatorCameraShake(x) => unimplemented_component(x)?,
            ComponentData::ShardCluster(x) => unimplemented_component(x)?,
            ComponentData::AdventureSpecialIntro(x) => unimplemented_component(x)?,
            ComponentData::AudioEffectZone(x) => unimplemented_component(x)?,
            ComponentData::CinematicCamera(x) => unimplemented_component(x)?,
            ComponentData::CinematicCameraFocalPoint(x) => unimplemented_component(x)?,
            ComponentData::SetAbilitiesTrigger(x) => unimplemented_component(x)?,
            ComponentData::LostToEchoesIntroCutscene(x) => unimplemented_component(x)?,
            ComponentData::CutsceneText(x) => unimplemented_component(x)?,
            ComponentData::UltraPlanet(x) => unimplemented_component(x)?,
            ComponentData::DeadCarLogic(x) => unimplemented_component(x)?,
            ComponentData::RollingBarrelDropperLogic(x) => unimplemented_component(x)?,
            ComponentData::AdventureFinishTrigger(x) => unimplemented_component(x)?,
            ComponentData::AchievementSettings(x) => unimplemented_component(x)?,
            ComponentData::InterpolateRTPCLogic(x) => unimplemented_component(x)?,
            ComponentData::TriggerCooldownLogic(x) => unimplemented_component(x)?,
            ComponentData::ShadowsChangedListener(x) => unimplemented_component(x)?,
            ComponentData::LookAtCamera(x) => unimplemented_component(x)?,
            ComponentData::CubeMapRenderer(x) => unimplemented_component(x)?,
            ComponentData::RealtimeReflectionRenderer(x) => unimplemented_component(x)?,
            ComponentData::VirusDropperDroneLogic(x) => unimplemented_component(x)?,
            ComponentData::OnCollisionBreakApartLogic(x) => unimplemented_component(x)?,
            ComponentData::CheatSettings(x) => unimplemented_component(x)?,
            ComponentData::IgnoreInCullGroups(x) => unimplemented_component(x)?,
            ComponentData::IgnoreInputTrigger(x) => unimplemented_component(x)?,
            ComponentData::PowerPosterLogic(x) => unimplemented_component(x)?,
            ComponentData::MusicZone(x) => unimplemented_component(x)?,
            ComponentData::LightsFlickerLogic(x) => unimplemented_component(x)?,
            ComponentData::CutsceneManagerLogic(x) => unimplemented_component(x)?,
            ComponentData::FadeOut(x) => unimplemented_component(x)?,
            ComponentData::Flock(x) => unimplemented_component(x)?,
            ComponentData::GPSTrigger(x) => unimplemented_component(x)?,
            ComponentData::SprintMode(x) => unimplemented_component(x)?,
            ComponentData::StuntMode(x) => unimplemented_component(x)?,
            ComponentData::SoccerMode(x) => unimplemented_component(x)?,
            ComponentData::FreeRoamMode(x) => unimplemented_component(x)?,
            ComponentData::ReverseTagMode(x) => unimplemented_component(x)?,
            ComponentData::LevelEditorPlayMode(x) => unimplemented_component(x)?,
            ComponentData::CoopSprintMode(x) => unimplemented_component(x)?,
            ComponentData::ChallengeMode(x) => unimplemented_component(x)?,
            ComponentData::AdventureMode(x) => unimplemented_component(x)?,
            ComponentData::SpeedAndStyleMode(x) => unimplemented_component(x)?,
            ComponentData::TrackmogrifyMode(x) => unimplemented_component(x)?,
            ComponentData::DemoMode(x) => unimplemented_component(x)?,
            ComponentData::MainMenuMode(x) => unimplemented_component(x)?,
            ComponentData::LostToEchoesMode(x) => unimplemented_component(x)?,
            ComponentData::NexusMode(x) => unimplemented_component(x)?,
            ComponentData::TheOtherSideMode(x) => unimplemented_component(x)?,
        }

        Ok(())
    }

    fn write_component_start(
        &mut self,
        component: &Component,
        scope_mark: i32,
    ) -> Result<(), Error> {
        self.write_start_scope(scope_mark)?;
        self.writer.write_i32::<LE>(component.id().into())?;
        self.writer.write_i32::<LE>(component.version)?;
        self.writer.write_u32::<LE>(component.guid)?;

        Ok(())
    }

    fn write_start_scope(&mut self, mark: i32) -> Result<(), Error> {
        self.writer.write_i32::<LE>(mark)?;

        // Temporary stand-in for scope length
        self.writer.write_i64::<LE>(mark.into())?;

        self.scope_stack.push(self.writer.stream_position()?);

        Ok(())
    }

    fn write_end_scope(&mut self, scope_info: i64) -> Result<(), Error> {
        let stack_pos = self
            .scope_stack
            .pop()
            .expect("unexpected empty scope stack");
        let section_len: i64 = (self.writer.stream_position()? - stack_pos).try_into()?;

        self.writer.seek(SeekFrom::Current(-(section_len + 8)))?;
        let value_to_write = if scope_info == -1 {
            section_len
        } else {
            scope_info
        };
        self.writer.write_i64::<LE>(value_to_write)?;
        self.writer.seek(SeekFrom::Current(section_len))?;

        Ok(())
    }

    fn write_empty(&mut self) -> Result<(), Error> {
        self.writer.write_i32::<LE>(EMPTY_MARK)?;

        Ok(())
    }

    fn write_string(&mut self, string: &str) -> Result<(), Error> {
        string::write(&mut self.writer, string)
    }
}

impl<W: Write + Seek> Visitor for Serializer<W> {
    const VISIT_DIRECTION: VisitDirection = VisitDirection::Out;

    fn visit_bool(&mut self, _name: &str, val: &mut bool) -> Result<(), Error> {
        self.writer.write_u8(*val as u8)?;

        Ok(())
    }

    fn visit_i32(&mut self, _name: &str, val: &mut i32) -> Result<(), Error> {
        if *val != INVALID_INT {
            self.writer.write_i32::<LE>(*val)?;
        } else {
            self.write_empty()?;
        }

        Ok(())
    }

    fn visit_u32(&mut self, _name: &str, val: &mut u32) -> Result<(), Error> {
        if *val != 0xFFFF_FF81 {
            self.writer.write_u32::<LE>(*val)?;
        } else {
            self.write_empty()?;
        }

        Ok(())
    }

    fn visit_i64(&mut self, _name: &str, val: &mut i64) -> Result<(), Error> {
        if *val != INVALID_INT as i64 {
            self.writer.write_i64::<LE>(*val)?;
        } else {
            self.write_empty()?;
        }

        Ok(())
    }

    fn visit_f32(&mut self, _name: &str, val: &mut f32) -> Result<(), Error> {
        if !util::f32_approx_equal(*val, INVALID_FLOAT) {
            self.writer.write_f32::<LE>(*val)?;
        } else {
            self.write_empty()?;
        }

        Ok(())
    }

    fn visit_vector_3(&mut self, _name: &str, val: &mut Vector3) -> Result<(), Error> {
        if !util::vector3_approx_equals(*val, INVALID_VECTOR_3) {
            self.writer.write_f32::<LE>(val.x)?;
            self.writer.write_f32::<LE>(val.y)?;
            self.writer.write_f32::<LE>(val.z)?;
        } else {
            self.write_empty()?;
        }

        Ok(())
    }

    fn visit_quaternion(&mut self, _name: &str, val: &mut Quaternion) -> Result<(), Error> {
        if !util::quaternion_approx_equals(*val, INVALID_QUATERNION) {
            self.writer.write_f32::<LE>(val.v.x)?;
            self.writer.write_f32::<LE>(val.v.y)?;
            self.writer.write_f32::<LE>(val.v.z)?;
            self.writer.write_f32::<LE>(val.s)?;
        } else {
            self.write_empty()?;
        }

        Ok(())
    }

    fn visit_children(&mut self, val: &mut Vec<GameObject>) -> Result<(), Error> {
        self.write_start_scope(55555555)?;

        let num_children: i32 = val.len().try_into()?;
        self.writer.write_i32::<LE>(num_children)?;

        for game_object in val {
            self.write_game_object(game_object)?;
        }

        self.write_end_scope(-1)?;

        Ok(())
    }
}
