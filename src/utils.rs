use base64::prelude::*;
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use std::io::{Read, Write};
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::*;

thread_local! {
    static DOC: Document = web_sys::window().unwrap().document().unwrap();
}
pub fn document() -> Document {
    DOC.with(|d| d.clone())
}

pub fn onkeypress<F: 'static + FnMut(KeyboardEvent)>(element: &Element, function: F) {
    let callback = Closure::wrap(Box::new(function) as Box<dyn FnMut(KeyboardEvent)>);
    element.add_event_listener_with_callback("keypress", callback.as_ref().unchecked_ref()).unwrap();
    callback.forget();
}

pub fn onclick<F: 'static + FnMut()>(element: &Element, function: F) {
    let callback = Closure::wrap(Box::new(function) as Box<dyn FnMut()>);
    element.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref()).unwrap();
    callback.forget();
}

//
// compression
//

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

//
// combinatorial logic
//

pub trait Thrush {
    // Function application in reverse order.
    // `T x f = f x` (apply function f to value x)
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

pub trait Kestrel {
    // Performs side effects while preserving the original value.
    // `K x y = x` (Kestrel ignores the second argument)
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
