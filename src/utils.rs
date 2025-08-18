use base64::prelude::*;
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use std::io::{Read, Write};

/// Performs side effects while preserving the original value
///
/// In combinatorial logic: `K x y = x` (Kestrel ignores the second argument)
/// In our case: `x.tap(f) = { f(&x); x }` (perform f as side effect, return x)
pub trait Kestrel {
    fn tap<F>(self, f: F) -> Self
    where
        F: FnOnce(&Self);
}
impl<T> Kestrel for T {
    fn tap<F>(self, f: F) -> Self
    where
        F: FnOnce(&Self),
    {
        f(&self);
        self
    }
}

/// Function application in reverse order  
///
/// In combinatorial logic: `T x f = f x` (apply function f to value x)
/// This is the reverse of normal function application `f(x)`
pub trait Thrush {
    fn pipe<U, F>(self, f: F) -> U
    where
        F: FnOnce(Self) -> U,
        Self: Sized;
}
impl<T> Thrush for T {
    fn pipe<U, F>(self, f: F) -> U
    where
        F: FnOnce(Self) -> U,
        Self: Sized,
    {
        f(self)
    }
}

pub fn compress_string(sdp: &str) -> String {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(sdp.as_bytes()).unwrap();
    BASE64_STANDARD.encode(encoder.finish().unwrap())
}

pub fn decompress_string(compressed_str: &str) -> String {
    let mut decompressed = Vec::new();
    ZlibDecoder::new(&BASE64_STANDARD.decode(compressed_str).unwrap()[..]).read_to_end(&mut decompressed).unwrap();
    std::str::from_utf8(&decompressed).unwrap().to_string()
}
