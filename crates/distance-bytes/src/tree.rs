//! The structural tree of a `.bytes` file: levels, layers, game objects and components.
//!
//! Component bodies other than `Transform` are kept as raw bytes for now.

use std::cell::RefCell;

use crate::component_id::ComponentId;
use crate::error::{ErrorKind, Result};
use crate::io::{Reader, TraceEvent, Writer, mark};
use crate::string::DString;

/// The root of a `.bytes` file.
#[derive(Debug, Clone, PartialEq)]
pub enum BytesFile {
    Level(Level),
    /// Non-level files (custom objects, profiles, replays, leaderboards, ...) are a single
    /// game object.
    GameObject(GameObject),
}

impl BytesFile {
    pub fn read(data: &[u8]) -> Result<Self> {
        Self::read_from(Reader::new(data))
    }

    /// Like [`BytesFile::read`], but also returns every value that was decoded, in read order.
    pub fn read_traced(data: &[u8]) -> (Result<Self>, Vec<TraceEvent>) {
        let trace = RefCell::new(Vec::new());
        let result = Self::read_from(Reader::with_trace(data, &trace));
        (result, trace.into_inner())
    }

    fn read_from(mut r: Reader) -> Result<Self> {
        let file = match r.peek_i32() {
            Some(mark::LEVEL) => BytesFile::Level(Level::read(&mut r)?),
            Some(mark::GAME_OBJECT) => BytesFile::GameObject(GameObject::read(&mut r)?),
            Some(other) => return Err(r.error(ErrorKind::UnknownRoot(other))),
            None => return Err(r.error(ErrorKind::UnexpectedEnd { needed: 4 })),
        };
        r.expect_end()?;
        Ok(file)
    }

