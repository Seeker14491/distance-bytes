//! The recorded run of a replay (`CarReplayData`), decoded as the game decodes it to play it.
//!
//! The game records the car's state every 3 frames, at 100 frames a second: its position,
//! velocity, rotation and angular velocity, and the player's inputs. Up to version 4, a replay
//! keeps these states as recorded (`state_buffer`). From version 5 it keeps them compressed
//! (`transform_buffer` and `directive_buffer`): the car's movement as cubic Bézier curves, each
//! through up to 256 states, and the inputs as runs of equal ones (version 5) or as linear
//! ramps (later versions). The game decompresses them into states again, which follow the
//! curves rather than the recorded positions, within about 3 cm.
//!
//! [`CarReplayData::states`] gives the states the game plays, and [`CarReplayData::events`] the
//! player events, such as respawns, jumps and checkpoints (see [`events`]).
//!
//! The states are the game's to the bit. They're computed as the game computes them, in the
//! same order and precision: the game's Mono keeps the values within an expression in double
//! precision, and rounds them to single precision when it stores them (locals, arguments, fields
//! and return values).

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::components::{CarReplayData, ComponentData};
use crate::error::{Error, ErrorKind, Result};
use crate::io::{Reader, Writer};
use crate::tree::{ComponentBody, ComponentKind, GameObject};

pub mod events;

pub use events::{PLAYER_EVENTS, PlayerEvent, ReplayEvent};

/// The car's state at one frame of a replay.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct CarState {
    /// The frame, at 100 frames a second.
    pub frame: i32,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    /// Quaternion as `[x, y, z, w]`.
    pub rotation: [f32; 4],
    /// In radians a second.
    pub angular_velocity: [f32; 3],
    /// The player's inputs, packed as the game packs them (`CarDirectives`). See
    /// [`CarState::inputs`].
    pub directives: i32,
}

impl CarState {
    /// The player's inputs.
    pub fn inputs(&self) -> Inputs {
        Inputs::from_bits(self.directives)
    }

    /// The time from the start of the recording, in seconds.
    pub fn time(&self) -> f64 {
        f64::from(self.frame) * 0.01
    }
}

/// The player's inputs at a state of a replay (`CarDirectives`), which the game keeps in a few
/// bits each.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Inputs {
    /// From -1 to 1, in 7 bits.
    pub steer: f32,
    /// Rotation in the air, from -1 to 1, in 5 bits each.
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    /// From -1 to 1, in 4 bits.
    pub gas: f32,
    /// From 0 to 1, in 4 bits.
    pub brake: f32,
    pub boost: bool,
    pub grip: bool,
}

impl Inputs {
    /// Unpacks inputs as the game does.
    pub fn from_bits(bits: i32) -> Self {
        let mut b = Bits::new(bits);
        Inputs {
            steer: b.float_neg1_to_1(7),
            pitch: b.float_neg1_to_1(5),
            yaw: b.float_neg1_to_1(5),
            roll: b.float_neg1_to_1(5),
            gas: b.float_neg1_to_1(4),
            brake: b.float_0_to_1(4),
            boost: b.bool(),
            grip: b.bool(),
        }
    }

    /// Packs inputs as the game does. It truncates, so inputs unpacked from bits can pack
    /// lower: the game's steering of one step right of full left packs as full left.
    pub fn to_bits(&self) -> i32 {
        let mut b = Bits::new(0);
        b.add_float_neg1_to_1(self.steer, 7);
        b.add_float_neg1_to_1(self.pitch, 5);
        b.add_float_neg1_to_1(self.yaw, 5);
        b.add_float_neg1_to_1(self.roll, 5);
        b.add_float_neg1_to_1(self.gas, 4);
        b.add_float_0_to_1(self.brake, 4);
        b.add_bool(self.boost);
        b.add_bool(self.grip);
        b.field
    }

    /// `CarDirectives.LerpFrom`.
    fn lerp(a: &Inputs, b: &Inputs, t: f32) -> Inputs {
        Inputs {
            steer: lerp(a.steer, b.steer, t),
            pitch: lerp(a.pitch, b.pitch, t),
            yaw: lerp(a.yaw, b.yaw, t),
            roll: lerp(a.roll, b.roll, t),
            gas: lerp(a.gas, b.gas, t),
            brake: lerp(a.brake, b.brake, t),
            boost: b.boost,
            grip: b.grip,
        }
    }
}

