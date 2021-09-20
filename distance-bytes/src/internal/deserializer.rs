use crate::internal::component::{Component, ComponentBuilder, ComponentData, RawComponentData};
use crate::internal::{
    string, util, ComponentId, GameObject, Quaternion, Serializable, Vector3, VisitDirection,
    Visitor, EMPTY_MARK,
};
use crate::DistanceDateTime;
use anyhow::Result;
use byteorder::{ReadBytesExt, LE};
use paste::paste;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::io::{Read, Seek, SeekFrom};
use std::{fmt, io, mem};
use tracing::{debug, warn};

pub fn read_game_object(reader: impl Read + Seek) -> Result<GameObject> {
    Deserializer::new(reader).read_game_object()
}

#[derive(Debug, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct Deserializer<R: Read + Seek> {
    reader: R,
    scope_info_stack: Vec<ScopeInfo>,
}

impl<R: Read + Seek> Deserializer<R> {
    fn new(reader: R) -> Self {
        Deserializer {
            reader,
            scope_info_stack: Vec::new(),
        }
    }

    fn read_game_object(&mut self) -> Result<GameObject> {
        let (prefab_name, guid) = self.read_game_object_start(true)?;
        let components = self.read_game_object_contents(guid)?;

        // FIXME: This might need to be false under some circumstances.
        let log_warn = true;

        self.read_end_scope(log_warn)?;

        let game_object = GameObject {
            name: prefab_name,
            guid,
            components,
        };

        Ok(game_object)
    }

    fn read_game_object_contents(&mut self, _guid: u32) -> Result<Vec<Component>> {
        self.read_components()
    }

    // TODO: Check if necessary, and if so, implement and call this function from the proper places.
    #[allow(dead_code)]
    fn add_object_to_references(&mut self, _guid: u32) {}

    fn read_components(&mut self) -> Result<Vec<Component>> {
        let mut num_components = 0;
        self.read_set_i32("numComponents", &mut num_components)?;
        let mut components = Vec::with_capacity(num_components.try_into()?);
        for _ in 0..num_components {
            if let Some(component) = self.read_component()? {
                components.push(component);
            }
        }

        Ok(components)
    }

    fn read_component(&mut self) -> Result<Option<Component>> {
        let mut component_id = ComponentId::Invalid_;
        let mut name = String::new();
        let mut component_version = 0;
        let mut guid = 0;

        let scope_mark = self.read_start_scope(true)?;
        match scope_mark {
            33333333 | 22222222 | 32323232 => {
                let mut raw_id = 0;
                self.read_set_i32("componentID", &mut raw_id)?;
                if let Ok(id_2) = ComponentId::try_from(raw_id) {
                    component_id = id_2;
                } else {
                    warn!(id = raw_id, "unknown componentID");
                }

                name = format!("{:?}", component_id);
                self.read_set_i32("componentVersion", &mut component_version)?;
            }
            23232323 => {
                self.read_set_string("componentVersion", &mut name)?;
            }
            mark => {
                name = "Invalid".to_owned();
                warn!(mark, "invalid component mark");
            }
        }

        self.read_set_u32("component GUID", &mut guid)?;
        self.set_current_scope_name(format!("Comp:{}", name));

        if component_id != ComponentId::Invalid_ {
            let component = self.read_component_helper(component_id, component_version, guid)?;
            Ok(Some(component))
        } else {
            debug!(name = name.as_str(), guid, "skipping unknown component");
            Ok(None)
        }
    }

    fn read_component_helper(
        &mut self,
        component_id: ComponentId,
        version: i32,
        guid: u32,
    ) -> Result<Component> {
        let is_default_component = self.is_empty_scope()?;
        let builder = DeserializerComponentDataBuilder {
            deserilizer: self,
            version,
            guid,
            is_default_component,
        };
        let component = Component::from_builder(component_id, builder)?;
        Ok(component)
    }

    fn check_and_adjust_for_scope_bounds<NextElement>(&mut self) -> Result<bool> {
        let scope_info = match self.scope_info_stack.last() {
            Some(info) => info,
            None => {
                return Ok(false);
            }
        };

        let stream_position = self.reader.stream_position()?;
        let size_of_next_element: u64 = mem::size_of::<NextElement>().try_into()?;
        let scope_end: u64 = scope_info.end_pos.try_into()?;
        if stream_position + size_of_next_element > scope_end {
            self.reader.seek(SeekFrom::Start(scope_end))?;

            return Ok(false);
        }

        Ok(true)
    }

