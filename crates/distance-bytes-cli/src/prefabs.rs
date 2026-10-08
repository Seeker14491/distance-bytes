//! Dumping prefab defaults from the game, and resolving and minimizing files with them.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use base64::Engine;
use distance_bytes::components::MaterialColors;
use distance_bytes::{
    BytesFile, ComponentData, ComponentId, DString, GameObject, PrefabDefaults, Unresolved, json,
};
use serde::Deserialize;

/// A line of the BytesTrace plugin's `/prefab-defaults` response.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum Dumped {
    Prefab {
        name: String,
        tag: Option<String>,
        /// Children tagged `PrefabContainer`.
        #[serde(default)]
        containers: Vec<String>,
        bytes: Option<String>,
        error: Option<String>,
    },
    Added {
        #[serde(rename = "type")]
        type_name: String,
        id: Option<i32>,
        bytes: Option<String>,
        error: Option<String>,
    },
}

/// Dumps the prefab defaults from the game: the prefabs the game lists, and those of the
/// objects in the files under `paths` that the game creates from the prefab of their name.
pub fn dump(server: &str, paths: &[PathBuf], out: &Path) -> Result<()> {
    let names = post(server, "prefab-names", b"")?;
    let mut names: BTreeSet<String> = names.lines().map(str::to_owned).collect();
    let mut defaults = PrefabDefaults::default();
    let mut failed = Vec::new();
    fetch(server, &names, &mut defaults, &mut failed)?;

    // The objects of files, and the children of objects whose prefab is a container.
    let mut more = BTreeSet::new();
    for path in super::binary_files(paths)? {
        let data = std::fs::read(&path)?;
        let Ok(BytesFile::Level(level)) = BytesFile::read(&data) else {
            continue;
        };
        let mut stack: Vec<&GameObject> = level.layers.iter().flat_map(|l| &l.objects).collect();
        while let Some(obj) = stack.pop() {
            let name = obj.name.to_string_lossy().into_owned();
            if defaults.get(&name).is_some_and(|p| p.container) {
                stack.extend(obj.children());
            }
            if !names.contains(&name) {
                more.insert(name);
            }
        }
    }
    if !more.is_empty() {
        fetch(server, &more, &mut defaults, &mut failed)?;
        names.extend(more);
    }

    for (name, error) in &failed {
        eprintln!("{name}: {error}");
    }
    std::fs::write(out, defaults.write())?;
    eprintln!(
        "{} prefabs ({} names failed) and {} added component types",
        defaults.prefabs().count(),
        failed.len(),
        defaults.added_components().count()
    );
    Ok(())
}

fn post(server: &str, endpoint: &str, body: &[u8]) -> Result<String> {
    Ok(ureq::post(&format!("{server}/{endpoint}"))
        .send(body)?
        .body_mut()
        .with_config()
        .limit(u64::MAX)
        .read_to_string()?)
}

fn fetch(
    server: &str,
    names: &BTreeSet<String>,
    defaults: &mut PrefabDefaults,
    failed: &mut Vec<(String, String)>,
) -> Result<()> {
    let body = names.iter().map(|n| format!("{n}\n")).collect::<String>();
    let response = post(server, "prefab-defaults", body.as_bytes())?;
    for line in response.lines().filter(|l| !l.is_empty()) {
        let dumped: Dumped =
            serde_json::from_str(line).with_context(|| format!("bad line: {line:.200}"))?;
        match dumped {
            Dumped::Prefab {
                name,
                tag,
                containers,
                bytes,
                error,
            } => {
                let Some(bytes) = bytes else {
                    failed.push((name, error.unwrap_or_default()));
                    continue;
                };
                if !containers.is_empty() {
                    bail!("{name} has children tagged PrefabContainer: {containers:?}");
                }
                let mut obj = object(&bytes).with_context(|| name.clone())?;
                // Prefabs the game builds from templates are named after the base prefab.
                obj.name = name.as_str().into();
                defaults.insert(obj, tag.as_deref() == Some("PrefabContainer"));
            }
            Dumped::Added {
                type_name,
                id,
                bytes,
                error,
            } => {
                let (Some(id), Some(bytes)) = (id, bytes) else {
                    failed.push((type_name, error.unwrap_or_default()));
                    continue;
                };
                let id = ComponentId(id);
                let obj = object(&bytes).with_context(|| type_name.clone())?;
                let c = obj
                    .components
                    .iter()
                    .find(|c| c.id() == Some(id))
                    .with_context(|| format!("{type_name} has no {id}"))?;
                defaults.insert_added(c.clone());
            }
        }
    }
    Ok(())
}

