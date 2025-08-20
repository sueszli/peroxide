use base64::prelude::*;
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use std::io::{Read, Write};

use super::combinatorics::Thrush;

const SDP_TEMPLATE: &str = r#"{"type":"{}","sdp":"v=0\r\no=- {} 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\na=extmap-allow-mixed\r\na=msid-semantic: WMS\r\nm=application {} UDP/DTLS/SCTP webrtc-datachannel\r\nc=IN IP4 {}\r\na=candidate:{} 1 udp {} {}.local {} typ host generation 0 network-cost 999\r\na=candidate:{} 1 udp {} {} {} typ srflx raddr 0.0.0.0 rport 0 generation 0 network-cost 999\r\na=ice-ufrag:{}\r\na=ice-pwd:{}\r\na=ice-options:trickle\r\na=fingerprint:sha-256 {}\r\na=setup:{}\r\na=mid:0\r\na=sctp-port:5000\r\na=max-message-size:262144\r\n"}"#;

pub fn compress(sdp: &str) -> Result<String, String> {
    let parts = if sdp.contains("\"type\":\"offer\"") { extract_sdp_values(sdp, "offer")? } else { extract_sdp_values(sdp, "answer")? };

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(parts.join("|").as_bytes()).map_err(|e| format!("Compression write error: {}", e))?;
    let compressed = encoder.finish().map_err(|e| format!("Compression finish error: {}", e))?;
    Ok(BASE64_STANDARD.encode(compressed))
}

pub fn decompress(compressed_str: &str) -> Result<String, String> {
    let decoded = BASE64_STANDARD.decode(compressed_str).map_err(|e| format!("Base64 decode error: {}", e))?;

    let mut decompressed = Vec::new();
    ZlibDecoder::new(&decoded[..]).read_to_end(&mut decompressed).map_err(|e| format!("Decompression error: {}", e))?;

    let parts_str = std::str::from_utf8(&decompressed).map_err(|e| format!("UTF-8 decode error: {}", e))?;
    let parts: Vec<&str> = parts_str.split('|').collect();

    if parts.len() != 16 {
        return Err(format!("Expected 16 parts, got {}", parts.len()));
    }

    let mut result = SDP_TEMPLATE.to_string();
    for part in &parts {
        result = result.replacen("{}", part, 1);
    }

    // drop the entire second candidate line
    if parts[8].is_empty() {
        let empty_candidate_pattern = "\\r\\na=candidate: 1 udp    typ srflx raddr 0.0.0.0 rport 0 generation 0 network-cost 999";
        if let Some(start) = result.find(empty_candidate_pattern) {
            if let Some(end_pos) = result[start..].find("\\r\\na=ice-ufrag") {
                let full_end = start + end_pos;
                result = format!("{}{}", &result[..start], &result[full_end..]);
            }
        }
    }

    Ok(result)
}

fn extract_sdp_values(sdp: &str, sdp_type: &str) -> Result<Vec<String>, String> {
    let mut values = Vec::new();

    // 0: type
    values.push(sdp_type.to_string());

    // 1: session id (o=- <session_id> 2 IN IP4...)
    let session_id = extract_between(sdp, "o=- ", " 2 IN IP4").ok_or("Failed to extract session ID")?;
    values.push(session_id);

    // 2: port (m=application <port> UDP/DTLS/SCTP...)
    let port = extract_between(sdp, "m=application ", " UDP/DTLS/SCTP").ok_or("Failed to extract port")?;
    values.push(port);

    // 3: connection IP (c=IN IP4 <ip>)
    let connection_ip = extract_between(sdp, "c=IN IP4 ", "\\r\\n").ok_or("Failed to extract connection IP")?;
    values.push(connection_ip);

    // Extract candidates (1 or 2)
    let candidates = extract_candidates(sdp)?;
    if candidates.is_empty() {
        return Err("No candidates found".to_string());
    }

    // 4-7: first candidate (host)
    values.extend(candidates[0].clone());

    // 8-11: second candidate (srflx) - use empty strings if not present
    if candidates.len() > 1 {
        values.extend(candidates[1].clone());
    } else {
        values.extend(vec!["".to_string(), "".to_string(), "".to_string(), "".to_string()]);
    }

    // 12: ice-ufrag
    let ice_ufrag = extract_between(sdp, "a=ice-ufrag:", "\\r\\n").ok_or("Failed to extract ice-ufrag")?;
    values.push(ice_ufrag);

    // 13: ice-pwd
    let ice_pwd = extract_between(sdp, "a=ice-pwd:", "\\r\\n").ok_or("Failed to extract ice-pwd")?;
    values.push(ice_pwd);

    // 14: fingerprint
    let fingerprint = extract_between(sdp, "a=fingerprint:sha-256 ", "\\r\\n").ok_or("Failed to extract fingerprint")?;
    values.push(fingerprint);

    // 15: setup
    let setup = extract_between(sdp, "a=setup:", "\\r\\n").ok_or("Failed to extract setup")?;
    values.push(setup);

    if values.len() != 16 {
        return Err(format!("Expected 16 values, extracted {}", values.len()));
    }

    Ok(values)
}

