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

/// Serialize a value into an OpenAPI `form` query parameter.
pub struct Form<'s, F>
where
    F: for<'a> EncodingFn<'a>,
{
    output: String,
    name: &'s str,
    explode: bool,
    encoder: F,
    state: State,
}

impl<'s, F> Form<'s, F>
where
    F: for<'a> EncodingFn<'a>,
{
    /// Serialize a `form` value into a new string to be used for web requests.
    ///
    /// If `explode` is `false`:
    /// - sequences and tuples use the name once and items are comma-separated (`name=item1,item2`)
    /// - maps and structs use the name once and keys and values are comma separated
    /// (`name=key1,value1,key2,value2`).
    ///
    /// If `explode` is `true`:
    /// - sequences repeat the name and separate with `&` (`name=item1&name=item2`)
    /// - maps and structs do not use the name and keys and values are separated with `=`
    /// (`key1=value1&key2=value2`)
    ///
    /// # Example
    ///
    /// ```
    /// use querylizer::{escape_query, Form};
    /// #[derive(serde::Serialize)]
    /// struct A {
    ///     a: i32,
    ///     b: String,
    /// }
    /// let a = A { a: 12, b: "#hello".to_owned() };
    /// let s = Form::to_string(
    ///     "value",
    ///     &a,
    ///     false,
    ///     escape_query
    /// ).unwrap();
    /// assert_eq!(s, "value=a,12,b,%23hello".to_owned());
    /// ```
    pub fn to_string<T>(
        name: &str,
        value: &T,
        explode: bool,
        encoder: F,
    ) -> Result<String, QuerylizerError>
    where
        T: Serialize,
    {
        let mut serializer = Form {
            output: String::new(),
            name,
            explode,
            encoder,
            state: State::Outer,
        };
        value.serialize(&mut serializer)?;
        Ok(serializer.output)
    }

    /// Append a `form` value onto an existing string to be used for web requests.
    ///
    /// If `explode` is `false`:
    /// - sequences and tuples use the name once and items are comma-separated (`name=item1,item2`)
    /// - maps and structs use the name once and keys and values are comma separated
    /// (`name=key1,value1,key2,value2`).
    ///
    /// If `explode` is `true`:
    /// - sequences repeat the name and separate with `&` (`name=item1&name=item2`)
    /// - maps and structs do not use the name and keys and values are separated with `=`
    /// (`key1=value1&key2=value2`)
    ///
    /// # Example
    ///
    /// ```
    /// use querylizer::{escape_query, Form};
    /// #[derive(serde::Serialize)]
    /// struct A {
    ///     a: i32,
    ///     b: String,
    /// }
    /// let a = A { a: 12, b: "#hello".to_owned() };
    /// let s = Form::extend(
    ///     "https://example.com/v1/?".to_owned(),
    ///     "value",
    ///     &a,
    ///     true,
    ///     escape_query
    /// ).unwrap();
    /// assert_eq!(s, "https://example.com/v1/?a=12&b=%23hello".to_owned());
    /// ```
    pub fn extend<T>(
        output: String,
        name: &str,
        value: &T,
        explode: bool,
        encoder: F,
    ) -> Result<String, QuerylizerError>
    where
        T: Serialize,
    {
        let mut serializer = Form {
            output,
            name,
            explode,
            encoder,
            state: State::Outer,
        };
        value.serialize(&mut serializer)?;
        Ok(serializer.output)
    }
}

