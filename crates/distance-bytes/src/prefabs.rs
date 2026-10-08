//! Prefab defaults: the values that a file leaves to the prefabs of its objects, as the game has
//! them, so that they can be filled in ([`Level::resolve`]) or left out again
//! ([`Level::minimize`]).
//!
//! ```no_run
//! use distance_bytes::{BytesFile, PrefabDefaults};
//!
//! let defaults = PrefabDefaults::read(&std::fs::read("prefab-defaults.bytes").unwrap()).unwrap();
//! let mut file = BytesFile::read(&std::fs::read("level.bytes").unwrap()).unwrap();
//! for unresolved in file.resolve(&defaults) {
//!     println!("{unresolved}");
//! }
//! ```
//!
//! The game reads an object by creating it from the prefab of its name and reading the file's
//! values into it. A component the prefab lacks is added if the game allows adding it. Empty
//! bodies and values keep what the object has. A child is found among the prefab's children by
//! name, except under a prefab tagged `PrefabContainer` (`Group`), where it's a new object
//! created from the prefab of its own name.
//!
//! The defaults are dumped from the game by the BytesTrace plugin (`dbytes prefabs`), with each
//! prefab's values after its components' `Awake`, as the game has them when it reads a level.
//! They're stored as a level with three layers: `Prefabs`, `Containers` (prefabs tagged
//! `PrefabContainer`) and `AddedComponents` (one object with a new component of every type that
//! the game adds to objects).

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::build::{NodeMut, walk_mut};
use crate::component_id::ComponentId;
use crate::components::{ComponentData, MaterialColors};
use crate::error::{Error, ErrorKind, Result};
use crate::fields::{FieldValue, Value, write_value};
use crate::io::Writer;
use crate::stack::grow;
use crate::string::DString;
use crate::tree::{
    BytesFile, Component, ComponentBody, ComponentKind, GameObject, Layer, Level,
    LevelSettingsData, Transform,
};
use crate::visit::{BodyEnd, DefaultsVisitor};

const PREFABS: &str = "Prefabs";
const CONTAINERS: &str = "Containers";
const ADDED: &str = "AddedComponents";

/// The prefab of the level settings object.
const LEVEL_SETTINGS: &str = "LevelSettings";

/// The prefabs of a game build, with every value.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PrefabDefaults {
    prefabs: Vec<Prefab>,
    by_name: HashMap<DString, usize>,
    added: Vec<Component>,
    added_by_id: HashMap<ComponentId, usize>,
}

/// A prefab, as the game creates an object from it.
#[derive(Debug, Clone, PartialEq)]
pub struct Prefab {
    /// The object, with every component and child and every value. Its name is the prefab's.
    pub object: GameObject,
    /// Whether the prefab is tagged `PrefabContainer`, as `Group` is. In a file, the children of
    /// its objects are new objects, created from the prefabs of their own names.
    pub container: bool,
}

impl PrefabDefaults {
    /// Reads defaults written by [`PrefabDefaults::write`].
    pub fn read(data: &[u8]) -> Result<Self> {
        match BytesFile::read(data)? {
            BytesFile::Level(level) => Self::from_level(level),
            BytesFile::GameObject(_) => Err(Error::new(0, ErrorKind::NotPrefabDefaults)),
        }
    }

    /// Defaults from the level that [`PrefabDefaults::to_level`] makes.
    pub fn from_level(level: Level) -> Result<Self> {
        let mut defaults = PrefabDefaults::default();
        for layer in level.layers {
            match layer.name.as_str() {
                Some(PREFABS) => layer
                    .objects
                    .into_iter()
                    .for_each(|obj| defaults.insert(obj, false)),
                Some(CONTAINERS) => layer
                    .objects
                    .into_iter()
                    .for_each(|obj| defaults.insert(obj, true)),
                Some(ADDED) => {
                    for mut obj in layer.objects {
                        for c in std::mem::take(&mut obj.components) {
                            defaults.insert_added(c);
                        }
                    }
                }
                _ => return Err(Error::new(0, ErrorKind::NotPrefabDefaults)),
            }
        }
        Ok(defaults)
    }

    /// The defaults as a level, which is how they're stored.
    pub fn to_level(&self) -> Level {
        let mut level = Level::new("Prefab defaults");
        let mut prefabs = Layer::new(PREFABS);
        let mut containers = Layer::new(CONTAINERS);
        for prefab in &self.prefabs {
            let layer = if prefab.container {
                &mut containers
            } else {
                &mut prefabs
            };
            layer.objects.push(prefab.object.clone());
        }
        let mut added = Layer::new(ADDED);
        let mut obj = GameObject::new(ADDED);
        obj.components = self.added.clone();
        added.objects.push(obj);
        level.layers = vec![prefabs, containers, added];
        level
    }

    pub fn write(&self) -> Vec<u8> {
        BytesFile::Level(self.to_level()).write()
    }

    /// Adds a prefab, named after the object, or replaces the one of that name.
    pub fn insert(&mut self, object: GameObject, container: bool) {
        let prefab = Prefab { object, container };
        match self.by_name.get(&prefab.object.name) {
            Some(&i) => self.prefabs[i] = prefab,
            None => {
                self.by_name
                    .insert(prefab.object.name.clone(), self.prefabs.len());
                self.prefabs.push(prefab);
            }
        }
    }

    /// Adds a component type that the game adds to objects whose prefab lacks it, as the game
    /// creates it, or replaces the one of its type. Components of unknown types are ignored.
    pub fn insert_added(&mut self, component: Component) {
        let Some(id) = component.id() else {
            return;
        };
        match self.added_by_id.get(&id) {
            Some(&i) => self.added[i] = component,
            None => {
                self.added_by_id.insert(id, self.added.len());
                self.added.push(component);
            }
        }
    }

