//! Round-trips every binary `.bytes` file under the directories listed in the
//! `DISTANCE_BYTES_CORPUS` environment variable (separated like `PATH`). Skipped if unset.

use std::path::{Path, PathBuf};

use distance_bytes::BytesFile;
use distance_bytes::io::mark;

fn collect(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_dir() {
        for entry in std::fs::read_dir(path).unwrap() {
            collect(&entry.unwrap().path(), out);
        }
    } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("bytes")) {
        out.push(path.to_owned());
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

    let mut checked = 0;
    let mut failures = Vec::new();
    for path in &files {
        let data = std::fs::read(path).unwrap();
        let head = data.get(..4).map(|b| i32::from_le_bytes(b.try_into().unwrap()));
        if !matches!(head, Some(mark::LEVEL | mark::GAME_OBJECT)) {
            continue; // XML or other non-binary files
        }
        checked += 1;
        match BytesFile::read(&data) {
            Err(e) => failures.push(format!("{}: {e}", path.display())),
            Ok(file) if file.write() != data => {
                failures.push(format!("{}: output differs", path.display()))
            }
            Ok(_) => {}
        }
    }
    assert!(checked > 0, "no binary .bytes files found in DISTANCE_BYTES_CORPUS");
    assert!(failures.is_empty(), "{} of {checked} files failed:\n{}", failures.len(), failures.join("\n"));
    eprintln!("{checked} files round-tripped");
}