impl<'a, 's, F> Serializer for &'a mut Form<'s, F>
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

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(QuerylizerError::UnsupportedValue)
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

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        use ser::SerializeSeq;
        let mut seq_serializer = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq_serializer.serialize_element(byte)?;
        }
        SerializeSeq::end(seq_serializer)?;
        Ok(())
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
        match self.state {
            State::Outer => {
                self.state = State::InnerFirst;
                Ok(self)
            }
            _ => Err(QuerylizerError::UnsupportedNesting),
        }
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        match self.state {
            State::Outer => {
                self.state = State::InnerFirst;
                Ok(self)
            }
            _ => Err(QuerylizerError::UnsupportedNesting),
        }
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        match self.state {
            State::Outer => {
                self.state = State::InnerFirst;
                Ok(self)
            }
            _ => Err(QuerylizerError::UnsupportedNesting),
        }
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        match self.state {
            State::Outer => {
                self.state = State::InnerFirst;
                Ok(self)
            }
            _ => Err(QuerylizerError::UnsupportedNesting),
        }
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
        impl<'a, 's, F> $trait for &'a mut Form<'s, F>
        where
            F: for<'b> EncodingFn<'b>,
        {
            type Ok = ();
            type Error = QuerylizerError;

            fn $serialize<T>(&mut self, value: &T) -> Result<(), Self::Error>
            where
                T: ?Sized + Serialize,
            {
                match self.state {
                    State::Outer => unreachable!(),
                    State::InnerFirst => {
                        self.state = State::InnerNext;
                        self.output.extend(self.encoder.call(&self.name));
                        self.output.push('=');
                    }
                    State::InnerNext => {
                        if self.explode {
                            self.output.push('&');
                            self.output.extend(self.encoder.call(&self.name));
                            self.output.push('=');
                        } else {
                            self.output.push(',');
                        }
                    }
                }
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

seq_serializer!(ser::SerializeSeq, serialize_element);
seq_serializer!(ser::SerializeTuple, serialize_element);
seq_serializer!(ser::SerializeTupleStruct, serialize_field);
seq_serializer!(ser::SerializeTupleVariant, serialize_field);

impl<'a, 's, F> ser::SerializeMap for &'a mut Form<'s, F>
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
                if !self.explode {
                    self.output.extend(self.encoder.call(self.name));
                    self.output.push('=');
                }
            }
            State::InnerNext => {
                self.output.push(if self.explode { '&' } else { ',' });
            }
        }
        key.serialize(&mut **self)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        match self.state {
            State::Outer => unreachable!(),
            _ => {
                self.output.push(if self.explode { '=' } else { ',' });
            }
        }
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
        impl<'a, 's, F> $trait for &'a mut Form<'s, F>
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
                        if !self.explode {
                            self.output.extend(self.encoder.call(&self.name));
                            self.output.push('=');
                        }
                    }
                    State::InnerNext => {
                        self.output.push(if self.explode { '&' } else { ',' });
                    }
                }
                key.serialize(&mut **self)?;
                match self.state {
                    State::Outer => unreachable!(),
                    _ => {
                        self.output.push(if self.explode { '=' } else { ',' });
                    }
                }
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

    use super::Form;

    #[test]
    fn test_bool() -> Result<(), QuerylizerError> {
        assert_eq!(
            Form::to_string("color", &true, false, passthrough),
            Err(QuerylizerError::UnsupportedValue)
        );
        Ok(())
    }

    #[test]
    fn test_i8() -> Result<(), QuerylizerError> {
        assert_eq!(
            Form::to_string("color", &-1i8, false, passthrough)?,
            "color=-1"
        );
        Ok(())
    }

    #[test]
    fn test_i16() -> Result<(), QuerylizerError> {
        assert_eq!(
            Form::to_string("color", &-1i16, false, passthrough)?,
            "color=-1"
        );
        Ok(())
    }

    #[test]
    fn test_i32() -> Result<(), QuerylizerError> {
        assert_eq!(
            Form::to_string("color", &-1i32, false, passthrough)?,
            "color=-1"
        );
        Ok(())
    }

    #[test]
    fn test_i64() -> Result<(), QuerylizerError> {
        assert_eq!(
            Form::to_string("color", &-1i64, false, passthrough)?,
            "color=-1"
        );
        Ok(())
    }

    #[test]
    fn test_i128() -> Result<(), QuerylizerError> {
        assert_eq!(
            Form::to_string("color", &-1i128, false, passthrough)?,
            "color=-1"
        );
        Ok(())
    }

    #[test]
    fn test_u8() -> Result<(), QuerylizerError> {
        assert_eq!(
            Form::to_string("color", &1u8, false, passthrough)?,
            "color=1"
        );
        Ok(())
    }

    #[test]
    fn test_u16() -> Result<(), QuerylizerError> {
        assert_eq!(
            Form::to_string("color", &1u16, false, passthrough)?,
            "color=1"
        );
        Ok(())
    }

    #[test]
    fn test_u32() -> Result<(), QuerylizerError> {
        assert_eq!(
            Form::to_string("color", &1u32, false, passthrough)?,
            "color=1"
        );
        Ok(())
    }

    #[test]
    fn test_u64() -> Result<(), QuerylizerError> {
        assert_eq!(
            Form::to_string("color", &1u64, false, passthrough)?,
            "color=1"
        );
        Ok(())
    }

    #[test]
    fn test_u128() -> Result<(), QuerylizerError> {
        assert_eq!(
            Form::to_string("color", &1u128, false, passthrough)?,
            "color=1"
        );
        Ok(())
    }

    #[test]
    fn test_f32() -> Result<(), QuerylizerError> {
        assert_eq!(
            Form::to_string("color", &0.25f32, false, passthrough)?,
            "color=0.25"
        );
        Ok(())
    }

    #[test]
    fn test_f64() -> Result<(), QuerylizerError> {
        assert_eq!(
            Form::to_string("color", &0.25f64, false, passthrough)?,
            "color=0.25"
        );
        Ok(())
    }

    #[test]
    fn test_char() -> Result<(), QuerylizerError> {
        assert_eq!(
            Form::to_string("color", &'d', false, passthrough)?,
            "color=d"
        );
        Ok(())
    }

    #[test]
    fn test_str() -> Result<(), QuerylizerError> {
        assert_eq!(
            Form::to_string("color", &"blue", false, passthrough)?,
            "color=blue"
        );
        Ok(())
    }

    #[test]
    fn test_bytes() -> Result<(), QuerylizerError> {
        assert_eq!(
            Form::to_string("color", b"blue", false, passthrough)?,
            "color=98,108,117,101"
        );
        assert_eq!(
            Form::to_string("color", b"blue", true, passthrough)?,
            "color=98&color=108&color=117&color=101"
        );
        Ok(())
    }

    #[test]
    fn test_none() -> Result<(), QuerylizerError> {
        assert_eq!(
            Form::to_string::<Option<u32>>("color", &None, false, passthrough)?,
            "color="
        );
        Ok(())
    }

    #[test]
    fn test_some() -> Result<(), QuerylizerError> {
        assert_eq!(
            Form::to_string("color", &Some(1u32), false, passthrough)?,
            "color=1"
        );
        Ok(())
    }

    #[test]
    fn test_unit() -> Result<(), QuerylizerError> {
        assert_eq!(
            Form::to_string("color", &(), false, passthrough)?,
            "color="
        );
        Ok(())
    }

    #[test]
    fn test_unit_struct() -> Result<(), QuerylizerError> {
        #[derive(Serialize)]
        struct T {}
        assert_eq!(
            Form::to_string("color", &T {}, false, passthrough),
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
            Form::to_string("color", &E::A, false, passthrough)?,
            "color="
        );
        Ok(())
    }

    #[test]
    fn test_newtype_struct() -> Result<(), QuerylizerError> {
        #[derive(Serialize)]
        struct Metres(u32);
        assert_eq!(
            Form::to_string("color", &Metres(5), false, passthrough)?,
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
            Form::to_string("color", &E::A(5), false, passthrough)?,
            "color=5"
        );
        Ok(())
    }

    #[test]
    fn test_seq() -> Result<(), QuerylizerError> {
        let v = vec!["blue", "black", "brown"];
        assert_eq!(
            Form::to_string("color", &v, false, passthrough)?,
            "color=blue,black,brown"
        );
        assert_eq!(
            Form::to_string("color", &v, true, passthrough)?,
            "color=blue&color=black&color=brown"
        );
        Ok(())
    }

    #[test]
    fn test_tuple() -> Result<(), QuerylizerError> {
        let t = ("blue", "black", "brown");
        assert_eq!(
            Form::to_string("color", &t, false, passthrough)?,
            "color=blue,black,brown"
        );
        assert_eq!(
            Form::to_string("color", &t, true, passthrough)?,
            "color=blue&color=black&color=brown"
        );
        Ok(())
    }

    #[test]
    fn test_tuple_struct() -> Result<(), QuerylizerError> {
        #[derive(Serialize)]
        struct Triple(&'static str, &'static str, &'static str);
        let v = Triple("blue", "black", "brown");
        assert_eq!(
            Form::to_string("color", &v, false, passthrough)?,
            "color=blue,black,brown"
        );
        assert_eq!(
            Form::to_string("color", &v, true, passthrough)?,
            "color=blue&color=black&color=brown"
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
            Form::to_string("color", &E::A(5, 'f'), false, passthrough)?,
            "color=5,f"
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
            Form::to_string("color", &m, false, passthrough)?,
            "color=B,150,G,200,R,100"
        );
        assert_eq!(
            Form::to_string("color", &m, true, passthrough)?,
            "B=150&G=200&R=100"
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
            Form::to_string("color", &test, false, passthrough).unwrap(),
            "color=R,100,G,200,B,150"
        );
        assert_eq!(
            Form::to_string("color", &test, true, passthrough).unwrap(),
            "R=100&G=200&B=150"
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
            Form::to_string("color", &test, false, passthrough).unwrap(),
            "color=R,100,G,200,B,150"
        );
        assert_eq!(
            Form::to_string("color", &test, true, passthrough).unwrap(),
            "R=100&G=200&B=150"
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
            Form::to_string("color", &test, false, passthrough),
            Err(QuerylizerError::UnsupportedNesting)
        );
    }
}
