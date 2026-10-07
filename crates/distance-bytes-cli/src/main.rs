use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use distance_bytes::io::mark;
use distance_bytes::{BytesFile, ComponentBody, ComponentKind, GameObject, LevelSettings};
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
    Roundtrip { paths: Vec<PathBuf> },
    /// Print statistics about the binary .bytes files under the given paths.
    Stats { paths: Vec<PathBuf> },
    /// Print every value our decoder reads from a file, as JSON lines.
    Trace { file: PathBuf },
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
        Command::Roundtrip { paths } => roundtrip(&paths),
        Command::Stats { paths } => stats(&paths),
        Command::Trace { file } => trace::print_trace(&file),
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
            println!("Level {:?} (version {})", level.name.to_string_lossy(), level.version);
            match &level.settings {
                LevelSettings::GameObject(obj) => dump_object(obj, 1),
                LevelSettings::Legacy { version, data } => {
                    println!("  LegacySettings version {version} ({} bytes)", data.len())
                }
            }
            for layer in &level.layers {
                println!("  Layer {:?} {:?} ({} objects)", layer.name.to_string_lossy(), layer.flags, layer.objects.len());
                for obj in &layer.objects {
                    dump_object(obj, 2);
                }
            }
        }
        BytesFile::GameObject(obj) => dump_object(obj, 0),
    }
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
                format!("pos {:?} rot {:?} scale {:?}", t.position, t.rotation, t.scale)
            }
            ComponentBody::Raw(b) => format!("{} bytes", b.len()),
        };
        println!("{indent}  - {kind}: {body}");
    }
    for child in obj.children() {
        dump_object(child, depth + 1);
    }
}

/// Finds files with a `.bytes` extension whose contents start with a binary root mark.
fn binary_files(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    fn walk(path: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        if path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                walk(&entry?.path(), out)?;
            }
        } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("bytes")) {
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
    let head = data.get(..4).map(|b| i32::from_le_bytes(b.try_into().unwrap()));
    matches!(head, Some(mark::LEVEL | mark::GAME_OBJECT))
}

fn roundtrip(paths: &[PathBuf]) -> Result<()> {
    let files = binary_files(paths)?;
    let failures = Mutex::new(Vec::new());
    let checked = files
        .par_iter()
        .map(|path| -> Result<usize> {
            let data = std::fs::read(path)?;
            if !is_binary(&data) {
                return Ok(0);
            }
            let problem = match BytesFile::read(&data) {
                Err(e) => Some(format!("read error {e}")),
                Ok(file) => {
                    let out = file.write();
                    (out != data).then(|| {
                        let at = out.iter().zip(&data).position(|(a, b)| a != b).unwrap_or(out.len().min(data.len()));
                        format!("output differs at {at:#x} (len {} vs {})", out.len(), data.len())
                    })
                }
            };
            if let Some(p) = problem {
                failures.lock().unwrap().push(format!("{}: {p}", path.display()));
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
    if failures.is_empty() { Ok(()) } else { std::process::exit(1) }
}

#[derive(Default)]
struct Stats {
    levels: BTreeMap<i32, usize>,
    object_files: usize,
    errors: Vec<String>,
    /// (mark, id, version) -> (count, empty bodies, total body bytes)
    components: BTreeMap<(i32, i32, i32), (usize, usize, usize)>,
    unknown_components: BTreeMap<String, usize>,
    transforms_decoded: usize,
    transforms_raw: usize,
    raw_bytes: usize,
    raw_bodies_with_object_mark: usize,
    objects: usize,
}

impl Stats {
    fn object(&mut self, obj: &GameObject) {
        self.objects += 1;
        for c in &obj.components {
            let body_len = match &c.body {
                ComponentBody::Raw(b) => {
                    self.raw_bytes += b.len();
                    let needle = mark::GAME_OBJECT.to_le_bytes();
                    if b.windows(4).any(|w| w == needle) {
                        self.raw_bodies_with_object_mark += 1;
                    }
                    b.len()
                }
                _ => 0,
            };
            match &c.kind {
                ComponentKind::Known { mark, id, version } => {
                    let e = self.components.entry((*mark, id.0, *version)).or_default();
                    e.0 += 1;
                    e.1 += usize::from(c.body == ComponentBody::Empty);
                    e.2 += body_len;
                    if *id == distance_bytes::ComponentId::TRANSFORM {
                        match c.body {
                            ComponentBody::Transform(_) => self.transforms_decoded += 1,
                            ComponentBody::Raw(_) => self.transforms_raw += 1,
                            ComponentBody::Empty => {}
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
}

fn stats(paths: &[PathBuf]) -> Result<()> {
    let files = binary_files(paths)?;
    let stats = Mutex::new(Stats::default());
    files.par_iter().try_for_each(|path| -> Result<()> {
        let data = std::fs::read(path)?;
        if !is_binary(&data) {
            return Ok(());
        }
        let file = BytesFile::read(&data);
        let mut local = Stats::default();
        match file {
            Err(e) => local.errors.push(format!("{}: {e}", path.display())),
            Ok(BytesFile::Level(level)) => {
                *local.levels.entry(level.version).or_default() += 1;
                if let LevelSettings::GameObject(obj) = &level.settings {
                    local.object(obj);
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
        let mut s = stats.lock().unwrap();
        for (k, v) in local.levels {
            *s.levels.entry(k).or_default() += v;
        }
        s.object_files += local.object_files;
        s.errors.extend(local.errors);
        for (k, v) in local.components {
            let e = s.components.entry(k).or_default();
            e.0 += v.0;
            e.1 += v.1;
            e.2 += v.2;
        }
        for (k, v) in local.unknown_components {
            *s.unknown_components.entry(k).or_default() += v;
        }
        s.transforms_decoded += local.transforms_decoded;
        s.transforms_raw += local.transforms_raw;
        s.raw_bytes += local.raw_bytes;
        s.raw_bodies_with_object_mark += local.raw_bodies_with_object_mark;
        s.objects += local.objects;
        Ok(())
    })?;

    let s = stats.into_inner().unwrap();
    println!("levels by version: {:?}", s.levels);
    println!("object files: {}", s.object_files);
    println!("read errors: {}", s.errors.len());
    for e in &s.errors {
        println!("  {e}");
    }
    println!("game objects: {}", s.objects);
    println!("transforms: {} decoded, {} raw", s.transforms_decoded, s.transforms_raw);
    println!("raw component bytes: {}", s.raw_bytes);
    println!("raw bodies containing a game object mark: {}", s.raw_bodies_with_object_mark);
    println!("unknown components: {:?}", s.unknown_components);
    println!("components ({} kinds of (mark, id, version)):", s.components.len());
    let mut rows: Vec<_> = s.components.iter().collect();
    rows.sort_by_key(|(_, v)| std::cmp::Reverse(v.2));
    println!("  {:>10} {:>9} {:>12}  component", "count", "empty", "raw bytes");
    for ((m, id, version), (count, empty, bytes)) in rows {
        let kind = match *m {
            mark::SERIAL_COMPONENT => "",
            mark::SERIALIZABLE => " [wrapper]",
            _ => " [alt mark]",
        };
        println!("  {count:>10} {empty:>9} {bytes:>12}  {} v{version}{kind}", distance_bytes::ComponentId(*id));
    }
    Ok(())
}
