use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use distance_bytes::io::mark;
use distance_bytes::json;
use distance_bytes::{
    BodyEnd, BytesFile, ComponentBody, ComponentData, ComponentId, ComponentKind, FieldValue,
    GameObject, LegacyLevelSettings, LegacySettingsBody, LevelSettingsData,
};
use rayon::prelude::*;

mod trace;

#[derive(Parser)]
#[command(about = "Tools for Distance .bytes files")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print the object tree of a file.
    Dump { file: PathBuf },
    /// Read and re-write every binary .bytes file under the given paths and check that the
    /// output is byte-identical.
    Roundtrip {
        paths: Vec<PathBuf>,
        /// Convert each file to JSON and back before writing it.
        #[arg(long)]
        json: bool,
    },
    /// Convert a .bytes file to JSON.
    ToJson {
        input: PathBuf,
        /// The JSON file to write; standard output if omitted.
        output: Option<PathBuf>,
        /// Write the JSON on one line.
        #[arg(long)]
        compact: bool,
    },
    /// Convert JSON written by `to-json` (or by hand) to a .bytes file.
    FromJson { input: PathBuf, output: PathBuf },
    /// Print statistics about the binary .bytes files under the given paths.
    Stats { paths: Vec<PathBuf> },
    /// Print every value our decoder reads from a file, as JSON lines.
    Trace { file: PathBuf },
    /// Re-write a level with a longer name, which moves everything after the name. Used to test
    /// that offsets inside the file are recomputed on write.
    Relocate {
        input: PathBuf,
        output: PathBuf,
        /// Number of characters to append to the level name.
        #[arg(long, default_value_t = 1)]
        pad: usize,
    },
    /// For every (component, version) pair that a hand-written serializer decodes, print the
    /// smallest files containing it, as `<component> v<version>\t<file>` lines. Used to pick
    /// files for `trace-check`.
    Coverage {
        paths: Vec<PathBuf>,
        /// Number of files to print per pair.
        #[arg(long, default_value_t = 2)]
        per: usize,
    },
    /// Write files for `trace-check` that have the decoded component versions (and legacy level
    /// settings versions) no file under the paths has. Each is a copy of the smallest file with
    /// that component, whose first one is replaced by a body of that version with every value
    /// set.
    Synth {
        /// The directory to write the files to.
        #[arg(long)]
        out: PathBuf,
        paths: Vec<PathBuf>,
    },
    /// Compare our decoder against the game's own reads, using the BytesTrace plugin.
    TraceCheck {
        paths: Vec<PathBuf>,
        #[arg(long, default_value = "http://localhost:18501")]
        server: String,
        /// Maximum mismatches to print per file.
        #[arg(long, default_value_t = 10)]
        max: usize,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Dump { file } => dump(&file),
        Command::Roundtrip { paths, json } => roundtrip(&paths, json),
        Command::ToJson {
            input,
            output,
            compact,
        } => to_json(&input, output.as_deref(), compact),
        Command::FromJson { input, output } => from_json(&input, &output),
        Command::Stats { paths } => stats(&paths),
        Command::Trace { file } => trace::print_trace(&file),
        Command::Relocate { input, output, pad } => relocate(&input, &output, pad),
        Command::Coverage { paths, per } => coverage(&paths, per),
        Command::Synth { out, paths } => synth(&paths, &out),
        Command::TraceCheck { paths, server, max } => {
            let files: Vec<_> = binary_files(&paths)?
                .into_iter()
                .filter(|p| std::fs::read(p).is_ok_and(|d| is_binary(&d)))
                .collect();
            trace::trace_check(&files, &server, max)
        }
    }
}

fn dump(path: &Path) -> Result<()> {
    let data = std::fs::read(path)?;
    let file = BytesFile::read(&data).with_context(|| path.display().to_string())?;
    match &file {
        BytesFile::Level(level) => {
            println!(
                "Level {:?} (version {})",
                level.name.to_string_lossy(),
                level.version
            );
            match &level.settings {
                LevelSettingsData::GameObject(obj) => dump_object(obj, 1),
                LevelSettingsData::Legacy { version, body } => match body {
                    LegacySettingsBody::Data { data, end } => {
                        println!(
                            "  LegacySettings version {version}: {}",
                            describe_data(data, end)
                        )
                    }
                    LegacySettingsBody::Raw(b) => {
                        println!("  LegacySettings version {version}: {} bytes", b.len())
                    }
                },
            }
            for layer in &level.layers {
                println!(
                    "  Layer {:?} {:?} ({} objects)",
                    layer.name.to_string_lossy(),
                    layer.flags,
                    layer.objects.len()
                );
                for obj in &layer.objects {
                    dump_object(obj, 2);
                }
            }
        }
        BytesFile::GameObject(obj) => dump_object(obj, 0),
    }
    Ok(())
}

