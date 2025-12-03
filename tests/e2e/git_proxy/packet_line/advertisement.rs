use bytes::{Bytes, BytesMut};

use super::pkt_line_string;

/// Create the Git advertisement output
/// The format is: <pkt-line header><flush packet><command output>
pub fn create_git_advertisement(service: &str, command_output: &[u8]) -> Bytes {
    let header = format!("# service={service}\n");
    let pkt_header = pkt_line_string(&header);

    let mut advertisement = BytesMut::new();
    advertisement.extend_from_slice(pkt_header.as_bytes());
    advertisement.extend_from_slice(b"0000");
    advertisement.extend_from_slice(command_output);
    advertisement.freeze()
}
