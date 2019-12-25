use std::fmt::Display;
use std::io::{self, Write};

use serde::ser::{self, Serialize};
use thiserror::Error;

use crate::varuint::encode_u64;

pub fn to_writer<W: Write, T: Serialize>(w: W, value: T) -> Result<(), Error> {
    let mut serializer = Serializer::new(w);
    value.serialize(&mut serializer)?;
    serializer.end()?;
    Ok(())
}

#[derive(Debug)]
pub struct Serializer<W: Write> {
    w: W,
}

impl<W: Write> Serializer<W> {
    pub fn new(w: W) -> Serializer<W> {
        Serializer { w }
    }

    /// This method should be called after a value has been serialized to ensure all output data written to writer.
    pub fn end(&mut self) -> Result<(), Error> {
        self.w.flush()?;
        Ok(())
    }
}

impl<'a, W: Write> ser::Serializer for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Compound<'a, W>;
    type SerializeTuple = Compound<'a, W>;
    type SerializeTupleStruct = Compound<'a, W>;
    type SerializeTupleVariant = Compound<'a, W>;
    type SerializeMap = Compound<'a, W>;
    type SerializeStruct = Compound<'a, W>;
    type SerializeStructVariant = Compound<'a, W>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        let bs: [u8; 1] = if v { [1] } else { [0] };

        self.w.write_all(&bs[..])?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        let bs = v.to_le_bytes();
        self.w.write_all(&bs[..])?;
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        let u = if v >= 0 {
            (v as u16) << 1
        } else {
            ((-(v + 1)) as u16) << 1 | 1
        };
        u.serialize(self)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        let u = if v >= 0 {
            (v as u32) << 1
        } else {
            ((-(v + 1)) as u32) << 1 | 1
        };
        u.serialize(self)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        let u = if v >= 0 {
            (v as u64) << 1
        } else {
            ((-(v + 1)) as u64) << 1 | 1
        };
        u.serialize(self)
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        let u = if v >= 0 {
            (v as u128) << 1
        } else {
            ((-(v + 1)) as u128) << 1 | 1
        };
        u.serialize(self)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        let bs = v.to_le_bytes();
        self.w.write_all(&bs[..])?;
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        encode_u64(&mut self.w, v as u64)?;
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        encode_u64(&mut self.w, v as u64)?;
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        encode_u64(&mut self.w, v)?;
        Ok(())
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        let upper = 0xff_ff_ff_ff_ff_ff_ff_ff & (v >> 64);
        let lower = 0xff_ff_ff_ff_ff_ff_ff_ff & v;
        encode_u64(&mut self.w, lower as u64).unwrap();
        encode_u64(&mut self.w, upper as u64).unwrap();
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        let bs = v.to_le_bytes();
        self.w.write_all(&bs[..])?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        let bs = v.to_le_bytes();
        self.w.write_all(&bs[..])?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let bs = (v as u32).to_le_bytes();
        self.w.write_all(&bs[..3])?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        encode_u64(&mut self.w, v.len() as u64)?;
        self.w.write_all(v.as_bytes())?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        encode_u64(&mut self.w, v.len() as u64)?;
        self.w.write_all(v)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        let bs = [0];
        self.w.write_all(&bs[..])?;
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        let bs = [1];
        self.w.write_all(&bs[..])?;
        value.serialize(self)?;
        Ok(())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        encode_u64(&mut self.w, variant_index as u64)?;
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)?;
        Ok(())
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        encode_u64(&mut self.w, variant_index as u64)?;
        value.serialize(self)?;
        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let len = len.ok_or(Error::NoSequenceSize)?;
        encode_u64(&mut self.w, len as u64)?;
        Ok(Compound { serializer: self })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(Compound { serializer: self })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(Compound { serializer: self })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        encode_u64(&mut self.w, variant_index as u64)?;
        Ok(Compound { serializer: self })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let len = len.ok_or(Error::NoSequenceSize)?;
        encode_u64(&mut self.w, len as u64)?;
        Ok(Compound { serializer: self })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(Compound { serializer: self })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        encode_u64(&mut self.w, variant_index as u64)?;
        Ok(Compound { serializer: self })
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct Compound<'a, W: Write> {
    serializer: &'a mut Serializer<W>,
}

