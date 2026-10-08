//! Building files: new levels, objects and components, GUIDs, and checks of references.
//!
//! ```
//! use distance_bytes::components::TeleporterEntrance;
//! use distance_bytes::{BytesFile, GameObject, Level};
//!
//! let mut level = Level::new("My level");
//! let teleporter = GameObject::new("Teleporter")
//!     .with_position([0.0, 10.0, 0.0])
//!     .with(TeleporterEntrance {
//!         teleports_to: Some(1),
//!         ..Default::default()
//!     });
//! level.layers[0].objects.push(teleporter);
//! level.assign_guids();
//! assert!(level.check().is_empty());
//! let bytes = BytesFile::Level(level).write();
//! ```
//!
//! Every object and component has a GUID, unique in the file, by which components refer to
//! objects and components. New ones have GUID 0 until [`Level::assign_guids`] gives them one.
//! Objects are created from a prefab by name, and are the same as the prefab except for the
//! components they list.

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::component_id::ComponentId;
use crate::components::{ComponentData, LevelSettings};
use crate::fields::{Field, FieldValue, Value, fields_of};
use crate::string::DString;
use crate::tree::{
    BytesFile, Component, ComponentBody, ComponentKind, GameObject, Layer, LayerFlags, Level,
    LevelSettingsData, Transform,
};
use crate::visit::{BodyEnd, ReferenceVisitor};

impl Level {
    /// A new level as the game's editor starts one: a `LevelSettings` object with the level's
    /// name and otherwise the prefab's settings, and an empty `Default` layer. The game enables
    /// Sprint mode when no modes are set. A level the game can play also needs objects such as
    /// a `LevelEditorCarSpawner` and a finish.
    pub fn new(name: impl Into<DString>) -> Self {
        let name = name.into();
        let settings = LevelSettings {
            level_name: Some(name.clone()),
            ..LevelSettings::default()
        };
        let settings = GameObject::new("LevelSettings")
            .with(Component::empty(ComponentId::TRANSFORM))
            .with(settings);
        let mut layer = Layer::new("Default");
        layer.flags = LayerFlags::V1 {
            version: 1,
            active: true,
            frozen: false,
            visible: true,
            is_background: false,
        };
        Level {
            name,
            version: Level::LATEST_VERSION,
            settings: LevelSettingsData::GameObject(settings),
            layers: vec![layer],
        }
    }

    /// The largest GUID of an object or component, or 0 if there are none.
    pub fn max_guid(&self) -> u32 {
        max_guid(level_roots(self))
    }

    /// Gives every object and component whose GUID is 0 a new one, counting up from the largest
    /// in the level, and returns how many it gave. An object comes before its components, and
    /// they come before its children.
    pub fn assign_guids(&mut self) -> usize {
        let max = self.max_guid();
        assign_guids(level_roots_mut(self), max)
    }

    /// Problems with the GUIDs and references of the level, which make the game lose links
    /// between objects or fail to load the level.
    pub fn check(&self) -> Vec<Problem> {
        check(level_roots(self))
    }
}

impl BytesFile {
    /// The largest GUID of an object or component, or 0 if there are none.
    pub fn max_guid(&self) -> u32 {
        match self {
            BytesFile::Level(level) => level.max_guid(),
            BytesFile::GameObject(obj) => max_guid(vec![obj]),
        }
    }

    /// See [`Level::assign_guids`].
    pub fn assign_guids(&mut self) -> usize {
        match self {
            BytesFile::Level(level) => level.assign_guids(),
            BytesFile::GameObject(obj) => {
                let max = max_guid(vec![obj]);
                assign_guids(vec![obj], max)
            }
        }
    }

    /// See [`Level::check`].
    pub fn check(&self) -> Vec<Problem> {
        match self {
            BytesFile::Level(level) => level.check(),
            BytesFile::GameObject(obj) => check(vec![obj]),
        }
    }
}

impl Layer {
    /// A new, empty layer with the game's flags for a new layer.
    pub fn new(name: impl Into<DString>) -> Self {
        Layer {
            name: name.into(),
            flags: LayerFlags::DEFAULT,
            objects: Vec::new(),
        }
    }
}