fn relocate(input: &Path, output: &Path, pad: usize) -> Result<()> {
    let data = std::fs::read(input)?;
    let mut file = BytesFile::read(&data).with_context(|| input.display().to_string())?;
    let BytesFile::Level(level) = &mut file else {
        anyhow::bail!("{} is not a level", input.display());
    };
    level.name = format!("{}{}", level.name.to_string_lossy(), "_".repeat(pad)).into();
    let out = file.write();
    std::fs::write(output, &out)?;
    println!(
        "wrote {} ({} bytes, was {})",
        output.display(),
        out.len(),
        data.len()
    );
    Ok(())
}

fn dump_object(obj: &GameObject, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{indent}{} #{}", obj.name.to_string_lossy(), obj.guid);
    for c in &obj.components {
        let kind = match &c.kind {
            ComponentKind::Known { id, version, .. } => format!("{id} v{version}"),
            ComponentKind::Unknown { name } => format!("unknown {:?}", name.to_string_lossy()),
        };
        let body = match &c.body {
            ComponentBody::Empty => "empty".to_owned(),
            ComponentBody::Transform(t) => {
                format!(
                    "pos {:?} rot {:?} scale {:?}",
                    t.position, t.rotation, t.scale
                )
            }
            ComponentBody::Fields(fields) => {
                let fields: Vec<_> = fields
                    .iter()
                    .map(|f| match &f.value {
                        FieldValue::Empty => format!("{} empty", f.name),
                        FieldValue::Value(v) => format!("{} = {v:?}", f.name),
                        FieldValue::Raw(b) => format!("{} raw ({} bytes)", f.name, b.len()),
                    })
                    .collect();
                format!("fields {}", fields.join(", "))
            }
            ComponentBody::Data { data, end } => describe_data(data, end),
            ComponentBody::Raw(b) => format!("{} bytes", b.len()),
        };
        println!("{indent}  - {kind}: {body}");
    }
    for child in obj.children() {
        dump_object(child, depth + 1);
    }
}

fn describe_data(data: &impl std::fmt::Debug, end: &BodyEnd) -> String {
    match end {
        BodyEnd::Complete => format!("{data:?}"),
        BodyEnd::Truncated(n) => format!("{data:?} (truncated after {n} values)"),
        BodyEnd::Trailing(b) => format!("{data:?} (+{} trailing bytes)", b.len()),
    }
}

/// Finds files with a `.bytes` extension whose contents start with a binary root mark.
fn binary_files(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    fn walk(path: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        if path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                walk(&entry?.path(), out)?;
            }
        } else if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("bytes"))
        {
            out.push(path.to_owned());
        }
        Ok(())
    }
    let mut files = Vec::new();
    for p in paths {
        walk(p, &mut files)?;
    }
    files.sort();
    Ok(files)
}

fn is_binary(data: &[u8]) -> bool {
    let head = data
        .get(..4)
        .map(|b| i32::from_le_bytes(b.try_into().unwrap()));
    matches!(head, Some(mark::LEVEL | mark::GAME_OBJECT))
}

fn to_json(input: &Path, output: Option<&Path>, compact: bool) -> Result<()> {
    let data = std::fs::read(input)?;
    let file = BytesFile::read(&data).with_context(|| input.display().to_string())?;
    let json = if compact {
        json::to_vec(&file)?
    } else {
        let mut v = json::to_string_pretty(&file)?.into_bytes();
        v.push(b'\n');
        v
    };
    match output {
        Some(path) => std::fs::write(path, json)?,
        None => std::io::Write::write_all(&mut std::io::stdout().lock(), &json)?,
    }
    Ok(())
}

fn from_json(input: &Path, output: &Path) -> Result<()> {
    let text = std::fs::read(input)?;
    let file: BytesFile = json::from_slice(&text).with_context(|| input.display().to_string())?;
    std::fs::write(output, file.write())?;
    Ok(())
}

