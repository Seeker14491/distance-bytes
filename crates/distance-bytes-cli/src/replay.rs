//! Replays: their decoded states and events, and a check of them against the game's.

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use distance_bytes::io::Writer;
use distance_bytes::replay::{self, CarState, ReplayEvent};
use distance_bytes::{BytesFile, ComponentBody, ComponentData, ComponentKind, GameObject};
use serde::Serialize;
use serde_json::{Map, Value};

/// Writes the states of a replay as JSON lines, with the inputs unpacked, or with `events`
/// its events.
pub fn print(input: &Path, output: Option<&Path>, events: bool) -> Result<()> {
    #[derive(Serialize)]
    struct Line<'a> {
        #[serde(flatten)]
        state: &'a CarState,
        inputs: replay::Inputs,
    }
    let data = std::fs::read(input)?;
    let file = BytesFile::read(&data).with_context(|| input.display().to_string())?;
    let replay = match &file {
        BytesFile::GameObject(obj) => obj.car_replay_data(),
        _ => None,
    };
    let Some((replay, version)) = replay else {
        bail!("{} isn't a replay", input.display());
    };
    let mut out: Box<dyn Write> = match output {
        Some(path) => Box::new(std::io::BufWriter::new(std::fs::File::create(path)?)),
        None => Box::new(std::io::stdout().lock()),
    };
    if events {
        for event in replay.events()? {
            writeln!(out, "{}", serde_json::to_string(&event)?)?;
        }
    } else {
        for state in &replay.states(version)? {
            let line = Line {
                state,
                inputs: state.inputs(),
            };
            writeln!(out, "{}", serde_json::to_string(&line)?)?;
        }
    }
    out.flush()?;
    Ok(())
}

/// What a check found.
#[derive(Default)]
struct Totals {
    checked: usize,
    differing: usize,
    values: usize,
    different_values: usize,
    /// The largest difference in position, velocity, rotation and angular velocity.
    largest: [f32; 4],
    /// Whether frames or inputs differ.
    other: bool,
    events: usize,
    different_events: usize,
}

/// Compares the states and events of the replays among `files` with those the game decodes,
/// using the BytesTrace plugin. With `older`, also those of each replay converted to versions 5
/// and 4, which no recorded replay has.
pub fn check(files: &[PathBuf], server: &str, max: usize, older: bool) -> Result<()> {
    let mut totals = Totals::default();
    for path in files {
        let data = std::fs::read(path)?;
        let Ok(BytesFile::GameObject(obj)) = BytesFile::read(&data) else {
            continue;
        };
        let Some((replay, version)) = obj.car_replay_data() else {
            continue;
        };
        let states = replay
            .states(version)
            .with_context(|| path.display().to_string())?;
        let mut files = vec![(version, data)];
        if older {
            files.push((5, with_states(&obj, 5, &states).write()));
            files.push((4, with_states(&obj, 4, &states).write()));
        }
        for (version, data) in files {
            let label = format!("{} (v{version})", path.display());
            // Read back what was converted, to check it as a file the game would load.
            let (states, events) = match BytesFile::read(&data)? {
                BytesFile::GameObject(obj) => {
                    let (replay, version) = obj.car_replay_data().context("no replay")?;
                    let states = replay.states(version).context(label.clone())?;
                    (states, replay.events().context(label.clone())?)
                }
                _ => bail!("{label}: not a game object"),
            };
            match game(server, "replay-states", &data)? {
                Some(game) => compare(&label, &states, &game, max, &mut totals)?,
                None => println!("{label}: the game didn't load it"),
            }
            if let Some(game) = game(server, "replay-events", &data)? {
                compare_events(&label, &events, &String::from_utf8(game)?, max, &mut totals)?;
            }
        }
    }
    println!(
        "{} replays checked, {} differ; {} of {} values differ",
        totals.checked, totals.differing, totals.different_values, totals.values
    );
    if totals.different_values > 0 {
        let [p, v, r, a] = totals.largest;
        println!(
            "largest differences: position {p}, velocity {v}, rotation {r}, angular velocity \
             {a}; frames or inputs differ: {}",
            totals.other
        );
    }
    println!(
        "{} events checked, {} differ",
        totals.events, totals.different_events
    );
    Ok(())
}

