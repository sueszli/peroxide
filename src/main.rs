use std::io::{Read, Write};

use flate2::{Compression, write::ZlibEncoder};
use base64::prelude::*;  // Add base64 to your Cargo.toml dependencies

fn main() {
    
    let sdp = r#"
    {"type":"offer","sdp":"v=0\r\no=- 3904384361787272233 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\na=extmap-allow-mixed\r\na=msid-semantic: WMS\r\nm=application 3083 UDP/DTLS/SCTP webrtc-datachannel\r\nc=IN IP4 91.141.58.14\r\na=candidate:216262018 1 udp 2113937151 0854a906-ecc4-4de5-a709-aef252cac4bc.local 58250 typ host generation 0 network-cost 999\r\na=candidate:1080638978 1 udp 1677729535 91.141.58.14 3083 typ srflx raddr 0.0.0.0 rport 0 generation 0 network-cost 999\r\na=ice-ufrag:9f/g\r\na=ice-pwd:rRZm7s2+83gVtu1Pior0dbQ9\r\na=ice-options:trickle\r\na=fingerprint:sha-256 BF:9B:FE:22:F7:DD:A7:15:5A:A8:D7:C3:5A:9F:91:76:AC:EB:5F:56:E6:EB:81:AE:44:43:F0:F8:F6:9B:51:E0\r\na=setup:actpass\r\na=mid:0\r\na=sctp-port:5000\r\na=max-message-size:262144\r\n"}
    "#.trim();
    // println!("Original SDP: {}", sdp);

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(sdp.as_bytes()).unwrap();
    let compressed = encoder.finish().unwrap();
    let compressed_str = BASE64_STANDARD.encode(&compressed);
    // println!("Compressed SDP: {}", compressed_str);

    let decoded = BASE64_STANDARD.decode(compressed_str).unwrap();
    let mut decoder = flate2::read::ZlibDecoder::new(&decoded[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).unwrap();
    let decompressed_str = std::str::from_utf8(&decompressed).unwrap();

    assert_eq!(sdp, decompressed_str.trim());
}