/// Converts a file to JSON and back.
fn through_json(file: &BytesFile) -> Result<BytesFile> {
    let text = json::to_string(file).context("to JSON")?;
    json::from_str(&text).context("from JSON")
}

fn roundtrip(paths: &[PathBuf], via_json: bool) -> Result<()> {
    let files = binary_files(paths)?;
    let failures = Mutex::new(Vec::new());
    let checked = files
        .par_iter()
        .map(|path| -> Result<usize> {
            let data = std::fs::read(path)?;
            if !is_binary(&data) {
                return Ok(0);
            }
            let read = BytesFile::read(&data).map_err(anyhow::Error::from);
            let read = if via_json {
                read.and_then(|f| through_json(&f))
            } else {
                read
            };
            let problem = match read {
                Err(e) => Some(format!("error: {e:#}")),
                Ok(file) => {
                    let out = file.write();
                    (out != data).then(|| {
                        let at = out
                            .iter()
                            .zip(&data)
                            .position(|(a, b)| a != b)
                            .unwrap_or(out.len().min(data.len()));
                        format!(
                            "output differs at {at:#x} (len {} vs {})",
                            out.len(),
                            data.len()
                        )
                    })
                }
            };
            if let Some(p) = problem {
                failures
                    .lock()
                    .unwrap()
                    .push(format!("{}: {p}", path.display()));
            }
            Ok(1)
        })
        .sum::<Result<usize>>()?;
    let mut failures = failures.into_inner().unwrap();
    failures.sort();
    for f in &failures {
        println!("FAIL {f}");
    }
    println!("{checked} binary files checked, {} failed", failures.len());
    if failures.is_empty() {
        Ok(())
    } else {
        std::process::exit(1)
    }
}

fn coverage(paths: &[PathBuf], per: usize) -> Result<()> {
    fn pairs(obj: &GameObject, out: &mut Vec<(i32, i32)>) {
        for c in &obj.components {
            if let (ComponentKind::Known { id, version, .. }, ComponentBody::Data { .. }) =
                (&c.kind, &c.body)
            {
                out.push((id.0, *version));
            }
        }
        for child in obj.children() {
            pairs(child, out);
        }
    }

    let files = binary_files(paths)?;
    // (id, version) -> files containing it, as (size, path)
    let found = Mutex::new(BTreeMap::<(i32, i32), Vec<(usize, &Path)>>::new());
    files.par_iter().try_for_each(|path| -> Result<()> {
        let data = std::fs::read(path)?;
        if !is_binary(&data) {
            return Ok(());
        }
        let mut local = Vec::new();
        match BytesFile::read(&data) {
            Err(_) => {}
            Ok(BytesFile::Level(level)) => {
                if let LevelSettingsData::GameObject(obj) = &level.settings {
                    pairs(obj, &mut local);
                }
                for obj in level.layers.iter().flat_map(|l| &l.objects) {
                    pairs(obj, &mut local);
                }
            }
            Ok(BytesFile::GameObject(obj)) => pairs(&obj, &mut local),
        }
        local.sort_unstable();
        local.dedup();
        let mut found = found.lock().unwrap();
        for pair in local {
            found.entry(pair).or_default().push((data.len(), path));
        }
        Ok(())
    })?;
    for ((id, version), mut files) in found.into_inner().unwrap() {
        files.sort();
        for (_, path) in files.iter().take(per) {
            println!(
                "{} v{version}\t{}",
                distance_bytes::ComponentId(id),
                path.display()
            );
        }
    }
    Ok(())
}

