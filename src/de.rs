use std::fmt::Display;
use std::io::{self, ErrorKind, Read};

use serde::de::Error as _;
use serde::de::{self, DeserializeOwned, IntoDeserializer, Unexpected, Visitor};
use thiserror::Error;

use crate::varuint::decode_u64;

pub fn from_reader<R: Read, T: DeserializeOwned>(r: R) -> Result<T, Error> {
    let mut deserializer = Deserializer::new(r);
    let value: T = de::Deserialize::deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(value)
}

#[derive(Debug)]
pub struct Deserializer<R: Read> {
    r: R,
}

impl<R: Read> Deserializer<R> {
    /// Create new `Deserializer`
    pub fn new(r: R) -> Deserializer<R> {
        Deserializer { r }
    }

    /// This method should be called after a value has been deserialized to ensure there is no
    /// trailing data in the input source.
    pub fn end(&mut self) -> Result<(), Error> {
        let mut bs = [0u8];

        match self.r.read_exact(&mut bs[..]) {
            Ok(_) => Err(Error::IncompleteRead),
            Err(e) => {
                if e.kind() == ErrorKind::UnexpectedEof {
                    Ok(())
                } else {
                    Err(Error::IO(e))
                }
            }
        }
    }

    fn parse_u16(&mut self) -> Result<u16, Error> {
        let v = decode_u64(&mut self.r)?;
        if v <= u16::max_value() as u64 {
            Ok(v as u16)
        } else {
            Err(Error::invalid_value(Unexpected::Unsigned(v as u64), &"u16"))
        }
    }

    fn parse_u32(&mut self) -> Result<u32, Error> {
        let v = decode_u64(&mut self.r)?;
        if v <= u32::max_value() as u64 {
            Ok(v as u32)
        } else {
            Err(Error::invalid_value(Unexpected::Unsigned(v as u64), &"u16"))
        }
    }

    fn parse_u128(&mut self) -> Result<u128, Error> {
        let lower = decode_u64(&mut self.r)?;
        let upper = decode_u64(&mut self.r)?;
        Ok((upper as u128) << 64 | (lower as u128))
    }
}