fn extract_between(text: &str, start_marker: &str, end_marker: &str) -> Option<String> {
    let start_pos = text.find(start_marker)? + start_marker.len();
    let remaining = &text[start_pos..];
    let end_pos = remaining.find(end_marker)?;
    Some(remaining[..end_pos].to_string())
}

fn extract_candidates(sdp: &str) -> Result<Vec<Vec<String>>, String> {
    sdp.match_indices("a=candidate:")
        .take(2)
        .enumerate()
        .map(|(i, (pos, _))| {
            let line = &sdp[pos..];
            let end = line.find("\\r\\n").ok_or("Candidate line not properly terminated")?;
            let parts: Vec<&str> = line[12..end].split_whitespace().collect();
            if parts.len() < 7 {
                return Err(format!("Malformed candidate line: {}", &line[12..end]));
            }
            let host = if i == 0 { parts[4].strip_suffix(".local").unwrap_or(parts[4]) } else { parts[4] };
            Ok(vec![parts[0].to_string(), parts[3].to_string(), host.to_string(), parts[5].to_string()])
        })
        .collect::<Result<Vec<_>, String>>()?
        .pipe(|candidates| if candidates.is_empty() { Err("No candidates found".to_string()) } else { Ok(candidates) })
}

#[cfg(test)]
mod tests {
    use super::*;

    const OFFER_SDP: &str = r#"{"type":"offer","sdp":"v=0\r\no=- 1234567890123456789 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\na=extmap-allow-mixed\r\na=msid-semantic: WMS\r\nm=application 1234 UDP/DTLS/SCTP webrtc-datachannel\r\nc=IN IP4 192.0.2.1\r\na=candidate:1234 1 udp 2113937151 fake-host-1234-abcd.local 5678 typ host generation 0 network-cost 999\r\na=candidate:5678 1 udp 1677729535 192.0.2.1 1234 typ srflx raddr 0.0.0.0 rport 0 generation 0 network-cost 999\r\na=ice-ufrag:1234\r\na=ice-pwd:fakePassword1234\r\na=ice-options:trickle\r\na=fingerprint:sha-256 12:34:56:78:90:AB:CD:EF:12:34:56:78:90:AB:CD:EF:12:34:56:78:90:AB:CD:EF:12:34:56:78:90:AB:CD:EF\r\na=setup:actpass\r\na=mid:0\r\na=sctp-port:5000\r\na=max-message-size:262144\r\n"}"#;

    const ANSWER_SDP: &str = r#"{"type":"answer","sdp":"v=0\r\no=- 1122334455667788990 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\na=extmap-allow-mixed\r\na=msid-semantic: WMS\r\nm=application 8888 UDP/DTLS/SCTP webrtc-datachannel\r\nc=IN IP4 198.51.100.1\r\na=candidate:555666777 1 udp 2113937151 fake-answer-host-uuid-abcd-efgh.local 44444 typ host generation 0 network-cost 999\r\na=candidate:888999000 1 udp 1677729535 198.51.100.2 8888 typ srflx raddr 0.0.0.0 rport 0 generation 0 network-cost 999\r\na=ice-ufrag:tEsT\r\na=ice-pwd:fakeAnswerPassword987654321\r\na=ice-options:trickle\r\na=fingerprint:sha-256 11:22:33:44:55:66:77:88:99:AA:BB:CC:DD:EE:FF:00:AB:CD:EF:12:34:56:78:90:A1:B2:C3:D4:E5:F6:07\r\na=setup:active\r\na=mid:0\r\na=sctp-port:5000\r\na=max-message-size:262144\r\n"}"#;

