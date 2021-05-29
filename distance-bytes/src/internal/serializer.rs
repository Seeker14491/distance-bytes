use crate::internal::component::{ComponentDataDispatch, RawComponentData};
use crate::internal::{
    string, util, Component, GameObject, Quaternion, Serializable, Vector3, VisitDirection,
    Visitor, EMPTY_MARK, INVALID_FLOAT, INVALID_INT, INVALID_QUATERNION, INVALID_VECTOR_3,
};
use anyhow::Result;
use byteorder::{WriteBytesExt, LE};
use std::convert::TryInto;
use std::io::{Seek, SeekFrom, Write};
use util::ApproximatelyEquals;

pub fn write_game_object(
    writer: impl Write + Seek,
    game_object: &mut GameObject,
) -> Result<()> {
    Serializer::new(writer).write_game_object(game_object)
}

#[derive(Debug, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Serializer<W: Write + Seek> {
    writer: W,
    scope_stack: Vec<u64>,
}

impl<W: Write + Seek> Serializer<W> {
    fn new(writer: W) -> Self {
        Serializer {
            writer,
            scope_stack: Vec::new(),
        }
    }

    fn write_game_object(&mut self, game_object: &mut GameObject) -> Result<()> {
        self.write_start_scope(66666666)?;
        self.write_string(&game_object.name)?;
        self.write_string("")?;
        self.writer.write_u32::<LE>(game_object.guid)?;

        self.write_components(&mut game_object.components)?;

        self.write_end_scope(-1)?;

        Ok(())
    }

    fn write_components(&mut self, components: &mut [Component]) -> Result<()> {
        self.writer.write_i32::<LE>(components.len().try_into()?)?;
        for component in components {
            self.write_component(component)?;
        }

        Ok(())
    }

    fn write_component(&mut self, component: &mut Component) -> Result<()> {
        // The game would actually write either `22222222` or `33333333` here, but any of these
        // values work the same when read.
        self.write_component_start(component, 32323232)?;

        self.write_component_helper(component)?;
        self.write_end_scope(-1)?;

        Ok(())
    }

    #[rustfmt::skip]
    fn write_component_helper(&mut self, component: &mut Component) -> Result<()> {
        let dispatcher = SerializerComponentDataDispatcher {
            serializer: self,
            version: component.version,
        };

        component.data.dispatch(dispatcher)
    }

    fn write_component_start(
        &mut self,
        component: &Component,
        scope_mark: i32,
    ) -> Result<()> {
        self.write_start_scope(scope_mark)?;
        self.writer.write_i32::<LE>(component.id().into())?;
        self.writer.write_i32::<LE>(component.version)?;
        self.writer.write_u32::<LE>(component.guid)?;

        Ok(())
    }

    fn write_start_scope(&mut self, mark: i32) -> Result<()> {
        self.writer.write_i32::<LE>(mark)?;

        // Temporary stand-in for scope length
        self.writer.write_i64::<LE>(mark.into())?;

        self.scope_stack.push(self.writer.stream_position()?);

        Ok(())
    }

    fn write_end_scope(&mut self, scope_info: i64) -> Result<()> {
        let stack_pos = self
            .scope_stack
            .pop()
            .expect("unexpected empty scope stack");
        let section_len: i64 = (self.writer.stream_position()? - stack_pos).try_into()?;

        self.writer.seek(SeekFrom::Current(-(section_len + 8)))?;
        let value_to_write = if scope_info == -1 {
            section_len
        } else {
            scope_info
        };
        self.writer.write_i64::<LE>(value_to_write)?;
        self.writer.seek(SeekFrom::Current(section_len))?;

        Ok(())
    }

    fn write_empty(&mut self) -> Result<()> {
        self.writer.write_i32::<LE>(EMPTY_MARK)?;

        Ok(())
    }

    fn write_string(&mut self, string: &str) -> Result<()> {
        string::write(&mut self.writer, string)
    }

    fn write_array_start(&mut self, _name: &str, len: i32) -> Result<()> {
        self.writer.write_i32::<LE>(11111111)?;
        self.writer.write_i32::<LE>(len)?;

        Ok(())
    }
}

impl<W: Write + Seek> Visitor for Serializer<W> {
    type Self_ = Self;

    const VISIT_DIRECTION: VisitDirection = VisitDirection::Out;

