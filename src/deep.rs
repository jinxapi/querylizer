// Copyright 2022 Jonathan Giddy
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use serde::{ser, Serialize, Serializer};

use crate::{EncodingFn, QuerylizerError};

enum State {
    // Top-level outside any container
    Outer,
    // Inside a container, but no elements yet
    InnerFirst,
    // Inside a container after first element
    InnerNext,
}

/// Serialize a value into an OpenAPI `deepObject` query parameter.
pub struct DeepObject<'s, F>
where
    F: for<'a> EncodingFn<'a>,
{
    output: &'s mut String,
    name: &'s str,
    encoder: F,
    state: State,
}

impl<'s, F> DeepObject<'s, F>
where
    F: for<'a> EncodingFn<'a>,
{
    /// Serialize a `deepObject` value into a new string to be used for web requests.
    ///
    /// # Example
    ///
    /// ```
    /// use querylizer::{encode_query, DeepObject};
    /// #[derive(serde::Serialize)]
    /// struct A {
    ///     a: i32,
    ///     b: String,
    /// }
    /// let a = A { a: 12, b: "#hello".to_owned() };
    /// let s = DeepObject::to_string(
    ///     "value",
    ///     &a,
    ///     encode_query
    /// ).unwrap();
    /// assert_eq!(s, "value[a]=12&value[b]=%23hello".to_owned());
    /// ```
    pub fn to_string<T>(name: &str, value: &T, encoder: F) -> Result<String, QuerylizerError>
    where
        T: Serialize,
    {
        let mut output = String::new();
        let mut serializer = DeepObject {
            output: &mut output,
            name,
            encoder,
            state: State::Outer,
        };
        value.serialize(&mut serializer)?;
        Ok(output)
    }

    /// Append a `deepObject` value onto an existing string to be used for web requests.
    ///
    /// # Example
    ///
    /// ```
    /// use querylizer::{encode_query, DeepObject};
    /// #[derive(serde::Serialize)]
    /// struct A {
    ///     a: i32,
    ///     b: String,
    /// }
    /// let a = A { a: 12, b: "#hello".to_owned() };
    /// let mut s = "https://example.com/v1/?".to_owned();
    /// DeepObject::extend(
    ///     &mut s,
    ///     "value",
    ///     &a,
    ///     encode_query
    /// ).unwrap();
    /// assert_eq!(s, "https://example.com/v1/?value[a]=12&value[b]=%23hello".to_owned());
    /// ```
    pub fn extend<T>(
        output: &mut String,
        name: &str,
        value: &T,
        encoder: F,
    ) -> Result<(), QuerylizerError>
    where
        T: Serialize,
    {
        let mut serializer = DeepObject {
            output,
            name,
            encoder,
            state: State::Outer,
        };
        value.serialize(&mut serializer)?;
        Ok(())
    }
}