/// What the game gives for a replay file at an endpoint: its states as it keeps them, or its
/// events as JSON lines.
fn game(server: &str, endpoint: &str, data: &[u8]) -> Result<Option<Vec<u8>>> {
    let response = ureq::post(&format!("{server}/{endpoint}"))
        .config()
        .http_status_as_error(false)
        .build()
        .send(data)?;
    if response.status() != 200 {
        return Ok(None);
    }
    Ok(Some(
        response
            .into_body()
            .with_config()
            .limit(u64::MAX)
            .read_to_vec()?,
    ))
}

fn compare(
    label: &str,
    ours: &[CarState],
    game: &[u8],
    max: usize,
    totals: &mut Totals,
) -> Result<()> {
    totals.checked += 1;
    // Each state has a frame, 13 floats and the inputs.
    totals.values += ours.len() * 15;
    if game == replay::write_states(ours) {
        return Ok(());
    }
    totals.differing += 1;
    let game = replay::read_states(game).context("the game's states")?;
    println!(
        "{label}: {} states, the game has {}",
        ours.len(),
        game.len()
    );
    let mut shown = 0;
    let mut show = |text: String| {
        if shown < max {
            println!("  {text}");
            shown += 1;
        }
    };
    for (i, (o, g)) in ours.iter().zip(&game).enumerate() {
        let fields: [(&str, &[f32], &[f32]); 4] = [
            ("position", &o.position, &g.position),
            ("velocity", &o.velocity, &g.velocity),
            ("rotation", &o.rotation, &g.rotation),
            ("angular_velocity", &o.angular_velocity, &g.angular_velocity),
        ];
        for (k, (name, ours, game)) in fields.into_iter().enumerate() {
            for (a, b) in ours.iter().zip(game) {
                if a.to_bits() != b.to_bits() {
                    totals.different_values += 1;
                    totals.largest[k] = totals.largest[k].max((a - b).abs());
                    show(format!(
                        "state {i} frame {}: {name} {a:?}, game {b:?}",
                        o.frame
                    ));
                }
            }
        }
        for (name, a, b) in [
            ("frame", o.frame, g.frame),
            ("directives", o.directives, g.directives),
        ] {
            if a != b {
                totals.different_values += 1;
                totals.other = true;
                show(format!("state {i}: {name} {a:#x}, game {b:#x}"));
            }
        }
    }
    Ok(())
}

/// Compares events with those the game reads, which are JSON lines with the frame, the event's
/// type and the fields of its data: floats as `{"f32": bits}` and structs as objects.
fn compare_events(
    label: &str,
    ours: &[ReplayEvent],
    game: &str,
    max: usize,
    totals: &mut Totals,
) -> Result<()> {
    let game: Vec<Value> = game
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?;
    let mut shown = 0;
    // Events past the end of the shorter list differ.
    totals.events += ours.len().max(game.len());
    if ours.len() != game.len() {
        println!(
            "{label}: {} events, the game has {}",
            ours.len(),
            game.len()
        );
        totals.different_events += ours.len().abs_diff(game.len());
        shown = max;
    }
    for (i, (o, g)) in ours.iter().zip(&game).enumerate() {
        let Value::Object(mut ours) = serde_json::to_value(o)? else {
            bail!("an event isn't an object");
        };
        let event = ours.remove("type");
        let frame = ours.remove("frame");
        let ours: Map<String, Value> = ours.into_iter().map(|(k, v)| (k, ours_value(v))).collect();
        let mut theirs = Map::new();
        if let Some(Value::Object(data)) = g.get("data") {
            for (name, value) in data {
                // Not written: the game sets it as it sends the event.
                if name == "silent_" {
                    continue;
                }
                theirs.insert(field_name(name), game_value(value));
            }
        }
        if frame.as_ref() != g.get("frame") || event.as_ref() != g.get("event") || ours != theirs {
            totals.different_events += 1;
            if shown < max {
                println!(
                    "{label}: event {i}: {}, game {g}",
                    serde_json::to_string(o)?
                );
                shown += 1;
            }
        }
    }
    Ok(())
}

/// Floats as their bits, to compare them exactly.
fn ours_value(v: Value) -> Value {
    match v {
        Value::Number(n) if n.is_f64() => {
            let f = n.as_f64().expect("a float") as f32;
            Value::from(format!("f32:{}", f.to_bits()))
        }
        Value::Array(a) => Value::Array(a.into_iter().map(ours_value).collect()),
        v => v,
    }
}