    pub fn write(&self) -> Vec<u8> {
        let mut w = Writer::new();
        match self {
            BytesFile::Level(level) => level.write(&mut w),
            BytesFile::GameObject(obj) => obj.write(&mut w),
        }
        w.finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Level {
    pub name: DString,
    /// The level file version, 0 to 4. The game writes version 4.
    pub version: i32,
    pub settings: LevelSettings,
    pub layers: Vec<Layer>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LevelSettings {
    /// Level version 2 and later: a game object with a `LevelSettings` component.
    GameObject(GameObject),
    /// Level versions 0 and 1.
    Legacy { version: i32, data: Vec<u8> },
}

impl Level {
    pub fn read(r: &mut Reader) -> Result<Self> {
        let (_, mut s) = r.scope(&[mark::LEVEL])?;
        let name = s.string()?;
        let layer_count = s.count()?;
        let version = s.i32()?;
        let settings = if version >= 2 {
            LevelSettings::GameObject(GameObject::read(&mut s)?)
        } else {
            let (_, mut ls) = s.scope(&[mark::LEGACY_LEVEL_SETTINGS])?;
            let version = ls.i32()?;
            LevelSettings::Legacy { version, data: ls.rest().to_vec() }
        };
        let layers = (0..layer_count)
            .map(|_| Layer::read(&mut s, version))
            .collect::<Result<_>>()?;
        s.expect_end()?;
        Ok(Level { name, version, settings, layers })
    }

    pub fn write(&self, w: &mut Writer) {
        w.scope(mark::LEVEL, |w| {
            w.string(&self.name);
            w.count(self.layers.len());
            w.i32(self.version);
            match &self.settings {
                LevelSettings::GameObject(obj) => obj.write(w),
                LevelSettings::Legacy { version, data } => w.scope(mark::LEGACY_LEVEL_SETTINGS, |w| {
                    w.i32(*version);
                    w.bytes(data);
                }),
            }
            for layer in &self.layers {
                layer.write(w, self.version);
            }
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    pub name: DString,
    pub flags: LayerFlags,
    pub objects: Vec<GameObject>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerFlags {
    /// Levels before version 3 have no layer flags.
    Absent,
    /// Layer version 0.
    V0 { unfrozen: bool, active: bool, visible: bool },
    /// Layer version 1 (the game reads any version >= 1 this way).
    V1 { version: i32, active: bool, frozen: bool, visible: bool, is_background: bool },
    /// A negative layer version: no flags follow.
    None { version: i32 },
}

impl LayerFlags {
    /// The flags of a new layer in the game.
    pub const DEFAULT: LayerFlags =
        LayerFlags::V1 { version: 1, active: false, frozen: false, visible: true, is_background: false };
}

impl Layer {
    fn read(r: &mut Reader, level_version: i32) -> Result<Self> {
        let (_, mut s) = r.scope(&[mark::LAYER])?;
        let name = s.string()?;
        let object_count = s.count()?;
        let flags = if level_version < 3 {
            LayerFlags::Absent
        } else {
            match s.i32()? {
                0 => LayerFlags::V0 { unfrozen: s.bool()?, active: s.bool()?, visible: s.bool()? },
                v @ 1.. => LayerFlags::V1 {
                    version: v,
                    active: s.bool()?,
                    frozen: s.bool()?,
                    visible: s.bool()?,
                    is_background: s.bool()?,
                },
                v => LayerFlags::None { version: v },
            }
        };
        let objects = (0..object_count)
            .map(|_| GameObject::read(&mut s))
            .collect::<Result<_>>()?;
        s.expect_end()?;
        Ok(Layer { name, flags, objects })
    }

    fn write(&self, w: &mut Writer, level_version: i32) {
        w.scope(mark::LAYER, |w| {
            w.string(&self.name);
            w.count(self.objects.len());
            if level_version >= 3 {
                // Levels before version 3 have no flags; if the level was upgraded without
                // setting them, write the game's defaults.
                let flags = match self.flags {
                    LayerFlags::Absent => LayerFlags::DEFAULT,
                    flags => flags,
                };
                match flags {
                    LayerFlags::Absent => unreachable!(),
                    LayerFlags::V0 { unfrozen, active, visible } => {
                        w.i32(0);
                        for b in [unfrozen, active, visible] {
                            w.u8(b.into());
                        }
                    }
                    LayerFlags::V1 { version, active, frozen, visible, is_background } => {
                        w.i32(version);
                        for b in [active, frozen, visible, is_background] {
                            w.u8(b.into());
                        }
                    }
                    LayerFlags::None { version } => w.i32(version),
                }
            }
            for obj in &self.objects {
                obj.write(w);
            }
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameObject {
    /// The prefab name, which is also the object's name.
    pub name: DString,
    /// Always written empty by the game.
    pub prefab: DString,
    /// Unique ID used for references between objects.
    pub guid: u32,
    pub components: Vec<Component>,
}

impl GameObject {
    pub fn read(r: &mut Reader) -> Result<Self> {
        let (_, mut s) = r.scope(&[mark::GAME_OBJECT])?;
        let name = s.string()?;
        let prefab = s.string()?;
        let guid = s.u32()?;
        let count = s.count()?;
        let components = (0..count).map(|_| Component::read(&mut s)).collect::<Result<_>>()?;
        s.expect_end()?;
        Ok(GameObject { name, prefab, guid, components })
    }

    pub fn write(&self, w: &mut Writer) {
        w.scope(mark::GAME_OBJECT, |w| {
            w.string(&self.name);
            w.string(&self.prefab);
            w.u32(self.guid);
            w.count(self.components.len());
            for c in &self.components {
                c.write(w);
            }
        });
    }

    /// The object's children, if it has a `Transform` with a decoded body.
    pub fn children(&self) -> &[GameObject] {
        self.components
            .iter()
            .find_map(|c| match &c.body {
                ComponentBody::Transform(t) => Some(&t.children[..]),
                _ => None,
            })
            .unwrap_or(&[])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    pub kind: ComponentKind,
    /// Unique ID used for references.
    pub guid: u32,
    pub body: ComponentBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentKind {
    Known {
        /// One of [`mark::SERIALIZABLE`], [`mark::SERIAL_COMPONENT`] or
        /// [`mark::SERIALIZABLE_ALT`].
        mark: i32,
        id: ComponentId,
        version: i32,
    },
    /// A component the game had no serializer for; only its type name is stored.
    Unknown { name: DString },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentBody {
    /// No data: the component is identical to the prefab's.
    Empty,
    Transform(Transform),
    /// Not decoded yet.
    Raw(Vec<u8>),
}

/// The body of a `Transform` component. `None` values are the same as the prefab's.
#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    pub position: Option<[f32; 3]>,
    /// Quaternion as `[x, y, z, w]`.
    pub rotation: Option<[f32; 4]>,
    pub scale: Option<[f32; 3]>,
    pub children: Vec<GameObject>,
}

impl Component {
    fn read(r: &mut Reader) -> Result<Self> {
        let (scope_mark, mut s) = r.scope(&[
            mark::SERIALIZABLE,
            mark::SERIAL_COMPONENT,
            mark::SERIALIZABLE_ALT,
            mark::UNKNOWN_COMPONENT,
        ])?;
        let kind = if scope_mark == mark::UNKNOWN_COMPONENT {
            ComponentKind::Unknown { name: s.string()? }
        } else {
            let id = ComponentId(s.i32()?);
            let version = s.i32()?;
            ComponentKind::Known { mark: scope_mark, id, version }
        };
        let guid = s.u32()?;
        let body = if s.is_empty() {
            ComponentBody::Empty
        } else if matches!(kind, ComponentKind::Known { id: ComponentId::TRANSFORM, .. }) {
            // Fall back to raw bytes if the body doesn't parse as a transform.
            match s.attempt(Transform::read) {
                Ok(transform) => ComponentBody::Transform(transform),
                Err(_) => ComponentBody::Raw(s.rest().to_vec()),
            }
        } else {
            ComponentBody::Raw(s.rest().to_vec())
        };
        Ok(Component { kind, guid, body })
    }

    fn write(&self, w: &mut Writer) {
        let scope_mark = match self.kind {
            ComponentKind::Known { mark, .. } => mark,
            ComponentKind::Unknown { .. } => mark::UNKNOWN_COMPONENT,
        };
        w.scope(scope_mark, |w| {
            match &self.kind {
                ComponentKind::Known { id, version, .. } => {
                    w.i32(id.0);
                    w.i32(*version);
                }
                ComponentKind::Unknown { name } => w.string(name),
            }
            w.u32(self.guid);
            match &self.body {
                ComponentBody::Empty => {}
                ComponentBody::Transform(t) => t.write(w),
                ComponentBody::Raw(bytes) => w.bytes(bytes),
            }
        });
    }

    pub fn id(&self) -> Option<ComponentId> {
        match self.kind {
            ComponentKind::Known { id, .. } => Some(id),
            ComponentKind::Unknown { .. } => None,
        }
    }
}

impl Transform {
    fn read(r: &mut Reader) -> Result<Self> {
        let position = r.optional(Reader::f32s::<3>)?;
        let rotation = r.optional(Reader::f32s::<4>)?;
        let scale = r.optional(Reader::f32s::<3>)?;
        let (_, mut s) = r.scope(&[mark::CHILDREN])?;
        let count = s.count()?;
        let children = (0..count).map(|_| GameObject::read(&mut s)).collect::<Result<_>>()?;
        s.expect_end()?;
        r.expect_end()?;
        Ok(Transform { position, rotation, scale, children })
    }

    fn write(&self, w: &mut Writer) {
        w.optional(self.position.as_ref(), |w, v| w.f32s(v));
        w.optional(self.rotation.as_ref(), |w, v| w.f32s(v));
        w.optional(self.scale.as_ref(), |w, v| w.f32s(v));
        w.scope(mark::CHILDREN, |w| {
            w.count(self.children.len());
            for child in &self.children {
                child.write(w);
            }
        });
    }
}