    fn empty_marker(&mut self) -> Result<bool> {
        const MARK_SIZE: usize = mem::size_of::<i32>();

        let mut buf = [0_u8; 4];
        if let Err(e) = self.reader.read_exact(&mut buf) {
            return if e.kind() == io::ErrorKind::UnexpectedEof {
                self.reader.seek(SeekFrom::Current(-(MARK_SIZE as i64)))?;
                Ok(false)
            } else {
                Err(e.into())
            };
        }

        let n = i32::from_le_bytes(buf);
        if n == EMPTY_MARK {
            Ok(true)
        } else {
            self.reader.seek(SeekFrom::Current(-(MARK_SIZE as i64)))?;
            Ok(false)
        }
    }

    fn is_empty_scope(&mut self) -> Result<bool> {
        if let Some(scope_info) = self.scope_info_stack.last() {
            Ok(self.reader.stream_position()? == u64::try_from(scope_info.end_pos)?)
        } else {
            warn!("ScopeInfo stack was empty when accessed");

            Ok(true)
        }
    }

    fn read_set_string(&mut self, _name: &str, val: &mut String) -> Result<()> {
        *val = string::read(&mut self.reader)?;

        Ok(())
    }

    fn read_game_object_start(&mut self, push_in_scope_stack: bool) -> Result<(String, u32)> {
        let mut name = String::new();
        let mut guid = 0;
        self.read_start_scope_with_mark(66666666, push_in_scope_stack)?;
        self.read_set_string("GameObject", &mut name)?;
        self.set_current_scope_name(format!("GO:{}", &name));
        self.read_set_string("Prefab", &mut String::new())?;
        self.read_set_u32("guid", &mut guid)?;

        Ok((name, guid))
    }

    fn read_start_scope(&mut self, push_in_scope_stack: bool) -> Result<i32> {
        let mark = self.reader.read_i32::<LE>()?;
        self.read_start_scope_helper(mark, push_in_scope_stack)?;

        Ok(mark)
    }

    fn read_start_scope_with_mark(&mut self, mark: i32, push_in_scope_stack: bool) -> Result<()> {
        let n = self.reader.read_i32::<LE>()?;
        if n == mark {
            self.read_start_scope_helper(mark, push_in_scope_stack)?;
        } else {
            warn!(
                expected_mark = mark,
                expected_mark_name = util::scope_mark_string(mark),
                found = n,
                "Expected mark wasn't found. Stack: {:?}",
                &self.scope_info_stack
            );
        }

        Ok(())
    }

    fn read_start_scope_helper(&mut self, mark: i32, push_in_scope_stack: bool) -> Result<()> {
        let scope_len: usize = self.reader.read_i64::<LE>()?.try_into()?;
        if push_in_scope_stack {
            let start = self.reader.stream_position()?.try_into()?;
            let end = start + scope_len;
            let new_scope_info = ScopeInfo::new(mark, start, end);
            self.scope_info_stack.push(new_scope_info);
        }

        Ok(())
    }

    fn read_end_scope(&mut self, log_warn: bool) -> Result<()> {
        if let Some(scope_info) = self.scope_info_stack.pop() {
            self.read_end_scope_helper(&scope_info, log_warn)?;
        } else {
            warn!("ScopeInfo stack was empty when accessed");
        }

        Ok(())
    }

    fn read_end_scope_helper(&mut self, scope_info: &ScopeInfo, log_warn: bool) -> Result<()> {
        let actual_pos = self.reader.stream_position()?;
        let info_pos: u64 = scope_info.end_pos.try_into()?;
        let str_1 = match actual_pos.cmp(&info_pos) {
            Ordering::Less => "understepped",
            Ordering::Equal => {
                return Ok(());
            }
            Ordering::Greater => "overstepped",
        };

        if log_warn {
            warn!(
                scope = scope_info.scope_mark_string(),
                "A scope was {} when reading. Stack: {:?}", str_1, &self.scope_info_stack
            );
        }

        self.reader.seek(SeekFrom::Start(info_pos))?;

        Ok(())
    }

    fn set_current_scope_name(&mut self, name: impl Into<Cow<'static, str>>) {
        if let Some(scope_info) = self.scope_info_stack.last_mut() {
            scope_info.name = name.into();
        }
    }