impl<'a, W: Write> ser::SerializeSeq for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeTuple for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeTupleStruct for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeTupleVariant for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeMap for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Error> {
        key.serialize(&mut *self.serializer)
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeStruct for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeStructVariant for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] io::Error),
    #[error("input sequence has no size hint")]
    NoSequenceSize,
    #[error("{0}")]
    Other(String),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Error {
        Error::Other(msg.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::collections::{HashMap, HashSet};

    use serde_derive::{Deserialize, Serialize};

    use crate::de::from_reader;

    #[test]
    fn serialize_i8() {
        let v = -1i8;

        let mut bs = Vec::new();
        to_writer(&mut bs, v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_i16() {
        let v = -1i16;

        let mut bs = Vec::new();
        to_writer(&mut bs, v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_i32() {
        let v = -1i32;

        let mut bs = Vec::new();
        to_writer(&mut bs, v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_i64() {
        let v = -1i64;

        let mut bs = Vec::new();
        to_writer(&mut bs, v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_i128() {
        let v = -1i128;

        let mut bs = Vec::new();
        to_writer(&mut bs, v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_u8() {
        let v = u8::max_value();

        let mut bs = Vec::new();
        to_writer(&mut bs, v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_u16() {
        let v = u16::max_value();

        let mut bs = Vec::new();
        to_writer(&mut bs, v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_u32() {
        let v = u32::max_value();

        let mut bs = Vec::new();
        to_writer(&mut bs, v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_u64() {
        let v = u64::max_value();

        let mut bs = Vec::new();
        to_writer(&mut bs, v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_u128() {
        let v = u128::max_value();

        let mut bs = Vec::new();
        to_writer(&mut bs, v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_f32() {
        let v = 13141.32f32;

        let mut bs = Vec::new();
        to_writer(&mut bs, v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_f64() {
        let v = 13141.32f64;

        let mut bs = Vec::new();
        to_writer(&mut bs, v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_char() {
        let v = '𡈼';

        let mut bs = Vec::new();
        to_writer(&mut bs, v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_str() {
        let v = "example例";

        let mut bs = Vec::new();
        to_writer(&mut bs, v).unwrap();
        let d: String = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, &d);
    }

    #[test]
    fn serialize_string() {
        let v = "example例".to_owned();

        let mut bs = Vec::new();
        to_writer(&mut bs, v.clone()).unwrap();
        let d: String = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_ref() {
        let v = 12345u64;

        let mut bs = Vec::new();
        to_writer(&mut bs, &v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_option_none() {
        let v = Option::<u64>::None;

        let mut bs = Vec::new();
        to_writer(&mut bs, &v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_option_some() {
        let v = Some(123u64);

        let mut bs = Vec::new();
        to_writer(&mut bs, &v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_array_empty() {
        let v: [char; 0] = [];

        let mut bs = Vec::new();
        to_writer(&mut bs, v).unwrap();
        let d: [char; 0] = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_array() {
        let v = [1.0f32, 2.0, 3.0];

        let mut bs = Vec::new();
        to_writer(&mut bs, v).unwrap();
        let d: [f32; 3] = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_vec() {
        let v = vec![1.0f32, 2.0, 3.0];

        let mut bs = Vec::new();
        to_writer(&mut bs, &v).unwrap();
        let d: Vec<f32> = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_hashset() {
        let mut v = HashSet::new();
        v.insert(1);
        v.insert(2);
        v.insert(4);

        let mut bs = Vec::new();
        to_writer(&mut bs, &v).unwrap();
        let d: HashSet<u64> = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_hashmap() {
        let mut v = HashMap::new();
        v.insert(1, "壱".to_string());
        v.insert(2, "弐".to_string());
        v.insert(4, "参".to_string());

        let mut bs = Vec::new();
        to_writer(&mut bs, &v).unwrap();
        let d: HashMap<u64, String> = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct UnitStruct;

    #[test]
    fn serialize_unit_struct() {
        let v = UnitStruct;

        let mut bs = Vec::new();
        to_writer(&mut bs, &v).unwrap();
        let d: UnitStruct = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct NewtypeStruct(u8);

    #[test]
    fn serialize_newtype_struct() {
        let v = NewtypeStruct(123);

        let mut bs = Vec::new();
        to_writer(&mut bs, &v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TupleStruct(u8, u16, u8);

    #[test]
    fn serialize_tuple_struct() {
        let v = TupleStruct(1, 60000, 2);

        let mut bs = Vec::new();
        to_writer(&mut bs, &v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct BasicStruct {
        id: u64,
        name: String,
        score: f32,
    }

    #[test]
    fn serialize_basic_struct() {
        let v = BasicStruct {
            id: 1249,
            name: "平塚 彩".to_owned(),
            score: 12.2,
        };

        let mut bs = Vec::new();
        to_writer(&mut bs, &v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum BasicEnum {
        Unit,
        Newtype(String),
        Tuple(u16, String),
        Struct { x: u8, y: u8 },
    }

    #[test]
    fn serialize_basic_enum_unit_variant() {
        let v = BasicEnum::Unit;

        let mut bs = Vec::new();
        to_writer(&mut bs, &v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_basic_enum_newtype_variant() {
        let v = BasicEnum::Newtype("abc".to_owned());

        let mut bs = Vec::new();
        to_writer(&mut bs, &v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_basic_enum_tuple_variant() {
        let v = BasicEnum::Tuple(123, "abc".to_owned());

        let mut bs = Vec::new();
        to_writer(&mut bs, &v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }

    #[test]
    fn serialize_basic_enum_struct_variant() {
        let v = BasicEnum::Struct { x: 1, y: 255 };

        let mut bs = Vec::new();
        to_writer(&mut bs, &v).unwrap();
        let d = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, d);
    }
}
