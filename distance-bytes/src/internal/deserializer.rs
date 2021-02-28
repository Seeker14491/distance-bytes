use crate::internal::component::{Component, ComponentData, RawComponentData};
use crate::internal::{
    string, util, ComponentId, GameObject, Quaternion, Serializable, Vector3, VisitDirection,
    Visitor, EMPTY_MARK,
};
use anyhow::{bail, Error};
use byteorder::{ReadBytesExt, LE};
use num_traits::FromPrimitive;
use paste::paste;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};
use std::io::{Read, Seek, SeekFrom};
use std::{fmt, io, mem};
use tracing::{debug, warn};

pub fn read_game_object(reader: impl Read + Seek) -> Result<GameObject, Error> {
    Deserializer::new(reader).read_game_object()
}

#[derive(Debug, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct Deserializer<R: Read + Seek> {
    reader: R,
    scope_info_stack: Vec<ScopeInfo>,
}

impl<R: Read + Seek> Deserializer<R> {
    fn new(reader: R) -> Self {
        Deserializer {
            reader,
            scope_info_stack: Vec::new(),
        }
    }

    fn read_game_object(&mut self) -> Result<GameObject, Error> {
        let (prefab_name, guid) = self.read_game_object_start(true)?;
        let components = self.read_game_object_contents(guid)?;

        // FIXME: This might need to be false under some circumstances.
        let log_warn = true;

        self.read_end_scope(log_warn)?;

        let game_object = GameObject {
            name: prefab_name,
            guid,
            components,
        };

        Ok(game_object)
    }

    fn read_game_object_contents(&mut self, _guid: u32) -> Result<Vec<Component>, Error> {
        self.read_components()
    }

    // TODO: Check if necessary, and if so, implement and call this function from the proper places.
    #[allow(dead_code)]
    fn add_object_to_references(&mut self, _guid: u32) {}

    fn read_components(&mut self) -> Result<Vec<Component>, Error> {
        let mut num_components = 0;
        self.read_set_i32("numComponents", &mut num_components)?;
        let mut components = Vec::with_capacity(num_components.try_into()?);
        for _ in 0..num_components {
            if let Some(component) = self.read_component()? {
                components.push(component);
            }
        }

        Ok(components)
    }

    fn read_component(&mut self) -> Result<Option<Component>, Error> {
        let mut component_id = ComponentId::Invalid_;
        let mut name = String::new();
        let mut component_version = 0;
        let mut guid = 0;

        let scope_mark = self.read_start_scope(true)?;
        match scope_mark {
            33333333 | 22222222 | 32323232 => {
                let mut raw_id = 0;
                self.read_set_i32("componentID", &mut raw_id)?;
                if let Some(id_2) = ComponentId::from_i32(raw_id) {
                    component_id = id_2;
                } else {
                    warn!(id = raw_id, "unknown componentID");
                }

                name = format!("{:?}", component_id);
                self.read_set_i32("componentVersion", &mut component_version)?;
            }
            23232323 => {
                self.read_set_string("componentVersion", &mut name)?;
            }
            mark => {
                name = "Invalid".to_owned();
                warn!(mark, "invalid component mark");
            }
        }

        self.read_set_u32("component GUID", &mut guid)?;
        self.set_current_scope_name(format!("Comp:{}", name));

        if component_id != ComponentId::Invalid_ {
            let component = self.read_component_helper(component_id, component_version, guid)?;
            Ok(Some(component))
        } else {
            debug!(name = name.as_str(), guid, "skipping unknown component");
            Ok(None)
        }
    }