    fn visit_bool(&mut self, _name: &str, value: &mut bool) -> Result<()> {
        self.writer.write_u8(*value as u8)?;

        Ok(())
    }

    fn visit_i32(&mut self, _name: &str, value: &mut i32) -> Result<()> {
        if *value != INVALID_INT {
            self.writer.write_i32::<LE>(*value)?;
        } else {
            self.write_empty()?;
        }

        Ok(())
    }

    fn visit_u32(&mut self, _name: &str, value: &mut u32) -> Result<()> {
        if *value != 0xFFFF_FF81 {
            self.writer.write_u32::<LE>(*value)?;
        } else {
            self.write_empty()?;
        }

        Ok(())
    }

    fn visit_i64(&mut self, _name: &str, value: &mut i64) -> Result<()> {
        if *value != INVALID_INT as i64 {
            self.writer.write_i64::<LE>(*value)?;
        } else {
            self.write_empty()?;
        }

        Ok(())
    }

    fn visit_f32(&mut self, _name: &str, value: &mut f32) -> Result<()> {
        if !value.approximately_equals(&INVALID_FLOAT) {
            self.writer.write_f32::<LE>(*value)?;
        } else {
            self.write_empty()?;
        }

        Ok(())
    }

    fn visit_string(&mut self, _name: &str, value: &mut Option<String>) -> Result<()> {
        match value {
            Some(s) => {
                self.write_string(s)?;
            }
            None => {
                self.write_empty()?;
            }
        }

        Ok(())
    }

    fn visit_vector_3(&mut self, _name: &str, value: &mut Vector3) -> Result<()> {
        if !value.approximately_equals(&INVALID_VECTOR_3) {
            self.writer.write_f32::<LE>(value.x)?;
            self.writer.write_f32::<LE>(value.y)?;
            self.writer.write_f32::<LE>(value.z)?;
        } else {
            self.write_empty()?;
        }

        Ok(())
    }

    fn visit_quaternion(&mut self, _name: &str, value: &mut Quaternion) -> Result<()> {
        if !value.approximately_equals(&INVALID_QUATERNION) {
            self.writer.write_f32::<LE>(value.v.x)?;
            self.writer.write_f32::<LE>(value.v.y)?;
            self.writer.write_f32::<LE>(value.v.z)?;
            self.writer.write_f32::<LE>(value.s)?;
        } else {
            self.write_empty()?;
        }

        Ok(())
    }

    fn visit_reference(&mut self, _name: &str, value: &mut u32) -> Result<()> {
        self.writer.write_u32::<LE>(*value)?;

        Ok(())
    }

    fn visit_reference_array(
        &mut self,
        array_name: &str,
        element_name: &str,
        value: &mut Vec<u32>,
    ) -> Result<()> {
        self.write_array_start(array_name, value.len().try_into()?)?;
        for v in value {
            self.visit_reference(element_name, v)?;
        }

        Ok(())
    }

    fn visit_array<T, F>(
        &mut self,
        _element_name: &str,
        array: &mut Vec<T>,
        mut visit_t_fn: F,
    ) -> Result<()>
    where
        T: Default,
        F: FnMut(&mut Self::Self_, &mut T) -> Result<()>,
    {
        self.write_array_start("Ints", array.len().try_into()?)?;
        for element in array {
            visit_t_fn(self, element)?;
        }

        Ok(())
    }

    fn visit_children(&mut self, value: &mut Vec<GameObject>) -> Result<()> {
        self.write_start_scope(55555555)?;

        let num_children: i32 = value.len().try_into()?;
        self.writer.write_i32::<LE>(num_children)?;

        for game_object in value {
            self.write_game_object(game_object)?;
        }

        self.write_end_scope(-1)?;

        Ok(())
    }
}

struct SerializerComponentDataDispatcher<'a, W: Write + Seek> {
    serializer: &'a mut Serializer<W>,
    version: i32,
}

impl<W: Write + Seek> ComponentDataDispatch for SerializerComponentDataDispatcher<'_, W> {
    fn implemented<T: Serializable>(&mut self, data: &mut T) -> Result<()> {
        data.accept(&mut self.serializer, self.version)
    }

    fn raw(&mut self, data: &RawComponentData) -> Result<()> {
        self.serializer.writer.write_all(&data.0)?;

        Ok(())
    }
}