    const ANSWER_SDP_SINGLE_CANDIDATE: &str = r#"{"type":"answer","sdp":"v=0\r\no=- 7777888899990000111 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\na=extmap-allow-mixed\r\na=msid-semantic: WMS\r\nm=application 6666 UDP/DTLS/SCTP webrtc-datachannel\r\nc=IN IP4 192.0.2.1\r\na=candidate:111222333 1 udp 2113937151 fake-single-host-uuid-9999-8888.local 77777 typ host generation 0 network-cost 999\r\na=ice-ufrag:sInG\r\na=ice-pwd:fakeSinglePassword555666777\r\na=ice-options:trickle\r\na=fingerprint:sha-256 99:88:77:66:55:44:33:22:11:00:FF:EE:DD:CC:BB:AA:89:AB:CD:EF:01:23:45:67:89:AB:CD:EF:12:34:56:78\r\na=setup:active\r\na=mid:0\r\na=sctp-port:5000\r\na=max-message-size:262144\r\n"}"#;

    #[test]
    fn test_extract_between() {
        assert_eq!(extract_between("abc 123 def", "abc ", " def"), Some("123".to_string()));
        assert_eq!(extract_between("no match", "xyz", "def"), None);
        assert_eq!(extract_between("start only", "start ", " missing"), None);
    }

    #[test]
    fn test_extract_sdp_values_offer() {
        let result = extract_sdp_values(OFFER_SDP, "offer").unwrap();
        assert_eq!(result.len(), 16);
        assert_eq!(result[0], "offer");
        assert_eq!(result[1], "1234567890123456789");
        assert_eq!(result[2], "1234");
        assert_eq!(result[3], "192.0.2.1");
        assert_eq!(result[4], "1234");
        assert_eq!(result[5], "2113937151");
        assert_eq!(result[6], "fake-host-1234-abcd");
        assert_eq!(result[7], "5678");
        assert_eq!(result[8], "5678");
        assert_eq!(result[9], "1677729535");
        assert_eq!(result[10], "192.0.2.1");
        assert_eq!(result[11], "1234");
        assert_eq!(result[12], "1234");
        assert_eq!(result[13], "fakePassword1234");
        assert_eq!(result[14], "12:34:56:78:90:AB:CD:EF:12:34:56:78:90:AB:CD:EF:12:34:56:78:90:AB:CD:EF:12:34:56:78:90:AB:CD:EF");
        assert_eq!(result[15], "actpass");
    }

    #[test]
    fn test_extract_sdp_values_answer() {
        let result = extract_sdp_values(ANSWER_SDP, "answer").unwrap();
        assert_eq!(result.len(), 16);
        assert_eq!(result[0], "answer");
        assert_eq!(result[1], "1122334455667788990");
        assert_eq!(result[2], "8888");
        assert_eq!(result[3], "198.51.100.1");
        assert_eq!(result[4], "555666777");
        assert_eq!(result[5], "2113937151");
        assert_eq!(result[6], "fake-answer-host-uuid-abcd-efgh");
        assert_eq!(result[7], "44444");
        assert_eq!(result[8], "888999000");
        assert_eq!(result[9], "1677729535");
        assert_eq!(result[10], "198.51.100.2");
        assert_eq!(result[11], "8888");
        assert_eq!(result[12], "tEsT");
        assert_eq!(result[13], "fakeAnswerPassword987654321");
        assert_eq!(result[14], "11:22:33:44:55:66:77:88:99:AA:BB:CC:DD:EE:FF:00:AB:CD:EF:12:34:56:78:90:A1:B2:C3:D4:E5:F6:07");
        assert_eq!(result[15], "active");
    }

    #[test]
    fn test_compress_decompress_offer_roundtrip() {
        let compressed = compress(OFFER_SDP).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(decompressed, OFFER_SDP);
    }

    #[test]
    fn test_compress_decompress_answer_roundtrip() {
        let compressed = compress(ANSWER_SDP).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(decompressed, ANSWER_SDP);
    }

    #[test]
    fn test_compress_invalid_sdp() {
        let invalid_sdp = r#"{"type":"offer","invalid":"data"}"#;
        assert!(compress(invalid_sdp).is_err());
    }