impl GameObject {
    /// A new object of the named prefab, such as `CubeGS`, which is the same as the prefab
    /// until components are added. Its GUID is 0.
    pub fn new(prefab: impl Into<DString>) -> Self {
        GameObject {
            name: prefab.into(),
            prefab: DString::default(),
            guid: 0,
            components: Vec::new(),
        }
    }

    /// Adds a component.
    pub fn with(mut self, component: impl Into<Component>) -> Self {
        self.components.push(component.into());
        self
    }

    /// Sets the position, relative to the parent.
    pub fn with_position(mut self, position: [f32; 3]) -> Self {
        self.transform_mut().position = Some(position);
        self
    }

    /// Sets the rotation, relative to the parent, as a quaternion `[x, y, z, w]`.
    pub fn with_rotation(mut self, rotation: [f32; 4]) -> Self {
        self.transform_mut().rotation = Some(rotation);
        self
    }

    /// Sets the scale, relative to the parent.
    pub fn with_scale(mut self, scale: [f32; 3]) -> Self {
        self.transform_mut().scale = Some(scale);
        self
    }

    /// Adds a child, positioned relative to the object.
    pub fn with_child(mut self, child: GameObject) -> Self {
        self.transform_mut().children.push(child);
        self
    }

    /// The object's `Transform` body. A `Transform` is added if the object has none, and a body
    /// that is the prefab's (or that didn't decode) becomes one with no values set.
    pub fn transform_mut(&mut self) -> &mut Transform {
        let i = match self
            .components
            .iter()
            .position(|c| c.id() == Some(ComponentId::TRANSFORM))
        {
            Some(i) => i,
            None => {
                // The game writes the transform first.
                let transform = Component::transform(Transform::default());
                self.components.insert(0, transform);
                0
            }
        };
        let body = &mut self.components[i].body;
        if !matches!(body, ComponentBody::Transform(_)) {
            *body = ComponentBody::Transform(Transform::default());
        }
        match body {
            ComponentBody::Transform(transform) => transform,
            _ => unreachable!(),
        }
    }

    /// The first component of a type.
    pub fn component(&self, id: ComponentId) -> Option<&Component> {
        self.components.iter().find(|c| c.id() == Some(id))
    }

    /// The first component of a type.
    pub fn component_mut(&mut self, id: ComponentId) -> Option<&mut Component> {
        self.components.iter_mut().find(|c| c.id() == Some(id))
    }
}

impl Component {
    /// A new component with a typed body, in the version the game writes. Its GUID is 0.
    pub fn new(data: impl Into<ComponentData>) -> Self {
        let data = data.into();
        let id = data.id();
        Component {
            kind: ComponentKind::Known {
                mark: id.written_mark(),
                id,
                version: data.latest_version(),
            },
            guid: 0,
            body: ComponentBody::Data {
                data,
                end: BodyEnd::Complete,
            },
        }
    }

    /// A new component that is the same as the prefab's, in the version the game writes. Its
    /// GUID is 0.
    pub fn empty(id: ComponentId) -> Self {
        Component {
            kind: ComponentKind::Known {
                mark: id.written_mark(),
                id,
                version: id.written_version().unwrap_or(0),
            },
            guid: 0,
            body: ComponentBody::Empty,
        }
    }

    /// A new `Transform`. Its GUID is 0.
    pub fn transform(transform: Transform) -> Self {
        Component {
            body: ComponentBody::Transform(transform),
            ..Component::empty(ComponentId::TRANSFORM)
        }
    }

    /// A new component in the game's generic format, in the version the game writes, or `None`
    /// if the game doesn't write the type in that format. Its GUID is 0.
    pub fn fields(id: ComponentId, fields: Vec<Field>) -> Option<Self> {
        let version = id.written_version()?;
        if id == ComponentId::TRANSFORM
            || ComponentData::decodes(id, version)
            || fields_of(id).is_none()
        {
            return None;
        }
        Some(Component {
            body: ComponentBody::Fields(fields),
            ..Component::empty(id)
        })
    }