    /// The prefab of a name.
    pub fn get(&self, name: &str) -> Option<&Prefab> {
        self.prefab(&DString::from(name))
    }

    fn prefab(&self, name: &DString) -> Option<&Prefab> {
        self.by_name.get(name).map(|&i| &self.prefabs[i])
    }

    /// A new component of a type the game adds to objects whose prefab lacks it, or `None` if
    /// the game doesn't add the type.
    pub fn added(&self, id: ComponentId) -> Option<&Component> {
        self.added_by_id.get(&id).map(|&i| &self.added[i])
    }

    pub fn prefabs(&self) -> impl Iterator<Item = &Prefab> {
        self.prefabs.iter()
    }

    pub fn added_components(&self) -> impl Iterator<Item = &Component> {
        self.added.iter()
    }
}

/// Something in a file that the game skips or merges with something else when it reads the
/// file. [`resolve`](Level::resolve) leaves what the game skips as it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unresolved {
    /// An object whose prefab the game doesn't have, with its GUID. The game skips it and its
    /// children.
    Prefab { name: DString, guid: u32 },
    /// A child that the prefab of its parent doesn't have, with its GUID.
    Child {
        parent: DString,
        name: DString,
        guid: u32,
    },
    /// A component that the object's prefab doesn't have and that the game doesn't add, with
    /// its GUID.
    Component {
        object: DString,
        component: ComponentId,
        guid: u32,
    },
    /// A component whose type the object's prefab has only as a subclass, with its GUID. The
    /// game doesn't read its values, but references to it go to the subclass's component,
    /// which gets its GUID (unless it already has one).
    Subclass {
        object: DString,
        component: ComponentId,
        found: ComponentId,
        guid: u32,
    },
    /// A child, or a component of an object, that the game reads into the same child or
    /// component as an earlier one of its name or type, with its GUID. References to it go
    /// there too, but only the earlier one's GUID is kept.
    Repeated {
        object: DString,
        /// The component's type, or `None` for a child.
        component: Option<ComponentId>,
        guid: u32,
    },
    /// A material of a `MeshRenderer` that the object's renderer doesn't have, or a color that
    /// its material doesn't have, with the component's GUID. The game reads the colors into
    /// the renderer's own materials by material and color name.
    ///
    /// `TextMesh`, `ParticlesGPU` and `GoldenSimples` replace their object's materials
    /// according to their values before the game reads the renderer. On an object with one of
    /// them, if none of the file's materials is the prefab renderer's, the file's are taken to
    /// be the renderer's, and none is reported.
    Material {
        object: DString,
        material: DString,
        /// The color's name, or `None` for the whole material.
        color: Option<DString>,
        guid: u32,
    },
}

impl fmt::Display for Unresolved {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unresolved::Prefab { name, guid } => write!(
                f,
                "the game skips object {name} {guid}, as it has no prefab of that name"
            ),
            Unresolved::Child { parent, name, guid } => write!(
                f,
                "the game skips child {name} {guid}, as the prefab {parent} has no such child"
            ),
            Unresolved::Component {
                object,
                component,
                guid,
            } => write!(
                f,
                "the game skips {component} {guid}, as the prefab {object} has none and the \
                 game doesn't add it"
            ),
            Unresolved::Subclass {
                object,
                component,
                found,
                guid,
            } => write!(
                f,
                "the game skips the values of {component} {guid}, as the prefab {object} has a \
                 {found} instead, which references to it go to"
            ),
            Unresolved::Repeated {
                object,
                component: Some(component),
                guid,
            } => write!(
                f,
                "the game reads {component} {guid} of {object} into an earlier {component}, \
                 which keeps its GUID here"
            ),
            Unresolved::Repeated {
                object,
                component: None,
                guid,
            } => write!(
                f,
                "the game reads child {object} {guid} into an earlier child of that name, which \
                 keeps its GUID here"
            ),
            Unresolved::Material {
                object,
                material,
                color: None,
                guid,
            } => write!(
                f,
                "the game skips material {material} of MeshRenderer {guid}, as the renderer of \
                 {object} has no such material"
            ),
            Unresolved::Material {
                object,
                material,
                color: Some(color),
                guid,
            } => write!(
                f,
                "the game skips color {color} of material {material} of MeshRenderer {guid}, as \
                 the material of {object} has no such color"
            ),
        }
    }
}

impl Level {
    /// Fills in every value that the level leaves to the prefabs of its objects, so that it has
    /// the values the game reads it with, and returns what the game skips or merges. Objects get
    /// the components and children of their prefabs that the level doesn't list, with GUID 0.
    ///
    /// Bodies stay in the level's component versions, with the prefab's values for those their
    /// version doesn't have. A `MeshRenderer` has the materials of the prefab's renderer, with
    /// the level's colors read into them by material and color name (see
    /// [`Unresolved::Material`] for an exception). Bodies that didn't decode, and bodies in a
    /// different format than the prefab's (the generic format of an old version), are left as
    /// they are.
    ///
    /// What the game does beyond reading the values isn't applied:
    /// - its upgrades of old versions;
    /// - its fixes of the values it reads: NaN or infinite `Transform` values, scales under 1e-5,
    ///   and a `Light`'s spot angle over 150 and intensity over 8;
    /// - values it sets from others as it reads them: those a `GoldenSimples` preset sets, the
    ///   material a `TextMesh` font sets, and the pong values of an animation curve without
    ///   custom ones.
    pub fn resolve(&mut self, defaults: &PrefabDefaults) -> Vec<Unresolved> {
        let mut out = Vec::new();
        if let LevelSettingsData::GameObject(obj) = &mut self.settings {
            resolve_root(obj, &DString::from(LEVEL_SETTINGS), defaults, &mut out);
        }
        for obj in self.layers.iter_mut().flat_map(|layer| &mut layer.objects) {
            let name = obj.name.clone();
            resolve_root(obj, &name, defaults, &mut out);
        }
        out
    }