/// A value the game gave, as `ours_value` gives ours: vectors as arrays, and structs of one
/// field (handles) as that field.
fn game_value(v: &Value) -> Value {
    let Value::Object(fields) = v else {
        return v.clone();
    };
    if let Some(bits) = fields.get("f32").and_then(Value::as_i64) {
        return Value::from(format!("f32:{}", bits as u32));
    }
    let keys: Vec<_> = fields.keys().map(String::as_str).collect();
    if keys == ["x", "y", "z"] || keys == ["x", "y", "z", "w"] {
        return Value::Array(fields.values().map(game_value).collect());
    }
    match fields.values().next() {
        Some(only) if fields.len() == 1 => game_value(only),
        _ => v.clone(),
    }
}

/// Our name of a field of the game's: `fastRespawn_` is `fast_respawn`, `planeLS` `plane_ls`.
fn field_name(name: &str) -> String {
    let mut out = String::new();
    let mut previous = None;
    for c in name.trim_end_matches('_').chars() {
        if c.is_ascii_uppercase() && previous.is_some_and(|p: char| p.is_ascii_lowercase()) {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
        previous = Some(c);
    }
    out
}

/// A copy of a replay's object whose replay data has `states` in `version`: version 5's
/// compression, or uncompressed for version 4.
fn with_states(obj: &GameObject, version: i32, states: &[CarState]) -> BytesFile {
    let mut obj = obj.clone();
    for c in &mut obj.components {
        if let (
            ComponentKind::Known { version: v, .. },
            ComponentBody::Data {
                data: ComponentData::CarReplayData(data),
                ..
            },
        ) = (&mut c.kind, &mut c.body)
        {
            *v = version;
            data.data_was_truncated = None;
            if version >= 5 {
                data.state_buffer.clear();
                data.transform_buffer = v5_transforms(states);
                data.directive_buffer = v5_directives(states);
            } else {
                data.state_buffer = replay::write_states(states);
                data.transform_buffer.clear();
                data.directive_buffer.clear();
            }
            break;
        }
    }
    BytesFile::GameObject(obj)
}

/// Splits states into segments of up to 256 states 3 frames apart, by `split`, which is given
/// the segment so far and the next state.
fn segments(
    states: &[CarState],
    mut split: impl FnMut(&[CarState], &CarState) -> bool,
) -> Vec<&[CarState]> {
    let mut out = Vec::new();
    let mut start = 0;
    for i in 1..=states.len() {
        let segment = &states[start..i];
        let end = match states.get(i) {
            None => true,
            Some(next) => {
                segment.len() == 256
                    || next.frame != segment[segment.len() - 1].frame + 3
                    || split(segment, next)
            }
        };
        if end {
            out.push(segment);
            start = i;
        }
    }
    out
}

/// Writes segments as packets: the position of the next, the first frame, the number of
/// states after the first, and `data`.
fn write_segments(segments: &[&[CarState]], data: impl Fn(&mut Writer, &[CarState])) -> Vec<u8> {
    let mut w = Writer::new();
    for s in segments {
        let mut packet = Writer::new();
        packet.u8((s.len() - 1) as u8);
        data(&mut packet, s);
        let packet = packet.finish();
        w.i32((w.pos() + 8 + packet.len()) as i32);
        w.i32(s[0].frame);
        w.bytes(&packet);
    }
    w.finish()
}

/// Version 5's curves: segments through each state's neighbors, of lengths that cover a
/// segment of one state, of two (with no states between the ends), and up to 256.
fn v5_transforms(states: &[CarState]) -> Vec<u8> {
    const LENGTHS: [usize; 8] = [1, 2, 3, 5, 17, 64, 256, 100];
    let mut n = 0;
    let segments = segments(states, |segment, _| {
        let end = segment.len() == LENGTHS[n % LENGTHS.len()];
        n += usize::from(end);
        end
    });
    write_segments(&segments, |w, s| {
        for state in [s[0], s[s.len() - 1]] {
            w.f32s(&state.position);
            w.f32s(&state.velocity);
            w.f32s(&state.rotation);
            w.f32s(&state.angular_velocity);
        }
    })
}

/// Version 5's inputs: runs of equal ones.
fn v5_directives(states: &[CarState]) -> Vec<u8> {
    let segments = segments(states, |segment, next| {
        next.directives != segment[0].directives
    });
    write_segments(&segments, |w, s| w.i32(s[0].directives))
}
