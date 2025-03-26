use base64::prelude::*;
use flate2::{Compression, write::ZlibEncoder};
use std::io::{Read, Write};

pub fn compress_string(sdp: &str) -> String {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(sdp.as_bytes()).unwrap();
    let compressed = encoder.finish().unwrap();
    return BASE64_STANDARD.encode(&compressed);
}

pub fn decompress_string(compressed_str: &str) -> String {
    let decoded = BASE64_STANDARD.decode(compressed_str).unwrap();
    let mut decoder = flate2::read::ZlibDecoder::new(&decoded[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).unwrap();
    return std::str::from_utf8(&decompressed).unwrap().to_string();
}