impl GameObject {
    /// The object's first `CarReplayData` with a decoded body, and its version. A replay file's
    /// object has one.
    pub fn car_replay_data(&self) -> Option<(&CarReplayData, i32)> {
        self.components
            .iter()
            .find_map(|c| match (&c.kind, &c.body) {
                (
                    ComponentKind::Known { version, .. },
                    ComponentBody::Data {
                        data: ComponentData::CarReplayData(data),
                        ..
                    },
                ) => Some((&**data, *version)),
                _ => None,
            })
    }
}

impl CarReplayData {
    /// The car's states, as the game decodes them from the buffers of a body of `version` to
    /// play the replay. An error's offset is into the buffer, not the file.
    pub fn states(&self, version: i32) -> Result<Vec<CarState>> {
        match version {
            ..=4 => read_states(&self.state_buffer),
            5 => decompress(
                &self.transform_buffer,
                &self.directive_buffer,
                bezier_segment,
                run_segment,
            ),
            _ => decompress(
                &self.transform_buffer,
                &self.directive_buffer,
                quantized_segment,
                ramp_segment,
            ),
        }
    }
}

/// Reads states as replays of version 4 and earlier keep them, and as the game decompresses
/// later ones into.
pub fn read_states(buffer: &[u8]) -> Result<Vec<CarState>> {
    packets(buffer)?
        .into_iter()
        .map(|(frame, mut r)| {
            let s = snapshot(&mut r)?;
            let directives = r.i32()?;
            r.expect_end()?;
            Ok(CarState {
                frame,
                position: s.pos,
                velocity: s.vel,
                rotation: s.rot,
                angular_velocity: s.rot_vel,
                directives,
            })
        })
        .collect()
}

/// Writes states as [`read_states`] reads them.
pub fn write_states(states: &[CarState]) -> Vec<u8> {
    let mut w = Writer::new();
    for s in states {
        // The position of the next packet, after this one's 64 bytes.
        w.i32((w.pos() + 64) as i32);
        w.i32(s.frame);
        w.f32s(&s.position);
        w.f32s(&s.velocity);
        w.f32s(&s.rotation);
        w.f32s(&s.angular_velocity);
        w.i32(s.directives);
    }
    w.finish()
}

/// The packets of a buffer, `PacketReaderGeneric`: each is the position of the next one, its
/// frame and its data.
fn packets(buffer: &[u8]) -> Result<Vec<(i32, Reader<'_>)>> {
    let mut r = Reader::new(buffer);
    let mut packets = Vec::new();
    while !r.is_empty() {
        let start = r.pos();
        let next = r.i32()?;
        let frame = r.i32()?;
        let len = usize::try_from(next)
            .ok()
            .and_then(|next| next.checked_sub(r.pos()))
            .filter(|&len| len <= r.remaining())
            .ok_or_else(|| Error::new(start, ErrorKind::InvalidPacket))?;
        packets.push((frame, r.sub(len)?));
    }
    Ok(packets)
}

/// `StateBufferDecompressorSplit.Decompress`: the states from the segments of the car's
/// movement and those of the inputs, which the game reads in step, 3 frames apart.
fn decompress(
    transforms: &[u8],
    directives: &[u8],
    transform_segment: fn(&mut Reader, Span) -> Result<Vec<Snapshot>>,
    directive_segment: fn(&mut Reader, Span) -> Result<Vec<i32>>,
) -> Result<Vec<CarState>> {
    let mut transforms = Stream::new(segments(transforms, transform_segment)?);
    let mut directives = Stream::new(segments(directives, directive_segment)?);
    let mut states = Vec::new();
    loop {
        transforms.next();
        directives.next();
        if transforms.frame == -1 && directives.frame == -1 {
            return Ok(states);
        }
        let s = transforms.current;
        states.push(CarState {
            frame: transforms.frame.max(directives.frame),
            position: s.pos,
            velocity: s.vel,
            rotation: s.rot,
            angular_velocity: s.rot_vel,
            directives: directives.current,
        });
    }
}

/// The frames a segment covers: from `start` to `end`, 3 apart.
#[derive(Clone, Copy)]
struct Span {
    start: i32,
    end: i32,
}