fn object(base64: &str) -> Result<GameObject> {
    let data = base64::engine::general_purpose::STANDARD.decode(base64)?;
    match BytesFile::read(&data)? {
        BytesFile::GameObject(obj) => Ok(obj),
        BytesFile::Level(_) => bail!("a level, not an object"),
    }
}

fn read_defaults(path: &Path) -> Result<PrefabDefaults> {
    let data = std::fs::read(path).with_context(|| path.display().to_string())?;
    PrefabDefaults::read(&data).with_context(|| path.display().to_string())
}

fn read_file(path: &Path) -> Result<BytesFile> {
    let data = std::fs::read(path)?;
    let file = if super::is_binary(&data) {
        BytesFile::read(&data)?
    } else {
        json::from_slice(&data)?
    };
    Ok(file)
}

fn write_file(path: &Path, file: &BytesFile) -> Result<()> {
    let json = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("json"));
    let data = if json {
        let mut v = json::to_string_pretty(file)?.into_bytes();
        v.push(b'\n');
        v
    } else {
        file.write()
    };
    std::fs::write(path, data)?;
    Ok(())
}

/// Checks on the binary files under `paths` that minimizing keeps what resolving gives, both for
/// a file and for its resolved form, and counts what the game skips.
pub fn check(paths: &[PathBuf], defaults: &Path) -> Result<()> {
    use rayon::prelude::*;
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    let defaults = read_defaults(defaults)?;
    let files = super::binary_files(paths)?;
    let skipped = Mutex::new(BTreeMap::<String, usize>::new());
    let failures = Mutex::new(Vec::new());
    let sizes = Mutex::new([0usize; 3]);
    files.par_iter().for_each(|path| {
        let Ok(data) = std::fs::read(path) else {
            return;
        };
        let Ok(file) = BytesFile::read(&data) else {
            return;
        };
        let mut resolved = file.clone();
        let mut repeated = false;
        for unresolved in resolved.resolve(&defaults) {
            use distance_bytes::Unresolved as U;
            let key = match unresolved {
                U::Prefab { name, .. } => format!("prefab {name}"),
                U::Child { parent, name, .. } => format!("child {parent}/{name}"),
                U::Component {
                    object, component, ..
                } => format!("component {object}.{component}"),
                U::Subclass {
                    object,
                    component,
                    found,
                    ..
                } => format!("subclass {object}.{component} as {found}"),
                U::Repeated {
                    object, component, ..
                } => {
                    repeated = true;
                    match component {
                        Some(component) => format!("repeated {object}.{component}"),
                        None => format!("repeated child {object}"),
                    }
                }
                U::Material {
                    object,
                    material,
                    color,
                    ..
                } => match color {
                    Some(color) => format!("color {object}.{material}.{color}"),
                    None => format!("material {object}.{material}"),
                },
            };
            *skipped.lock().unwrap().entry(key).or_default() += 1;
        }
        let fail = |what: &str| {
            failures
                .lock()
                .unwrap()
                .push(format!("{}: {what}", path.display()))
        };
        // Compared as written, since a NaN isn't equal to itself.
        let resolved_bytes = resolved.write();
        let mut minimized = file.clone();
        minimized.minimize(&defaults);
        let minimized_bytes = minimized.write();
        let mut again = minimized.clone();
        again.resolve(&defaults);
        if again.write() != resolved_bytes {
            fail("resolving the minimized file gives something else");
        }
        let mut again = resolved.clone();
        again.minimize(&defaults);
        again.resolve(&defaults);
        if again.write() != resolved_bytes {
            fail(if repeated {
                "repeats: resolving the minimized resolved file gives something else"
            } else {
                "resolving the minimized resolved file gives something else"
            });
        }
        if BytesFile::read(&minimized_bytes).map(|f| f.write()).ok()
            != Some(minimized_bytes.clone())
        {
            fail("the minimized file doesn't read back");
        }
        let mut s = sizes.lock().unwrap();
        s[0] += data.len();
        s[1] += minimized_bytes.len();
        s[2] += resolved_bytes.len();
    });
    let skipped = skipped.into_inner()?;
    let total: usize = skipped.values().sum();
    println!("the game skips {total} things:");
    let mut by_count: Vec<_> = skipped.into_iter().collect();
    by_count.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    for (key, n) in by_count.iter().take(40) {
        println!("{n:>8}  {key}");
    }
    let [original, minimized, resolved] = sizes.into_inner()?;
    println!("bytes: {original} original, {minimized} minimized, {resolved} resolved");
    let mut failures = failures.into_inner()?;
    failures.sort();
    for f in failures.iter().take(30) {
        println!("{f}");
    }
    println!("{} files checked, {} failed", files.len(), failures.len());
    Ok(())
}