    /// Leaves out every value that is the same as what the game reads in its place: a body
    /// becomes empty, a `Transform` value or a generic field the empty marker, and a
    /// `MeshRenderer` color is left out, where it is the same as the prefab's (or a new
    /// component's). Bodies of a component version older than the game's aren't emptied or cut,
    /// as the game upgrades only those it reads.
    ///
    /// Components and children are kept, as the game writes every one.
    pub fn minimize(&mut self, defaults: &PrefabDefaults) {
        if let LevelSettingsData::GameObject(obj) = &mut self.settings {
            minimize_root(obj, &DString::from(LEVEL_SETTINGS), defaults);
        }
        for obj in self.layers.iter_mut().flat_map(|layer| &mut layer.objects) {
            let name = obj.name.clone();
            minimize_root(obj, &name, defaults);
        }
    }
}

impl BytesFile {
    /// See [`Level::resolve`]. A file of one object is resolved as a level's object.
    pub fn resolve(&mut self, defaults: &PrefabDefaults) -> Vec<Unresolved> {
        match self {
            BytesFile::Level(level) => level.resolve(defaults),
            BytesFile::GameObject(obj) => obj.resolve(defaults),
        }
    }

    /// See [`Level::minimize`].
    pub fn minimize(&mut self, defaults: &PrefabDefaults) {
        match self {
            BytesFile::Level(level) => level.minimize(defaults),
            BytesFile::GameObject(obj) => obj.minimize(defaults),
        }
    }
}

impl GameObject {
    /// [`Level::resolve`] for an object of a level's layer.
    pub fn resolve(&mut self, defaults: &PrefabDefaults) -> Vec<Unresolved> {
        let mut out = Vec::new();
        let name = self.name.clone();
        resolve_root(self, &name, defaults, &mut out);
        out
    }

    /// [`Level::minimize`] for an object of a level's layer.
    pub fn minimize(&mut self, defaults: &PrefabDefaults) {
        let name = self.name.clone();
        minimize_root(self, &name, defaults);
    }
}

/// Resolves an object that the game creates from the prefab of a name.
fn resolve_root(
    obj: &mut GameObject,
    prefab: &DString,
    defaults: &PrefabDefaults,
    out: &mut Vec<Unresolved>,
) {
    let Some(prefab) = defaults.prefab(prefab) else {
        out.push(Unresolved::Prefab {
            name: obj.name.clone(),
            guid: obj.guid,
        });
        return;
    };
    let mut created = prefab.object.clone();
    walk_mut(vec![&mut created], |node| match node {
        NodeMut::Object(obj) => obj.guid = 0,
        NodeMut::Component(c) => c.guid = 0,
    });
    let file = std::mem::replace(obj, created);
    read_into(obj, file, prefab.container, defaults, out);
}

/// Components that replace the materials of their object's renderer according to their values:
/// `TextMesh` with its font's, `ParticlesGPU` with its blend mode's, and `GoldenSimples`.
const MATERIAL_SETTERS: [ComponentId; 3] = [
    ComponentId::TEXT_MESH,
    ComponentId::PARTICLES_GPU,
    ComponentId::GOLDEN_SIMPLES,
];

/// Reads an object of a file into the object the game has for it, as the game does.
fn read_into(
    obj: &mut GameObject,
    mut file: GameObject,
    container: bool,
    defaults: &PrefabDefaults,
    out: &mut Vec<Unresolved>,
) {
    grow(|| {
        obj.name = file.name.clone();
        obj.prefab = std::mem::take(&mut file.prefab);
        obj.guid = file.guid;
        // Whether one of the object's components (the prefab's, or one the game adds) replaces
        // its renderer's materials.
        let sets = |c: &Component| c.id().filter(|id| MATERIAL_SETTERS.contains(id));
        let sets_materials = obj.components.iter().any(|c| sets(c).is_some())
            || file
                .components
                .iter()
                .any(|c| sets(c).is_some_and(|id| defaults.added(id).is_some()));
        let mut read = HashSet::new();
        for c in std::mem::take(&mut file.components) {
            let Some(id) = c.id() else {
                obj.components.push(c);
                continue;
            };
            // The game reads every component of a type into the object's first one of the type
            // or a subclass.
            let found = obj
                .components
                .iter()
                .position(|o| o.id().is_some_and(|o| o.is_a(id)));
            if let Some(i) = found {
                let found = &mut obj.components[i];
                if found.id() == Some(id) {
                    let name = obj.name.clone();
                    let guid = found.guid;
                    let repeated = !read.insert(i);
                    if repeated {
                        out.push(Unresolved::Repeated {
                            object: name.clone(),
                            component: Some(id),
                            guid: c.guid,
                        });
                    }
                    read_component(found, c, &name, container, sets_materials, defaults, out);
                    if repeated {
                        found.guid = guid;
                    }
                    continue;
                }
                // It doesn't read a subclass's values, but references go to it.
                out.push(Unresolved::Subclass {
                    object: obj.name.clone(),
                    component: id,
                    found: found.id().unwrap(),
                    guid: c.guid,
                });
                if found.guid == 0 {
                    found.guid = c.guid;
                } else {
                    obj.components.push(c);
                }
            } else if let Some(added) = defaults.added(id) {
                let mut added = added.clone();
                read_component(
                    &mut added,
                    c,
                    &obj.name,
                    container,
                    sets_materials,
                    defaults,
                    out,
                );
                obj.components.push(added);
            } else {
                out.push(Unresolved::Component {
                    object: obj.name.clone(),
                    component: id,
                    guid: c.guid,
                });
                obj.components.push(c);
            }
        }
    })
}

