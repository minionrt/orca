pub mod advertisement;
pub mod errors;
pub mod parse_commands;
pub mod parse_tests;

/// Create a packet line as a String.
/// The first 4 characters are the total length (header + payload) in hexadecimal.
pub fn pkt_line_string(s: &str) -> String {
    let total_len = s.len() + 4;
    format!("{total_len:04x}{s}")
}