    /// The component's references to other objects and components by GUID, with their names
    /// in the game, leaving out those that are 0 (none). These are the references of a typed
    /// body in its version, and reference values in the generic format.
    pub fn references(&self) -> Vec<(String, u32)> {
        let mut references = Vec::new();
        match &self.body {
            ComponentBody::Data { data, .. } => {
                let mut v = ReferenceVisitor {
                    f: |name: &'static str, guid: &mut u32| references.push((name.into(), *guid)),
                    found: false,
                };
                // Visit a copy, as visiting can normalize values the way writing does.
                let _ = data.clone().visit(&mut v, self.data_version());
            }
            ComponentBody::Fields(fields) => {
                for field in fields {
                    if let FieldValue::Value(value) = &field.value {
                        let mut value = value.clone();
                        value_references(&mut value, &mut |guid| {
                            references.push((field.name.to_string(), *guid))
                        });
                    }
                }
            }
            _ => {}
        }
        references.retain(|&(_, guid)| guid != 0);
        references
    }

    /// Calls `f` on every reference, including those that are 0, which it may change, such as
    /// to renumber GUIDs.
    pub fn references_mut(&mut self, mut f: impl FnMut(&mut u32)) {
        let version = self.data_version();
        match &mut self.body {
            ComponentBody::Data { data, .. } => {
                // Visiting can normalize values the way writing does, so keep the visited copy
                // only if it has references.
                let mut copy = data.clone();
                let mut v = ReferenceVisitor {
                    f: |_: &'static str, guid: &mut u32| f(guid),
                    found: false,
                };
                let _ = copy.visit(&mut v, version);
                if v.found {
                    *data = copy;
                }
            }
            ComponentBody::Fields(fields) => {
                for field in fields {
                    if let FieldValue::Value(value) = &mut field.value {
                        value_references(value, &mut f);
                    }
                }
            }
            _ => {}
        }
    }

    /// The version a typed body is written in.
    fn data_version(&self) -> i32 {
        match (&self.kind, &self.body) {
            (ComponentKind::Known { version, .. }, _) => *version,
            (ComponentKind::Unknown { .. }, ComponentBody::Data { data, .. }) => {
                data.latest_version()
            }
            _ => 0,
        }
    }
}

impl From<ComponentData> for Component {
    fn from(data: ComponentData) -> Self {
        Component::new(data)
    }
}

impl From<Transform> for Component {
    fn from(transform: Transform) -> Self {
        Component::transform(transform)
    }
}

impl Field {
    /// A field with a value.
    pub fn new(name: impl Into<DString>, value: Value) -> Self {
        Field {
            name: name.into(),
            value: FieldValue::Value(value),
        }
    }
}

fn value_references(value: &mut Value, f: &mut impl FnMut(&mut u32)) {
    match value {
        Value::Reference(guid) => f(guid),
        Value::Array(items) => items.iter_mut().for_each(|v| value_references(v, f)),
        _ => {}
    }
}

/// A problem with the GUIDs or references of a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// More than one object or component has this GUID. References to it go to the last one
    /// the game loads.
    DuplicateGuid(u32),
    /// A reference to a GUID that nothing has. The game logs a warning and leaves the
    /// reference empty.
    MissingTarget { from: Source, guid: u32 },
    /// A reference to an object or component of a type the game can't use there. The game
    /// fails to load the level.
    WrongTarget {
        from: Source,
        guid: u32,
        target: Target,
        /// The component types the game accepts.
        expected: &'static [ComponentId],
    },
}

/// Where a reference is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// The GUID of the component with the reference.
    pub guid: u32,
    /// The component's type, if the game knows it.
    pub component: Option<ComponentId>,
    /// The reference's name in the game.
    pub field: String,
}

