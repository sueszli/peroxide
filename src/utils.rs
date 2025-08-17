use base64::prelude::*;
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use std::io::{Read, Write};

pub fn compress_string(sdp: &str) -> String {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(sdp.as_bytes()).unwrap();
    BASE64_STANDARD.encode(encoder.finish().unwrap())
}

pub fn decompress_string(compressed_str: &str) -> String {
    let mut decompressed = Vec::new();
    ZlibDecoder::new(&BASE64_STANDARD.decode(compressed_str).unwrap()[..])
        .read_to_end(&mut decompressed)
        .unwrap();
    std::str::from_utf8(&decompressed).unwrap().to_string()
}