/// Compares resolving the levels under `paths` with what the game has after reading them, which
/// the BytesTrace plugin writes with every value. GUIDs aren't compared, nor the bodies of
/// components in other versions than the game's, which the game upgrades.
pub fn game_check(paths: &[PathBuf], defaults: &Path, server: &str, max: usize) -> Result<()> {
    let defaults = read_defaults(defaults)?;
    let files = super::binary_files(paths)?;
    // Objects nest hundreds deep, and comparing them recurses.
    std::thread::scope(|s| {
        std::thread::Builder::new()
            .stack_size(512 << 20)
            .spawn_scoped(s, || game_check_files(&files, &defaults, server, max))?
            .join()
            .unwrap()
    })
}

fn game_check_files(
    files: &[PathBuf],
    defaults: &PrefabDefaults,
    server: &str,
    max: usize,
) -> Result<()> {
    use std::collections::BTreeMap;

    let mut tally = BTreeMap::<String, usize>::new();
    let mut upgraded = BTreeMap::<String, usize>::new();
    let (mut checked, mut failed_loads, mut differing) = (0, 0, 0);
    for path in files {
        let data = std::fs::read(path)?;
        let Ok(BytesFile::Level(level)) = BytesFile::read(&data) else {
            continue;
        };
        let response = ureq::post(&format!("{server}/level-bytes-full"))
            .config()
            .http_status_as_error(false)
            .build()
            .send(&data)?;
        if response.status() != 200 {
            println!("{}: the game didn't load it", path.display());
            failed_loads += 1;
            continue;
        }
        let game = response
            .into_body()
            .with_config()
            .limit(u64::MAX)
            .read_to_vec()?;
        let BytesFile::Level(game) = BytesFile::read(&game)? else {
            bail!("the game wrote an object, not a level");
        };
        checked += 1;
        let mut ours = level;
        let mut skipped = HashSet::new();
        let mut materials = HashMap::<u32, Vec<_>>::new();
        for unresolved in ours.resolve(defaults) {
            match unresolved {
                Unresolved::Component { guid, .. } | Unresolved::Child { guid, .. } => {
                    skipped.insert(guid);
                }
                Unresolved::Material {
                    material,
                    color,
                    guid,
                    ..
                } => materials.entry(guid).or_default().push((material, color)),
                _ => {}
            }
        }
        skipped.remove(&0);
        materials.remove(&0);
        let mut diffs = Diffs {
            defaults,
            found: Vec::new(),
            upgraded: Vec::new(),
            guids: HashMap::new(),
            skipped,
            materials,
            mapping: true,
        };
        // First match the game's GUIDs, which it renumbers, to ours.
        diffs.level(&game, &ours);
        diffs.mapping = false;
        diffs.level(&game, &ours);
        for key in diffs.upgraded.drain(..) {
            *upgraded.entry(key).or_default() += 1;
        }
        if !diffs.found.is_empty() {
            differing += 1;
            println!("{}: {} differences", path.display(), diffs.found.len());
            for (_, detail) in diffs.found.iter().take(max) {
                println!("  {detail}");
            }
        }
        for (key, _) in diffs.found {
            *tally.entry(key).or_default() += 1;
        }
    }
    for (title, tally) in [
        ("differences by kind:", tally),
        (
            "not compared: upgraded, rounded or fixed by the game, or skipped as resolving says:",
            upgraded,
        ),
    ] {
        let mut by_count: Vec<_> = tally.into_iter().collect();
        by_count.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        println!("{title}");
        for (key, n) in by_count.iter().take(60) {
            println!("{n:>8}  {key}");
        }
    }
    println!(
        "{checked} levels compared, {differing} with differences, {failed_loads} the game didn't load"
    );
    Ok(())
}

