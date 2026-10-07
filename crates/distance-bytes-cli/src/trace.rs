//! Comparing our decoder against traces from the game (the BytesTrace plugin).

use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use distance_bytes::{BytesFile, TraceEvent, TraceValue};
use serde::Deserialize;

/// One line of the game's trace.
#[derive(Debug, Deserialize)]
pub struct GameEvent {
    #[serde(default)]
    pub o: usize,
    #[serde(default)]
    pub l: usize,
    pub t: String,
    #[serde(default)]
    pub n: Option<String>,
    #[serde(default)]
    pub v: serde_json::Value,
    #[serde(default)]
    pub mark: Option<i32>,
    #[serde(default)]
    pub len: Option<i64>,
    #[serde(default)]
    pub end: Option<usize>,
    #[serde(default)]
    pub ok: Option<bool>,
    /// Read past the end of its scope; the game seeked back afterwards.
    #[serde(default)]
    pub overread: bool,
}

pub fn event_json(e: &TraceEvent) -> String {
    let (t, v) = match &e.value {
        TraceValue::Scope { mark, len } => {
            return format!(r#"{{"o":{},"l":{},"t":"scope","mark":{mark},"len":{len}}}"#, e.offset, e.len);
        }
        TraceValue::Int(v) => ("int", v.to_string()),
        TraceValue::UInt(v) => ("uint", v.to_string()),
        TraceValue::Short(v) => ("short", v.to_string()),
        TraceValue::UShort(v) => ("ushort", v.to_string()),
        TraceValue::Long(v) => ("long", v.to_string()),
        TraceValue::Float(bits) => ("float", (*bits as i32).to_string()),
        TraceValue::Double(bits) => ("double", (*bits as i64).to_string()),
        TraceValue::Char(v) => ("char", v.to_string()),
        TraceValue::Bool(v) => ("bool", v.to_string()),
        TraceValue::String(s) => ("string", serde_json::to_string(s).unwrap()),
        TraceValue::Empty => ("empty", "null".to_owned()),
    };
    format!(r#"{{"o":{},"l":{},"t":"{t}","v":{v}}}"#, e.offset, e.len)
}

/// Prints our trace of a file in the same format as the game's.
pub fn print_trace(path: &Path) -> Result<()> {
    let data = std::fs::read(path)?;
    let (result, events) = BytesFile::read_traced(&data);
    for e in &events {
        println!("{}", event_json(e));
    }
    result.with_context(|| path.display().to_string())?;
    Ok(())
}

fn matches(ours: &TraceEvent, game: &GameEvent) -> bool {
    if ours.len != game.l {
        return false;
    }
    let v = &game.v;
    // Values read through the game's generic reader are typed by the C# type of the field
    // (e.g. "generic:System.Single" or an enum), so only their value is compared.
    let is = |t: &str| game.t == t || game.t.starts_with("generic:");
    match &ours.value {
        TraceValue::Scope { mark, len } => {
            game.t == "scope" && game.mark == Some(*mark) && game.len == Some(*len)
        }
        TraceValue::Int(x) => is("int") && v.as_i64() == Some(i64::from(*x)),
        TraceValue::UInt(x) => is("uint") && v.as_u64() == Some(u64::from(*x)),
        TraceValue::Short(x) => is("short") && v.as_i64() == Some(i64::from(*x)),
        TraceValue::UShort(x) => is("ushort") && v.as_u64() == Some(u64::from(*x)),
        TraceValue::Long(x) => is("long") && v.as_i64() == Some(*x),
        TraceValue::Float(bits) => is("float") && v.as_i64() == Some(i64::from(*bits as i32)),
        TraceValue::Double(bits) => is("double") && v.as_i64() == Some(*bits as i64),
        TraceValue::Char(x) => is("char") && v.as_u64() == Some(u64::from(*x)),
        TraceValue::Bool(x) => is("bool") && v.as_bool() == Some(*x),
        TraceValue::String(s) => is("string") && v.as_str() == Some(s),
        TraceValue::Empty => game.t == "empty",
    }
}

fn describe(e: &GameEvent) -> String {
    let mut s = format!("{} {:?}", e.t, e.n.as_deref().unwrap_or(""));
    match (&e.mark, &e.len) {
        (Some(mark), Some(len)) => write!(s, " mark {mark} len {len}").unwrap(),
        _ => write!(s, " = {}", e.v).unwrap(),
    }
    s
}

pub struct CheckReport {
    pub game_ok: bool,
    pub mismatches: Vec<String>,
    pub not_read_by_game: usize,
    pub game_skips: Vec<String>,
    pub game_bytes: usize,
    pub decoded_game_bytes: usize,
}

/// Compares our trace of `data` with the game's.
pub fn check(data: &[u8], game: &[GameEvent], max_mismatches: usize) -> Result<CheckReport> {
    let (result, ours) = BytesFile::read_traced(data);
    result?;

    let mut by_offset: HashMap<usize, Vec<&GameEvent>> = HashMap::new();
    let mut game_ok = false;
    let mut game_skips = Vec::new();
    // Bytes read by the game, and the subset that we decode too.
    let mut game_read = vec![false; data.len()];
    for e in game {
        match e.t.as_str() {
            "result" => game_ok = e.ok == Some(true),
            "skip" => {
                let end = e.end.unwrap_or(0);
                let what = if e.o > end { "overstepped, back" } else { "skipped" };
                game_skips.push(format!("{:#x}: {what} to {end:#x}", e.o));
            }
            _ if e.overread => {}
            _ => {
                by_offset.entry(e.o).or_default().push(e);
                let end = (e.o + e.l).min(data.len());
                game_read[e.o.min(end)..end].fill(true);
            }
        }
    }

    let mut mismatches = Vec::new();
    let mut not_read_by_game = 0;
    let mut decoded = vec![false; data.len()];
    for ours in &ours {
        match by_offset.get(&ours.offset) {
            None => not_read_by_game += 1,
            Some(candidates) => {
                if candidates.iter().any(|g| matches(ours, g)) {
                    decoded[ours.offset..ours.offset + ours.len].fill(true);
                } else if mismatches.len() < max_mismatches {
                    let theirs: Vec<_> = candidates.iter().map(|g| describe(g)).collect();
                    mismatches.push(format!(
                        "{:#x}: ours {} | game {}",
                        ours.offset,
                        event_json(ours),
                        theirs.join(" / ")
                    ));
                } else {
                    mismatches.push(String::new());
                }
            }
        }
    }

    let game_bytes = game_read.iter().filter(|&&b| b).count();
    let decoded_game_bytes = game_read.iter().zip(&decoded).filter(|&(&g, &d)| g && d).count();
    Ok(CheckReport { game_ok, mismatches, not_read_by_game, game_skips, game_bytes, decoded_game_bytes })
}

pub fn fetch_game_trace(server: &str, data: &[u8]) -> Result<Vec<GameEvent>> {
    let endpoint = match data.get(..4).map(|b| i32::from_le_bytes(b.try_into().unwrap())) {
        Some(distance_bytes::io::mark::LEVEL) => "level-bytes-trace",
        Some(distance_bytes::io::mark::GAME_OBJECT) => "gameobject-bytes-trace",
        _ => bail!("not a binary .bytes file"),
    };
    let body = ureq::post(&format!("{server}/{endpoint}"))
        .send(data)?
        .body_mut()
        .with_config()
        .limit(u64::MAX)
        .read_to_string()?;
    body.lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str(l).with_context(|| format!("bad trace line: {l}")))
        .collect()
}

pub fn trace_check(files: &[PathBuf], server: &str, max_mismatches: usize) -> Result<()> {
    let (mut total_game, mut total_decoded, mut failed) = (0usize, 0usize, 0usize);
    for path in files {
        let data = std::fs::read(path)?;
        let game = fetch_game_trace(server, &data).with_context(|| path.display().to_string())?;
        let report = check(&data, &game, max_mismatches).with_context(|| path.display().to_string())?;
        total_game += report.game_bytes;
        total_decoded += report.decoded_game_bytes;
        let pct = 100.0 * report.decoded_game_bytes as f64 / report.game_bytes.max(1) as f64;
        let ok = report.mismatches.is_empty() && report.game_ok;
        let status = if ok { "ok  " } else { "FAIL" };
        failed += usize::from(!ok);
        println!(
            "{status} {} — game load {}, {} mismatches, {:.1}% of game-read bytes decoded, {} values not read by game, {} game skips",
            path.display(),
            if report.game_ok { "ok" } else { "FAILED" },
            report.mismatches.len(),
            pct,
            report.not_read_by_game,
            report.game_skips.len(),
        );
        for m in report.mismatches.iter().filter(|m| !m.is_empty()) {
            println!("    {m}");
        }
        for s in report.game_skips.iter().take(max_mismatches) {
            println!("    game {s}");
        }
    }
    println!(
        "{} files, {failed} failed (mismatches or game load failure); {:.2}% of all game-read bytes decoded",
        files.len(),
        100.0 * total_decoded as f64 / total_game.max(1) as f64
    );
    if failed > 0 { std::process::exit(1) } else { Ok(()) }
}
