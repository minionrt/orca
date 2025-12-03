use flate2::read::GzDecoder;
use std::io::Read;

/// Decompresses a gzip compressed byte array
pub async fn decompress_gzip(body_bytes: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = GzDecoder::new(body_bytes);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}
