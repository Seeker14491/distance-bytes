//! Pretty JSON that keeps arrays of plain values, such as vectors and colors, on one line.

use std::io::{self, Write};

use serde_json::ser::{CompactFormatter, Formatter};

#[derive(Default)]
pub(crate) struct Pretty {
    depth: usize,
    stack: Vec<Frame>,
    /// After `begin_array_value`: whether the value is the array's first.
    pending: Option<bool>,
}

#[derive(Clone, Copy, PartialEq)]
enum Frame {
    Object {
        empty: bool,
    },
    /// An array whose values so far are on one line (or that has none).
    Inline,
    /// An array with a value on each line, once one of its values is an array or object.
    Multiline,
}

impl Pretty {
    fn newline<W: ?Sized + Write>(&self, w: &mut W) -> io::Result<()> {
        const SPACES: &[u8; 64] = &[b' '; 64];
        w.write_all(b"\n")?;
        let mut n = self.depth * 2;
        while n > 0 {
            let k = n.min(SPACES.len());
            w.write_all(&SPACES[..k])?;
            n -= k;
        }
        Ok(())
    }

    /// Before an array element that is a plain value.
    fn plain<W: ?Sized + Write>(&mut self, w: &mut W) -> io::Result<()> {
        let Some(first) = self.pending.take() else {
            return Ok(());
        };
        if !first {
            w.write_all(b",")?;
        }
        match self.stack.last() {
            Some(Frame::Multiline) => self.newline(w),
            _ if first => Ok(()),
            _ => w.write_all(b" "),
        }
    }

    /// Before an array or object.
    fn container<W: ?Sized + Write>(&mut self, w: &mut W) -> io::Result<()> {
        let Some(first) = self.pending.take() else {
            return Ok(());
        };
        if !first {
            w.write_all(b",")?;
        }
        if let Some(frame) = self.stack.last_mut() {
            *frame = Frame::Multiline;
        }
        self.newline(w)
    }
}

macro_rules! plain {
    ($($method:ident($ty:ty);)*) => {
        $(fn $method<W: ?Sized + Write>(&mut self, w: &mut W, v: $ty) -> io::Result<()> {
            self.plain(w)?;
            CompactFormatter.$method(w, v)
        })*
    };
}

impl Formatter for Pretty {
    plain! {
        write_bool(bool);
        write_i8(i8);
        write_i16(i16);
        write_i32(i32);
        write_i64(i64);
        write_i128(i128);
        write_u8(u8);
        write_u16(u16);
        write_u32(u32);
        write_u64(u64);
        write_u128(u128);
        write_f32(f32);
        write_f64(f64);
        write_number_str(&str);
        write_raw_fragment(&str);
    }

    fn write_null<W: ?Sized + Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.plain(w)?;
        w.write_all(b"null")
    }

    fn begin_string<W: ?Sized + Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.plain(w)?;
        w.write_all(b"\"")
    }

    fn begin_array<W: ?Sized + Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.container(w)?;
        self.depth += 1;
        self.stack.push(Frame::Inline);
        w.write_all(b"[")
    }

    fn end_array<W: ?Sized + Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.depth -= 1;
        if self.stack.pop() == Some(Frame::Multiline) {
            self.newline(w)?;
        }
        w.write_all(b"]")
    }

    fn begin_array_value<W: ?Sized + Write>(&mut self, _w: &mut W, first: bool) -> io::Result<()> {
        self.pending = Some(first);
        Ok(())
    }

    fn end_array_value<W: ?Sized + Write>(&mut self, _w: &mut W) -> io::Result<()> {
        Ok(())
    }

    fn begin_object<W: ?Sized + Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.container(w)?;
        self.depth += 1;
        self.stack.push(Frame::Object { empty: true });
        w.write_all(b"{")
    }

    fn end_object<W: ?Sized + Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.depth -= 1;
        if self.stack.pop() == Some(Frame::Object { empty: false }) {
            self.newline(w)?;
        }
        w.write_all(b"}")
    }

    fn begin_object_key<W: ?Sized + Write>(&mut self, w: &mut W, first: bool) -> io::Result<()> {
        if let Some(Frame::Object { empty }) = self.stack.last_mut() {
            *empty = false;
        }
        if !first {
            w.write_all(b",")?;
        }
        self.newline(w)
    }

    fn end_object_key<W: ?Sized + Write>(&mut self, _w: &mut W) -> io::Result<()> {
        Ok(())
    }

    fn begin_object_value<W: ?Sized + Write>(&mut self, w: &mut W) -> io::Result<()> {
        w.write_all(b": ")
    }

    fn end_object_value<W: ?Sized + Write>(&mut self, _w: &mut W) -> io::Result<()> {
        Ok(())
    }
}