/// The segments of a buffer, `SplitPacket.Decoder`, each with its first frame and its values.
fn segments<T>(
    buffer: &[u8],
    segment: fn(&mut Reader, Span) -> Result<Vec<T>>,
) -> Result<Vec<(i32, Vec<T>)>> {
    packets(buffer)?
        .into_iter()
        .map(|(start, mut r)| {
            let count = r.bytes(1)?[0];
            let span = Span {
                start,
                end: start.wrapping_add(i32::from(count) * 3),
            };
            let values = segment(&mut r, span)?;
            r.expect_end()?;
            Ok((start, values))
        })
        .collect()
}

/// The values of one buffer as the decompressor goes through them: the current one and its
/// frame, which is -1 after the last.
struct Stream<T> {
    segments: std::vec::IntoIter<(i32, Vec<T>)>,
    queue: VecDeque<T>,
    frame: i32,
    current: T,
}

impl<T: Default> Stream<T> {
    fn new(segments: Vec<(i32, Vec<T>)>) -> Self {
        Stream {
            segments: segments.into_iter(),
            queue: VecDeque::new(),
            frame: 0,
            current: T::default(),
        }
    }

    fn next(&mut self) {
        if self.queue.is_empty() {
            self.frame = match self.segments.next() {
                Some((start, values)) => {
                    self.queue.extend(values);
                    start
                }
                None => -1,
            };
        } else {
            self.frame = self.frame.wrapping_add(3);
        }
        // After the last, the last value stays.
        if let Some(value) = self.queue.pop_front() {
            self.current = value;
        }
    }
}

/// The frames from `start` to `end`, 3 apart, with whether to include the ends.
fn frames(span: Span, first: bool, last: bool) -> impl Iterator<Item = i32> {
    let start = i64::from(span.start) + if first { 0 } else { 3 };
    let end = i64::from(span.end) + if last { 1 } else { 0 };
    (start..end).step_by(3).map(|i| i as i32)
}

/// `SplitPacketTransformBezier.Decoder` (version 5): the states at both ends, with the curve
/// between them from their velocities.
fn bezier_segment(r: &mut Reader, span: Span) -> Result<Vec<Snapshot>> {
    let s0 = snapshot(r)?;
    let s1 = snapshot(r)?;
    let mut out = vec![s0];
    let num = span.end.wrapping_sub(span.start);
    if num > 3 {
        let num2 = 1.0 / num as f32;
        let (pb, rb) = bezier_interpolators(&s0, &s1, num);
        out.extend(curve_states(
            &pb,
            &rb,
            span,
            num2,
            frames(span, false, false),
        ));
    }
    if num > 0 {
        out.push(s1);
    }
    Ok(out)
}

/// `SplitPacketTransformBezier.DecoderQuantized` (version 6 and later): the curves themselves,
/// with their inner control points and the rotations quantized.
fn quantized_segment(r: &mut Reader, span: Span) -> Result<Vec<Snapshot>> {
    let p0 = r.f32s::<3>()?;
    let p3 = r.f32s::<3>()?;
    // The inner control points' offsets from the ends, divided by the distance between the ends.
    let d0 = vec2(r.i32()?);
    let d1 = vec2(r.i32()?);
    let dz = vec2(r.i32()?);
    let rb = [
        quaternion(r.i32()?),
        quaternion(r.i32()?),
        quaternion(r.i32()?),
        quaternion(r.i32()?),
    ];
    let len = max(distance(p0, p3), 1.0);
    let p1 = add(scale([d0[0], d0[1], dz[0]], len), p0);
    let p2 = add(scale([d1[0], d1[1], dz[1]], len), p3);
    let pb = [p0, p1, p2, p3];
    let num = span.end.wrapping_sub(span.start).max(1);
    let num2 = 1.0 / num as f32;
    Ok(curve_states(&pb, &rb, span, num2, frames(span, true, true)).collect())
}

/// The states on a segment's curves at `frames`.
fn curve_states(
    pb: &[V3; 4],
    rb: &[Q; 4],
    span: Span,
    num2: f32,
    frames: impl Iterator<Item = i32>,
) -> impl Iterator<Item = Snapshot> {
    let dpb = bezier_derivative(pb);
    let drb = quaternion_bezier_derivative(rb);
    let num3 = num2 * 100.0;
    frames.map(move |i| {
        let t = i.wrapping_sub(span.start) as f32 * num2;
        Snapshot {
            pos: bezier(pb, t),
            rot: quaternion_bezier(rb, t),
            vel: scale(bezier2(&dpb, t), num3),
            rot_vel: scale(bezier2(&drb, t), num3),
        }
    })
}

