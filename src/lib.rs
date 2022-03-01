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

use std::fmt::Display;

use serde::ser;
use thiserror::Error;

pub use deep::DeepObject;
pub use form::Form;
pub use simple::Simple;

#[derive(Error, PartialEq, Debug)]
pub enum QuerylizerError {
    #[error("serialization error")]
    SerializationError(String),
    #[error("nested containers not supported")]
    UnsupportedNesting,
    #[error("unsupported value")]
    UnsupportedValue,
    #[error("unknown error")]
    Unknown,
}

impl ser::Error for QuerylizerError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        QuerylizerError::SerializationError(msg.to_string())
    }
}

// See https://datatracker.ietf.org/doc/html/rfc3986#appendix-A

const UNRESERVED: &percent_encoding::AsciiSet = &percent_encoding::NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

const PCHAR_SIMPLE: &percent_encoding::AsciiSet = &UNRESERVED
    .remove(b'!')
    .remove(b'$')
    .remove(b'&')
    .remove(b'\'')
    .remove(b'(')
    .remove(b')')
    .remove(b'*')
    .remove(b'+')
    .remove(b',')
    .remove(b';')
    .remove(b'=')
    .remove(b':')
    .remove(b'@');

const QUERY_SIMPLE: &percent_encoding::AsciiSet = &PCHAR_SIMPLE.remove(b'/').remove(b'?');

// allowReserved allows all reserved characters to be not percent encoded

const QUERY_SIMPLE_ALLOW_RESERVED: &percent_encoding::AsciiSet = &QUERY_SIMPLE
    .remove(b':')
    .remove(b'/')
    .remove(b'?')
    .remove(b'#')
    .remove(b'[')
    .remove(b']')
    .remove(b'@');

/// Encode a string to allow it to be added to a URL path.
pub fn escape_path(s: &str) -> impl Iterator<Item = &str> {
    percent_encoding::utf8_percent_encode(s, PCHAR_SIMPLE)
}

/// Encode a string to allow it to be added to a URL query.
pub fn escape_query(s: &str) -> impl Iterator<Item = &str> {
    percent_encoding::utf8_percent_encode(s, QUERY_SIMPLE)
}

/// Encode a string to allow it to be added to a URL query, but allowing reserved
/// characters to pass unencoded.  Since this allows `&` and `#` to appear in the
/// query value, it should only be used when the URL query contains a single parameter.
pub fn escape_query_allow_reserved(s: &str) -> impl Iterator<Item = &str> {
    percent_encoding::utf8_percent_encode(s, QUERY_SIMPLE_ALLOW_RESERVED)
}

/// An identity function that does not encode any characters.  This can be passed to
/// the `querylizer` serializers if no encoding should be done.
pub fn passthrough(s: &str) -> impl Iterator<Item = &str> {
    ::std::iter::once(s)
}

// Use a trait to represent `Fn(&str) -> impl Iterator<Item=&str>`, to allow it to
// be stored in a struct. Derived from https://stackoverflow.com/a/63558160/2644842
pub trait EncodingFn<'a> {
    type Iter: Iterator<Item = &'a str>;
    fn call(&self, arg: &'a str) -> Self::Iter;
}

impl<'a, I, F> EncodingFn<'a> for F
where
    F: Fn(&'a str) -> I,
    I: Iterator<Item = &'a str>,
{
    type Iter = I;
    fn call(&self, s: &'a str) -> I {
        self(s)
    }
}

mod deep;
mod form;
mod simple;