/// What sets an object's renderer when the game reads the object, which resolving leaves as the
/// file or the prefab has it.
#[derive(Clone, Copy)]
enum RendererSetBy {
    /// A `GoldenSimples` preset, which sets the colors (`GoldenSimples.CreateNewMaterial`).
    Preset,
    /// A `TextMesh` font, which sets the material (`TextMeshWrapper.SetFont`), with the
    /// `TextMesh` color as its `_Color`.
    Font(i32),
}

/// Differences between what the game has after reading a level and what resolving it gives, as
/// `(kind, detail)`.
struct Diffs<'a> {
    defaults: &'a PrefabDefaults,
    found: Vec<(String, String)>,
    /// What isn't compared: components the game upgrades, values it rounds or fixes, and what
    /// it skips.
    upgraded: Vec<String>,
    /// The game's GUIDs to ours.
    guids: HashMap<u32, u32>,
    /// The GUIDs of the components and children that resolving reports the game skips.
    skipped: HashSet<u32>,
    /// The materials and colors of `MeshRenderer`s, by GUID, that resolving reports the game
    /// skips, as `(material, color)`.
    materials: HashMap<u32, Vec<(DString, Option<DString>)>>,
    /// Whether this pass only matches GUIDs.
    mapping: bool,
}

impl Diffs<'_> {
    fn add(&mut self, key: String, detail: String) {
        if !self.mapping {
            self.found.push((key, detail));
        }
    }

    /// Whether resolving reports that the game skips the component or child of a GUID, which
    /// is then counted as not compared.
    fn skip(&mut self, guid: u32, key: impl FnOnce() -> String) -> bool {
        let skipped = self.skipped.contains(&guid);
        if skipped && !self.mapping {
            self.upgraded.push(format!("skipped {}", key()));
        }
        skipped
    }

    fn level(&mut self, game: &distance_bytes::Level, ours: &distance_bytes::Level) {
        // The level settings object isn't compared: the plugin reuses it from level to level, so
        // that it keeps values a level leaves empty from the previous one.
        let mut layers = &game.layers[..];
        // The game adds the background layer of "ultra new level" to levels older than version 4,
        // before their own.
        if ours.version < 4
            && layers.len() == ours.layers.len() + 1
            && layers[0].name == "Background"
        {
            layers = &layers[1..];
            if !self.mapping {
                self.upgraded.push("background layer added".into());
            }
        }
        for (gl, ol) in layers.iter().zip(&ours.layers) {
            // The game skips objects without a prefab.
            let ours: Vec<_> = ol
                .objects
                .iter()
                .filter(|o| self.defaults.get(&o.name.to_string_lossy()).is_some())
                .collect();
            if gl.objects.len() != ours.len() {
                self.add(
                    "object count".into(),
                    format!(
                        "layer {}: {} objects, ours {}",
                        gl.name,
                        gl.objects.len(),
                        ours.len()
                    ),
                );
            }
            for (i, (g, o)) in gl.objects.iter().zip(ours).enumerate() {
                // The game names an object made from a spline road template after the road,
                // such as `SplineRoad` for `SplineRoadStraight`.
                if g.name != o.name
                    && o.name
                        .to_string_lossy()
                        .starts_with(&*g.name.to_string_lossy())
                {
                    if !self.mapping {
                        self.upgraded.push(format!("named {} {}", g.name, o.name));
                    }
                } else if g.name != o.name {
                    self.add(
                        "object order".into(),
                        format!(
                            "layer {} object {i}: {} vs ours {}",
                            gl.name, g.name, o.name
                        ),
                    );
                    break;
                }
                self.object(&format!("{}[{i}]", g.name), g, o);
            }
        }
    }

    fn object(&mut self, path: &str, game: &GameObject, ours: &GameObject) {
        if self.mapping {
            self.guids.insert(game.guid, ours.guid);
        }
        let set_by = ours.components.iter().find_map(|c| match &c.body {
            distance_bytes::ComponentBody::Data { data, .. } => match data {
                ComponentData::GoldenSimples(g) if g.preset.is_some_and(|p| p != 0) => {
                    Some(RendererSetBy::Preset)
                }
                ComponentData::TextMesh(t) => t.font.map(RendererSetBy::Font),
                _ => None,
            },
            _ => None,
        });
        let mut used = vec![false; ours.components.len()];
        for gc in &game.components {
            let found = ours
                .components
                .iter()
                .enumerate()
                .position(|(i, o)| !used[i] && o.id() == gc.id() && gc.id().is_some());
            let id = gc.id().map_or("unknown".into(), |id| id.to_string());
            match found {
                Some(i) => {
                    used[i] = true;
                    if self.mapping {
                        self.guids.insert(gc.guid, ours.components[i].guid);
                    }
                    self.component(
                        &format!("{path}.{id}"),
                        &game.name,
                        gc,
                        &ours.components[i],
                        set_by,
                    );
                }
                None => self.add(
                    format!("missing component {}.{id}", game.name),
                    format!("{path}: no {id}"),
                ),
            }
        }
        for (i, oc) in ours.components.iter().enumerate() {
            if !used[i] {
                let id = oc.id().map_or("unknown".into(), |id| id.to_string());
                if self.skip(oc.guid, || format!("{}.{id}", ours.name)) {
                    continue;
                }
                self.add(
                    format!("extra component {}.{id}", ours.name),
                    format!("{path}: extra {id}"),
                );
            }
        }
    }

    fn component(
        &mut self,
        path: &str,
        object: &distance_bytes::DString,
        game: &distance_bytes::Component,
        ours: &distance_bytes::Component,
        set_by: Option<RendererSetBy>,
    ) {
        use distance_bytes::{ComponentBody as B, ComponentKind as K, FieldValue};
        let version = |c: &distance_bytes::Component| match c.kind {
            K::Known { version, .. } => version,
            K::Unknown { .. } => 0,
        };
        let id = game.id().map_or("unknown".into(), |id| id.to_string());
        let key = format!("{object}.{id}");
        // Without the materials and colors that resolving reports the game skips.
        let stripped;
        let ours = match self.materials.get(&ours.guid) {
            Some(skipped) if !self.mapping => {
                for (material, _) in skipped {
                    self.upgraded
                        .push(format!("skipped material {object}.{material}"));
                }
                stripped = without_materials(ours, skipped);
                &stripped
            }
            _ => ours,
        };
        // With the game's references renumbered to our GUIDs.
        let mut game = game.clone();
        if !self.mapping {
            game.references_mut(|guid| {
                if let Some(&ours) = self.guids.get(guid) {
                    *guid = ours;
                }
            });
            // Unity fills an empty material slot with its default material when the game reads
            // colors into a renderer.
            if let (
                B::Data {
                    data: ComponentData::MeshRenderer(g),
                    ..
                },
                B::Data {
                    data: ComponentData::MeshRenderer(o),
                    ..
                },
            ) = (&mut game.body, &ours.body)
            {
                let standard =
                    |m: &MaterialColors| m.material.as_ref().is_some_and(|n| *n == "Standard");
                if !o.materials.iter().any(standard) && g.materials.iter().any(standard) {
                    g.materials.retain(|m| !standard(m));
                    self.upgraded
                        .push(format!("empty material slot filled {object}"));
                }
            }
        }
        let game = &game;
        match (&game.body, &ours.body) {
            (B::Transform(g), B::Transform(o)) => {
                let values = |t: &distance_bytes::Transform| {
                    [
                        ("position", t.position.map(|v| v.to_vec())),
                        ("rotation", t.rotation.map(|v| v.to_vec())),
                        ("scale", t.scale.map(|v| v.to_vec())),
                    ]
                };
                for ((name, g), (_, o)) in values(g).into_iter().zip(values(o)) {
                    let bits = |v: &Option<Vec<f32>>| {
                        v.as_ref()
                            .map(|v| v.iter().map(|f| f.to_bits()).collect::<Vec<_>>())
                    };
                    if bits(&g) == bits(&o) {
                        continue;
                    }
                    // The game normalizes rotations, which changes their last bits.
                    let close = match (&g, &o) {
                        (Some(g), Some(o)) => g
                            .iter()
                            .zip(o)
                            .all(|(a, b)| (a - b).abs() <= 4.0 * f32::EPSILON * a.abs().max(1e-3)),
                        _ => false,
                    };
                    if close && name == "rotation" {
                        self.upgraded.push(format!("rotation normalized {object}"));
                        continue;
                    }
                    if let Some(o) = &o
                        && bits(&g) == bits(&Some(sanitize(name, o)))
                    {
                        self.upgraded.push(format!("{name} made valid {object}"));
                        continue;
                    }
                    self.add(
                        format!("transform {name} {object}"),
                        format!("{path}.{name}: {g:?}, ours {o:?}"),
                    );
                }
                let mut used = vec![false; o.children.len()];
                for gc in &g.children {
                    let found = o
                        .children
                        .iter()
                        .enumerate()
                        .position(|(i, oc)| !used[i] && oc.name == gc.name);
                    match found {
                        Some(i) => {
                            used[i] = true;
                            self.object(&format!("{path}/{}", gc.name), gc, &o.children[i]);
                        }
                        None => self.add(
                            format!("missing child {object}/{}", gc.name),
                            format!("{path}: no child {}", gc.name),
                        ),
                    }
                }
                for (i, oc) in o.children.iter().enumerate() {
                    if !used[i] {
                        if self.skip(oc.guid, || format!("child {object}/{}", oc.name)) {
                            continue;
                        }
                        self.add(
                            format!("extra child {object}/{}", oc.name),
                            format!("{path}: extra child {}", oc.name),
                        );
                    }
                }
            }
            _ if self.mapping => {}
            _ if version(game) != version(ours) => {
                self.upgraded.push(format!("{key} v{}", version(ours)));
            }
            (B::Fields(g), B::Fields(o)) => {
                for gf in g {
                    let of = o.iter().find(|f| f.name == gf.name);
                    let same = match (&gf.value, of.map(|f| &f.value)) {
                        (FieldValue::Value(a), Some(FieldValue::Value(b))) => {
                            json::to_string(a).ok() == json::to_string(b).ok()
                        }
                        (a, b) => Some(a) == b,
                    };
                    if !same {
                        self.add(
                            format!("field {key}.{}", gf.name),
                            format!(
                                "{path}.{}: {:?}, ours {:?}",
                                gf.name,
                                gf.value,
                                of.map(|f| &f.value)
                            ),
                        );
                    }
                }
            }
            (B::Data { data: gd, .. }, B::Data { data: od, .. }) => {
                let as_json = |c: &distance_bytes::Component| {
                    let text = json::to_string(c).unwrap_or_default();
                    let value: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
                    value.get("data").cloned().unwrap_or_default()
                };
                let (g, o) = (as_json(game), as_json(ours));
                if g != o {
                    if let Some(why) = set_on_read(gd, od, set_by) {
                        self.upgraded.push(format!("{why} {object}"));
                        return;
                    }
                    let keys: BTreeSet<&String> = g
                        .as_object()
                        .into_iter()
                        .chain(o.as_object())
                        .flat_map(|m| m.keys())
                        .collect();
                    for k in keys {
                        // The game clamps spot angles to 150 degrees
                        // (`LightWrapper.maxSpotlightAngle_`).
                        let clamped = game.id() == Some(ComponentId::LIGHT)
                            && k == "spot_angle"
                            && g.get(k).and_then(|v| v.as_f64()) == Some(150.0)
                            && o.get(k).and_then(|v| v.as_f64()).is_some_and(|v| v > 150.0);
                        // The game sets an animation curve's pong values from its other values
                        // when they aren't custom (`AnimatorBase.VisitCurve`).
                        let pong = k == "curve"
                            && o[k].get("custom_pong_values") == Some(&false.into())
                            && g.get(k).map(without_pong) == o.get(k).map(without_pong);
                        if clamped {
                            self.upgraded.push(format!("spot angle clamped {object}"));
                        } else if pong {
                            self.upgraded
                                .push(format!("pong values follow the curve {object}"));
                        } else if g.get(k) != o.get(k) {
                            self.add(
                                format!("value {key}.{k}"),
                                format!(
                                    "{path}.{k}: {:.120}, ours {:.120}",
                                    g.get(k).map_or("-".into(), |v| v.to_string()),
                                    o.get(k).map_or("-".into(), |v| v.to_string())
                                ),
                            );
                        }
                    }
                }
            }
            (B::Raw(_), _) | (_, B::Raw(_)) | (B::Empty, B::Empty) => {}
            (g, o) => self.add(
                format!("body {key}"),
                format!("{path}: {} body, ours {}", body_kind(g), body_kind(o)),
            ),
        }
    }
}