impl<'a, 's, F> Serializer for &'a mut DeepObject<'s, F>
where
    F: for<'b> EncodingFn<'b>,
{
    type Ok = ();

    // The error type when some error occurs during serialization.
    type Error = QuerylizerError;

    // Associated types for keeping track of additional state while serializing
    // compound data structures like sequences and maps. In this case no
    // additional state is required beyond what is already stored in the
    // Serializer struct.
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(if v { "true" } else { "false" })
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i32(i32::from(v))
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i32(i32::from(v))
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.serialize_str(buffer.format(v))
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.serialize_str(buffer.format(v))
    }
    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.serialize_str(buffer.format(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u32(u32::from(v))
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u32(u32::from(v))
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.serialize_str(buffer.format(v))
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.serialize_str(buffer.format(v))
    }
    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.serialize_str(buffer.format(v))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        let mut buffer = dtoa::Buffer::new();
        self.serialize_str(buffer.format(v))
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        let mut buffer = dtoa::Buffer::new();
        self.serialize_str(buffer.format(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let mut buf = [0u8; 4];
        let s = v.encode_utf8(&mut buf);
        self.serialize_str(s)?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        if let State::Outer = self.state {
            self.output.extend(self.encoder.call(self.name));
            self.output.push('=');
        }
        self.output.extend(self.encoder.call(v));
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(QuerylizerError::UnsupportedValue)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        if let State::Outer = self.state {
            self.serialize_str("")
        } else {
            Err(QuerylizerError::UnsupportedNesting)
        }
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        if let State::Outer = self.state {
            value.serialize(self)
        } else {
            Err(QuerylizerError::UnsupportedNesting)
        }
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        if let State::Outer = self.state {
            self.serialize_str("")
        } else {
            Err(QuerylizerError::UnsupportedNesting)
        }
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(QuerylizerError::UnsupportedNesting)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        if let State::Outer = self.state {
            self.serialize_str("")
        } else {
            Err(QuerylizerError::UnsupportedNesting)
        }
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(QuerylizerError::UnsupportedValue)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(QuerylizerError::UnsupportedValue)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(QuerylizerError::UnsupportedValue)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(QuerylizerError::UnsupportedValue)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        match self.state {
            State::Outer => {
                self.state = State::InnerFirst;
                Ok(self)
            }
            _ => Err(QuerylizerError::UnsupportedNesting),
        }
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        match self.state {
            State::Outer => {
                self.state = State::InnerFirst;
                Ok(self)
            }
            _ => Err(QuerylizerError::UnsupportedNesting),
        }
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        match self.state {
            State::Outer => {
                self.state = State::InnerFirst;
                Ok(self)
            }
            _ => Err(QuerylizerError::UnsupportedNesting),
        }
    }
}

macro_rules! seq_serializer {
    ($trait:ty, $serialize:ident) => {
        impl<'a, 's, F> $trait for &'a mut DeepObject<'s, F>
        where
            F: for<'b> EncodingFn<'b>,
        {
            type Ok = ();
            type Error = QuerylizerError;

            fn $serialize<T>(&mut self, _value: &T) -> Result<(), Self::Error>
            where
                T: ?Sized + Serialize,
            {
                Err(QuerylizerError::UnsupportedValue)
            }

            fn end(self) -> Result<(), Self::Error> {
                Err(QuerylizerError::UnsupportedValue)
            }
        }
    };
}

seq_serializer!(ser::SerializeSeq, serialize_element);
seq_serializer!(ser::SerializeTuple, serialize_element);
seq_serializer!(ser::SerializeTupleStruct, serialize_field);
seq_serializer!(ser::SerializeTupleVariant, serialize_field);

impl<'a, 's, F> ser::SerializeMap for &'a mut DeepObject<'s, F>
where
    F: for<'b> EncodingFn<'b>,
{
    type Ok = ();
    type Error = QuerylizerError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        match self.state {
            State::Outer => unreachable!(),
            State::InnerFirst => {
                self.state = State::InnerNext;
            }
            State::InnerNext => {
                self.output.push('&');
            }
        }
        self.output.extend(self.encoder.call(self.name));
        self.output.push('[');
        key.serialize(&mut **self)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.output.push_str("]=");
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Self::Error> {
        match self.state {
            State::Outer => unreachable!(),
            State::InnerFirst => Err(QuerylizerError::UnsupportedValue),
            State::InnerNext => {
                self.state = State::Outer;
                Ok(())
            }
        }
    }
}

macro_rules! struct_serializer {
    ($trait:ty) => {
        impl<'a, 's, F> $trait for &'a mut DeepObject<'s, F>
        where
            F: for<'b> EncodingFn<'b>,
        {
            type Ok = ();
            type Error = QuerylizerError;

            fn serialize_field<T: ?Sized>(
                &mut self,
                key: &'static str,
                value: &T,
            ) -> Result<(), Self::Error>
            where
                T: Serialize,
            {
                match self.state {
                    State::Outer => unreachable!(),
                    State::InnerFirst => {
                        self.state = State::InnerNext;
                    }
                    State::InnerNext => {
                        self.output.push('&');
                    }
                }
                self.output.extend(self.encoder.call(&self.name));
                self.output.push('[');
                self.output.extend(self.encoder.call(key));
                self.output.push_str("]=");
                value.serialize(&mut **self)
            }

            fn end(self) -> Result<(), Self::Error> {
                match self.state {
                    State::Outer => unreachable!(),
                    State::InnerFirst => Err(QuerylizerError::UnsupportedValue),
                    State::InnerNext => {
                        self.state = State::Outer;
                        Ok(())
                    }
                }
            }
        }
    };
}

struct_serializer!(ser::SerializeStruct);
struct_serializer!(ser::SerializeStructVariant);

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use crate::{passthrough, QuerylizerError};

    use super::DeepObject;

    #[test]
    fn test_bool() -> Result<(), QuerylizerError> {
        assert_eq!(
            DeepObject::to_string("color", &true, passthrough)?,
            "color=true"
        );
        assert_eq!(
            DeepObject::to_string("color", &false, passthrough)?,
            "color=false"
        );
        Ok(())
    }

    #[test]
    fn test_i8() -> Result<(), QuerylizerError> {
        assert_eq!(
            DeepObject::to_string("color", &-1i8, passthrough)?,
            "color=-1"
        );
        Ok(())
    }

    #[test]
    fn test_i16() -> Result<(), QuerylizerError> {
        assert_eq!(
            DeepObject::to_string("color", &-1i16, passthrough)?,
            "color=-1"
        );
        Ok(())
    }

    #[test]
    fn test_i32() -> Result<(), QuerylizerError> {
        assert_eq!(
            DeepObject::to_string("color", &-1i32, passthrough)?,
            "color=-1"
        );
        Ok(())
    }

    #[test]
    fn test_i64() -> Result<(), QuerylizerError> {
        assert_eq!(
            DeepObject::to_string("color", &-1i64, passthrough)?,
            "color=-1"
        );
        Ok(())
    }

    #[test]
    fn test_i128() -> Result<(), QuerylizerError> {
        assert_eq!(
            DeepObject::to_string("color", &-1i128, passthrough)?,
            "color=-1"
        );
        Ok(())
    }

    #[test]
    fn test_u8() -> Result<(), QuerylizerError> {
        assert_eq!(
            DeepObject::to_string("color", &1u8, passthrough)?,
            "color=1"
        );
        Ok(())
    }

    #[test]
    fn test_u16() -> Result<(), QuerylizerError> {
        assert_eq!(
            DeepObject::to_string("color", &1u16, passthrough)?,
            "color=1"
        );
        Ok(())
    }

    #[test]
    fn test_u32() -> Result<(), QuerylizerError> {
        assert_eq!(
            DeepObject::to_string("color", &1u32, passthrough)?,
            "color=1"
        );
        Ok(())
    }

    #[test]
    fn test_u64() -> Result<(), QuerylizerError> {
        assert_eq!(
            DeepObject::to_string("color", &1u64, passthrough)?,
            "color=1"
        );
        Ok(())
    }

    #[test]
    fn test_u128() -> Result<(), QuerylizerError> {
        assert_eq!(
            DeepObject::to_string("color", &1u128, passthrough)?,
            "color=1"
        );
        Ok(())
    }

    #[test]
    fn test_f32() -> Result<(), QuerylizerError> {
        assert_eq!(
            DeepObject::to_string("color", &0.25f32, passthrough)?,
            "color=0.25"
        );
        Ok(())
    }

    #[test]
    fn test_f64() -> Result<(), QuerylizerError> {
        assert_eq!(
            DeepObject::to_string("color", &0.25f64, passthrough)?,
            "color=0.25"
        );
        Ok(())
    }

    #[test]
    fn test_char() -> Result<(), QuerylizerError> {
        assert_eq!(
            DeepObject::to_string("color", &'d', passthrough)?,
            "color=d"
        );
        Ok(())
    }

    #[test]
    fn test_str() -> Result<(), QuerylizerError> {
        assert_eq!(
            DeepObject::to_string("color", &"blue", passthrough)?,
            "color=blue"
        );
        Ok(())
    }

    #[test]
    fn test_bytes() -> Result<(), QuerylizerError> {
        assert_eq!(
            DeepObject::to_string("color", b"blue", passthrough),
            Err(QuerylizerError::UnsupportedValue)
        );
        Ok(())
    }

    #[test]
    fn test_none() -> Result<(), QuerylizerError> {
        assert_eq!(
            DeepObject::to_string::<Option<u32>>("color", &None, passthrough)?,
            "color="
        );
        Ok(())
    }

    #[test]
    fn test_some() -> Result<(), QuerylizerError> {
        assert_eq!(
            DeepObject::to_string("color", &Some(1u32), passthrough)?,
            "color=1"
        );
        Ok(())
    }

    #[test]
    fn test_unit() -> Result<(), QuerylizerError> {
        assert_eq!(DeepObject::to_string("color", &(), passthrough)?, "color=");
        Ok(())
    }

    #[test]
    fn test_unit_struct() -> Result<(), QuerylizerError> {
        #[derive(Serialize)]
        struct T {}
        assert_eq!(
            DeepObject::to_string("color", &T {}, passthrough),
            Err(QuerylizerError::UnsupportedValue)
        );
        Ok(())
    }

    #[test]
    fn test_unit_variant() -> Result<(), QuerylizerError> {
        #[derive(Serialize)]
        enum E {
            A,
        }
        assert_eq!(
            DeepObject::to_string("color", &E::A, passthrough)?,
            "color="
        );
        Ok(())
    }

    #[test]
    fn test_newtype_struct() -> Result<(), QuerylizerError> {
        #[derive(Serialize)]
        struct Metres(u32);
        assert_eq!(
            DeepObject::to_string("color", &Metres(5), passthrough)?,
            "color=5"
        );
        Ok(())
    }

    #[test]
    fn test_newtype_variant() -> Result<(), QuerylizerError> {
        #[derive(Serialize)]
        enum E {
            A(u32),
        }
        assert_eq!(
            DeepObject::to_string("color", &E::A(5), passthrough)?,
            "color=5"
        );
        Ok(())
    }

    #[test]
    fn test_seq() -> Result<(), QuerylizerError> {
        let v = vec!["blue", "black", "brown"];
        assert_eq!(
            DeepObject::to_string("color", &v, passthrough),
            Err(QuerylizerError::UnsupportedValue)
        );
        Ok(())
    }

    #[test]
    fn test_tuple() -> Result<(), QuerylizerError> {
        let t = ("blue", "black", "brown");
        assert_eq!(
            DeepObject::to_string("color", &t, passthrough),
            Err(QuerylizerError::UnsupportedValue)
        );
        Ok(())
    }

    #[test]
    fn test_tuple_struct() -> Result<(), QuerylizerError> {
        #[derive(Serialize)]
        struct Triple(&'static str, &'static str, &'static str);
        let v = Triple("blue", "black", "brown");
        assert_eq!(
            DeepObject::to_string("color", &v, passthrough),
            Err(QuerylizerError::UnsupportedValue)
        );
        Ok(())
    }

    #[test]
    fn test_tuple_variant() -> Result<(), QuerylizerError> {
        #[derive(Serialize)]
        enum E {
            A(u32, char),
        }
        assert_eq!(
            DeepObject::to_string("color", &E::A(5, 'f'), passthrough),
            Err(QuerylizerError::UnsupportedValue)
        );
        Ok(())
    }

    #[test]
    fn test_map() -> Result<(), QuerylizerError> {
        let mut m = std::collections::BTreeMap::new();
        m.insert("R", 100);
        m.insert("G", 200);
        m.insert("B", 150);
        assert_eq!(
            DeepObject::to_string("color", &m, passthrough)?,
            "color[B]=150&color[G]=200&color[R]=100"
        );
        Ok(())
    }

    #[test]
    fn test_struct() {
        #[derive(Serialize)]
        struct Test {
            #[serde(rename = "R")]
            r: u32,
            #[serde(rename = "G")]
            g: u32,
            #[serde(rename = "B")]
            b: u32,
        }

        let test = Test {
            r: 100,
            g: 200,
            b: 150,
        };
        assert_eq!(
            DeepObject::to_string("color", &test, passthrough).unwrap(),
            "color[R]=100&color[G]=200&color[B]=150"
        );
    }

    #[test]
    fn test_struct_variant() {
        #[derive(Serialize)]
        struct Test {
            #[serde(rename = "R")]
            r: u32,
            #[serde(rename = "G")]
            g: u32,
            #[serde(rename = "B")]
            b: u32,
        }
        #[derive(Serialize)]
        enum E {
            T(Test),
        }

        let test = E::T(Test {
            r: 100,
            g: 200,
            b: 150,
        });
        assert_eq!(
            DeepObject::to_string("color", &test, passthrough).unwrap(),
            "color[R]=100&color[G]=200&color[B]=150"
        );
    }

    #[test]
    fn test_unsupported_nesting() {
        #[derive(Serialize)]
        struct Test {
            #[serde(rename = "R")]
            r: u32,
            #[serde(rename = "G")]
            g: u32,
            #[serde(rename = "B")]
            b: u32,
        }

        #[derive(Serialize)]
        struct Outer {
            t: Test,
        }
        let test = Outer {
            t: Test {
                r: 100,
                g: 200,
                b: 150,
            },
        };
        assert_eq!(
            DeepObject::to_string("color", &test, passthrough),
            Err(QuerylizerError::UnsupportedNesting)
        );
    }
}
