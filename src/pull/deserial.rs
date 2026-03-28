use crate::db::{Attr, Db, Eid, Ent, Val};
use crate::space::Space;
use crate::QueryError;
use serde::de::{DeserializeSeed, Error, Visitor};

struct PullSeqAccess<'a, 'de: 'a, T: Space> {
    attrs: Vec<Attr>,
    de: &'a mut PullDeserializer<'de, T>,
    index: usize,
}

impl<'de, 'a, S: Space> PullSeqAccess<'a, 'de, S> {
    fn new(attrs: Vec<Attr>, de: &'a mut PullDeserializer<'de, S>) -> Self {
        Self {
            attrs,
            de,
            index: 0,
        }
    }
}

impl<'de, 'a, S: Space> serde::de::SeqAccess<'de> for PullSeqAccess<'a, 'de, S> {
    type Error = QueryError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.index >= self.attrs.len() {
            Ok(None)
        } else {
            let attr = self.attrs[self.index];
            self.de.current_attr.push(attr);
            let result = seed.deserialize(&mut *self.de);
            self.de.current_attr.pop();
            self.index += 1;
            result.map(Some)
        }
    }
}

pub struct PullDeserializer<'de, S: Space> {
    eid: Eid,
    db: &'de Db<S>,
    current_attr: Vec<Attr>,
}

impl<'de, S: Space> PullDeserializer<'de, S> {
    pub fn new(eid: Eid, db: &'de Db<S>) -> Self {
        Self {
            eid,
            db,
            current_attr: Vec::new(),
        }
    }
    fn read_current_val(&self) -> Result<(Attr, Option<Val>), QueryError> {
        let ent = Ent::Id(self.eid);
        let attr = *self.current_attr.last().expect("current_attr is empty");
        let val = self.db.find_val(ent, attr)?;
        Ok((attr, val))
    }
}

impl<'de, T: Space> serde::Deserializer<'de> for &mut PullDeserializer<'de, T> {
    type Error = QueryError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (attr, val) = self.read_current_val()?;
        match val {
            Some(Val::U32(v)) => visitor.visit_i32(v as i32),
            _ => Err(Error::custom(format!(
                "Expected u32 value for attribute '{attr}', found {:?}",
                (attr, val)
            ))),
        }
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (attr, val) = self.read_current_val()?;
        match val {
            Some(Val::U32(v)) => visitor.visit_u32(v),
            _ => Err(Error::custom(format!(
                "Expected u32 value for attribute '{attr}', found {:?}",
                (attr, val)
            ))),
        }
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_str<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (attr, val) = self.read_current_val()?;
        match val {
            Some(Val::String(v)) => visitor.visit_string(v),
            _ => Err(Error::custom(format!(
                "Expected string value for attribute '{attr}', found {:?}",
                (attr, val)
            ))),
        }
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let attrs = fields.iter().map(|f| Attr(name, f)).collect::<Vec<_>>();
        let access = PullSeqAccess::new(attrs, self);
        visitor.visit_seq(access)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
}