impl<'de, R: Read> de::Deserializer<'de> for &mut Deserializer<R> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported("deserialize_any"))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut bs = [0u8];
        self.r.read_exact(&mut bs[..])?;

        match bs[0] {
            0 => visitor.visit_bool(false),
            1 => visitor.visit_bool(true),
            v => Err(Error::invalid_value(
                Unexpected::Unsigned(v as u64),
                &"0 or 1",
            )),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut bs = [0u8];
        self.r.read_exact(&mut bs[..])?;
        visitor.visit_i8(i8::from_le_bytes(bs))
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut bs = [0u8; 2];
        self.r.read_exact(&mut bs[..])?;
        visitor.visit_i16(i16::from_le_bytes(bs))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut bs = [0u8; 4];
        self.r.read_exact(&mut bs[..])?;
        visitor.visit_i32(i32::from_le_bytes(bs))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut bs = [0u8; 8];
        self.r.read_exact(&mut bs[..])?;
        visitor.visit_i64(i64::from_le_bytes(bs))
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut bs = [0u8; 16];
        self.r.read_exact(&mut bs[..])?;
        visitor.visit_i128(i128::from_le_bytes(bs))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut bs = [0u8];
        self.r.read_exact(&mut bs[..])?;
        visitor.visit_u8(u8::from_le_bytes(bs))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse_u16()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_u32()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let v = decode_u64(&mut self.r)?;
        visitor.visit_u64(v)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u128(self.parse_u128()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut bs = [0u8; 4];
        self.r.read_exact(&mut bs[..])?;
        visitor.visit_f32(f32::from_le_bytes(bs))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut bs = [0u8; 8];
        self.r.read_exact(&mut bs[..])?;
        visitor.visit_f64(f64::from_le_bytes(bs))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut bs = [0u8; 4];
        self.r.read_exact(&mut bs[..3])?;
        let v = u32::from_le_bytes(bs);
        if let Some(ch) = std::char::from_u32(v) {
            visitor.visit_char(ch)
        } else {
            Err(Error::invalid_value(
                Unexpected::Unsigned(v as u64),
                &"Unicode codepoint",
            ))
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let len = decode_u64(&mut self.r)? as usize;

        let mut bs = vec![0u8; len];
        self.r.read_exact(&mut bs)?;

        match String::from_utf8(bs) {
            Ok(s) => visitor.visit_string(s),
            Err(_) => Err(Error::custom("invalid UTF-8 sequence")),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let len = decode_u64(&mut self.r)? as usize;

        let mut bs = vec![0u8; len];
        self.r.read_exact(&mut bs)?;

        visitor.visit_byte_buf(bs)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut bs = [0u8];
        self.r.read_exact(&mut bs[..])?;

        match bs[0] {
            0 => visitor.visit_none(),
            1 => visitor.visit_some(self),
            v => Err(Error::invalid_value(
                Unexpected::Unsigned(v as u64),
                &"None (0) or Some (1)",
            )),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let len = decode_u64(&mut self.r)? as usize;
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        struct Access<'a, R: Read> {
            deserializer: &'a mut Deserializer<R>,
            len: usize,
        }

        impl<'de, 'a, R: Read> de::SeqAccess<'de> for Access<'a, R> {
            type Error = Error;

            fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
            where
                T: de::DeserializeSeed<'de>,
            {
                if self.len > 0 {
                    self.len -= 1;
                    let value =
                        serde::de::DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }

            fn size_hint(&self) -> Option<usize> {
                Some(self.len)
            }
        }

        visitor.visit_seq(Access {
            deserializer: self,
            len,
        })
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        struct Access<'a, R: Read> {
            deserializer: &'a mut Deserializer<R>,
            len: usize,
        }

        impl<'de, 'a, R: Read> de::MapAccess<'de> for Access<'a, R> {
            type Error = Error;

            fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
            where
                T: de::DeserializeSeed<'de>,
            {
                if self.len > 0 {
                    self.len -= 1;
                    let value =
                        serde::de::DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }

            fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Error>
            where
                T: de::DeserializeSeed<'de>,
            {
                let value = serde::de::DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
                Ok(value)
            }

            fn size_hint(&self) -> Option<usize> {
                Some(self.len)
            }
        }

        let len = decode_u64(&mut self.r)? as usize;

        visitor.visit_map(Access {
            deserializer: self,
            len,
        })
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        impl<'de, 'a, R: Read> de::EnumAccess<'de> for &'a mut Deserializer<R> {
            type Error = Error;
            type Variant = Self;

            fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Error>
            where
                V: de::DeserializeSeed<'de>,
            {
                let idx = decode_u64(&mut self.r)? as u32;
                let val: Result<_, Error> = seed.deserialize(idx.into_deserializer());
                Ok((val?, self))
            }
        }

        visitor.visit_enum(self)
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported("deserialize_identifier"))
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported("deserialize_ignored_any"))
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'de, 'a, R: Read> de::VariantAccess<'de> for &'a mut Deserializer<R> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        serde::de::DeserializeSeed::deserialize(seed, self)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: serde::de::Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_tuple(self, len, visitor)
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: serde::de::Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_tuple(self, fields.len(), visitor)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("decode finished but input bytes left")]
    IncompleteRead,
    #[error("{0}")]
    IO(#[from] io::Error),
    #[error("{0} is unsupported")]
    Unsupported(&'static str),
    #[error("{0}")]
    Other(String),
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Error {
        Error::Other(msg.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::collections::{HashMap, HashSet};

    use serde_derive::Deserialize;

    use crate::varuint::encode_u64;

    #[test]
    fn deserialize_bool_false() {
        let bs = [0u8];
        let v: bool = from_reader(&bs[..]).unwrap();
        assert!(!v);
    }

    #[test]
    fn deserialize_bool_true() {
        let bs = [1u8];
        let v: bool = from_reader(&bs[..]).unwrap();
        assert!(v);
    }

    #[test]
    fn deserialize_bool_fails_with_2() {
        let bs = [2u8];
        let _ = from_reader::<&[u8], bool>(&bs[..]).unwrap_err();
    }

    #[test]
    fn deserialize_i8() {
        let to_be = -1i8;
        let bs = to_be.to_le_bytes();
        let v: i8 = from_reader(&bs[..]).unwrap();
        assert_eq!(v, to_be);
    }

    #[test]
    fn deserialize_i16() {
        let to_be = -1i16;
        let bs = to_be.to_le_bytes();
        let v: i16 = from_reader(&bs[..]).unwrap();
        assert_eq!(v, to_be);
    }

    #[test]
    fn deserialize_i32() {
        let to_be = -1i32;
        let bs = to_be.to_le_bytes();
        let v: i32 = from_reader(&bs[..]).unwrap();
        assert_eq!(v, to_be);
    }

    #[test]
    fn deserialize_i64() {
        let to_be = -1i64;
        let bs = to_be.to_le_bytes();
        let v: i64 = from_reader(&bs[..]).unwrap();
        assert_eq!(v, to_be);
    }

    #[test]
    fn deserialize_i128() {
        let to_be = -1i128;
        let bs = to_be.to_le_bytes();
        let v: i128 = from_reader(&bs[..]).unwrap();
        assert_eq!(v, to_be);
    }

    #[test]
    fn deserialize_u8() {
        let to_be = 0x12u8;
        let bs = to_be.to_le_bytes();
        let v: u8 = from_reader(&bs[..]).unwrap();
        assert_eq!(v, to_be);
    }

    #[test]
    fn deserialize_u16() {
        let to_be = u16::max_value();
        let mut bs = Vec::new();
        encode_u64(&mut bs, to_be as u64).unwrap();

        let v: u16 = from_reader(&bs[..]).unwrap();
        assert_eq!(v, to_be);
    }

    #[test]
    fn deserialize_u32() {
        let to_be = u32::max_value();
        let mut bs = Vec::new();
        encode_u64(&mut bs, to_be as u64).unwrap();

        let v: u32 = from_reader(&bs[..]).unwrap();
        assert_eq!(v, to_be);
    }

    #[test]
    fn deserialize_u64() {
        let to_be = u64::max_value();
        let mut bs = Vec::new();
        encode_u64(&mut bs, to_be as u64).unwrap();

        let v: u64 = from_reader(&bs[..]).unwrap();
        assert_eq!(v, to_be);
    }

    #[test]
    fn deserialize_u128() {
        let to_be = 0x123456789abcdef0123456789abcdefu128;

        let upper = 0xff_ff_ff_ff_ff_ff_ff_ff & (to_be >> 64);
        let lower = 0xff_ff_ff_ff_ff_ff_ff_ff & to_be;

        let mut bs = Vec::new();
        encode_u64(&mut bs, lower as u64).unwrap();
        encode_u64(&mut bs, upper as u64).unwrap();

        let v: u128 = from_reader(&bs[..]).unwrap();
        assert_eq!(v, to_be);
    }

    #[test]
    fn deserialize_f32() {
        let to_be = 123.45678f32;
        let bs = to_be.to_le_bytes();
        let v: f32 = from_reader(&bs[..]).unwrap();
        assert_eq!(v, to_be);
    }

    #[test]
    fn deserialize_f64() {
        let to_be = 123.45678f64;
        let bs = to_be.to_le_bytes();
        let v: f64 = from_reader(&bs[..]).unwrap();
        assert_eq!(v, to_be);
    }

    #[test]
    fn deserialize_char_a() {
        let bs = [0x41, 0x00, 0x00]; // A
        let v: char = from_reader(&bs[..]).unwrap();
        assert_eq!(v, 'A');
    }

    #[test]
    fn deserialize_char_2byte() {
        let bs = [0x9e, 0x8a, 0x00]; // 語
        let v: char = from_reader(&bs[..]).unwrap();
        assert_eq!(v, '語');
    }

    #[test]
    fn deserialize_char_3byte() {
        let bs = [0x3c, 0x12, 0x02]; // 𡈼
        let v: char = from_reader(&bs[..]).unwrap();
        assert_eq!(v, '𡈼');
    }

    #[test]
    fn deserialize_str() {
        let to_be = "sample例";
        let mut bs = Vec::new();
        encode_u64(&mut bs, to_be.len() as u64).unwrap();
        bs.extend(to_be.as_bytes().iter());

        let v: String = from_reader(bs.as_slice()).unwrap();
        assert_eq!(&v, to_be);
    }

    #[test]
    fn deserialize_long_str() {
        let mut to_be = String::new();
        for _ in 0..0x100000 {
            to_be.push_str("sample text");
        }

        let mut bs = Vec::new();
        encode_u64(&mut bs, to_be.len() as u64).unwrap();
        bs.extend(to_be.as_bytes().iter());

        let v: String = from_reader(bs.as_slice()).unwrap();
        assert_eq!(v, to_be);
    }

    #[test]
    fn deserialize_option_none_u8() {
        let bs = [0u8];
        let v: Option<u8> = from_reader(&bs[..]).unwrap();
        assert_eq!(v, None);
    }

    #[test]
    fn deserialize_option_some_u8() {
        let bs = [1u8, 123];
        let v: Option<u8> = from_reader(&bs[..]).unwrap();
        assert_eq!(v, Some(123));
    }

    #[test]
    fn deserialize_unit() {
        let bs: [u8; 0] = [];
        let v: () = from_reader(&bs[..]).unwrap();
        assert_eq!(v, ());
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct UnitStruct;

    #[test]
    fn deserialize_unit_struct() {
        let bs: [u8; 0] = [];
        let v: UnitStruct = from_reader(&bs[..]).unwrap();
        assert_eq!(v, UnitStruct);
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct NewtypeStruct(u8);

    #[test]
    fn deserialize_newtype_struct() {
        let bs = [123u8];
        let v: NewtypeStruct = from_reader(&bs[..]).unwrap();
        assert_eq!(v, NewtypeStruct(123));
    }

    #[test]
    fn deserialize_vec() {
        let bs = [3u8, 1, 2, 3];
        let v: Vec<u8> = from_reader(&bs[..]).unwrap();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn deserialize_hashset() {
        let bs = [3u8, 1, 2, 3];
        let v: HashSet<u8> = from_reader(&bs[..]).unwrap();

        let mut to_be = HashSet::<u8>::new();
        to_be.insert(1);
        to_be.insert(2);
        to_be.insert(3);

        assert_eq!(v, to_be);
    }

    #[test]
    fn deserialize_tuple() {
        let bs = [1u8, 2, 3];
        let v: (u8, u16, u8) = from_reader(&bs[..]).unwrap();
        assert_eq!(v, (1u8, 2u16, 3u8));
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct TupleStruct(u8, u16, u8);

    #[test]
    fn deserialize_tuple_struct() {
        let bs = [1u8, 2, 3];
        let v: TupleStruct = from_reader(&bs[..]).unwrap();
        assert_eq!(v, TupleStruct(1u8, 2u16, 3u8));
    }

    #[test]
    fn deserialize_hashmap() {
        let mut bs = vec![3u8];
        bs.push(1);
        encode_u64(&mut bs, 1024).unwrap();
        bs.push(2);
        encode_u64(&mut bs, 1025).unwrap();
        bs.push(3);
        encode_u64(&mut bs, 1026).unwrap();

        let v: HashMap<u8, u16> = from_reader(&bs[..]).unwrap();

        let mut to_be = HashMap::<u8, u16>::new();
        to_be.insert(1, 1024);
        to_be.insert(2, 1025);
        to_be.insert(3, 1026);

        assert_eq!(v, to_be);
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct BasicStruct {
        id: u64,
        name: String,
        score: f32,
    }

    #[test]
    fn deserialize_struct() {
        let actual_name = "岸田　宏";

        let mut bs = Vec::<u8>::new();

        encode_u64(&mut bs, 123).unwrap();
        encode_u64(&mut bs, actual_name.len() as u64).unwrap();
        bs.extend(actual_name.as_bytes());
        bs.extend(&97.3f32.to_le_bytes()[..]);

        let v: BasicStruct = from_reader(&bs[..]).unwrap();
        assert_eq!(v.id, 123);
        assert_eq!(&v.name, actual_name);
        assert_eq!(v.score, 97.3f32);
    }

    #[derive(Debug, PartialEq, Deserialize)]
    enum BasicEnum {
        UnitA,
        UnitB,
        Newtype(String),
        Tuple(u16, String),
        Struct { x: u8, y: u8 },
    }

    #[test]
    fn deserialize_enum_unit_variant_a() {
        let bs = [0u8];
        let v: BasicEnum = from_reader(&bs[..]).unwrap();
        assert_eq!(v, BasicEnum::UnitA);
    }

    #[test]
    fn deserialize_enum_unit_variant_b() {
        let bs = [1u8];
        let v: BasicEnum = from_reader(&bs[..]).unwrap();
        assert_eq!(v, BasicEnum::UnitB);
    }

    #[test]
    fn deserialize_enum_newtype_variant() {
        let bs = [2u8, 4, b'b', b'i', b'i', b'm'];
        let v: BasicEnum = from_reader(&bs[..]).unwrap();
        assert_eq!(v, BasicEnum::Newtype("biim".to_owned()));
    }

    #[test]
    fn deserialize_enum_tuple_variant() {
        let mut bs = vec![3u8];
        encode_u64(&mut bs, 0x1234).unwrap();
        encode_u64(&mut bs, 3).unwrap();
        bs.extend(b"Abe");

        let v: BasicEnum = from_reader(&bs[..]).unwrap();
        assert_eq!(v, BasicEnum::Tuple(0x1234, "Abe".to_owned()));
    }
}