    #[test]
    fn test_decompress_invalid_base64() {
        assert!(decompress("invalid_base64!@#").is_err());
    }

    #[test]
    fn test_decompress_invalid_compressed_data() {
        let invalid_data = BASE64_STANDARD.encode("not_compressed_data");
        assert!(decompress(&invalid_data).is_err());
    }

    #[test]
    fn test_decompress_wrong_part_count() {
        let parts = vec!["a", "b", "c"];
        let data = parts.join("|");
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(data.as_bytes()).unwrap();
        let compressed = BASE64_STANDARD.encode(encoder.finish().unwrap());

        let result = decompress(&compressed);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected 16 parts, got 3"));
    }

    #[test]
    fn test_extract_candidates() {
        let sdp_part = r#"a=candidate:1234 1 udp 2113937151 fake-host-1234-abcd.local 5678 typ host generation 0 network-cost 999\r\na=candidate:5678 1 udp 1677729535 192.0.2.1 1234 typ srflx raddr 0.0.0.0 rport 0 generation 0 network-cost 999\r\n"#;

        let candidates = extract_candidates(sdp_part).unwrap();
        assert_eq!(candidates.len(), 2);

        assert_eq!(candidates[0][0], "1234");
        assert_eq!(candidates[0][1], "2113937151");
        assert_eq!(candidates[0][2], "fake-host-1234-abcd");
        assert_eq!(candidates[0][3], "5678");

        assert_eq!(candidates[1][0], "5678");
        assert_eq!(candidates[1][1], "1677729535");
        assert_eq!(candidates[1][2], "192.0.2.1");
        assert_eq!(candidates[1][3], "1234");
    }

    #[test]
    fn test_extract_candidates_insufficient() {
        let sdp_part = r#"a=candidate:1234 1 udp 2113937151 fake-host-1234-abcd.local 5678\r\n"#;

        assert!(extract_candidates(sdp_part).is_err());
    }

    #[test]
    fn test_extract_candidates_single() {
        let sdp_part = r#"a=candidate:1234 1 udp 2113937151 fake-host-1234-abcd.local 5678 typ host generation 0 network-cost 999\r\n"#;

        let candidates = extract_candidates(sdp_part).unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0][0], "1234");
        assert_eq!(candidates[0][1], "2113937151");
        assert_eq!(candidates[0][2], "fake-host-1234-abcd");
        assert_eq!(candidates[0][3], "5678");
    }

    #[test]
    fn test_missing_fields() {
        let incomplete_sdp = r#"{"type":"offer","sdp":"v=0\r\no=- 123 2 IN IP4 127.0.0.1\r\n"}"#;
        assert!(extract_sdp_values(incomplete_sdp, "offer").is_err());
    }

    #[test]
    fn test_compression_efficiency() {
        let offer_compressed = compress(OFFER_SDP).unwrap();
        let answer_compressed = compress(ANSWER_SDP).unwrap();

        assert!(offer_compressed.len() < OFFER_SDP.len() / 2);
        assert!(answer_compressed.len() < ANSWER_SDP.len() / 2);

        assert!(BASE64_STANDARD.decode(&offer_compressed).is_ok());
        assert!(BASE64_STANDARD.decode(&answer_compressed).is_ok());
    }

    #[test]
    fn test_single_candidate_extraction() {
        let result = extract_sdp_values(ANSWER_SDP_SINGLE_CANDIDATE, "answer").unwrap();
        assert_eq!(result.len(), 16);
        assert_eq!(result[0], "answer");
        assert_eq!(result[4], "111222333"); // First candidate ID
        assert_eq!(result[8], ""); // Second candidate ID should be empty
        assert_eq!(result[9], ""); // Second candidate priority should be empty
        assert_eq!(result[10], ""); // Second candidate host should be empty
        assert_eq!(result[11], ""); // Second candidate port should be empty
    }

    #[test]
    fn test_compress_decompress_single_candidate_roundtrip() {
        let compressed = compress(ANSWER_SDP_SINGLE_CANDIDATE).unwrap();
        let decompressed = decompress(&compressed).unwrap();

        // The decompressed version should match the original
        assert_eq!(decompressed, ANSWER_SDP_SINGLE_CANDIDATE);
    }
}