fn read_component(
    c: &mut Component,
    mut file: Component,
    object: &DString,
    container: bool,
    sets_materials: bool,
    defaults: &PrefabDefaults,
    out: &mut Vec<Unresolved>,
) {
    c.guid = file.guid;
    let version = version(&c.kind);
    let file_version = version_of(&file);
    if matches!(file.body, ComponentBody::Empty) {
        // Keep the file's version, unless the typed body has none such (an old version in the
        // generic format).
        if let ComponentBody::Data { data, .. } = &c.body
            && !ComponentData::decodes(data.id(), file_version)
        {
            return;
        }
        c.kind = file.kind;
        return;
    }
    match (&mut c.body, &mut file.body) {
        (ComponentBody::Transform(t), ComponentBody::Transform(f)) => {
            t.position = f.position.or(t.position);
            t.rotation = f.rotation.or(t.rotation);
            t.scale = f.scale.or(t.scale);
            let mut read = HashSet::new();
            for child in std::mem::take(&mut f.children) {
                read_child(t, child, object, container, &mut read, defaults, out);
            }
            c.kind = file.kind;
            return;
        }
        (ComponentBody::Fields(fields), ComponentBody::Fields(f)) => {
            for field in std::mem::take(f) {
                match fields.iter().position(|o| o.name == field.name) {
                    // The game keeps the value for an empty field and for one it can't read.
                    Some(i) => {
                        if let FieldValue::Value(_) = field.value {
                            fields[i] = field;
                        }
                    }
                    // A field the game doesn't have, which it skips.
                    None => fields.push(field),
                }
            }
            c.kind = file.kind;
            return;
        }
        (ComponentBody::Data { data, .. }, ComponentBody::Data { data: f, end })
            if data.id() == f.id() =>
        {
            for v in [file_version, version] {
                fill(f, data, v);
            }
            // The game reads the colors into the renderer's materials, which components such as
            // TextMesh replace before it reads them. So on such an object, where none of the
            // file's materials is one of the prefab renderer's, the file's are taken to be the
            // renderer's.
            if let (ComponentData::MeshRenderer(base), ComponentData::MeshRenderer(f)) =
                (&*data, &mut *f)
                && (!sets_materials
                    || f.materials.is_empty()
                    || f.materials
                        .iter()
                        .any(|m| base_material(m, &base.materials).is_some()))
            {
                let file = std::mem::replace(&mut f.materials, base.materials.clone());
                for (material, color) in read_materials(&mut f.materials, file) {
                    out.push(Unresolved::Material {
                        object: object.clone(),
                        material,
                        color,
                        guid: c.guid,
                    });
                }
            }
            // The missing values are filled in.
            if let BodyEnd::Truncated(_) = end {
                *end = BodyEnd::Complete;
            }
        }
        _ => {}
    }
    c.kind = file.kind;
    c.body = file.body;
}

/// Reads a child of an object of a file into the object's `Transform`. `read` has the children
/// already read.
fn read_child(
    t: &mut Transform,
    child: GameObject,
    parent: &DString,
    container: bool,
    read: &mut HashSet<usize>,
    defaults: &PrefabDefaults,
    out: &mut Vec<Unresolved>,
) {
    if container {
        let mut child = child;
        let name = child.name.clone();
        resolve_root(&mut child, &name, defaults, out);
        t.children.push(child);
    } else if let Some(i) = t.children.iter().position(|o| o.name == child.name) {
        // The game reads every child of a name into the first one.
        let found = &mut t.children[i];
        let guid = found.guid;
        let repeated = !read.insert(i);
        if repeated {
            out.push(Unresolved::Repeated {
                object: child.name.clone(),
                component: None,
                guid: child.guid,
            });
        }
        read_into(found, child, false, defaults, out);
        if repeated {
            found.guid = guid;
        }
    } else {
        out.push(Unresolved::Child {
            parent: parent.clone(),
            name: child.name.clone(),
            guid: child.guid,
        });
        t.children.push(child);
    }
}

/// Fills the values that a typed body leaves out in a version from another body of its type.
fn fill(data: &mut ComponentData, from: &ComponentData, version: i32) {
    let mut v = DefaultsVisitor::collect();
    if from.clone().visit(&mut v, version).is_err() {
        return;
    }
    let mut v = v.fill();
    // Visiting can normalize values the way writing does, so keep the copy only if it changed.
    let mut copy = data.clone();
    if copy.visit(&mut v, version).is_ok() && v.filled {
        *data = copy;
    }
}

/// Reads the material colors of a file's `MeshRenderer` into the renderer's, as the game does:
/// into the renderer's first material of a name, and its first color of a name. Materials and
/// colors that the renderer doesn't have are appended, and returned as `(material, color)`.
fn read_materials(
    materials: &mut Vec<MaterialColors>,
    file: Vec<MaterialColors>,
) -> Vec<(DString, Option<DString>)> {
    let mut skipped = Vec::new();
    let colors: Vec<usize> = materials.iter().map(|m| m.colors.len()).collect();
    for m in file {
        let found = (0..colors.len())
            .position(|i| m.material.is_some() && materials[i].material == m.material);
        let name = m.material.clone().unwrap_or_default();
        let Some(i) = found else {
            skipped.push((name, None));
            materials.push(m);
            continue;
        };
        for color in m.colors {
            let found = materials[i].colors[..colors[i]]
                .iter()
                .position(|o| color.name.is_some() && o.name == color.name);
            match found {
                Some(j) => {
                    if color.color.is_some() {
                        materials[i].colors[j].color = color.color;
                    }
                }
                None => {
                    skipped.push((name.clone(), Some(color.name.clone().unwrap_or_default())));
                    materials[i].colors.push(color);
                }
            }
        }
    }
    skipped
}