fn synth(paths: &[PathBuf], out: &Path) -> Result<()> {
    fn components(obj: &GameObject, out: &mut Vec<(Option<ComponentId>, i32, bool)>) {
        for c in &obj.components {
            if let ComponentKind::Known { id, version, .. } = c.kind {
                out.push((
                    Some(id),
                    version,
                    matches!(c.body, ComponentBody::Data { .. }),
                ));
            }
        }
        for child in obj.children() {
            components(child, out);
        }
    }
    fn replace(
        obj: &mut GameObject,
        id: ComponentId,
        version: i32,
        data: &mut Option<ComponentData>,
    ) -> bool {
        for c in &mut obj.components {
            if let ComponentKind::Known {
                id: found,
                version: v,
                ..
            } = &mut c.kind
                && *found == id
            {
                *v = version;
                c.body = ComponentBody::Data {
                    data: data.take().unwrap(),
                    end: BodyEnd::Complete,
                };
                return true;
            }
        }
        obj.components.iter_mut().any(|c| match &mut c.body {
            ComponentBody::Transform(t) => t
                .children
                .iter_mut()
                .any(|child| replace(child, id, version, data)),
            _ => false,
        })
    }

    #[derive(Default)]
    struct Seen {
        /// The smallest file with the component, as (size, path).
        host: Option<(usize, PathBuf)>,
        /// The versions decoded in some file.
        decoded: BTreeSet<i32>,
    }
    let files = binary_files(paths)?;
    // By component, with `None` for legacy level settings.
    let seen = Mutex::new(BTreeMap::<Option<ComponentId>, Seen>::new());
    files.par_iter().try_for_each(|path| -> Result<()> {
        let data = std::fs::read(path)?;
        if !is_binary(&data) {
            return Ok(());
        }
        let mut found = Vec::new();
        match BytesFile::read(&data) {
            Err(_) => {}
            Ok(BytesFile::Level(level)) => {
                match &level.settings {
                    LevelSettingsData::GameObject(obj) => components(obj, &mut found),
                    LevelSettingsData::Legacy { version, body } => found.push((
                        None,
                        *version,
                        matches!(body, LegacySettingsBody::Data { .. }),
                    )),
                }
                for obj in level.layers.iter().flat_map(|l| &l.objects) {
                    components(obj, &mut found);
                }
            }
            Ok(BytesFile::GameObject(obj)) => components(&obj, &mut found),
        }
        let mut seen = seen.lock().unwrap();
        for (id, version, decoded) in found {
            let s = seen.entry(id).or_default();
            if s.host
                .as_ref()
                .is_none_or(|(size, p)| (data.len(), path) < (*size, p))
            {
                s.host = Some((data.len(), path.clone()));
            }
            if decoded {
                s.decoded.insert(version);
            }
        }
        Ok(())
    })?;

    std::fs::create_dir_all(out)?;
    let mut written = 0;
    for (id, s) in seen.into_inner().unwrap() {
        let (_, host) = s.host.expect("every seen component has a host");
        let file = BytesFile::read(&std::fs::read(&host)?)?;
        let Some(id) = id else {
            // The game reads legacy level settings above version 8 the same as version 8.
            for version in (0..=8).filter(|v| !s.decoded.contains(v)) {
                let mut file = file.clone();
                let BytesFile::Level(level) = &mut file else {
                    unreachable!()
                };
                let data = Box::new(LegacyLevelSettings::sample(version));
                level.settings = LevelSettingsData::Legacy {
                    version,
                    body: LegacySettingsBody::Data {
                        data,
                        end: BodyEnd::Complete,
                    },
                };
                std::fs::write(
                    out.join(format!("LegacyLevelSettings_v{version}.bytes")),
                    file.write(),
                )?;
                written += 1;
            }
            continue;
        };
        for version in 0..=64 {
            if s.decoded.contains(&version) {
                continue;
            }
            let Some(sample) = ComponentData::sample(id, version) else {
                continue;
            };
            let mut file = file.clone();
            let mut data = Some(sample);
            let replaced = match &mut file {
                BytesFile::Level(level) => {
                    let settings = match &mut level.settings {
                        LevelSettingsData::GameObject(obj) => Some(obj),
                        LevelSettingsData::Legacy { .. } => None,
                    };
                    settings
                        .into_iter()
                        .chain(level.layers.iter_mut().flat_map(|l| &mut l.objects))
                        .any(|obj| replace(obj, id, version, &mut data))
                }
                BytesFile::GameObject(obj) => replace(obj, id, version, &mut data),
            };
            assert!(replaced, "{id} not found in {}", host.display());
            std::fs::write(out.join(format!("{id}_v{version}.bytes")), file.write())?;
            written += 1;
        }
    }
    println!("{written} files written");
    Ok(())
}

#[derive(Default)]
struct ComponentStats {
    count: usize,
    empty: usize,
    /// Bodies decoded as generic fields.
    fields: usize,
    /// Bodies decoded by a hand-written serializer.
    data: usize,
    /// Decoded bodies with fewer values than their version reads.
    truncated: usize,
    /// Decoded bodies with bytes after the values their version reads.
    trailing: usize,
    raw: usize,
    raw_bytes: usize,
    /// Field values that didn't decode as their type, and their bytes.
    raw_values: usize,
    raw_value_bytes: usize,
    typed_values: usize,
}