/// `SplitPacketDirectiveRLE.Decoder` (version 5): the same inputs throughout.
fn run_segment(r: &mut Reader, span: Span) -> Result<Vec<i32>> {
    let bits = r.i32()?;
    let count = span.end.wrapping_sub(span.start) / 3 + 1;
    Ok(vec![bits; count.max(0) as usize])
}

/// `SplitPacketDirectiveLinear.Decoder` (version 6 and later): the inputs at both ends, and
/// between them inputs interpolated from them.
fn ramp_segment(r: &mut Reader, span: Span) -> Result<Vec<i32>> {
    let bits0 = r.i32()?;
    let bits1 = r.i32()?;
    let mut out = vec![bits0];
    let num = span.end.wrapping_sub(span.start);
    if num > 3 {
        let num2 = 1.0 / num as f32;
        let (first, last) = (Inputs::from_bits(bits0), Inputs::from_bits(bits1));
        // The game takes the frame for the time into the segment, so past the first segment
        // the interpolation is clamped to the last inputs.
        out.extend(
            frames(span, false, false)
                .map(|i| Inputs::lerp(&first, &last, i as f32 * num2).to_bits()),
        );
    }
    if num > 0 {
        out.push(bits1);
    }
    Ok(out)
}

// --- The game's math ---
//
// Values are `f32` as the game stores them. Within an expression the game computes in double
// precision, which matters where an expression has more than one operation: those are computed
// in `f64` and rounded once.

type V3 = [f32; 3];
/// `[x, y, z, w]`.
type Q = [f32; 4];

/// `RigidbodyStateTransceiver.Snapshot`.
#[derive(Clone, Copy, Default)]
struct Snapshot {
    pos: V3,
    rot: Q,
    vel: V3,
    rot_vel: V3,
}

fn snapshot(r: &mut Reader) -> Result<Snapshot> {
    Ok(Snapshot {
        pos: r.f32s()?,
        vel: r.f32s()?,
        rot: r.f32s()?,
        rot_vel: r.f32s()?,
    })
}

fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn neg(a: V3) -> V3 {
    [-a[0], -a[1], -a[2]]
}

fn sqr_magnitude(a: V3) -> f32 {
    let [x, y, z] = a.map(f64::from);
    (x * x + y * y + z * z) as f32
}

fn distance(a: V3, b: V3) -> f32 {
    sqr_magnitude(sub(a, b)).sqrt()
}

/// `Mathf.Max`.
fn max(a: f32, b: f32) -> f32 {
    if a > b { a } else { b }
}

fn clamp(v: f32, min: f32, max: f32) -> f32 {
    if v < min {
        min
    } else if v > max {
        max
    } else {
        v
    }
}

/// `Mathf.Lerp`, which clamps `t`.
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    let (a, b, t) = (f64::from(a), f64::from(b), f64::from(clamp(t, 0.0, 1.0)));
    (a + (b - a) * t) as f32
}

/// `Quaternion * Quaternion`.
fn qmul(l: Q, r: Q) -> Q {
    let (l, r) = (l.map(f64::from), r.map(f64::from));
    [
        l[3] * r[0] + l[0] * r[3] + l[1] * r[2] - l[2] * r[1],
        l[3] * r[1] + l[1] * r[3] + l[2] * r[0] - l[0] * r[2],
        l[3] * r[2] + l[2] * r[3] + l[0] * r[1] - l[1] * r[0],
        l[3] * r[3] - l[0] * r[0] - l[1] * r[1] - l[2] * r[2],
    ]
    .map(|x| x as f32)
}

fn qscale(q: Q, s: f32) -> Q {
    [q[0] * s, q[1] * s, q[2] * s, q[3] * s]
}

/// `QuaternionEx.Negate`.
fn qneg(q: Q) -> Q {
    q.map(|x| -x)
}

fn qdot(a: Q, b: Q) -> f32 {
    let (a, b) = (a.map(f64::from), b.map(f64::from));
    (a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]) as f32
}

/// `QuaternionEx.AddWithPositiveDot`.
fn qadd_positive(a: Q, b: Q) -> Q {
    let b = if qdot(a, b) < 0.0 { qneg(b) } else { b };
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}