    fn read_component_helper(
        &mut self,
        component_id: ComponentId,
        version: i32,
        guid: u32,
    ) -> Result<Component, Error> {
        fn implemented_component<C: Serializable>(
            visitor: impl Visitor,
            f: fn(C) -> ComponentData,
            default_component: bool,
            version: i32,
            guid: u32,
        ) -> Result<Component, Error> {
            let mut inner_component = C::default();
            if !default_component {
                inner_component.accept(visitor, version)?;
            }
            let component_data = f(inner_component);
            let component = Component {
                version: C::VERSION,
                guid,
                data: component_data,
            };

            Ok(component)
        }

        let is_default_component = self.is_empty_scope()?;

        let mut unimplemented_component =
            |f: fn(RawComponentData) -> ComponentData| -> Result<Component, Error> {
                let component_data = if is_default_component {
                    f(RawComponentData::default())
                } else {
                    let current_pos: usize = self.reader.stream_position()?.try_into()?;
                    let data_len = self
                        .scope_info_stack
                        .last()
                        .map(|scope_info| scope_info.end_pos - current_pos)
                        .unwrap_or(0);

                    let mut data = vec![0; data_len];
                    self.reader.read_exact(&mut data)?;

                    f(RawComponentData(data))
                };

                let component = Component {
                    version,
                    guid,
                    data: component_data,
                };

                Ok(component)
            };

        #[rustfmt::skip]
        let component = match component_id {
            ComponentId::Transform => {
                implemented_component(self, ComponentData::Transform, is_default_component, version, guid)?
            }
            ComponentId::MeshRenderer => unimplemented_component(ComponentData::MeshRenderer)?,
            ComponentId::TextMesh => unimplemented_component(ComponentData::TextMesh)?,
            ComponentId::Light => unimplemented_component(ComponentData::Light)?,
            ComponentId::LensFlare => unimplemented_component(ComponentData::LensFlare)?,
            ComponentId::Projector => unimplemented_component(ComponentData::Projector)?,
            ComponentId::SphereCollider => unimplemented_component(ComponentData::SphereCollider)?,
            ComponentId::BoxCollider => unimplemented_component(ComponentData::BoxCollider)?,
            ComponentId::CapsuleCollider => unimplemented_component(ComponentData::CapsuleCollider)?,
            ComponentId::BezierSplineTrack => unimplemented_component(ComponentData::BezierSplineTrack)?,
            ComponentId::TrackSegment => unimplemented_component(ComponentData::TrackSegment)?,
            ComponentId::TrackLink => unimplemented_component(ComponentData::TrackLink)?,
            ComponentId::RigidbodyAxisRotationLogic => unimplemented_component(ComponentData::RigidbodyAxisRotationLogic)?,
            ComponentId::BackAndForthSawLogic => unimplemented_component(ComponentData::BackAndForthSawLogic)?,
            ComponentId::CheckpointLogic => unimplemented_component(ComponentData::CheckpointLogic)?,
            ComponentId::LightFlickerLogic => unimplemented_component(ComponentData::LightFlickerLogic)?,
            ComponentId::Group => unimplemented_component(ComponentData::Group)?,
            ComponentId::TutorialBoxText => unimplemented_component(ComponentData::TutorialBoxText)?,
            ComponentId::FlyingRingLogic => unimplemented_component(ComponentData::FlyingRingLogic)?,
            ComponentId::PopupBlockerLogic => unimplemented_component(ComponentData::PopupBlockerLogic)?,
            ComponentId::PulseLight => unimplemented_component(ComponentData::PulseLight)?,
            ComponentId::PulseMaterial => unimplemented_component(ComponentData::PulseMaterial)?,
            ComponentId::SmoothRandomPosition => unimplemented_component(ComponentData::SmoothRandomPosition)?,
            ComponentId::SoccerGoalLogic => unimplemented_component(ComponentData::SoccerGoalLogic)?,
            ComponentId::VirusMineLogic => unimplemented_component(ComponentData::VirusMineLogic)?,
            ComponentId::BrightenCarHeadlights => unimplemented_component(ComponentData::BrightenCarHeadlights)?,
            ComponentId::GameData => unimplemented_component(ComponentData::GameData)?,
            ComponentId::GraphicsSettings => unimplemented_component(ComponentData::GraphicsSettings)?,
            ComponentId::AudioSettings => unimplemented_component(ComponentData::AudioSettings)?,
            ComponentId::ControlsSettings => unimplemented_component(ComponentData::ControlsSettings)?,
            ComponentId::Profile => unimplemented_component(ComponentData::Profile)?,
            ComponentId::ToolInputCombos => unimplemented_component(ComponentData::ToolInputCombos)?,
            ComponentId::ColorPreset => unimplemented_component(ComponentData::ColorPreset)?,
            ComponentId::LocalLeaderboard => unimplemented_component(ComponentData::LocalLeaderboard)?,
            ComponentId::AxisRotationLogic => unimplemented_component(ComponentData::AxisRotationLogic)?,
            ComponentId::ParticleEmitLogic => unimplemented_component(ComponentData::ParticleEmitLogic)?,
            ComponentId::VirusSpiritSpawner => unimplemented_component(ComponentData::VirusSpiritSpawner)?,
            ComponentId::PulseRotateOnTrigger => unimplemented_component(ComponentData::PulseRotateOnTrigger)?,
            ComponentId::TeleporterEntrance => unimplemented_component(ComponentData::TeleporterEntrance)?,
            ComponentId::TeleporterExit => unimplemented_component(ComponentData::TeleporterExit)?,
            ComponentId::ControlScheme => unimplemented_component(ComponentData::ControlScheme)?,
            ComponentId::DeviceToSchemeLinks => unimplemented_component(ComponentData::DeviceToSchemeLinks)?,
            ComponentId::ObjectSpawnCircle => unimplemented_component(ComponentData::ObjectSpawnCircle)?,
            ComponentId::InterpolateToPositionOnTrigger => unimplemented_component(ComponentData::InterpolateToPositionOnTrigger)?,
            ComponentId::EngageBrokenPieces => unimplemented_component(ComponentData::EngageBrokenPieces)?,
            ComponentId::GravityToggle => unimplemented_component(ComponentData::GravityToggle)?,
            ComponentId::CarSpawner => unimplemented_component(ComponentData::CarSpawner)?,
            ComponentId::RaceStartCarSpawner => unimplemented_component(ComponentData::RaceStartCarSpawner)?,
            ComponentId::LevelEditorCarSpawner => unimplemented_component(ComponentData::LevelEditorCarSpawner)?,
            ComponentId::InfoDisplayLogic => unimplemented_component(ComponentData::InfoDisplayLogic)?,
            ComponentId::MusicTrigger => unimplemented_component(ComponentData::MusicTrigger)?,
            ComponentId::TabPopulator => unimplemented_component(ComponentData::TabPopulator)?,
            ComponentId::AdventureAbilitySettings => unimplemented_component(ComponentData::AdventureAbilitySettings)?,
            ComponentId::IndicatorDisplayLogic => unimplemented_component(ComponentData::IndicatorDisplayLogic)?,
            ComponentId::PulseCoreLogic => unimplemented_component(ComponentData::PulseCoreLogic)?,
            ComponentId::PulseAll => unimplemented_component(ComponentData::PulseAll)?,
            ComponentId::TeleporterExitCheckpoint => unimplemented_component(ComponentData::TeleporterExitCheckpoint)?,
            ComponentId::LevelSettings => unimplemented_component(ComponentData::LevelSettings)?,
            ComponentId::WingCorruptionZone => unimplemented_component(ComponentData::WingCorruptionZone)?,
            ComponentId::GenerateCreditsNames => unimplemented_component(ComponentData::GenerateCreditsNames)?,
            ComponentId::IntroCutsceneLightFadeIn => unimplemented_component(ComponentData::IntroCutsceneLightFadeIn)?,
            ComponentId::QuarantineTrigger => unimplemented_component(ComponentData::QuarantineTrigger)?,
            ComponentId::CarScreenTextDecodeTrigger => unimplemented_component(ComponentData::CarScreenTextDecodeTrigger)?,
            ComponentId::GlitchFieldLogic => unimplemented_component(ComponentData::GlitchFieldLogic)?,
            ComponentId::FogSkyboxAmbientChangeTrigger => unimplemented_component(ComponentData::FogSkyboxAmbientChangeTrigger)?,
            ComponentId::FinalCountdownLogic => unimplemented_component(ComponentData::FinalCountdownLogic)?,
            ComponentId::SetActiveOnIntroCutsceneStarted => unimplemented_component(ComponentData::SetActiveOnIntroCutsceneStarted)?,
            ComponentId::RaceEndLogic => unimplemented_component(ComponentData::RaceEndLogic)?,
            ComponentId::EnableAbilitiesTrigger => unimplemented_component(ComponentData::EnableAbilitiesTrigger)?,
            ComponentId::SphericalGravity => unimplemented_component(ComponentData::SphericalGravity)?,
            ComponentId::CreditsNameOrbLogic => unimplemented_component(ComponentData::CreditsNameOrbLogic)?,
            ComponentId::DisableLocalCarWarnings => unimplemented_component(ComponentData::DisableLocalCarWarnings)?,
            ComponentId::CustomName => unimplemented_component(ComponentData::CustomName)?,
            ComponentId::SplineSegment => unimplemented_component(ComponentData::SplineSegment)?,
            ComponentId::WarningPulseLight => unimplemented_component(ComponentData::WarningPulseLight)?,
            ComponentId::RumbleZone => unimplemented_component(ComponentData::RumbleZone)?,
            ComponentId::HideOnVirusSpiritEvent => unimplemented_component(ComponentData::HideOnVirusSpiritEvent)?,
            ComponentId::TrackAttachment => unimplemented_component(ComponentData::TrackAttachment)?,
            ComponentId::LevelPlaylist => unimplemented_component(ComponentData::LevelPlaylist)?,
            ComponentId::ProfileProgress => unimplemented_component(ComponentData::ProfileProgress)?,
            ComponentId::GeneralSettings => unimplemented_component(ComponentData::GeneralSettings)?,
            ComponentId::WorkshopPublishedFileInfos => unimplemented_component(ComponentData::WorkshopPublishedFileInfos)?,
            ComponentId::WarpAnchor => unimplemented_component(ComponentData::WarpAnchor)?,
            ComponentId::SetActiveOnMIDIEvent => unimplemented_component(ComponentData::SetActiveOnMIDIEvent)?,
            ComponentId::TurnLightOnNearCar => unimplemented_component(ComponentData::TurnLightOnNearCar)?,
            ComponentId::Traffic => unimplemented_component(ComponentData::Traffic)?,
            ComponentId::TrackManipulatorNode => unimplemented_component(ComponentData::TrackManipulatorNode)?,
            ComponentId::AudioEventTrigger => unimplemented_component(ComponentData::AudioEventTrigger)?,
            ComponentId::LevelEditorSettings => unimplemented_component(ComponentData::LevelEditorSettings)?,
            ComponentId::EmpireProximityDoorLogic => unimplemented_component(ComponentData::EmpireProximityDoorLogic)?,
            ComponentId::Biodome => unimplemented_component(ComponentData::Biodome)?,
            ComponentId::TunnelHorrorLogic => unimplemented_component(ComponentData::TunnelHorrorLogic)?,
            ComponentId::VirusSpiritWarpTeaserLogic => unimplemented_component(ComponentData::VirusSpiritWarpTeaserLogic)?,
            ComponentId::CarReplayData => unimplemented_component(ComponentData::CarReplayData)?,
            ComponentId::LevelImageCamera => unimplemented_component(ComponentData::LevelImageCamera)?,
            ComponentId::ParticlesGPU => unimplemented_component(ComponentData::ParticlesGPU)?,
            ComponentId::KillGridBox => unimplemented_component(ComponentData::KillGridBox)?,
            ComponentId::GoldenSimples => {
                implemented_component(self, ComponentData::GoldenSimples, is_default_component, version, guid)?
            }
            ComponentId::SetActiveAfterWarp => unimplemented_component(ComponentData::SetActiveAfterWarp)?,
            ComponentId::AmbientAudioObject => unimplemented_component(ComponentData::AmbientAudioObject)?,
            ComponentId::BiodomeAudioInterpolator => unimplemented_component(ComponentData::BiodomeAudioInterpolator)?,
            ComponentId::MoveElectricityAlongWire => unimplemented_component(ComponentData::MoveElectricityAlongWire)?,
            ComponentId::ActivationRampLogic => unimplemented_component(ComponentData::ActivationRampLogic)?,
            ComponentId::ZEventTrigger => unimplemented_component(ComponentData::ZEventTrigger)?,
            ComponentId::ZEventListener => unimplemented_component(ComponentData::ZEventListener)?,
            ComponentId::BlackPortalLogic => unimplemented_component(ComponentData::BlackPortalLogic)?,
            ComponentId::VRSettings => unimplemented_component(ComponentData::VRSettings)?,
            ComponentId::CutsceneCamera => unimplemented_component(ComponentData::CutsceneCamera)?,
            ComponentId::ProfileStats => unimplemented_component(ComponentData::ProfileStats)?,
            ComponentId::InterpolateToRotationOnTrigger => unimplemented_component(ComponentData::InterpolateToRotationOnTrigger)?,
            ComponentId::MoveAlongAttachedTrack => unimplemented_component(ComponentData::MoveAlongAttachedTrack)?,
            ComponentId::ShowDuringGlitch => unimplemented_component(ComponentData::ShowDuringGlitch)?,
            ComponentId::AddCameraNoise => unimplemented_component(ComponentData::AddCameraNoise)?,
            ComponentId::CarVoiceTrigger => unimplemented_component(ComponentData::CarVoiceTrigger)?,
            ComponentId::HoverScreenSpecialObjectTrigger => unimplemented_component(ComponentData::HoverScreenSpecialObjectTrigger)?,
            ComponentId::ReplaySettings => unimplemented_component(ComponentData::ReplaySettings)?,
            ComponentId::CutsceneCamForTrailer => unimplemented_component(ComponentData::CutsceneCamForTrailer)?,
            ComponentId::LevelInfos => unimplemented_component(ComponentData::LevelInfos)?,
            ComponentId::AchievementTrigger => unimplemented_component(ComponentData::AchievementTrigger)?,
            ComponentId::ArenaCarSpawner => unimplemented_component(ComponentData::ArenaCarSpawner)?,
            ComponentId::Animated => unimplemented_component(ComponentData::Animated)?,
            ComponentId::BlinkInTrigger => unimplemented_component(ComponentData::BlinkInTrigger)?,
            ComponentId::CarScreenImageTrigger => unimplemented_component(ComponentData::CarScreenImageTrigger)?,
            ComponentId::ExcludeFromEMP => unimplemented_component(ComponentData::ExcludeFromEMP)?,
            ComponentId::InfiniteCooldownTrigger => unimplemented_component(ComponentData::InfiniteCooldownTrigger)?,
            ComponentId::DiscoverableStuntArea => unimplemented_component(ComponentData::DiscoverableStuntArea)?,
            ComponentId::ForceVolume => unimplemented_component(ComponentData::ForceVolume)?,
            ComponentId::AdventureModeCompleteTrigger => unimplemented_component(ComponentData::AdventureModeCompleteTrigger)?,
            ComponentId::CountdownTextMeshLogic => unimplemented_component(ComponentData::CountdownTextMeshLogic)?,
            ComponentId::AbilitySignButtonColorLogic => unimplemented_component(ComponentData::AbilitySignButtonColorLogic)?,
            ComponentId::GoldenAnimator => unimplemented_component(ComponentData::GoldenAnimator)?,
            ComponentId::AnimatorAudio => unimplemented_component(ComponentData::AnimatorAudio)?,
            ComponentId::AnimatorCameraShake => unimplemented_component(ComponentData::AnimatorCameraShake)?,
            ComponentId::ShardCluster => unimplemented_component(ComponentData::ShardCluster)?,
            ComponentId::AdventureSpecialIntro => unimplemented_component(ComponentData::AdventureSpecialIntro)?,
            ComponentId::AudioEffectZone => unimplemented_component(ComponentData::AudioEffectZone)?,
            ComponentId::CinematicCamera => unimplemented_component(ComponentData::CinematicCamera)?,
            ComponentId::CinematicCameraFocalPoint => unimplemented_component(ComponentData::CinematicCameraFocalPoint)?,
            ComponentId::SetAbilitiesTrigger => unimplemented_component(ComponentData::SetAbilitiesTrigger)?,
            ComponentId::LostToEchoesIntroCutscene => unimplemented_component(ComponentData::LostToEchoesIntroCutscene)?,
            ComponentId::CutsceneText => unimplemented_component(ComponentData::CutsceneText)?,
            ComponentId::UltraPlanet => unimplemented_component(ComponentData::UltraPlanet)?,
            ComponentId::DeadCarLogic => unimplemented_component(ComponentData::DeadCarLogic)?,
            ComponentId::RollingBarrelDropperLogic => unimplemented_component(ComponentData::RollingBarrelDropperLogic)?,
            ComponentId::AdventureFinishTrigger => unimplemented_component(ComponentData::AdventureFinishTrigger)?,
            ComponentId::AchievementSettings => unimplemented_component(ComponentData::AchievementSettings)?,
            ComponentId::InterpolateRTPCLogic => unimplemented_component(ComponentData::InterpolateRTPCLogic)?,
            ComponentId::TriggerCooldownLogic => unimplemented_component(ComponentData::TriggerCooldownLogic)?,
            ComponentId::ShadowsChangedListener => unimplemented_component(ComponentData::ShadowsChangedListener)?,
            ComponentId::LookAtCamera => unimplemented_component(ComponentData::LookAtCamera)?,
            ComponentId::CubeMapRenderer => unimplemented_component(ComponentData::CubeMapRenderer)?,
            ComponentId::RealtimeReflectionRenderer => unimplemented_component(ComponentData::RealtimeReflectionRenderer)?,
            ComponentId::VirusDropperDroneLogic => unimplemented_component(ComponentData::VirusDropperDroneLogic)?,
            ComponentId::OnCollisionBreakApartLogic => unimplemented_component(ComponentData::OnCollisionBreakApartLogic)?,
            ComponentId::CheatSettings => unimplemented_component(ComponentData::CheatSettings)?,
            ComponentId::IgnoreInCullGroups => unimplemented_component(ComponentData::IgnoreInCullGroups)?,
            ComponentId::IgnoreInputTrigger => unimplemented_component(ComponentData::IgnoreInputTrigger)?,
            ComponentId::PowerPosterLogic => unimplemented_component(ComponentData::PowerPosterLogic)?,
            ComponentId::MusicZone => unimplemented_component(ComponentData::MusicZone)?,
            ComponentId::LightsFlickerLogic => unimplemented_component(ComponentData::LightsFlickerLogic)?,
            ComponentId::CutsceneManagerLogic => unimplemented_component(ComponentData::CutsceneManagerLogic)?,
            ComponentId::FadeOut => unimplemented_component(ComponentData::FadeOut)?,
            ComponentId::Flock => unimplemented_component(ComponentData::Flock)?,
            ComponentId::GPSTrigger => unimplemented_component(ComponentData::GPSTrigger)?,
            ComponentId::SprintMode => unimplemented_component(ComponentData::SprintMode)?,
            ComponentId::StuntMode => unimplemented_component(ComponentData::StuntMode)?,
            ComponentId::SoccerMode => unimplemented_component(ComponentData::SoccerMode)?,
            ComponentId::FreeRoamMode => unimplemented_component(ComponentData::FreeRoamMode)?,
            ComponentId::ReverseTagMode => unimplemented_component(ComponentData::ReverseTagMode)?,
            ComponentId::LevelEditorPlayMode => unimplemented_component(ComponentData::LevelEditorPlayMode)?,
            ComponentId::CoopSprintMode => unimplemented_component(ComponentData::CoopSprintMode)?,
            ComponentId::ChallengeMode => unimplemented_component(ComponentData::ChallengeMode)?,
            ComponentId::AdventureMode => unimplemented_component(ComponentData::AdventureMode)?,
            ComponentId::SpeedAndStyleMode => unimplemented_component(ComponentData::SpeedAndStyleMode)?,
            ComponentId::TrackmogrifyMode => unimplemented_component(ComponentData::TrackmogrifyMode)?,
            ComponentId::DemoMode => unimplemented_component(ComponentData::DemoMode)?,
            ComponentId::MainMenuMode => unimplemented_component(ComponentData::MainMenuMode)?,
            ComponentId::LostToEchoesMode => unimplemented_component(ComponentData::LostToEchoesMode)?,
            ComponentId::NexusMode => unimplemented_component(ComponentData::NexusMode)?,
            ComponentId::TheOtherSideMode => unimplemented_component(ComponentData::TheOtherSideMode)?,
            _ => bail!("unserializable component `{:?}` encountered", component_id),
        };

        Ok(component)
    }