fn version(kind: &ComponentKind) -> i32 {
    match kind {
        ComponentKind::Known { version, .. } => *version,
        ComponentKind::Unknown { .. } => 0,
    }
}

/// The version a component's body is read in.
fn version_of(c: &Component) -> i32 {
    match (&c.kind, &c.body) {
        (ComponentKind::Unknown { .. }, ComponentBody::Data { data, .. }) => data.latest_version(),
        (kind, _) => version(kind),
    }
}

/// Minimizes an object that the game creates from the prefab of a name.
fn minimize_root(obj: &mut GameObject, prefab: &DString, defaults: &PrefabDefaults) {
    if let Some(prefab) = defaults.prefab(prefab) {
        minimize_against(obj, &prefab.object, prefab.container, defaults);
    }
}

/// Minimizes an object against the object the game has before reading it.
fn minimize_against(
    obj: &mut GameObject,
    created: &GameObject,
    container: bool,
    defaults: &PrefabDefaults,
) {
    grow(|| {
        let mut seen = HashSet::new();
        for c in &mut obj.components {
            let Some(id) = c.id() else {
                continue;
            };
            // Later components of a type are read into the first one, after it.
            if !seen.insert(id) {
                continue;
            }
            let found = created
                .components
                .iter()
                .find(|o| o.id().is_some_and(|o| o.is_a(id)));
            let base = match found {
                Some(found) if found.id() == Some(id) => found,
                // The game doesn't read the values of a component whose type the object has
                // only as a subclass.
                Some(_) => continue,
                None => match defaults.added(id) {
                    Some(added) => added,
                    None => continue,
                },
            };
            minimize_component(c, base, container, defaults);
        }
    })
}

fn minimize_component(
    c: &mut Component,
    base: &Component,
    container: bool,
    defaults: &PrefabDefaults,
) {
    let version = version_of(c);
    let current = c.id().and_then(ComponentId::written_version) == Some(version);
    match (&mut c.body, &base.body) {
        (ComponentBody::Transform(t), ComponentBody::Transform(b)) => {
            minimize_value(&mut t.position, b.position);
            minimize_value(&mut t.rotation, b.rotation);
            minimize_value(&mut t.scale, b.scale);
            let mut seen = HashSet::new();
            for child in &mut t.children {
                if container {
                    let name = child.name.clone();
                    minimize_root(child, &name, defaults);
                } else if seen.insert(child.name.clone())
                    && let Some(created) = b.children.iter().find(|o| o.name == child.name)
                {
                    // Later children of a name are read into the first one, after it.
                    minimize_against(child, created, false, defaults);
                }
            }
            if current && *t == Transform::default() {
                c.body = ComponentBody::Empty;
            }
        }
        (ComponentBody::Fields(fields), ComponentBody::Fields(b)) => {
            for field in fields.iter_mut() {
                if let FieldValue::Value(value) = &field.value
                    && let Some(base) = b.iter().find(|o| o.name == field.name)
                    && let FieldValue::Value(base) = &base.value
                    && same_value(value, base)
                {
                    field.value = FieldValue::Empty;
                }
            }
            if current && fields.iter().all(|f| f.value == FieldValue::Empty) {
                c.body = ComponentBody::Empty;
            }
        }
        (ComponentBody::Data { data, end }, ComponentBody::Data { data: b, .. })
            if current && data.id() == b.id() =>
        {
            if let (ComponentData::MeshRenderer(m), ComponentData::MeshRenderer(b)) =
                (&mut *data, b)
                && matches!(end, BodyEnd::Complete)
            {
                minimize_materials(&mut m.materials, &b.materials);
                // The game reads nothing from an empty list.
                if m.materials.is_empty() {
                    c.body = ComponentBody::Empty;
                    return;
                }
            }
            // Bytes after the values are skipped.
            let end = match end {
                BodyEnd::Trailing(_) => BodyEnd::Complete,
                end => end.clone(),
            };
            let (mut w, mut wb) = (Writer::new(), Writer::new());
            data.write(&mut w, version, &end);
            b.write(&mut wb, version, &end);
            if w.finish() == wb.finish() {
                c.body = ComponentBody::Empty;
            }
        }
        _ => {}
    }
}

/// Leaves out the colors of a `MeshRenderer` that are the same as the renderer's, and the
/// materials left without colors, unless a material or color comes more than once, as the game
/// reads them by name. Materials and colors that the renderer doesn't have are kept, and if
/// there are such materials, so are all of the renderer's.
fn minimize_materials(materials: &mut Vec<MaterialColors>, base: &[MaterialColors]) {
    let mut names = HashSet::new();
    if !materials.iter().all(|m| names.insert(&m.material)) {
        return;
    }
    let mut shared = true;
    for m in materials.iter_mut() {
        let Some(b) = base_material(m, base) else {
            shared = false;
            continue;
        };
        let mut names = HashSet::new();
        if !m.colors.iter().all(|c| names.insert(&c.name)) {
            continue;
        }
        m.colors.retain(|c| {
            let same = b
                .colors
                .iter()
                .find(|o| c.name.is_some() && o.name == c.name)
                .is_some_and(|o| bits(o.color) == bits(c.color));
            !same
        });
    }
    // On some objects, the file's materials are read into the renderer's only if one of them is
    // the renderer's (see `read_component`), so with others, those are kept.
    if shared {
        materials.retain(|m| !m.colors.is_empty());
    }
}

