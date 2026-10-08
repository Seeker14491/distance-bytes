//! Replays: their decoded states, and a check of them against the game's.

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use distance_bytes::io::Writer;
use distance_bytes::replay::{self, CarState};
use distance_bytes::{BytesFile, ComponentBody, ComponentData, ComponentKind, GameObject};
use serde::Serialize;

/// The replay data of a replay file, with its version.
fn states_of(path: &Path) -> Result<Option<(i32, Vec<CarState>)>> {
    let data = std::fs::read(path)?;
    let Ok(BytesFile::GameObject(obj)) = BytesFile::read(&data) else {
        return Ok(None);
    };
    let Some((replay, version)) = obj.car_replay_data() else {
        return Ok(None);
    };
    let states = replay
        .states(version)
        .with_context(|| path.display().to_string())?;
    Ok(Some((version, states)))
}

/// Writes the states of a replay as JSON lines, with the inputs unpacked.
pub fn print(input: &Path, output: Option<&Path>) -> Result<()> {
    #[derive(Serialize)]
    struct Line<'a> {
        #[serde(flatten)]
        state: &'a CarState,
        inputs: replay::Inputs,
    }
    let Some((_, states)) = states_of(input)? else {
        bail!("{} isn't a replay", input.display());
    };
    let mut out: Box<dyn Write> = match output {
        Some(path) => Box::new(std::io::BufWriter::new(std::fs::File::create(path)?)),
        None => Box::new(std::io::stdout().lock()),
    };
    for state in &states {
        let line = Line {
            state,
            inputs: state.inputs(),
        };
        writeln!(out, "{}", serde_json::to_string(&line)?)?;
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
}

/// Compares the states of the replays among `files` with those the game decodes, using the
/// BytesTrace plugin. With `older`, also those of each replay converted to versions 5 and 4,
/// which no recorded replay has.
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
            let ours = match BytesFile::read(&data)? {
                BytesFile::GameObject(obj) => {
                    let (replay, version) = obj.car_replay_data().context("no replay")?;
                    replay.states(version).context(label.clone())?
                }
                _ => bail!("{label}: not a game object"),
            };
            match game_states(server, &data)? {
                Some(game) => compare(&label, &ours, &game, max, &mut totals)?,
                None => println!("{label}: the game didn't load it"),
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
    Ok(())
}

/// The states the game decodes from a replay file, as it keeps them.
fn game_states(server: &str, data: &[u8]) -> Result<Option<Vec<u8>>> {
    let response = ureq::post(&format!("{server}/replay-states"))
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
