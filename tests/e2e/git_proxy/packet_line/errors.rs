use bytes::Bytes;

/// Create a Git error message in the proper side‑band pkt‑line format.
pub fn create_git_error_message(message: &str) -> Bytes {
    let full_message = format!("error: {message}\n");

    let mut line = Vec::new();
    line.push(0x03);
    line.extend_from_slice(full_message.as_bytes());

    let total_len = line.len() + 4;

    let header = format!("{total_len:04x}");
    let mut pkt_line = header.into_bytes();
    pkt_line.extend_from_slice(&line);

    pkt_line.extend_from_slice(b"0000");

    Bytes::from(pkt_line)
}