/// What the game sets when it reads a body, which resolving leaves as the file or the prefab has
/// it, if that's why `game` differs from `ours`.
fn set_on_read(
    game: &ComponentData,
    ours: &ComponentData,
    set_by: Option<RendererSetBy>,
) -> Option<&'static str> {
    match (game, ours) {
        // The game applies a preset to the other values, and then resets it.
        (ComponentData::GoldenSimples(g), ComponentData::GoldenSimples(o))
            if o.preset.is_some_and(|p| p != 0) && g.preset == Some(0) =>
        {
            Some("preset applied")
        }
        (ComponentData::MeshRenderer(g), ComponentData::MeshRenderer(_)) => match set_by? {
            RendererSetBy::Preset => Some("colors set by preset"),
            RendererSetBy::Font(font) => {
                let name = if font == 1 {
                    "DenseBoldFontMat"
                } else {
                    "PetitaBoldFontMat"
                };
                matches!(&g.materials[..], [m] if m.material.as_ref().is_some_and(|n| *n == name))
                    .then_some("material set by font")
            }
        },
        _ => None,
    }
}

/// An animation curve without its pong values.
fn without_pong(curve: &serde_json::Value) -> serde_json::Value {
    let mut curve = curve.clone();
    if let Some(values) = curve.as_object_mut() {
        values.retain(|k, _| !k.starts_with("pong_"));
    }
    curve
}