/// The material of the renderer's materials that the game reads a file's material into.
fn base_material<'a>(m: &MaterialColors, base: &'a [MaterialColors]) -> Option<&'a MaterialColors> {
    base.iter()
        .find(|b| m.material.is_some() && b.material == m.material)
}

fn bits<const N: usize>(value: Option<[f32; N]>) -> Option<[u32; N]> {
    value.map(|v| v.map(f32::to_bits))
}

fn minimize_value<const N: usize>(value: &mut Option<[f32; N]>, base: Option<[f32; N]>) {
    if value.is_some() && bits(*value) == bits(base) {
        *value = None;
    }
}

fn same_value(a: &Value, b: &Value) -> bool {
    let (mut wa, mut wb) = (Writer::new(), Writer::new());
    write_value(&mut wa, a);
    write_value(&mut wb, b);
    wa.finish() == wb.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{
        Group, LevelSettings, MaterialColor, MeshRenderer, SplineSegment, TeleporterEntrance,
        TeleporterExit, TrackSegment,
    };

    const ENTRANCE: ComponentId = ComponentId::TELEPORTER_ENTRANCE;
    const IDENTITY: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

    fn entrance(teleports_to: i32) -> TeleporterEntrance {
        TeleporterEntrance {
            teleports_to: Some(teleports_to),
            include_in_track_calculation: Some(true),
            bank_id: Some(4),
        }
    }

    /// An object as a prefab has it, with every `Transform` value.
    fn full(name: &str, position: [f32; 3]) -> GameObject {
        GameObject::new(name)
            .with_position(position)
            .with_rotation(IDENTITY)
            .with_scale([1.0; 3])
    }

    fn thing() -> GameObject {
        full("Thing", [1.0, 2.0, 3.0])
            .with_scale([2.0; 3])
            .with(entrance(7))
            .with_child(full("A", [0.0, 1.0, 0.0]))
            .with_child(full("B", [0.0, 2.0, 0.0]))
    }

    fn defaults() -> PrefabDefaults {
        let mut defaults = PrefabDefaults::default();
        defaults.insert(thing(), false);
        defaults.insert(full("Road", [0.0; 3]).with(SplineSegment::default()), false);
        let settings = LevelSettings {
            level_name: Some("Prefab".into()),
            ..LevelSettings::default()
        };
        defaults.insert(full(LEVEL_SETTINGS, [0.0; 3]).with(settings), false);
        // Last, as the file has containers after the other prefabs.
        defaults.insert(full("Group", [0.0; 3]).with(Group::default()), true);
        defaults.insert_added(Component::new(TeleporterExit { link_id: Some(1) }));
        defaults
    }

    fn transform(obj: &GameObject) -> &Transform {
        match &obj.component(ComponentId::TRANSFORM).unwrap().body {
            ComponentBody::Transform(t) => t,
            body => panic!("{body:?}"),
        }
    }

    fn with_guid(mut c: Component, guid: u32) -> Component {
        c.guid = guid;
        c
    }

    fn with_version(mut c: Component, v: i32) -> Component {
        if let ComponentKind::Known { version, .. } = &mut c.kind {
            *version = v;
        }
        c
    }

    #[test]
    fn defaults_round_trip() {
        let defaults = defaults();
        assert_eq!(PrefabDefaults::read(&defaults.write()).unwrap(), defaults);
        assert!(defaults.get("Group").unwrap().container);
        assert!(!defaults.get("Thing").unwrap().container);
        assert_eq!(defaults.get("Nothing"), None);
        let exit = defaults.added(ComponentId::TELEPORTER_EXIT).unwrap();
        assert_eq!(exit.id(), Some(ComponentId::TELEPORTER_EXIT));
        assert_eq!(defaults.added(ENTRANCE), None);

        let mut level = Level::new("Not defaults");
        level.layers.push(Layer::new(PREFABS));
        let data = BytesFile::Level(level).write();
        assert_eq!(
            PrefabDefaults::read(&data).unwrap_err().kind,
            ErrorKind::NotPrefabDefaults
        );
    }

    #[test]
    fn empty_bodies_are_the_prefabs() {
        let mut obj = GameObject::new("Thing")
            .with(with_guid(Component::empty(ComponentId::TRANSFORM), 2))
            .with(with_guid(Component::empty(ENTRANCE), 3));
        obj.guid = 1;
        assert_eq!(obj.resolve(&defaults()), []);
        let mut expected = thing();
        expected.guid = 1;
        expected.components[0].guid = 2;
        expected.components[1].guid = 3;
        assert_eq!(obj, expected);
    }

    #[test]
    fn values_and_children_are_filled_in() {
        let mut obj = GameObject::new("Thing")
            .with_position([5.0; 3])
            .with_child(GameObject::new("B").with_scale([3.0; 3]))
            .with_child(GameObject::new("C"));
        let unresolved = obj.resolve(&defaults());
        let expected = Unresolved::Child {
            parent: "Thing".into(),
            name: "C".into(),
            guid: 0,
        };
        assert_eq!(unresolved, [expected]);
        let t = transform(&obj);
        assert_eq!(t.position, Some([5.0; 3]));
        assert_eq!(t.rotation, Some(IDENTITY));
        assert_eq!(t.scale, Some([2.0; 3]));
        // The prefab's components and children that the file doesn't list are kept.
        assert_eq!(obj.components[1], Component::new(entrance(7)));
        assert_eq!(t.children.len(), 3);
        assert_eq!(t.children[0], full("A", [0.0, 1.0, 0.0]));
        let b = full("B", [0.0, 2.0, 0.0]).with_scale([3.0; 3]);
        assert_eq!(t.children[1], b);
        assert_eq!(t.children[2], GameObject::new("C"));
    }

    #[test]
    fn typed_bodies_are_filled_in_their_version() {
        let old = TeleporterEntrance {
            teleports_to: Some(3),
            ..TeleporterEntrance::default()
        };
        let mut obj = GameObject::new("Thing").with(with_version(Component::new(old), 2));
        assert_eq!(obj.resolve(&defaults()), []);
        let expected = with_version(Component::new(entrance(3)), 2);
        assert_eq!(obj.components[1], expected);
    }

    #[test]
    fn container_children_are_created_from_their_prefabs() {
        let mut obj = GameObject::new("Group")
            .with_child(GameObject::new("Thing").with_position([9.0; 3]))
            .with_child(GameObject::new("Nothing"));
        let unresolved = obj.resolve(&defaults());
        let expected = Unresolved::Prefab {
            name: "Nothing".into(),
            guid: 0,
        };
        assert_eq!(unresolved, [expected]);
        assert_eq!(obj.components[1], Component::new(Group::default()));
        let children = &transform(&obj).children;
        assert_eq!(children[0], thing().with_position([9.0; 3]));
        assert_eq!(children[1], GameObject::new("Nothing"));
    }

    #[test]
    fn components_are_added_if_the_game_adds_them() {
        let exit = ComponentId::TELEPORTER_EXIT;
        let group = ComponentId::GROUP;
        let mut obj = GameObject::new("Thing")
            .with(with_guid(Component::empty(exit), 5))
            .with(with_guid(Component::empty(group), 6));
        let unresolved = obj.resolve(&defaults());
        let expected = Unresolved::Component {
            object: "Thing".into(),
            component: group,
            guid: 6,
        };
        assert_eq!(unresolved, [expected]);
        let added = Component::new(TeleporterExit { link_id: Some(1) });
        assert_eq!(obj.components[2], with_guid(added, 5));
        assert_eq!(obj.components[3], with_guid(Component::empty(group), 6));
    }

    #[test]
    fn subclasses_get_the_guid_but_not_the_values() {
        let segment = TrackSegment {
            include_in_track_calculation: Some(true),
            ..TrackSegment::default()
        };
        let mut obj = GameObject::new("Road").with(with_guid(Component::new(segment), 9));
        let unresolved = obj.resolve(&defaults());
        let expected = Unresolved::Subclass {
            object: "Road".into(),
            component: ComponentId::TRACK_SEGMENT,
            found: ComponentId::SPLINE_SEGMENT,
            guid: 9,
        };
        assert_eq!(unresolved, [expected]);
        assert_eq!(obj.components.len(), 2);
        let found = Component::new(SplineSegment::default());
        assert_eq!(obj.components[1], with_guid(found, 9));
    }

    #[test]
    fn repeats_are_read_into_the_first() {
        let mut a = GameObject::new("A").with_position([1.0; 3]);
        a.guid = 7;
        let mut again = GameObject::new("A").with_scale([4.0; 3]);
        again.guid = 8;
        let mut obj = GameObject::new("Thing")
            .with_child(a)
            .with_child(again)
            .with(with_guid(Component::new(entrance(1)), 5))
            .with(with_guid(Component::new(entrance(2)), 6));
        let unresolved = obj.resolve(&defaults());
        let expected = [
            Unresolved::Repeated {
                object: "A".into(),
                component: None,
                guid: 8,
            },
            Unresolved::Repeated {
                object: "Thing".into(),
                component: Some(ENTRANCE),
                guid: 6,
            },
        ];
        assert_eq!(unresolved, expected);
        assert_eq!(obj.components.len(), 2);
        assert_eq!(obj.components[1], with_guid(Component::new(entrance(2)), 5));
        let children = &transform(&obj).children;
        assert_eq!(children.len(), 2);
        let mut a = full("A", [1.0; 3]).with_scale([4.0; 3]);
        a.guid = 7;
        assert_eq!(children[0], a);
    }

    #[test]
    fn minimize_leaves_out_the_prefabs_values() {
        let defaults = defaults();
        let mut obj = thing().with_position([5.0; 3]);
        obj.minimize(&defaults);
        let empty = |name| GameObject::new(name).with(Component::empty(ComponentId::TRANSFORM));
        let transform = Transform {
            position: Some([5.0; 3]),
            children: vec![empty("A"), empty("B")],
            ..Transform::default()
        };
        let expected = GameObject::new("Thing")
            .with(transform)
            .with(Component::empty(ENTRANCE));
        assert_eq!(obj, expected);

        // The game upgrades the bodies of older versions, so they're kept.
        let old = with_version(Component::new(entrance(7)), 3);
        let mut obj = GameObject::new("Thing").with(old.clone());
        obj.minimize(&defaults);
        assert_eq!(obj.components[0], old);
        let old = with_version(Component::transform(Transform::default()), 1);
        let mut obj = GameObject::new("Thing").with(old.clone());
        obj.minimize(&defaults);
        assert_eq!(obj.components[0], old);
    }

    fn renderer(materials: &[(&str, &[(&str, f32)])]) -> Component {
        let materials = materials
            .iter()
            .map(|(name, colors)| MaterialColors {
                material: Some((*name).into()),
                colors: colors
                    .iter()
                    .map(|(name, v)| MaterialColor {
                        name: Some((*name).into()),
                        color: Some([*v; 4]),
                    })
                    .collect(),
            })
            .collect();
        Component::new(MeshRenderer {
            materials,
            ..MeshRenderer::default()
        })
    }

    #[test]
    fn materials_are_read_by_name() {
        let mut defaults = PrefabDefaults::default();
        let lamp = renderer(&[
            ("M1", &[("_Color", 0.1), ("_Emit", 0.2)]),
            ("M2", &[("_Color", 0.3)]),
        ]);
        defaults.insert(full("Lamp", [0.0; 3]).with(lamp.clone()), false);

        let file = renderer(&[
            ("M2", &[("_Color", 0.5)]),
            ("M3", &[]),
            ("M1", &[("_Spec", 0.6)]),
        ]);
        let mut obj = GameObject::new("Lamp").with(with_guid(file, 4));
        let skipped = |material: &str, color: Option<&str>| Unresolved::Material {
            object: "Lamp".into(),
            material: material.into(),
            color: color.map(DString::from),
            guid: 4,
        };
        let expected = [skipped("M3", None), skipped("M1", Some("_Spec"))];
        assert_eq!(obj.resolve(&defaults), expected);
        let resolved = renderer(&[
            ("M1", &[("_Color", 0.1), ("_Emit", 0.2), ("_Spec", 0.6)]),
            ("M2", &[("_Color", 0.5)]),
            ("M3", &[]),
        ]);
        assert_eq!(obj.components[1], with_guid(resolved, 4));
        // On an object whose materials a component replaces, without any of the renderer's
        // materials, the file's are taken to be the renderer's.
        let file = with_guid(renderer(&[("M3", &[("_Color", 0.5)])]), 4);
        let mut obj = GameObject::new("Lamp").with(file.clone());
        assert_eq!(obj.resolve(&defaults), [skipped("M3", None)]);
        let resolved = renderer(&[
            ("M1", &[("_Color", 0.1), ("_Emit", 0.2)]),
            ("M2", &[("_Color", 0.3)]),
            ("M3", &[("_Color", 0.5)]),
        ]);
        assert_eq!(obj.components[1], with_guid(resolved, 4));
        let particles = Component::empty(ComponentId::PARTICLES_GPU);
        let glow = full("Glow", [0.0; 3]).with(lamp).with(particles);
        defaults.insert(glow, false);
        let mut obj = GameObject::new("Glow").with(file.clone());
        assert_eq!(obj.resolve(&defaults), []);
        assert_eq!(obj.components[1], file);

        // Only the colors that differ are kept.
        let file = renderer(&[
            ("M1", &[("_Color", 0.1), ("_Emit", 0.7)]),
            ("M2", &[("_Color", 0.3)]),
        ]);
        let mut obj = GameObject::new("Lamp").with(file);
        obj.minimize(&defaults);
        let expected = renderer(&[("M1", &[("_Emit", 0.7)])]);
        assert_eq!(obj.components[0], expected);
        let file = renderer(&[("M2", &[("_Color", 0.3)]), ("M1", &[("_Color", 0.1)])]);
        let mut obj = GameObject::new("Lamp").with(file);
        obj.minimize(&defaults);
        let expected = Component::empty(ComponentId::MESH_RENDERER);
        assert_eq!(obj.components[0], expected);
        // With materials the renderer doesn't have, all of the renderer's are kept.
        let file = renderer(&[("M2", &[("_Color", 0.3)]), ("M3", &[])]);
        let mut obj = GameObject::new("Lamp").with(file);
        obj.minimize(&defaults);
        let expected = renderer(&[("M2", &[]), ("M3", &[])]);
        assert_eq!(obj.components[0], expected);
        // A material that comes twice is read twice.
        let twice = renderer(&[("M2", &[("_Color", 0.3)]), ("M2", &[("_Color", 0.4)])]);
        let mut obj = GameObject::new("Lamp").with(twice.clone());
        obj.minimize(&defaults);
        assert_eq!(obj.components[0], twice);
    }

    #[test]
    fn minimize_keeps_what_resolve_gives() {
        let defaults = defaults();
        let mut level = Level::new("Test");
        let group = GameObject::new("Group")
            .with_child(thing().with_child(GameObject::new("A").with_scale([3.0; 3])))
            .with_child(GameObject::new("Road").with(TrackSegment::default()));
        level.layers[0].objects = vec![
            thing().with(TeleporterExit { link_id: Some(1) }),
            GameObject::new("Thing").with_position([1.0; 3]),
            group,
            GameObject::new("Nothing").with_position([1.0; 3]),
        ];
        level.assign_guids();
        let mut resolved = level.clone();
        // The repeated child, the subclass and the object without a prefab.
        let unresolved = resolved.resolve(&defaults);
        assert_eq!(unresolved.len(), 3, "{unresolved:?}");
        // The level settings object is resolved against `LevelSettings`, and keeps its name.
        let settings = |level: &Level| match &level.settings {
            LevelSettingsData::GameObject(obj) => obj.clone(),
            _ => unreachable!(),
        };
        assert_eq!(transform(&settings(&resolved)).scale, Some([1.0; 3]));
        assert_eq!(
            settings(&resolved).components[1],
            settings(&level).components[1]
        );

        let mut minimized = level.clone();
        minimized.minimize(&defaults);
        let size = |level: &Level| BytesFile::Level(level.clone()).write().len();
        assert!(size(&minimized) < size(&level));
        let mut again = minimized.clone();
        again.resolve(&defaults);
        assert_eq!(again, resolved);
        let mut twice = minimized.clone();
        twice.minimize(&defaults);
        assert_eq!(twice, minimized);
    }
}
