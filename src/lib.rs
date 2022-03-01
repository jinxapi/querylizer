use std::fmt::Display;

use serde::ser;
use thiserror::Error;

pub use simple::Simple;

#[derive(Error, PartialEq, Debug)]
pub enum QuerylizerError {
    #[error("serialization error")]
    SerializationError(String),
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

pub fn escape_path_simple(s: &str) -> impl Iterator<Item = &str> {
    percent_encoding::utf8_percent_encode(s, PCHAR_SIMPLE)
}

pub fn escape_query_simple(s: &str) -> impl Iterator<Item = &str> {
    percent_encoding::utf8_percent_encode(s, QUERY_SIMPLE)
}

pub fn escape_query_simple_allow_reserved(s: &str) -> impl Iterator<Item = &str> {
    percent_encoding::utf8_percent_encode(s, QUERY_SIMPLE_ALLOW_RESERVED)
}

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

mod simple;