    fn check_and_adjust_for_scope_bounds<NextElement>(&mut self) -> Result<bool, Error> {
        let scope_info = match self.scope_info_stack.last() {
            Some(info) => info,
            None => {
                return Ok(false);
            }
        };

        let stream_position = self.reader.stream_position()?;
        let size_of_next_element: u64 = mem::size_of::<NextElement>().try_into()?;
        let scope_end: u64 = scope_info.end_pos.try_into()?;
        if stream_position + size_of_next_element > scope_end {
            self.reader.seek(SeekFrom::Start(scope_end))?;

            return Ok(false);
        }

        Ok(true)
    }

    fn empty_marker(&mut self) -> Result<bool, Error> {
        const MARK_SIZE: usize = mem::size_of::<i32>();

        let mut buf = [0_u8; 4];
        if let Err(e) = self.reader.read_exact(&mut buf) {
            return if e.kind() == io::ErrorKind::UnexpectedEof {
                Ok(false)
            } else {
                Err(e.into())
            };
        }

        let n = i32::from_le_bytes(buf);
        if n == EMPTY_MARK {
            Ok(true)
        } else {
            self.reader.seek(SeekFrom::Current(-(MARK_SIZE as i64)))?;
            Ok(false)
        }
    }

    fn is_empty_scope(&mut self) -> Result<bool, Error> {
        if let Some(scope_info) = self.scope_info_stack.last() {
            Ok(self.reader.stream_position()? == u64::try_from(scope_info.end_pos)?)
        } else {
            warn!("ScopeInfo stack was empty when accessed");

            Ok(true)
        }
    }

