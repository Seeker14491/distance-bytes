use crate::{
    domain::GameObject,
    serialization::{
        error::{failure, BytesDeserializeError, BytesDeserializeErrorKind, SliceWithOffset},
        string::string,
    },
};
use mint::{Quaternion, Vector3};
use nom::{
    bytes::complete::take,
    multi::{count, fill},
    number::complete::{le_f32, le_i32, le_i64, le_u32},
    IResult, Parser,
};
use std::convert::TryInto;

mod component;
pub(crate) mod error;
mod string;

type BytesParseResult<'a, O> = IResult<Input<'a>, O, BytesDeserializeError>;
type Input<'a> = SliceWithOffset<'a, u8>;

pub(crate) fn read_game_object(input: Input<'_>) -> BytesParseResult<'_, GameObject> {
    let (after_game_object, (scope_mark, scope)) = read_scope(input)?;
    if scope_mark != 66666666 {
        return Err(failure(input, BytesDeserializeErrorKind::GameObject));
    }

    let (scope, name) = string(scope)?;
    let (scope, prefab) = string(scope)?;
    let (scope, guid) = le_u32(scope)?;

    let err_input = scope;
    let (scope, num_components) = le_i32(scope)?;
    let num_components: usize = num_components
        .try_into()
        .map_err(|_| failure(err_input, BytesDeserializeErrorKind::GameObject))?;

    let (_scope, components) = count(component::component, num_components)(scope)?;

    let game_object = GameObject {
        name,
        prefab,
        guid,
        components,
    };

    Ok((after_game_object, game_object))
}

fn read_scope(input: Input<'_>) -> BytesParseResult<'_, (i32, Input<'_>)> {
    let (input, scope_mark) = read_scope_mark(input)?;
    let err_input = input;
    let (input, scope_len) = le_i64(input)?;
    let scope_len: usize = scope_len
        .try_into()
        .map_err(|_| failure(err_input, BytesDeserializeErrorKind::Scope))?;

    let (input, scope_data) = take(scope_len)(input)?;

    Ok((input, (scope_mark, scope_data)))
}

fn read_scope_mark(input: Input<'_>) -> BytesParseResult<'_, i32> {
    le_i32(input)
}

fn read_vector_3(input: Input<'_>) -> BytesParseResult<'_, Option<Vector3<f32>>> {
    make_optional_read(read_vector_3_raw)(input)
}

fn read_vector_3_raw(input: Input<'_>) -> BytesParseResult<'_, Vector3<f32>> {
    let mut buf = [0.0_f32; 3];
    let (input, _) = fill(le_f32, &mut buf)(input)?;

    Ok((input, Vector3::from(buf)))
}

fn read_quaternion(input: Input<'_>) -> BytesParseResult<'_, Option<Quaternion<f32>>> {
    make_optional_read(read_quaternion_raw)(input)
}

fn read_quaternion_raw(input: Input<'_>) -> BytesParseResult<'_, Quaternion<f32>> {
    let mut buf = [0.0_f32; 4];
    let (input, _) = fill(le_f32, &mut buf)(input)?;

    Ok((input, Quaternion::from(buf)))
}

fn make_optional_read<'a, O, F>(
    mut f: F,
) -> impl FnMut(Input<'a>) -> BytesParseResult<'a, Option<O>>
where
    F: Parser<Input<'a>, O, BytesDeserializeError>,
{
    move |input| match le_i32::<_, BytesDeserializeError>(input) {
        Ok((input, n)) if n == 0x7FFF_FFFD => Ok((input, None)),
        _ => {
            let (input, value) = f.parse(input)?;

            Ok((input, Some(value)))
        }
    }
}
