//! Round-trips every binary `.bytes` file under the directories listed in the
//! `DISTANCE_BYTES_CORPUS` environment variable (separated like `PATH`), directly and through
//! JSON. Skipped if unset.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use distance_bytes::io::mark;
use distance_bytes::{BytesFile, json};

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
    let text = match json::to_string(&file) {
        Ok(text) => text,
        Err(e) => return Some(format!("to JSON: {e}")),
    };
    match json::from_str::<BytesFile>(&text) {
        Err(e) => Some(format!("from JSON: {e}")),
        Ok(back) if back.write() != data => Some("output through JSON differs".into()),
        Ok(_) => None,
    }
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
            s.spawn(|| {
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
            });
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
    eprintln!("{checked} files round-tripped, directly and through JSON");
}