    fn read_set_string(&mut self, _name: &str, val: &mut String) -> Result<(), Error> {
        *val = string::read(&mut self.reader)?;

        Ok(())
    }

    fn read_game_object_start(
        &mut self,
        push_in_scope_stack: bool,
    ) -> Result<(String, u32), Error> {
        let mut name = String::new();
        let mut guid = 0;
        self.read_start_scope_with_mark(66666666, push_in_scope_stack)?;
        self.read_set_string("GameObject", &mut name)?;
        self.set_current_scope_name(format!("GO:{}", &name));
        self.read_set_string("Prefab", &mut String::new())?;
        self.read_set_u32("guid", &mut guid)?;

        Ok((name, guid))
    }

    fn read_start_scope(&mut self, push_in_scope_stack: bool) -> Result<i32, Error> {
        let mark = self.reader.read_i32::<LE>()?;
        self.read_start_scope_helper(mark, push_in_scope_stack)?;

        Ok(mark)
    }

    fn read_start_scope_with_mark(
        &mut self,
        mark: i32,
        push_in_scope_stack: bool,
    ) -> Result<(), Error> {
        let n = self.reader.read_i32::<LE>()?;
        if n == mark {
            self.read_start_scope_helper(mark, push_in_scope_stack)?;
        } else {
            warn!(
                expected_mark = mark,
                expected_mark_name = util::scope_mark_string(mark),
                found = n,
                "Expected mark wasn't found. Stack: {:?}",
                &self.scope_info_stack
            );
        }

        Ok(())
    }

