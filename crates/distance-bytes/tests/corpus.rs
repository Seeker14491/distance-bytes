//! Round-trips every binary `.bytes` file under the directories listed in the
//! `DISTANCE_BYTES_CORPUS` environment variable (separated like `PATH`), directly and through
//! JSON, and with the `schema` feature validates the JSON against the full schema. Also checks
//! that minimizing each file against `data/prefab-defaults.bytes` keeps what resolving it gives.
//! Skipped if unset.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LazyLock, Mutex};

use distance_bytes::io::mark;
use distance_bytes::{BytesFile, PrefabDefaults, json};

static DEFAULTS: LazyLock<PrefabDefaults> = LazyLock::new(|| {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/prefab-defaults.bytes");
    PrefabDefaults::read(&std::fs::read(path).unwrap()).unwrap()
});

fn collect(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_dir() {
        for entry in std::fs::read_dir(path).unwrap() {
            collect(&entry.unwrap().path(), out);
        }
    } else if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("bytes"))
    {
        out.push(path.to_owned());
    }
}

/// Why a file doesn't round-trip, if it doesn't.
fn check(data: &[u8]) -> Option<String> {
    let file = match BytesFile::read(data) {
        Ok(file) => file,
        Err(e) => return Some(format!("read error {e}")),
    };
    if file.write() != data {
        return Some("output differs".into());
    }
    // Compared as written, since a NaN isn't equal to itself.
    let mut resolved = file.clone();
    resolved.resolve(&DEFAULTS);
    let mut minimized = file.clone();
    minimized.minimize(&DEFAULTS);
    let minimized_data = minimized.write();
    if BytesFile::read(&minimized_data).map(|f| f.write()).ok() != Some(minimized_data) {
        return Some("the minimized file doesn't read back".into());
    }
    minimized.resolve(&DEFAULTS);
    if minimized.write() != resolved.write() {
        return Some("resolving the minimized file gives something else".into());
    }
    let text = match json::to_string(&file) {
        Ok(text) => text,
        Err(e) => return Some(format!("to JSON: {e}")),
    };
    match json::from_str::<BytesFile>(&text) {
        Err(e) => return Some(format!("from JSON: {e}")),
        Ok(back) if back.write() != data => return Some("output through JSON differs".into()),
        Ok(_) => {}
    }
    #[cfg(feature = "schema")]
    if let Some(problem) = schema_errors(&text) {
        return Some(problem);
    }
    None
}

/// Where JSON doesn't fit the full schema, if it doesn't.
#[cfg(feature = "schema")]
fn schema_errors(text: &str) -> Option<String> {
    static FULL: std::sync::LazyLock<jsonschema::Validator> = std::sync::LazyLock::new(|| {
        let options = json::SchemaOptions {
            full: true,
            components: None,
        };
        jsonschema::validator_for(&json::schema(&options)).unwrap()
    });
    let mut de = serde_json::Deserializer::from_str(text);
    de.disable_recursion_limit();
    let value: serde_json::Value = serde::Deserialize::deserialize(&mut de).unwrap();
    if FULL.is_valid(&value) {
        return None;
    }
    let errors: Vec<_> = FULL
        .iter_errors(&value)
        .take(3)
        .map(|e| format!("{} (schema {})", e.instance_path(), e.schema_path()))
        .collect();
    Some(format!(
        "JSON doesn't fit the schema at {}",
        errors.join(", ")
    ))
}

#[test]
fn corpus_round_trip() {
    let Some(dirs) = std::env::var_os("DISTANCE_BYTES_CORPUS") else {
        eprintln!("DISTANCE_BYTES_CORPUS not set; skipping");
        return;
    };
    let mut files = Vec::new();
    for dir in std::env::split_paths(&dirs) {
        collect(&dir, &mut files);
    }

    let next = AtomicUsize::new(0);
    let checked = AtomicUsize::new(0);
    let failures = Mutex::new(Vec::new());
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    std::thread::scope(|s| {
        for _ in 0..threads {
            // Parsing and validating JSON recurses without growing the stack, and a level nests
            // 469 deep.
            let thread = std::thread::Builder::new().stack_size(64 << 20);
            thread
                .spawn_scoped(s, || {
                    while let Some(path) = files.get(next.fetch_add(1, Ordering::Relaxed)) {
                        let data = std::fs::read(path).unwrap();
                        let head = data
                            .get(..4)
                            .map(|b| i32::from_le_bytes(b.try_into().unwrap()));
                        if !matches!(head, Some(mark::LEVEL | mark::GAME_OBJECT)) {
                            continue; // XML or other non-binary files
                        }
                        checked.fetch_add(1, Ordering::Relaxed);
                        if let Some(problem) = check(&data) {
                            let failure = format!("{}: {problem}", path.display());
                            failures.lock().unwrap().push(failure);
                        }
                    }
                })
                .unwrap();
        }
    });

    let checked = checked.into_inner();
    let mut failures = failures.into_inner().unwrap();
    failures.sort();
    assert!(
        checked > 0,
        "no binary .bytes files found in DISTANCE_BYTES_CORPUS"
    );
    assert!(
        failures.is_empty(),
        "{} of {checked} files failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
    let validated = if cfg!(feature = "schema") {
        ", and validated against the schema"
    } else {
        ""
    };
    eprintln!(
        "{checked} files round-tripped, directly and through JSON{validated}; minimizing them \
         keeps what resolving gives"
    );
}