/// `QuaternionEx.Normalized`.
fn normalized(q: Q) -> Q {
    let m = qdot(q, q).sqrt();
    if m != 0.0 {
        qscale(q, 1.0 / m)
    } else {
        [0.0, 0.0, 0.0, 1.0]
    }
}

/// `QuaternionEx.Conjugate`, which negates `w`.
fn conjugate(q: Q) -> Q {
    [q[0], q[1], q[2], -q[3]]
}

/// `QuaternionEx.GetDeltaRot`: the rotation as an axis scaled by its angle.
fn delta_rot(q: Q) -> V3 {
    let q = if q[3] < 0.0 { qneg(q) } else { q };
    if q[3].abs() > 0.9999 {
        return [0.0; 3];
    }
    let num = (1.0 - q[3] * q[3]).sqrt();
    let num2 = (f64::from(q[3]).acos() as f32) * 2.0;
    scale([q[0] / num, q[1] / num, q[2] / num], num2)
}

/// `QuaternionEx.Rotation`: the rotation by an axis scaled by its angle.
fn rotation(delta: V3) -> Q {
    let angle = sqr_magnitude(delta).sqrt();
    if angle == 0.0 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let axis = [delta[0] / angle, delta[1] / angle, delta[2] / angle];
    let f = angle * 0.5;
    let w = f64::from(f).cos() as f32;
    let s = f64::from(f).sin() as f32;
    [axis[0] * s, axis[1] * s, axis[2] * s, w]
}

/// `3 * a * b`, in double precision.
fn weight(a: f32, b: f32) -> f32 {
    (3.0 * f64::from(a) * f64::from(b)) as f32
}

/// `Bezier3V3.B`.
fn bezier(p: &[V3; 4], t: f32) -> V3 {
    let num = t * t;
    let num2 = num * t;
    let num3 = 1.0 - t;
    let num4 = num3 * num3;
    let num5 = num4 * num3;
    add(
        add(
            add(scale(p[0], num5), scale(p[1], weight(num4, t))),
            scale(p[2], weight(num3, num)),
        ),
        scale(p[3], num2),
    )
}

/// `Bezier3V3.Derivative`.
fn bezier_derivative(p: &[V3; 4]) -> [V3; 3] {
    [
        scale(sub(p[1], p[0]), 3.0),
        scale(sub(p[2], p[1]), 3.0),
        scale(sub(p[3], p[2]), 3.0),
    ]
}

/// `Bezier2V3.B`.
fn bezier2(p: &[V3; 3], t: f32) -> V3 {
    let num = t * t;
    let num2 = 1.0 - t;
    let num3 = num2 * num2;
    add(
        add(scale(p[0], num3), scale(p[1], 2.0 * num2 * t)),
        scale(p[2], num),
    )
}

/// `Bezier3Q.B`.
fn quaternion_bezier(p: &[Q; 4], t: f32) -> Q {
    let num = t * t;
    let s = num * t;
    let num2 = 1.0 - t;
    let num3 = num2 * num2;
    let s2 = num3 * num2;
    let q = qscale(p[0], s2);
    let q = qadd_positive(q, qscale(p[1], weight(num3, t)));
    let q = qadd_positive(q, qscale(p[2], weight(num2, num)));
    let q = qadd_positive(q, qscale(p[3], s));
    normalized(q)
}

/// `Bezier3Q.Derivative`.
fn quaternion_bezier_derivative(p: &[Q; 4]) -> [V3; 3] {
    let d = |a: Q, b: Q| scale(delta_rot(qmul(b, conjugate(a))), 3.0);
    [d(p[0], p[1]), d(p[1], p[2]), d(p[2], p[3])]
}

/// `SplitPacketTransformBezier.CreateBezierInterpolators`: curves between two states, with
/// control points from their velocities.
fn bezier_interpolators(s0: &Snapshot, s1: &Snapshot, len: i32) -> ([V3; 4], [Q; 4]) {
    let len = if len == 0 { 1 } else { len };
    let num = (f64::from(len as f32) / 3.0 / 100.0) as f32;
    let p1 = add(scale(s0.vel, num), s0.pos);
    let p2 = add(scale(neg(s1.vel), num), s1.pos);
    let r1 = qmul(rotation(scale(s0.rot_vel, num)), s0.rot);
    let r2 = qmul(rotation(scale(neg(s1.rot_vel), num)), s1.rot);
    ([s0.pos, p1, p2, s1.pos], [s0.rot, r1, r2, s1.rot])
}