    fn read_start_scope_helper(
        &mut self,
        mark: i32,
        push_in_scope_stack: bool,
    ) -> Result<(), Error> {
        let scope_len: usize = self.reader.read_i64::<LE>()?.try_into()?;
        if push_in_scope_stack {
            let start = self.reader.stream_position()?.try_into()?;
            let end = start + scope_len;
            let new_scope_info = ScopeInfo::new(mark, start, end);
            self.scope_info_stack.push(new_scope_info);
        }

        Ok(())
    }

    fn read_end_scope(&mut self, log_warn: bool) -> Result<(), Error> {
        if let Some(scope_info) = self.scope_info_stack.pop() {
            self.read_end_scope_helper(&scope_info, log_warn)?;
        } else {
            warn!("ScopeInfo stack was empty when accessed");
        }

        Ok(())
    }

    fn read_end_scope_helper(
        &mut self,
        scope_info: &ScopeInfo,
        log_warn: bool,
    ) -> Result<(), Error> {
        let actual_pos = self.reader.stream_position()?;
        let info_pos: u64 = scope_info.end_pos.try_into()?;
        let str_1 = match actual_pos.cmp(&info_pos) {
            Ordering::Less => "understepped",
            Ordering::Equal => {
                return Ok(());
            }
            Ordering::Greater => "overstepped",
        };

        if log_warn {
            warn!(
                scope = scope_info.scope_mark_string(),
                "A scope was {} when reading. Stack: {:?}", str_1, &self.scope_info_stack
            );
        }

        self.reader.seek(SeekFrom::Start(info_pos))?;

        Ok(())
    }