    fn read_set_u8(&mut self, _name: &str, val: &mut u8) -> Result<()> {
        if self.check_and_adjust_for_scope_bounds::<u8>()? {
            *val = self.reader.read_u8()?;
        }

        Ok(())
    }

    fn read_array_start(&mut self) -> Result<i32> {
        let mut mark = 0;
        self.read_set_i32("arrayMark", &mut mark)?;

        let mut len = -1;
        if mark == 11111111 {
            self.read_set_i32("array size", &mut len)?;
        } else {
            warn!("expected array mark `11111111` when reading the start of the array; found {} instead", mark);
        }

        Ok(len)
    }

    fn read_dictionary_start(&mut self) -> Result<i32> {
        let mut mark = 0;
        self.read_set_i32("dictionaryMark", &mut mark)?;

        let mut len = -1;
        if mark == 12121212 {
            self.read_set_i32("dictionarySize", &mut len)?;
        } else {
            warn!("expected dictionary mark `12121212` when reading the start of the dictionary; found {} instead", mark);
        }

        Ok(len)
    }
}

impl<R: Read + Seek> Visitor for Deserializer<R> {
    type Self_ = Self;

    const VISIT_DIRECTION: VisitDirection = VisitDirection::In;

    fn visit_bool(&mut self, name: &str, value: &mut bool) -> Result<()> {
        if !self.empty_marker()? {
            let mut n = *value as u8;
            self.read_set_u8(name, &mut n)?;
            *value = n != 0;
        }

        Ok(())
    }

    fn visit_i32(&mut self, name: &str, value: &mut i32) -> Result<()> {
        if !self.empty_marker()? {
            self.read_set_i32(name, value)?;
        }

        Ok(())
    }

    fn visit_u32(&mut self, name: &str, value: &mut u32) -> Result<()> {
        if !self.empty_marker()? {
            self.read_set_u32(name, value)?;
        }

        Ok(())
    }

    fn visit_i64(&mut self, name: &str, value: &mut i64) -> Result<()> {
        if !self.empty_marker()? {
            self.read_set_i64(name, value)?;
        }

        Ok(())
    }

    fn visit_f32(&mut self, name: &str, value: &mut f32) -> Result<()> {
        if !self.empty_marker()? {
            self.read_set_f32(name, value)?;
        }

        Ok(())
    }

    fn visit_f64(&mut self, name: &str, value: &mut f64) -> Result<()> {
        if !self.empty_marker()? {
            self.read_set_f64(name, value)?;
        }

        Ok(())
    }

    fn visit_string(&mut self, name: &str, value: &mut Option<String>) -> Result<()> {
        if !self.empty_marker()? {
            let mut s = String::new();
            self.read_set_string(name, &mut s)?;
            *value = Some(s);
        }

        Ok(())
    }

    fn visit_datetime(&mut self, name: &str, value: &mut DistanceDateTime) -> Result<()> {
        if !self.empty_marker()? {
            self.read_set_i64(name, &mut value.0)?;
        }

        Ok(())
    }

    fn visit_vector_3(&mut self, _name: &str, value: &mut Vector3) -> Result<()> {
        if !self.empty_marker()? {
            self.read_set_f32("x", &mut value.x)?;
            self.read_set_f32("y", &mut value.y)?;
            self.read_set_f32("z", &mut value.z)?;
        }

        Ok(())
    }

    fn visit_quaternion(&mut self, _name: &str, value: &mut Quaternion) -> Result<()> {
        if !self.empty_marker()? {
            self.read_set_f32("x", &mut value.v.x)?;
            self.read_set_f32("y", &mut value.v.y)?;
            self.read_set_f32("z", &mut value.v.z)?;
            self.read_set_f32("w", &mut value.s)?;
        }

        Ok(())
    }

    fn visit_reference(&mut self, name: &str, value: &mut u32) -> Result<()> {
        self.read_set_u32(name, value)?;

        Ok(())
    }

