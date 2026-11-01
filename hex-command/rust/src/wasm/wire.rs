//! Batch response encoding across the WASM boundary through reusable buffers.
//! This private stream preserves the public JS contract, including nulls and floats.
use serde::ser::Error as _;
use serde::{Serialize, ser::*};
use std::{cell::RefCell, fmt};
use wasm_bindgen::prelude::*;
#[wasm_bindgen(inline_js = r#"
const text = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
const staticStrings = new Map();
export function decode_engine_response(bytes, utf8) {
  const strings = utf8.length ? text.decode(utf8) : "";
  let textAt = 0;
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  let at = 0;
  function read() {
    const tag = bytes[at++];
    switch (tag) {
      case 1: return null;
      case 2: return false;
      case 3: return true;
      case 4: { const n = view.getInt32(at, true); at += 4; return n; }
      case 5: { const n = view.getFloat64(at, true); at += 8; return n; }
      case 6: { const n = view.getUint32(at, true); at += 4;
        const value = strings.slice(textAt, textAt + n); textAt += n; return value; }
      case 7: { const value = []; while (bytes[at] !== 0) value.push(read()); at++; return value; }
      case 8: { const value = {}; while (bytes[at] !== 0) { const key = read(); value[key] = read(); } at++; return value; }
      case 9: { const n = view.getUint32(at, true); at += 4; return n; }
      case 10: { const n = view.getUint32(at, true); at += 4; const v = bytes.slice(at, at + n); at += n; return v; }
      case 11: { const low = view.getBigUint64(at, true); const high = view.getBigInt64(at + 8, true); at += 16; return low + (high << 64n); }
      case 12: { const low = view.getBigUint64(at, true); const high = view.getBigUint64(at + 8, true); at += 16; return low + (high << 64n); }
      case 13: { const pointer = view.getUint32(at, true); at += 4; const length = bytes[at++];
        const key = pointer * 256 + length;
        let value = staticStrings.get(key);
        if (value === undefined) { value = text.decode(new Uint8Array(bytes.buffer, pointer, length)); staticStrings.set(key, value); }
        return value; }
      default: throw new Error('Invalid engine response tag');
    }
  }
  const value = read();
  if (at !== bytes.length || textAt !== strings.length) throw new Error('Invalid engine response length');
  return value;
}
"#)]
extern "C" {
    fn decode_engine_response(bytes: &[u8], utf8: &[u8]) -> JsValue;
}
#[derive(Default)]
struct Encoder(Vec<u8>, Vec<u8>);
thread_local! {static BUFFER:RefCell<Encoder>=RefCell::new(Encoder::default());}
pub(super) fn value<T: Serialize>(value: &T) -> Result<JsValue, JsError> {
    BUFFER.with_borrow_mut(|encoder| {
        encoder.0.clear();
        encoder.1.clear();
        value
            .serialize(&mut *encoder)
            .map_err(|error| JsError::new(&error.0))?;
        Ok(decode_engine_response(&encoder.0, &encoder.1))
    })
}
#[derive(Debug)]
struct Error(String);
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
impl serde::ser::Error for Error {
    fn custom<T: fmt::Display>(message: T) -> Self {
        Self(message.to_string())
    }
}
type Result<T, E = Error> = std::result::Result<T, E>;
impl Encoder {
    fn static_string(&mut self, value: &'static str) {
        if value.len() >= 256 {
            self.string(value);
            return;
        }
        self.0.push(13);
        self.0
            .extend_from_slice(&(value.as_ptr() as u32).to_le_bytes());
        self.0.push(value.len() as u8);
    }
    fn string(&mut self, value: &str) {
        self.0.push(6);
        self.0
            .extend_from_slice(&(value.encode_utf16().count() as u32).to_le_bytes());
        self.1.extend_from_slice(value.as_bytes());
    }
    fn compound(&mut self, tag: u8, ends: usize) -> Compound<'_> {
        self.0.push(tag);
        Compound {
            encoder: self,
            ends,
        }
    }
}
struct Compound<'a> {
    encoder: &'a mut Encoder,
    ends: usize,
}
impl Compound<'_> {
    fn finish(self) -> Result<()> {
        for _ in 0..self.ends {
            self.encoder.0.push(0);
        }
        Ok(())
    }
}
impl<'a> serde::Serializer for &'a mut Encoder {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Compound<'a>;
    type SerializeTuple = Compound<'a>;
    type SerializeTupleStruct = Compound<'a>;
    type SerializeTupleVariant = Compound<'a>;
    type SerializeMap = Compound<'a>;
    type SerializeStruct = Compound<'a>;
    type SerializeStructVariant = Compound<'a>;
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.0.push(if v { 3 } else { 2 });
        Ok(())
    }
    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i32(v.into())
    }
    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i32(v.into())
    }
    fn serialize_i32(self, v: i32) -> Result<()> {
        self.0.push(4);
        self.0.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result<()> {
        if v.unsigned_abs() > 9007199254740991 {
            return Err(Error::custom("integer exceeds JS precision"));
        }
        self.serialize_f64(v as f64)
    }
    fn serialize_i128(self, v: i128) -> Result<()> {
        self.0.push(11);
        self.0.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }
    fn serialize_u128(self, v: u128) -> Result<()> {
        self.0.push(12);
        self.0.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u32(v.into())
    }
    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u32(v.into())
    }
    fn serialize_u32(self, v: u32) -> Result<()> {
        self.0.push(9);
        self.0.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<()> {
        if v > 9007199254740991 {
            return Err(Error::custom("integer exceeds JS precision"));
        }
        self.serialize_f64(v as f64)
    }
    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(v.into())
    }
    fn serialize_f64(self, v: f64) -> Result<()> {
        self.0.push(5);
        self.0.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(v.encode_utf8(&mut [0; 4]))
    }
    fn serialize_str(self, v: &str) -> Result<()> {
        self.string(v);
        Ok(())
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.0.push(10);
        self.0.extend_from_slice(&(v.len() as u32).to_le_bytes());
        self.0.extend_from_slice(v);
        Ok(())
    }
    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }
    fn serialize_some<T: Serialize + ?Sized>(self, v: &T) -> Result<()> {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<()> {
        self.0.push(1);
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        self.serialize_unit()
    }
    fn serialize_unit_variant(self, _: &'static str, _: u32, v: &'static str) -> Result<()> {
        self.static_string(v);
        Ok(())
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(self, _: &'static str, v: &T) -> Result<()> {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        key: &'static str,
        v: &T,
    ) -> Result<()> {
        self.0.push(8);
        self.static_string(key);
        v.serialize(&mut *self)?;
        self.0.push(0);
        Ok(())
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Compound<'a>> {
        Ok(self.compound(7, 1))
    }
    fn serialize_tuple(self, _: usize) -> Result<Compound<'a>> {
        Ok(self.compound(7, 1))
    }
    fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Compound<'a>> {
        Ok(self.compound(7, 1))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        key: &'static str,
        _: usize,
    ) -> Result<Compound<'a>> {
        self.0.push(8);
        self.static_string(key);
        Ok(self.compound(7, 2))
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Compound<'a>> {
        Ok(self.compound(8, 1))
    }
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Compound<'a>> {
        Ok(self.compound(8, 1))
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        key: &'static str,
        _: usize,
    ) -> Result<Compound<'a>> {
        self.0.push(8);
        self.static_string(key);
        Ok(self.compound(8, 2))
    }
    fn collect_str<T: fmt::Display + ?Sized>(self, value: &T) -> Result<()> {
        struct Writer<'a>(&'a mut Vec<u8>);
        impl fmt::Write for Writer<'_> {
            fn write_str(&mut self, text: &str) -> fmt::Result {
                self.0.extend_from_slice(text.as_bytes());
                Ok(())
            }
        }
        self.0.push(6);
        let start = self.0.len();
        self.0.extend_from_slice(&[0; 4]);
        let text_start = self.1.len();
        fmt::write(&mut Writer(&mut self.1), format_args!("{value}")).map_err(Error::custom)?;
        let length = std::str::from_utf8(&self.1[text_start..])
            .expect("formatted UTF-8")
            .encode_utf16()
            .count() as u32;
        self.0[start..start + 4].copy_from_slice(&length.to_le_bytes());
        Ok(())
    }
}
macro_rules! sequence {
    ($trait:ident,$method:ident) => {
        impl $trait for Compound<'_> {
            type Ok = ();
            type Error = Error;
            fn $method<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<()> {
                v.serialize(&mut *self.encoder)
            }
            fn end(self) -> Result<()> {
                self.finish()
            }
        }
    };
}
sequence!(SerializeSeq, serialize_element);
sequence!(SerializeTuple, serialize_element);
sequence!(SerializeTupleStruct, serialize_field);
sequence!(SerializeTupleVariant, serialize_field);
impl SerializeMap for Compound<'_> {
    type Ok = ();
    type Error = Error;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<()> {
        v.serialize(&mut *self.encoder)
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<()> {
        v.serialize(&mut *self.encoder)
    }
    fn end(self) -> Result<()> {
        self.finish()
    }
}
macro_rules! record {
    ($trait:ident) => {
        impl $trait for Compound<'_> {
            type Ok = ();
            type Error = Error;
            fn serialize_field<T: Serialize + ?Sized>(
                &mut self,
                key: &'static str,
                v: &T,
            ) -> Result<()> {
                self.encoder.static_string(key);
                v.serialize(&mut *self.encoder)
            }
            fn end(self) -> Result<()> {
                self.finish()
            }
        }
    };
}
record!(SerializeStruct);
record!(SerializeStructVariant);