    fn set_current_scope_name(&mut self, name: impl Into<Cow<'static, str>>) {
        if let Some(scope_info) = self.scope_info_stack.last_mut() {
            scope_info.name = name.into();
        }
    }

    fn read_set_u8(&mut self, _name: &str, val: &mut u8) -> Result<(), Error> {
        if self.check_and_adjust_for_scope_bounds::<u8>()? {
            *val = self.reader.read_u8()?;
        }

        Ok(())
    }
}

impl<R: Read + Seek> Visitor for Deserializer<R> {
    const VISIT_DIRECTION: VisitDirection = VisitDirection::In;

    fn visit_bool(&mut self, name: &str, value: &mut bool) -> Result<(), Error> {
        if !self.empty_marker()? {
            let mut n = *value as u8;
            self.read_set_u8(name, &mut n)?;
            *value = n != 0;
        }

        Ok(())
    }

    fn visit_i32(&mut self, name: &str, value: &mut i32) -> Result<(), Error> {
        if !self.empty_marker()? {
            self.read_set_i32(name, value)?;
        }

        Ok(())
    }

    fn visit_u32(&mut self, name: &str, value: &mut u32) -> Result<(), Error> {
        if !self.empty_marker()? {
            self.read_set_u32(name, value)?;
        }

        Ok(())
    }

