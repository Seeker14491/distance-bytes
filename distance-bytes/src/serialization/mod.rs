mod component;
mod string;

use crate::{
    serialization::{component::component, string::string},
    util, GameObject,
};
use arrayvec::ArrayVec;
use combine::{
    attempt,
    error::StreamError,
    parser::{
        byte::num::{le_f32, le_i32, le_i64, le_u32},
        range::{length_prefix, range},
        repeat::count_min_max,
    },
    stream::PointerOffset,
    EasyParser, Parser as _, RangeStream, Stream,
};
use mint::{Quaternion, Vector3};
use std::convert::TryFrom;

type Input<'a> = combine::easy::Stream<&'a [u8]>;
type Error<'a> = combine::easy::Error<u8, &'a [u8]>;

pub(crate) fn finalize_errors(
    initial_slice: &[u8],
    errors: combine::easy::Errors<u8, &[u8], PointerOffset<[u8]>>,
) -> combine::easy::Errors<u8, String, usize> {
    let errors = errors.map_position(|pos| pos.translate_position(initial_slice));
    errors.map_range(|range| util::format_byte_slice(range, 10))
}

// #[derive(Debug, PartialEq)]
// pub struct BytesParseError<'a> {
//     offset: usize,
//     errors: Vec<combine::stream::easy::Error<u8, &'a [u8]>>,
// }
//
// impl<'a> BytesParseError<'a> {
//     pub(crate) fn new(
//         initial_slice: &[u8],
//         errors: combine::easy::Errors<u8, &'a [u8], PointerOffset<[u8]>>,
//     ) -> Self {
//         BytesParseError {
//             offset: errors.position.translate_position(initial_slice),
//             errors: errors.errors,
//         }
//     }
// }
//
// impl<'a> Display for BytesParseError<'a> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         write!(f, "Error at offset {}", self.offset)?;
//         if self.errors.is_empty() {
//             writeln!(f, ".")?;
//         } else {
//             writeln!(f, ":")?;
//         }
//
//         for error in &self.errors {
//             match error {
//                 Error::Unexpected(c) => writeln!(f, "Unexpected `{}`", c)?,
//                 Error::Expected(s) => writeln!(f, "Unexpected `{}`", s)?,
//                 Error::Message(msg) => msg.fmt(f)?,
//                 Error::Other(err) => err.fmt(f)?,
//             }
//         }
//
//         Ok(())
//     }
// }
//
// impl<'a> std::error::Error for BytesParseError<'a> {}

pub(crate) trait Parser<'a, O>: combine::Parser<Input<'a>, Output = O> {}
impl<'a, O, T> Parser<'a, O> for T where T: combine::Parser<Input<'a>, Output = O> {}

pub(crate) fn game_object<'a>() -> impl Parser<'a, GameObject> {
    let game_object_scope = scope_with_scope_mark_satisfying(|x| x == 66666666);

    game_object_scope.flat_map(|(_scope_mark, data)| {
        let name = string();
        let prefab = string();
        let guid = le_u32();
        let num_components = le_i32().and_then(usize::try_from);
        let components = num_components.then(|n| count_min_max(n, n, component()));

        let ((name, prefab, guid, components), _input) =
            (name, prefab, guid, components).easy_parse(data)?;

        let game_object = GameObject {
            name,
            prefab,
            guid,
            components,
        };

        Ok(game_object)
    })
}

fn scope<'a>() -> impl Parser<'a, (i32, &'a [u8])> {
    let scope_data = length_prefix(le_i64());

    (scope_mark(), scope_data)
}

fn scope_with_scope_mark_satisfying<'a>(
    f: impl FnMut(i32) -> bool,
) -> impl Parser<'a, (i32, &'a [u8])> {
    let scope_data = length_prefix(le_i64());

    (scope_mark_satisfying(f), scope_data)
}

fn scope_mark<'a>() -> impl Parser<'a, i32> {
    le_i32()
}

fn scope_mark_satisfying<'a>(mut f: impl FnMut(i32) -> bool) -> impl Parser<'a, i32> {
    scope_mark().and_then(move |mark| {
        if f(mark) {
            Ok(mark)
        } else {
            Err(Error::unexpected_format(format!("scope mark {}", mark)))
        }
    })
}

fn vector_3<'a>() -> impl Parser<'a, Option<Vector3<f32>>> {
    let p = || {
        count_min_max(3, 3, le_f32())
            .map(|buf: ArrayVec<[f32; 3]>| buf.into_inner().unwrap().into())
    };

    optional(p)
}

fn quaternion<'a>() -> impl Parser<'a, Option<Quaternion<f32>>> {
    let p = || {
        count_min_max(4, 4, le_f32())
            .map(|buf: ArrayVec<[f32; 4]>| buf.into_inner().unwrap().into())
    };

    optional(p)
}

fn optional<'a, I, P>(
    mut f: impl FnMut() -> P,
) -> impl combine::Parser<I, Output = Option<P::Output>> + 'a
where
    I: Stream<Token = u8, Range = &'a [u8]> + RangeStream + 'a,
    P: combine::Parser<I> + 'a,
    P::Output: Clone,
{
    const EMPTY_MARKER: &[u8] = &0x7FFF_FFFD_i32.to_le_bytes();

    let empty = range(EMPTY_MARKER).map(|_| None);
    attempt(empty).or(f().map(Some))
}