impl ComponentStats {
    fn add(&mut self, other: &ComponentStats) {
        self.count += other.count;
        self.empty += other.empty;
        self.fields += other.fields;
        self.data += other.data;
        self.truncated += other.truncated;
        self.trailing += other.trailing;
        self.raw += other.raw;
        self.raw_bytes += other.raw_bytes;
        self.raw_values += other.raw_values;
        self.raw_value_bytes += other.raw_value_bytes;
        self.typed_values += other.typed_values;
    }
}

#[derive(Default)]
struct Stats {
    levels: BTreeMap<i32, usize>,
    object_files: usize,
    errors: Vec<String>,
    /// Keyed by (mark, id, version).
    components: BTreeMap<(i32, i32, i32), ComponentStats>,
    unknown_components: BTreeMap<String, usize>,
    /// (component id, field name, value length) -> count
    raw_field_values: BTreeMap<(i32, String, usize), usize>,
    transforms_decoded: usize,
    transforms_raw: usize,
    legacy_settings_decoded: usize,
    legacy_settings_raw: usize,
    raw_bodies_with_object_mark: usize,
    objects: usize,
}

impl Stats {
    fn object(&mut self, obj: &GameObject) {
        self.objects += 1;
        for c in &obj.components {
            if let ComponentBody::Raw(b) = &c.body {
                let needle = mark::GAME_OBJECT.to_le_bytes();
                if b.windows(4).any(|w| w == needle) {
                    self.raw_bodies_with_object_mark += 1;
                }
            }
            match &c.kind {
                ComponentKind::Known { mark, id, version } => {
                    let e = self.components.entry((*mark, id.0, *version)).or_default();
                    e.count += 1;
                    match &c.body {
                        ComponentBody::Empty => e.empty += 1,
                        ComponentBody::Transform(_) => {}
                        ComponentBody::Data { end, .. } => {
                            e.data += 1;
                            match end {
                                BodyEnd::Complete => {}
                                BodyEnd::Truncated(_) => e.truncated += 1,
                                BodyEnd::Trailing(_) => e.trailing += 1,
                            }
                        }
                        ComponentBody::Fields(fields) => {
                            e.fields += 1;
                            for f in fields {
                                match &f.value {
                                    FieldValue::Empty => {}
                                    FieldValue::Value(_) => e.typed_values += 1,
                                    FieldValue::Raw(b) => {
                                        e.raw_values += 1;
                                        e.raw_value_bytes += b.len();
                                        let key = (id.0, f.name.to_string(), b.len());
                                        *self.raw_field_values.entry(key).or_default() += 1;
                                    }
                                }
                            }
                        }
                        ComponentBody::Raw(b) => {
                            e.raw += 1;
                            e.raw_bytes += b.len();
                        }
                    }
                    if *id == distance_bytes::ComponentId::TRANSFORM {
                        match c.body {
                            ComponentBody::Transform(_) => self.transforms_decoded += 1,
                            ComponentBody::Raw(_) => self.transforms_raw += 1,
                            _ => {}
                        }
                    }
                }
                ComponentKind::Unknown { name } => {
                    *self.unknown_components.entry(name.to_string()).or_default() += 1;
                }
            }
        }
        for child in obj.children() {
            self.object(child);
        }
    }

    fn merge(&mut self, other: Stats) {
        for (k, v) in other.levels {
            *self.levels.entry(k).or_default() += v;
        }
        self.object_files += other.object_files;
        self.errors.extend(other.errors);
        for (k, v) in &other.components {
            self.components.entry(*k).or_default().add(v);
        }
        for (k, v) in other.unknown_components {
            *self.unknown_components.entry(k).or_default() += v;
        }
        for (k, v) in other.raw_field_values {
            *self.raw_field_values.entry(k).or_default() += v;
        }
        self.transforms_decoded += other.transforms_decoded;
        self.transforms_raw += other.transforms_raw;
        self.legacy_settings_decoded += other.legacy_settings_decoded;
        self.legacy_settings_raw += other.legacy_settings_raw;
        self.raw_bodies_with_object_mark += other.raw_bodies_with_object_mark;
        self.objects += other.objects;
    }
}

