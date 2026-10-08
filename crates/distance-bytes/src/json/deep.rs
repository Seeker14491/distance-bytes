//! Reading deeply nested objects without deep recursion.
//!
//! Objects nest hundreds of levels deep (469 in one Workshop level), and reading one level of
//! objects takes several levels of recursion through the nested JSON values. Natively the stack
//! grows as needed, but on wasm it can't, and a browser's stack holds about 500 levels. So
//! [`from_str`](super::from_str) and [`from_slice`](super::from_slice) keep the children of the
//! objects [`DEPTH`] levels down as text, and read them separately once the stack has unwound
//! to the children of a root object.

use std::cell::RefCell;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;

use super::Error;
use crate::string::DString;
use crate::tree::{ComponentBody, GameObject};

/// The depth of the objects whose children are kept as text, roots being at depth 0.
const DEPTH: usize = 64;

/// What a `from_str` or `from_slice` keeps track of.
struct State {
    /// The address of the text, to find kept children in it.
    text: usize,
    /// How many `children` arrays are being read.
    depth: usize,
    /// Children kept as text, with where they start in the text.
    kept: Vec<(usize, String)>,
    /// An error in kept children, with where they start in the text.
    error: Option<(usize, Error)>,
}

thread_local! {
    /// The states of the `from_str` and `from_slice` calls running, innermost last.
    static STATES: RefCell<Vec<State>> = const { RefCell::new(Vec::new()) };
}

/// Runs `f` on the state of the innermost `from_str` or `from_slice`, if one is running.
fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> Option<R> {
    STATES.with_borrow_mut(|states| states.last_mut().map(f))
}

/// Runs `read`, which reads `text`.
pub(super) fn reading<T>(text: &[u8], read: impl FnOnce() -> Result<T, Error>) -> Result<T, Error> {
    struct Pop;
    impl Drop for Pop {
        fn drop(&mut self) {
            STATES.with_borrow_mut(Vec::pop);
        }
    }
    STATES.with_borrow_mut(|states| {
        states.push(State {
            text: text.as_ptr() as usize,
            depth: 0,
            kept: Vec::new(),
            error: None,
        })
    });
    let _pop = Pop;
    read().map_err(|e| match with_state(|s| s.error.take()).flatten() {
        Some((offset, error)) => error.within(text, offset),
        None => e,
    })
}

/// Reads the `children` of a `Transform`, for `#[serde(deserialize_with)]`.
pub(crate) fn children<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<GameObject>, D::Error> {
    let Some(depth) = with_state(|s| s.depth) else {
        return Vec::deserialize(d);
    };
    if depth == DEPTH {
        let text = <&RawValue>::deserialize(d)?.get();
        let index = with_state(|s| {
            let offset = (text.as_ptr() as usize).wrapping_sub(s.text);
            s.kept.push((offset, text.to_owned()));
            s.kept.len() - 1
        });
        // Stands in for the children until they're read, with the index as its GUID.
        return Ok(vec![GameObject {
            name: DString::default(),
            prefab: DString::default(),
            guid: index.unwrap_or_default() as u32,
            components: Vec::new(),
        }]);
    }
    with_state(|s| s.depth += 1);
    let children = Vec::deserialize(d);
    with_state(|s| s.depth -= 1);
    let mut children = children?;
    if depth == 0 && read_kept(&mut children).is_err() {
        // `reading` replaces this with the error.
        return Err(D::Error::custom("error in deeply nested children"));
    }
    Ok(children)
}

/// Reads the children kept as text among the descendants of `children`, which are the children
/// of a root object.
fn read_kept(children: &mut Vec<GameObject>) -> Result<(), ()> {
    // The children of objects at `DEPTH` are all kept, so they're found without recursion.
    let mut kept = Vec::new();
    let mut objects = vec![(children, 1)];
    while let Some((children, depth)) = objects.pop() {
        for obj in children {
            for c in &mut obj.components {
                if let ComponentBody::Transform(t) = &mut c.body
                    && !t.children.is_empty()
                {
                    if depth == DEPTH {
                        kept.push(&mut t.children);
                    } else {
                        objects.push((&mut t.children, depth + 1));
                    }
                }
            }
        }
    }
    for children in kept {
        let index = children[0].guid as usize;
        let Some((offset, text)) =
            with_state(|s| s.kept.get_mut(index).map(std::mem::take)).flatten()
        else {
            return Err(());
        };
        match super::from_str(&text) {
            Ok(objects) => *children = objects,
            Err(e) => {
                with_state(|s| s.error = Some((offset, e)));
                return Err(());
            }
        }
    }
    Ok(())
}

/// The path of the value that starts at `offset` in JSON `text`, as `serde_path_to_error`
/// writes paths. The text before it is valid JSON.
pub(super) fn path_at(text: &[u8], offset: usize) -> String {
    enum Frame {
        /// An object, with the key of the current value.
        Object(Option<String>),
        /// An array, with the index of the current value.
        Array(usize),
    }
    let mut frames = Vec::new();
    let mut i = 0;
    while i < offset.min(text.len()) {
        match text[i] {
            b'{' => frames.push(Frame::Object(None)),
            b'[' => frames.push(Frame::Array(0)),
            b'}' | b']' => drop(frames.pop()),
            b',' => match frames.last_mut() {
                Some(Frame::Object(key)) => *key = None,
                Some(Frame::Array(index)) => *index += 1,
                None => {}
            },
            b'"' => {
                let start = i;
                i += 1;
                while i < text.len() && text[i] != b'"' {
                    i += if text[i] == b'\\' { 2 } else { 1 };
                }
                // A string in an object before a value is a key.
                if let Some(Frame::Object(key @ None)) = frames.last_mut() {
                    let string = text.get(start..=i).unwrap_or_default();
                    *key = Some(serde_json::from_slice(string).unwrap_or_default());
                }
            }
            _ => {}
        }
        i += 1;
    }
    let mut path = String::new();
    for frame in frames {
        match frame {
            Frame::Array(index) => path += &format!("[{index}]"),
            Frame::Object(key) => {
                if !path.is_empty() {
                    path.push('.');
                }
                path += key.as_deref().unwrap_or("?");
            }
        }
    }
    path
}