/// A `MeshRenderer` without the materials and colors that resolving appends as skipped.
fn without_materials(
    c: &distance_bytes::Component,
    skipped: &[(DString, Option<DString>)],
) -> distance_bytes::Component {
    use distance_bytes::{ComponentBody as B, ComponentData};
    let mut c = c.clone();
    if let B::Data {
        data: ComponentData::MeshRenderer(m),
        ..
    } = &mut c.body
    {
        // Each was appended after those before it.
        for (material, color) in skipped.iter().rev() {
            let named = |o: &MaterialColors| o.material.as_ref() == Some(material);
            match color {
                None => {
                    if let Some(i) = m.materials.iter().rposition(named) {
                        m.materials.remove(i);
                    }
                }
                Some(color) => {
                    if let Some(o) = m.materials.iter_mut().find(|o| named(o))
                        && let Some(j) = o
                            .colors
                            .iter()
                            .rposition(|c| c.name.as_ref() == Some(color))
                    {
                        o.colors.remove(j);
                    }
                }
            }
        }
    }
    c
}

/// A `Transform` value as the game sets it when it reads a file (`TransformWrapper.Visit`):
/// infinite or NaN values become the identity, and a scale under 1e-5 its absolute value, at
/// least 1e-5.
fn sanitize(name: &str, value: &[f32]) -> Vec<f32> {
    // A rotation's `w` isn't checked.
    let valid = value.iter().take(3).all(|v| v.is_finite());
    match name {
        "position" if !valid => vec![0.0; 3],
        "rotation" if !valid => vec![0.0, 0.0, 0.0, 1.0],
        "scale" if !valid => vec![1.0; 3],
        "scale" if value.iter().any(|&v| v < 1e-5) => {
            value.iter().map(|v| v.abs().max(1e-5)).collect()
        }
        _ => value.to_vec(),
    }
}

fn body_kind(body: &distance_bytes::ComponentBody) -> &'static str {
    use distance_bytes::ComponentBody as B;
    match body {
        B::Empty => "empty",
        B::Transform(_) => "transform",
        B::Fields(_) => "fields",
        B::Data { .. } => "typed",
        B::Raw(_) => "raw",
    }
}

/// Resolves or minimizes a binary or JSON file, writing JSON if the output ends in `.json`.
pub fn resolve(input: &Path, output: &Path, defaults: &Path, minimize: bool) -> Result<()> {
    let defaults = read_defaults(defaults)?;
    let mut file = read_file(input).with_context(|| input.display().to_string())?;
    if minimize {
        file.minimize(&defaults);
    } else {
        for unresolved in file.resolve(&defaults) {
            eprintln!("warning: {unresolved}");
        }
    }
    write_file(output, &file)
}