fn stats(paths: &[PathBuf]) -> Result<()> {
    let files = binary_files(paths)?;
    let stats = Mutex::new(Stats::default());
    files.par_iter().try_for_each(|path| -> Result<()> {
        let data = std::fs::read(path)?;
        if !is_binary(&data) {
            return Ok(());
        }
        let mut local = Stats::default();
        match BytesFile::read(&data) {
            Err(e) => local.errors.push(format!("{}: {e}", path.display())),
            Ok(BytesFile::Level(level)) => {
                *local.levels.entry(level.version).or_default() += 1;
                match &level.settings {
                    LevelSettingsData::GameObject(obj) => local.object(obj),
                    LevelSettingsData::Legacy {
                        body: LegacySettingsBody::Data { .. },
                        ..
                    } => local.legacy_settings_decoded += 1,
                    LevelSettingsData::Legacy {
                        body: LegacySettingsBody::Raw(_),
                        ..
                    } => local.legacy_settings_raw += 1,
                }
                for layer in &level.layers {
                    for obj in &layer.objects {
                        local.object(obj);
                    }
                }
            }
            Ok(BytesFile::GameObject(obj)) => {
                local.object_files += 1;
                local.object(&obj);
            }
        }
        stats.lock().unwrap().merge(local);
        Ok(())
    })?;

    let s = stats.into_inner().unwrap();
    let mut total = ComponentStats::default();
    for v in s.components.values() {
        total.add(v);
    }
    println!("levels by version: {:?}", s.levels);
    println!("object files: {}", s.object_files);
    println!("read errors: {}", s.errors.len());
    for e in &s.errors {
        println!("  {e}");
    }
    println!("game objects: {}", s.objects);
    println!(
        "transforms: {} decoded, {} raw",
        s.transforms_decoded, s.transforms_raw
    );
    println!(
        "legacy level settings: {} decoded, {} raw",
        s.legacy_settings_decoded, s.legacy_settings_raw
    );
    println!(
        "generic field bodies: {}, with {} typed values and {} raw values ({} bytes)",
        total.fields, total.typed_values, total.raw_values, total.raw_value_bytes
    );
    println!("raw field values by component, field and length:");
    let mut raw_fields: Vec<_> = s.raw_field_values.iter().collect();
    raw_fields.sort_by_key(|(_, n)| std::cmp::Reverse(**n));
    for ((id, name, len), n) in raw_fields {
        let id = distance_bytes::ComponentId(*id);
        let ty = distance_bytes::field_type(id, name);
        println!("  {n:>8}  {id}.{name} ({len} bytes), type {ty:?}");
    }
    println!(
        "bodies decoded by hand-written serializers: {} ({} truncated, {} with trailing bytes)",
        total.data, total.truncated, total.trailing
    );
    println!("raw bodies: {} ({} bytes)", total.raw, total.raw_bytes);
    println!(
        "raw bodies containing a game object mark: {}",
        s.raw_bodies_with_object_mark
    );
    println!("unknown components: {:?}", s.unknown_components);
    println!(
        "components ({} kinds of (mark, id, version)):",
        s.components.len()
    );
    let mut rows: Vec<_> = s.components.iter().collect();
    rows.sort_by_key(|(_, v)| std::cmp::Reverse(v.raw_bytes + v.raw_value_bytes));
    println!(
        "  {:>10} {:>9} {:>9} {:>7} {:>7} {:>9} {:>9} {:>12} {:>9} {:>9} {:>9}  component",
        "count",
        "empty",
        "data",
        "short",
        "long",
        "fields",
        "raw",
        "raw bytes",
        "typed",
        "raw vals",
        "raw val b"
    );
    for ((m, id, version), v) in rows {
        let kind = match *m {
            mark::SERIAL_COMPONENT => "",
            mark::SERIALIZABLE => " [wrapper]",
            _ => " [alt mark]",
        };
        println!(
            "  {:>10} {:>9} {:>9} {:>7} {:>7} {:>9} {:>9} {:>12} {:>9} {:>9} {:>9}  {} v{version}{kind}",
            v.count,
            v.empty,
            v.data,
            v.truncated,
            v.trailing,
            v.fields,
            v.raw,
            v.raw_bytes,
            v.typed_values,
            v.raw_values,
            v.raw_value_bytes,
            distance_bytes::ComponentId(*id)
        );
    }
    Ok(())
}