/// A quantized rotation: the index of its largest component in 2 bits, and the others divided
/// by it in 10 bits each (`SegmentQuantized.RebuildQuat`).
fn quaternion(bits: i32) -> Q {
    let mut b = Bits::new(bits);
    let big = b.int(2) as usize;
    let v = [
        b.float_neg1_to_1(10),
        b.float_neg1_to_1(10),
        b.float_neg1_to_1(10),
    ];
    let num = 1.0 / (sqr_magnitude(v) + 1.0).sqrt();
    let mut q = [0.0; 4];
    q[big] = num;
    let mut j = 0;
    for x in v {
        if j == big {
            j += 1;
        }
        q[j] = x * num;
        j += 1;
    }
    q
}

/// Two values from -1 to 1 in 16 bits each (`SegmentQuantized.RebuildVec2`).
fn vec2(bits: i32) -> [f32; 2] {
    let mut b = Bits::new(bits);
    [b.float_neg1_to_1(16), b.float_neg1_to_1(16)]
}

/// `BitEncoder`: values packed into the bits of an `int`, from the lowest.
struct Bits {
    field: i32,
    index: u32,
}

impl Bits {
    fn new(field: i32) -> Self {
        Bits { field, index: 0 }
    }

    fn mask(bits: u32) -> i32 {
        ((1u32 << bits) - 1) as i32
    }

    // Shifts take the count modulo 32, as in C#.
    fn int(&mut self, bits: u32) -> i32 {
        let v = self.field.wrapping_shr(self.index) & Self::mask(bits);
        self.index += bits;
        v
    }

    fn bool(&mut self) -> bool {
        self.int(1) != 0
    }

    fn float_0_to_1(&mut self, bits: u32) -> f32 {
        self.int(bits) as f32 / Self::mask(bits) as f32
    }

    fn float_neg1_to_1(&mut self, bits: u32) -> f32 {
        let num3 = self.int(bits) as f32 / (Self::mask(bits) - 1) as f32;
        num3 * 2.0 - 1.0
    }

    fn add(&mut self, v: i32, bits: u32) {
        self.field |= v.wrapping_shl(self.index);
        self.index += bits;
    }

    fn add_bool(&mut self, b: bool) {
        self.add(b.into(), 1);
    }

    // The products are truncated from double precision.
    fn add_float_0_to_1(&mut self, f: f32, bits: u32) {
        let f = clamp(f, 0.0, 1.0);
        self.add((f64::from(f) * f64::from(Self::mask(bits))) as i32, bits);
    }