    fn visit_i64(&mut self, name: &str, value: &mut i64) -> Result<(), Error> {
        if !self.empty_marker()? {
            self.read_set_i64(name, value)?;
        }

        Ok(())
    }

    fn visit_f32(&mut self, name: &str, value: &mut f32) -> Result<(), Error> {
        if !self.empty_marker()? {
            self.read_set_f32(name, value)?;
        }

        Ok(())
    }

    fn visit_vector_3(&mut self, _name: &str, value: &mut Vector3) -> Result<(), Error> {
        if !self.empty_marker()? {
            self.read_set_f32("x", &mut value.x)?;
            self.read_set_f32("y", &mut value.y)?;
            self.read_set_f32("z", &mut value.z)?;
        }

        Ok(())
    }

    fn visit_quaternion(&mut self, _name: &str, value: &mut Quaternion) -> Result<(), Error> {
        if !self.empty_marker()? {
            self.read_set_f32("x", &mut value.v.x)?;
            self.read_set_f32("y", &mut value.v.y)?;
            self.read_set_f32("z", &mut value.v.z)?;
            self.read_set_f32("w", &mut value.s)?;
        }

        Ok(())
    }

    fn visit_children(&mut self, value: &mut Vec<GameObject>) -> Result<(), Error> {
        self.read_start_scope_with_mark(55555555, true)?;
        let mut num_children = 0;
        self.read_set_i32("numberOfChildren", &mut num_children)?;
        self.set_current_scope_name(format!("ChildNum:{}", num_children));
        for _ in 0..num_children {
            let child = self.read_game_object()?;
            value.push(child);
        }

        Ok(())
    }
}

macro_rules! impl_read_set {
    ($type_:ty) => {
        impl<R: Read + Seek> Deserializer<R> {
            paste! {
                fn [<read_set_ $type_>](&mut self, _name: &str, val: &mut $type_) -> Result<(), Error> {
                    if self.check_and_adjust_for_scope_bounds::<$type_>()? {
                        *val = self.reader.[<read_ $type_>]::<LE>()?;
                    }

                    Ok(())
                }
            }
        }
    };
}

impl_read_set!(i32);
impl_read_set!(u32);
impl_read_set!(i64);
impl_read_set!(f32);

#[derive(Debug, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct ScopeInfo {
    name: Cow<'static, str>,
    scope_mark: i32,
    start_pos: usize,
    end_pos: usize,
}

impl ScopeInfo {
    pub fn new(scope_mark: i32, start_pos: usize, end_pos: usize) -> Self {
        ScopeInfo {
            name: "".into(),
            scope_mark,
            start_pos,
            end_pos,
        }
    }

    pub fn scope_mark_string(&self) -> &'static str {
        util::scope_mark_string(self.scope_mark)
    }
}

impl Display for ScopeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.name, self.scope_mark_string())
    }
}