/// What has a GUID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// An object, with its name.
    Object(DString),
    /// A component, with its type if the game knows it.
    Component(Option<ComponentId>),
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Problem::DuplicateGuid(guid) => write!(f, "GUID {guid} is used more than once"),
            Problem::MissingTarget { from, guid } => {
                write!(f, "{from} refers to GUID {guid}, which nothing has")
            }
            Problem::WrongTarget {
                from,
                guid,
                target,
                expected,
            } => {
                let expected: Vec<_> = expected.iter().map(ToString::to_string).collect();
                write!(
                    f,
                    "{from} refers to GUID {guid}, {target}, where the game takes only {}",
                    expected.join(" or ")
                )
            }
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}` of ", self.field)?;
        match self.component {
            Some(id) => write!(f, "{id} {}", self.guid),
            None => write!(f, "component {}", self.guid),
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Target::Object(name) => write!(f, "the object {name}"),
            Target::Component(Some(id)) => write!(f, "a {id}"),
            Target::Component(None) => f.write_str("a component of unknown type"),
        }
    }
}

/// The component types that a reference of a typed body can point to, or none if any can.
/// The game casts the target to the type of the field, and fails to load the level if it isn't
/// one.
fn expected_targets(id: ComponentId, field: &str) -> &'static [ComponentId] {
    use ComponentId as C;
    // The game's classes and their subclasses with component types.
    const TRACK_SEGMENT: &[ComponentId] = &[C::TRACK_SEGMENT, C::SPLINE_SEGMENT];
    const TRACK_LINK_PARENT: &[ComponentId] = &[C::TRACK_SEGMENT, C::SPLINE_SEGMENT, C::GROUP];
    match (id, field) {
        (C::GROUP | C::TRACK_SEGMENT, "Links") => &[C::TRACK_LINK],
        (C::TRACK_LINK, "SegRef") => TRACK_LINK_PARENT,
        (C::TRACK_LINK, "LinkRef") => &[C::TRACK_LINK],
        (C::TRACK_LINK, "TrackNodeRef") => &[C::TRACK_MANIPULATOR_NODE],
        (C::TRACK_ATTACHMENT, "Attached To") => TRACK_SEGMENT,
        _ => &[],
    }
}

fn level_roots(level: &Level) -> Vec<&GameObject> {
    let settings = match &level.settings {
        LevelSettingsData::GameObject(obj) => Some(obj),
        LevelSettingsData::Legacy { .. } => None,
    };
    let objects = level.layers.iter().flat_map(|layer| &layer.objects);
    settings.into_iter().chain(objects).collect()
}

fn level_roots_mut(level: &mut Level) -> Vec<&mut GameObject> {
    let settings = match &mut level.settings {
        LevelSettingsData::GameObject(obj) => Some(obj),
        LevelSettingsData::Legacy { .. } => None,
    };
    let objects = level.layers.iter_mut().flat_map(|layer| &mut layer.objects);
    settings.into_iter().chain(objects).collect()
}

enum Node<'a> {
    Object(&'a GameObject),
    Component(&'a Component),
}

enum NodeMut<'a> {
    Object(&'a mut GameObject),
    Component(&'a mut Component),
}

/// Calls `f` on every object and component under the roots: an object, then its components,
/// then its children. Objects nest hundreds deep, so this doesn't recurse.
fn walk<'a>(roots: Vec<&'a GameObject>, mut f: impl FnMut(Node<'a>)) {
    let mut stack: Vec<_> = roots.into_iter().rev().collect();
    while let Some(obj) = stack.pop() {
        f(Node::Object(obj));
        let start = stack.len();
        for c in &obj.components {
            f(Node::Component(c));
            if let ComponentBody::Transform(transform) = &c.body {
                stack.extend(&transform.children);
            }
        }
        stack[start..].reverse();
    }
}

/// [`walk`] with changes.
fn walk_mut(roots: Vec<&mut GameObject>, mut f: impl FnMut(NodeMut<'_>)) {
    let mut stack: Vec<_> = roots.into_iter().rev().collect();
    while let Some(obj) = stack.pop() {
        f(NodeMut::Object(&mut *obj));
        let start = stack.len();
        for c in &mut obj.components {
            f(NodeMut::Component(&mut *c));
            if let ComponentBody::Transform(transform) = &mut c.body {
                stack.extend(&mut transform.children);
            }
        }
        stack[start..].reverse();
    }
}

fn max_guid(roots: Vec<&GameObject>) -> u32 {
    let mut max = 0;
    walk(roots, |node| {
        let guid = match node {
            Node::Object(obj) => obj.guid,
            Node::Component(c) => c.guid,
        };
        max = max.max(guid);
    });
    max
}

fn assign_guids(roots: Vec<&mut GameObject>, max: u32) -> usize {
    let mut last = max;
    let mut assigned = 0;
    walk_mut(roots, |node| {
        let guid = match node {
            NodeMut::Object(obj) => &mut obj.guid,
            NodeMut::Component(c) => &mut c.guid,
        };
        if *guid == 0
            && let Some(next) = last.checked_add(1)
        {
            *guid = next;
            last = next;
            assigned += 1;
        }
    });
    assigned
}

fn check(roots: Vec<&GameObject>) -> Vec<Problem> {
    let mut problems = Vec::new();
    // As in the game, the last object or component with a GUID is the one references go to.
    let mut targets = HashMap::new();
    let mut duplicates = HashSet::new();
    walk(roots.clone(), |node| {
        let (guid, target) = match node {
            Node::Object(obj) => (obj.guid, Target::Object(obj.name.clone())),
            Node::Component(c) => (c.guid, Target::Component(c.id())),
        };
        if guid != 0 && targets.insert(guid, target).is_some() && duplicates.insert(guid) {
            problems.push(Problem::DuplicateGuid(guid));
        }
    });
    walk(roots, |node| {
        let Node::Component(c) = node else {
            return;
        };
        for (field, guid) in c.references() {
            let expected = c.id().map_or(&[][..], |id| expected_targets(id, &field));
            let from = Source {
                guid: c.guid,
                component: c.id(),
                field,
            };
            match targets.get(&guid) {
                None => problems.push(Problem::MissingTarget { from, guid }),
                Some(target) => {
                    let fits = match target {
                        Target::Component(Some(id)) => expected.contains(id),
                        _ => false,
                    };
                    if !expected.is_empty() && !fits {
                        problems.push(Problem::WrongTarget {
                            from,
                            guid,
                            target: target.clone(),
                            expected,
                        });
                    }
                }
            }
        }
    });
    problems
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{
        Group, ProfileProgress, TeleporterEntrance, TeleporterExit, TrackLink, TrackSegment,
    };

    fn guids(level: &Level) -> Vec<u32> {
        let mut guids = Vec::new();
        walk(level_roots(level), |node| {
            guids.push(match node {
                Node::Object(obj) => obj.guid,
                Node::Component(c) => c.guid,
            })
        });
        guids
    }

    #[test]
    fn new_level() {
        let mut level = Level::new("Test");
        assert_eq!(level.assign_guids(), 3);
        assert_eq!(guids(&level), [1, 2, 3]);
        assert_eq!(level.check(), []);
        let file = BytesFile::Level(level);
        assert_eq!(BytesFile::read(&file.write()).unwrap(), file);
        let json = crate::json::to_string(&file).unwrap();
        assert!(json.contains(r#""data":{"level_name":"Test"}"#), "{json}");
    }

    #[test]
    fn new_components_are_in_the_written_version() {
        let c = Component::from(TeleporterEntrance::default());
        let id = ComponentId::from_name("TeleporterEntrance").unwrap();
        let expected = ComponentKind::Known {
            mark: id.written_mark(),
            id,
            version: 4,
        };
        assert_eq!(c.kind, expected);

        let flicker = ComponentId::LIGHT_FLICKER_LOGIC;
        let fields = vec![Field::new("FlickerModNumber", Value::Int(2))];
        let c = Component::fields(flicker, fields).unwrap();
        assert!(matches!(c.kind, ComponentKind::Known { version: 0, .. }));
        // These are written with their own serializers.
        assert_eq!(Component::fields(id, vec![]), None);
        assert_eq!(Component::fields(ComponentId::TRANSFORM, vec![]), None);
    }

    #[test]
    fn guids_go_above_the_largest() {
        let mut level = Level::new("Test");
        let child = GameObject::new("CubeGS").with(TeleporterExit::default());
        let mut obj = GameObject::new("Group")
            .with_position([1.0, 2.0, 3.0])
            .with_child(child)
            .with(Group::default());
        obj.guid = 10;
        level.layers[0].objects.push(obj);
        level.assign_guids();
        // The settings, then the object (already 10), its Transform and Group, then the child
        // and its TeleporterExit.
        assert_eq!(guids(&level), [11, 12, 13, 10, 14, 15, 16, 17]);
        assert_eq!(level.assign_guids(), 0);
        let obj = &level.layers[0].objects[0];
        assert_eq!(obj.components[0].id(), Some(ComponentId::TRANSFORM));
        assert_eq!(obj.children()[0].name, "CubeGS");
    }

    fn track(link: TrackLink) -> Level {
        let mut level = Level::new("Test");
        let segment = TrackSegment::default();
        let mut segment = GameObject::new("SplineRoad").with(segment);
        segment.components[0].guid = 5;
        let mut link = GameObject::new("SnapPoint").with(link);
        link.components[0].guid = 6;
        level.layers[0].objects = vec![segment, link];
        level.assign_guids();
        level
    }

    #[test]
    fn references_are_checked() {
        let link = |seg_ref, link_ref| TrackLink {
            seg_ref,
            link_ref,
            ..TrackLink::default()
        };
        assert_eq!(track(link(5, 6)).check(), []);

        let from = |field: &str| Source {
            guid: 6,
            component: Some(ComponentId::TRACK_LINK),
            field: field.into(),
        };
        assert_eq!(
            track(link(5, 99)).check(),
            [Problem::MissingTarget {
                from: from("LinkRef"),
                guid: 99,
            }]
        );
        let problems = track(link(6, 5)).check();
        assert_eq!(problems.len(), 2);
        assert_eq!(
            problems[0],
            Problem::WrongTarget {
                from: from("SegRef"),
                guid: 6,
                target: Target::Component(Some(ComponentId::TRACK_LINK)),
                expected: &[
                    ComponentId::TRACK_SEGMENT,
                    ComponentId::SPLINE_SEGMENT,
                    ComponentId::GROUP
                ],
            }
        );
        assert_eq!(
            problems[0].to_string(),
            "`SegRef` of TrackLink 6 refers to GUID 6, a TrackLink, where the game takes only \
             TrackSegment or SplineSegment or Group"
        );

        let mut level = track(link(0, 0));
        level.layers[0].objects[1].components[0].guid = 5;
        assert_eq!(level.check(), [Problem::DuplicateGuid(5)]);
    }

    #[test]
    fn references_can_be_renumbered() {
        let link = TrackLink {
            seg_ref: 5,
            link_ref: 6,
            track_node_ref: 7, // Version 1 only, so not a reference in version 2.
            ..TrackLink::default()
        };
        let mut c = Component::from(link);
        assert_eq!(
            c.references(),
            [("SegRef".to_string(), 5), ("LinkRef".to_string(), 6)]
        );
        c.references_mut(|guid| *guid += 100);
        assert_eq!(
            c.references(),
            [("SegRef".to_string(), 105), ("LinkRef".to_string(), 106)]
        );

        // References in the generic format.
        let saw = ComponentId::BACK_AND_FORTH_SAW_LOGIC;
        let mut c = Component {
            body: ComponentBody::Fields(vec![
                Field::new("SawTrail", Value::Reference(8)),
                Field::new("MoveSpeedMultiplier", Value::Float(1.0)),
            ]),
            ..Component::empty(saw)
        };
        c.kind = ComponentKind::Known {
            mark: saw.written_mark(),
            id: saw,
            version: 0,
        };
        c.references_mut(|guid| *guid += 1);
        assert_eq!(c.references(), [("SawTrail".to_string(), 9)]);

        // Bodies without references are left as they are, though visiting them changes them.
        let progress = ProfileProgress {
            total_medal_count: Some(1 << 40),
            ..ProfileProgress::default()
        };
        let mut c = Component::new(progress);
        c.kind = ComponentKind::Known {
            mark: c.id().unwrap().written_mark(),
            id: c.id().unwrap(),
            version: 1,
        };
        let before = c.clone();
        c.references_mut(|_| unreachable!());
        assert_eq!(c, before);
    }

    /// GUIDs are assigned and checked without recursion.
    #[test]
    fn deep_nesting() {
        let mut obj = GameObject::new("Leaf");
        for _ in 0..20_000 {
            obj = GameObject::new("Group").with_child(obj);
        }
        let mut level = Level::new("Deep");
        level.layers[0].objects.push(obj);
        let ok = std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(move || level.assign_guids() == 40_004 && level.check().is_empty())
            .unwrap()
            .join()
            .unwrap();
        assert!(ok);
    }
}