    fn visit_reference_array(
        &mut self,
        _array_name: &str,
        element_name: &str,
        value: &mut Vec<u32>,
    ) -> Result<()> {
        let len: usize = self.read_array_start()?.try_into().unwrap_or(0);
        value.clear();
        value.resize(len, 0);

        for reference in value {
            self.visit_reference(element_name, reference)?;
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
        let array_len = usize::try_from(self.read_array_start()?);
        if let Ok(len) = array_len {
            array.clear();
            array.resize_with(len, T::default);
            for element in array {
                visit_t_fn(self, element)?;
            }
        }

        Ok(())
    }

    fn visit_dictionary_generic<Key, Value, KeyAcceptor, ValueAcceptor>(
        &mut self,
        _name: &str,
        value: &mut Option<HashMap<Key, Value>>,
        mut key_acceptor: KeyAcceptor,
        mut value_acceptor: ValueAcceptor,
        default_key: Key,
        default_value: Value,
    ) -> Result<()>
    where
        Key: Clone + Hash + Eq,
        Value: Clone,
        KeyAcceptor: FnMut(&mut Self, &mut Key) -> Result<()>,
        ValueAcceptor: FnMut(&mut Self, &mut Value) -> Result<()>,
    {
        let len = self.read_dictionary_start()?;

        let dictionary = match value {
            Some(x) => x,
            None => {
                *value = Some(HashMap::with_capacity(len.try_into().unwrap_or(0)));
                value.as_mut().unwrap()
            }
        };
        dictionary.clear();

        for _ in 0..len {
            let mut key = default_key.clone();
            key_acceptor(self, &mut key)?;

            let mut value = default_value.clone();
            value_acceptor(self, &mut value)?;

            dictionary.insert(key, value);
        }

        Ok(())
    }

    fn visit_children(&mut self, value: &mut Vec<GameObject>) -> Result<()> {
        self.read_start_scope_with_mark(55555555, true)?;
        let mut num_children = 0;
        self.read_set_i32("numberOfChildren", &mut num_children)?;
        self.set_current_scope_name(format!("ChildNum:{}", num_children));
        for _ in 0..num_children {
            let child = self.read_game_object()?;
            value.push(child);
        }

        Ok(())
    }
}

macro_rules! impl_read_set {
    ($type_:ty) => {
        impl<R: Read + Seek> Deserializer<R> {
            paste! {
                fn [<read_set_ $type_>](&mut self, _name: &str, val: &mut $type_) -> Result<()> {
                    if self.check_and_adjust_for_scope_bounds::<$type_>()? {
                        *val = self.reader.[<read_ $type_>]::<LE>()?;
                    }

                    Ok(())
                }
            }
        }
    };
}

impl_read_set!(i32);
impl_read_set!(u32);
impl_read_set!(i64);
impl_read_set!(f32);
impl_read_set!(f64);

#[derive(Debug, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct ScopeInfo {
    name: Cow<'static, str>,
    scope_mark: i32,
    start_pos: usize,
    end_pos: usize,
}

impl ScopeInfo {
    pub fn new(scope_mark: i32, start_pos: usize, end_pos: usize) -> Self {
        ScopeInfo {
            name: "".into(),
            scope_mark,
            start_pos,
            end_pos,
        }
    }

    pub fn scope_mark_string(&self) -> &'static str {
        util::scope_mark_string(self.scope_mark)
    }
}

impl Display for ScopeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.name, self.scope_mark_string())
    }
}

struct DeserializerComponentDataBuilder<'a, R: Read + Seek> {
    deserilizer: &'a mut Deserializer<R>,
    version: i32,
    guid: u32,
    is_default_component: bool,
}

impl<R: Read + Seek> ComponentBuilder for DeserializerComponentDataBuilder<'_, R> {
    fn implemented<T: Serializable>(
        &mut self,
        component_data_constructor: fn(T) -> ComponentData,
        implemented_version: i32,
    ) -> Result<Component> {
        let mut inner_component = T::default();
        if !self.is_default_component {
            inner_component.accept(&mut self.deserilizer, self.version)?;
        }
        let component_data = component_data_constructor(inner_component);
        let component = Component {
            version: implemented_version,
            guid: self.guid,
            data: component_data,
        };

        Ok(component)
    }

    fn raw(
        &mut self,
        component_data_constructor: fn(RawComponentData) -> ComponentData,
    ) -> Result<Component> {
        let component_data = if self.is_default_component {
            component_data_constructor(RawComponentData::default())
        } else {
            let current_pos: usize = self.deserilizer.reader.stream_position()?.try_into()?;
            let data_len = self
                .deserilizer
                .scope_info_stack
                .last()
                .map(|scope_info| scope_info.end_pos - current_pos)
                .unwrap_or(0);

            let mut data = vec![0; data_len];
            self.deserilizer.reader.read_exact(&mut data)?;

            component_data_constructor(RawComponentData(data))
        };
        let component = Component {
            version: self.version,
            guid: self.guid,
            data: component_data,
        };

        Ok(component)
    }
}
