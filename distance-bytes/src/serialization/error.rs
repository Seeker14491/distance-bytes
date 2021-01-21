use crate::{domain::ComponentId};
use nom::{Err::Failure, InputIter, InputLength, InputTake, Needed, Slice, error::ParseError, lib::std::fmt::Formatter};
use std::{
    fmt::{self, Debug, Display},
    ops::RangeFrom,
};


#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct BytesDeserializeError {
    errors: Vec<(usize, BytesDeserializeErrorKind)>,
}

impl Display for BytesDeserializeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Bytes deserialization error:")?;
        for (offset, error_kind) in &self.errors {
            writeln!(f, "{} at offset {}", error_kind, offset)?;
        }

        Ok(())
    }
}

impl std::error::Error for BytesDeserializeError {}

impl<'a> ParseError<SliceWithOffset<'a, u8>> for BytesDeserializeError {
    fn from_error_kind(input: SliceWithOffset<'a, u8>, kind: nom::error::ErrorKind) -> Self {
        BytesDeserializeError {
            errors: vec![(input.offset, BytesDeserializeErrorKind::Nom(kind))],
        }
    }

    fn append(
        input: SliceWithOffset<'a, u8>,
        kind: nom::error::ErrorKind,
        mut other: Self,
    ) -> Self {
        other
            .errors
            .push((input.offset, BytesDeserializeErrorKind::Nom(kind)));
        other
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum BytesDeserializeErrorKind {
    Nom(nom::error::ErrorKind),
    Scope,
    GameObject,
    Component(Option<ComponentId>),
    String,
}

impl Display for BytesDeserializeErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BytesDeserializeErrorKind::Nom(error_kind) => {
                write!(f, "Nom ErrorKind: {:?}", error_kind)
            }
            BytesDeserializeErrorKind::Scope => {
                write!(f, "in scope")
            }
            BytesDeserializeErrorKind::GameObject => {
                write!(f, "in GameObject")
            }
            BytesDeserializeErrorKind::Component(Some(component)) => {
                write!(f, "in {:?} component", component)
            }
            BytesDeserializeErrorKind::Component(None) => {
                write!(f, "in component",)
            }
            BytesDeserializeErrorKind::String => {
                write!(f, "in String")
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct SliceWithOffset<'a, T> {
    pub slice: &'a [T],
    pub offset: usize,
}

impl<'a, T> From<&'a [T]> for SliceWithOffset<'a, T> {
    fn from(slice: &'a [T]) -> Self {
        SliceWithOffset { slice, offset: 0 }
    }
}

impl<'a, T> AsRef<[T]> for SliceWithOffset<'a, T> {
    fn as_ref(&self) -> &[T] {
        self.slice
    }
}

impl<T> Slice<RangeFrom<usize>> for SliceWithOffset<'_, T> {
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        let SliceWithOffset { slice, offset } = *self;
        SliceWithOffset {
            slice: &slice[range.clone()],
            offset: offset + range.start,
        }
    }
}

impl<'a> InputIter for SliceWithOffset<'a, u8> {
    type Item = u8;
    type Iter = std::iter::Enumerate<Self::IterElem>;
    type IterElem = std::iter::Copied<std::slice::Iter<'a, u8>>;

    #[inline]
    fn iter_indices(&self) -> Self::Iter {
        self.iter_elements().enumerate()
    }

    #[inline]
    fn iter_elements(&self) -> Self::IterElem {
        self.slice.iter().copied()
    }

    #[inline]
    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.slice.iter().position(|b| predicate(*b))
    }

    #[inline]
    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        if self.slice.len() >= count {
            Ok(count)
        } else {
            Err(Needed::new(count - self.slice.len()))
        }
    }
}

impl<'a, T> InputLength for SliceWithOffset<'a, T> {
    #[inline]
    fn input_len(&self) -> usize {
        self.slice.len()
    }
}

impl<'a, T> InputTake for SliceWithOffset<'a, T> {
    #[inline]
    fn take(&self, count: usize) -> Self {
        let SliceWithOffset { slice, offset } = *self;

        SliceWithOffset {
            slice: &slice[0..count],
            offset,
        }
    }

    #[inline]
    fn take_split(&self, count: usize) -> (Self, Self) {
        let SliceWithOffset { slice, offset } = *self;
        let (prefix, suffix) = slice.split_at(count);

        (
            SliceWithOffset {
                slice: suffix,
                offset: offset + count,
            },
            SliceWithOffset {
                slice: prefix,
                offset,
            },
        )
    }
}

pub(crate) fn failure(
    input: SliceWithOffset<'_, u8>,
    error_kind: BytesDeserializeErrorKind,
) -> nom::Err<BytesDeserializeError> {
    Failure(BytesDeserializeError {
        errors: vec![(input.offset, error_kind)],
    })
}

// TODO
// pub(crate) fn context<O>(
//     result: IResult<Input<'_>, O, Option<BytesDeserializeError>>,
//     context: BytesDeserializeErrorKind,
// ) -> BytesParseResult<'_, O> {
//     result.map_err(|e| {
//         match e {
//             nom::Err::Incomplete(x) => nom::Err::Incomplete(x),
//             nom::Err::Error(Some(e)) => {},
//             nom::Err::Error(None) => {}
//             Failure(e) => {}
//         }
//     })
// }