    fn add_float_neg1_to_1(&mut self, f: f32, bits: u32) {
        let f = (clamp(f, -1.0, 1.0) + 1.0) * 0.5;
        self.add(
            (f64::from(f) * f64::from(Self::mask(bits) - 1)) as i32,
            bits,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Writes a packet of a buffer: the position of the next, the frame and `data`.
    fn packet(w: &mut Writer, frame: i32, data: &[u8]) {
        w.i32((w.pos() + 8 + data.len()) as i32);
        w.i32(frame);
        w.bytes(data);
    }

    /// The data of a segment: the number of states after the first, then `values`.
    fn segment(count: u8, values: impl FnOnce(&mut Writer)) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(count);
        values(&mut w);
        w.finish()
    }

    /// Quantized, the identity rotation: `w` is the largest, and the others are 0.
    const IDENTITY: i32 = 3 | 511 << 2 | 511 << 12 | 511 << 22;
    /// Quantized, two zeros.
    const ZEROS: i32 = 32767 | 32767 << 16;

    fn steer(steer: f32) -> i32 {
        Inputs {
            steer,
            ..Inputs::default()
        }
        .to_bits()
    }

    #[test]
    fn uncompressed_states_round_trip() {
        let states = [
            CarState {
                frame: 2,
                position: [1.0, 2.0, 3.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                directives: 0x4000_0000,
                ..CarState::default()
            },
            CarState {
                frame: 5,
                velocity: [-0.5, 0.25, 8.0],
                angular_velocity: [0.0, -1.0, 0.0],
                ..CarState::default()
            },
        ];
        let buffer = write_states(&states);
        assert_eq!(buffer.len(), 128);
        assert_eq!(read_states(&buffer).unwrap(), states);
        let replay = CarReplayData {
            state_buffer: buffer,
            ..CarReplayData::default()
        };
        assert_eq!(replay.states(4).unwrap(), states);
    }

    #[test]
    fn inputs() {
        // From a recorded replay: full gas, and boosting.
        let inputs = Inputs::from_bits(0x439e_f7bf);
        assert_eq!(
            inputs,
            Inputs {
                gas: 1.0,
                boost: true,
                ..Inputs::default()
            }
        );
        assert_eq!(inputs.to_bits(), 0x439e_f7bf);
        // Packing truncates: the steering just right of full left packs as full left.
        assert_eq!(Inputs::from_bits(1).steer, -0.984127);
        assert_eq!(Inputs::from_bits(1).to_bits(), 0);
    }

    #[test]
    fn quantized_segments() {
        // A straight curve over frames 0 to 6, with no rotation.
        let mut transforms = Writer::new();
        let data = segment(2, |w| {
            w.f32s(&[0.0, 0.0, 0.0]);
            w.f32s(&[0.0, 0.0, 6.0]);
            for bits in [ZEROS, ZEROS, ZEROS, IDENTITY, IDENTITY, IDENTITY, IDENTITY] {
                w.i32(bits);
            }
        });
        packet(&mut transforms, 0, &data);
        // Inputs from full left to full right over frames 0 to 6, and again over 9 to 15.
        let mut directives = Writer::new();
        let ramp = segment(2, |w| {
            w.i32(steer(-1.0));
            w.i32(steer(1.0));
        });
        packet(&mut directives, 0, &ramp);
        packet(&mut directives, 9, &ramp);
        let replay = CarReplayData {
            transform_buffer: transforms.finish(),
            directive_buffer: directives.finish(),
            ..CarReplayData::default()
        };
        let states = replay.states(7).unwrap();

        let frames: Vec<_> = states.iter().map(|s| s.frame).collect();
        assert_eq!(frames, [0, 3, 6, 9, 12, 15]);
        let z: Vec<_> = states.iter().map(|s| s.position[2]).collect();
        // After the last curve, its end stays.
        assert_eq!(z, [0.0, 3.0, 6.0, 6.0, 6.0, 6.0]);
        assert!(states.iter().all(|s| s.rotation == [0.0, 0.0, 0.0, 1.0]));
        // The curve's inner control points are at its ends, so it starts and ends at rest.
        assert_eq!(states[0].velocity, [0.0; 3]);
        assert_eq!(states[1].velocity, [0.0, 0.0, 150.00002]);
        let steering: Vec<_> = states.iter().map(|s| s.inputs().steer).collect();
        // The game interpolates by the frame rather than the time into the segment, so in
        // the second segment it's past the end and gives the last inputs.
        assert_eq!(steering, [-1.0, 0.0, 1.0, -1.0, 1.0, 1.0]);
    }

    #[test]
    fn version_5_segments() {
        let s0 = [
            0.0, 1.0, 0.0, 0.0, 0.0, 30.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
        ];
        let s1 = [
            0.0, 1.0, 3.6, 0.0, 0.0, 30.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
        ];
        let mut transforms = Writer::new();
        packet(
            &mut transforms,
            0,
            &segment(4, |w| w.f32s(&[s0, s1].concat())),
        );
        let mut directives = Writer::new();
        packet(&mut directives, 0, &segment(4, |w| w.i32(steer(0.0))));
        let replay = CarReplayData {
            transform_buffer: transforms.finish(),
            directive_buffer: directives.finish(),
            ..CarReplayData::default()
        };
        let states = replay.states(5).unwrap();

        // Moving at a steady 30 m/s from z = 0 to 3.6 over 12 frames.
        let z: Vec<_> = states.iter().map(|s| s.position[2]).collect();
        assert_eq!(z, [0.0, 0.9, 1.8, 2.7, 3.6]);
        assert!(states.iter().all(|s| (s.velocity[2] - 30.0).abs() < 1e-4));
        assert!(states.iter().all(|s| s.directives == steer(0.0)));
    }

    #[test]
    fn invalid_packets() {
        let mut w = Writer::new();
        w.i32(100);
        w.i32(0);
        w.u8(0);
        let replay = CarReplayData {
            transform_buffer: w.finish(),
            ..CarReplayData::default()
        };
        let e = replay.states(7).unwrap_err();
        assert_eq!((e.offset, e.kind), (0, ErrorKind::InvalidPacket));
    }
}
