//! This module contains a parser for the Git protocol's "update-requests".
//! This parser collects all ref modification commands from a "git-receive-pack" request.
//! As this parser is used for preventing unauthorized ref updates, it is crucial that
//! whenver a ref update contains modification commands, all of them are correctly parsed and returned.
//!
//! Below is the grammar for the "update-requests" as per the [Git documentation](https://github.com/git/git/blob/master/Documentation/gitprotocol-pack.txt):
//!
//! ```text
//! update-requests   =  *shallow ( command-list | push-cert )
//!
//! shallow           =  PKT-LINE("shallow" SP obj-id)
//!
//! command-list      =  PKT-LINE(command NUL capability-list)
//!              *PKT-LINE(command)
//!              flush-pkt
//!
//! command           =  create / delete / update
//! create            =  zero-id SP new-id  SP name
//! delete            =  old-id  SP zero-id SP name
//! update            =  old-id  SP new-id  SP name
//!
//! old-id            =  obj-id
//! new-id            =  obj-id
//!
//! push-cert         = PKT-LINE("push-cert" NUL capability-list LF)
//!             PKT-LINE("certificate version 0.1" LF)
//!             PKT-LINE("pusher" SP ident LF)
//!             PKT-LINE("pushee" SP url LF)
//!             PKT-LINE("nonce" SP nonce LF)
//!             *PKT-LINE("push-option" SP push-option LF)
//!             PKT-LINE(LF)
//!             *PKT-LINE(command LF)
//!             *PKT-LINE(gpg-signature-lines LF)
//!             PKT-LINE("push-cert-end" LF)
//!
//! push-option       =  1*( VCHAR | SP )
//! ```
//!
//! The [grammar for packet-lines](https://github.com/git/git/blob/master/Documentation/gitprotocol-common.txt) is as follows:
//!
//! ```text
//! pkt-line     =  data-pkt / flush-pkt
//!
//! data-pkt     =  pkt-len pkt-payload
//! pkt-len      =  4*(HEXDIG)
//! pkt-payload  =  (pkt-len - 4)*(OCTET)
//!
//! flush-pkt    = "0000"
//! ```
//!
//! We rely on `gix_packetline::decode` to parse packet lines.
use nom::{
    IResult, Needed, Parser,
    branch::alt,
    bytes::complete::take_while_m_n,
    character::complete::char,
    combinator::rest,
    error::{Error, ErrorKind},
    multi::many0,
};
use std::str;

pub(super) const ZERO_ID: &str = "0000000000000000000000000000000000000000";

/// A ref modification command.
#[derive(Debug, PartialEq, Clone)]
pub enum RefModification {
    Create {
        new_id: String,
        ref_name: String,
    },
    Delete {
        old_id: String,
        ref_name: String,
    },
    Update {
        old_id: String,
        new_id: String,
        ref_name: String,
    },
}

impl RefModification {
    /// Get the name of the ref being modified.
    pub fn ref_name(&self) -> &str {
        match self {
            RefModification::Create { ref_name, .. } => ref_name,
            RefModification::Delete { ref_name, .. } => ref_name,
            RefModification::Update { ref_name, .. } => ref_name,
        }
    }
}

/// Parse the raw packet–line body to extract all ref modification commands.
pub fn parse_update_requests(
    body: &[u8],
) -> Result<Vec<RefModification>, Box<dyn std::error::Error>> {
    let (_remaining, cmds) = update_requests(body).map_err(|e| format!("Parsing error: {e:?}"))?;
    Ok(cmds)
}

/// ```text
/// update-requests = *shallow ( command-list | push-cert )
/// ```
pub(super) fn update_requests(input: &[u8]) -> IResult<&[u8], Vec<RefModification>> {
    let (input, _shallows) = many0(shallow).parse(input)?;
    alt((command_list, push_cert)).parse(input)
}

/// A shallow line must be exactly "shallow" SP <40-hex-digit-objid>
pub(super) fn shallow(input: &[u8]) -> IResult<&[u8], &str> {
    let (input, line) = take_data_line(input)?;
    if let Some(obj_part) = line.strip_prefix("shallow ") {
        // Consume exactly 40 hex digits and check that nothing extra follows.
        let (rem, _) = take_while_m_n::<_, &str, nom::error::Error<&str>>(40, 40, |c: char| {
            c.is_ascii_hexdigit()
        })(obj_part)
        .map_err(|_| nom::Err::Error(Error::new(obj_part.as_bytes(), ErrorKind::LengthValue)))?;
        if !rem.is_empty() {
            return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
        }
        Ok((input, line))
    } else {
        Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)))
    }
}

/// ```text
/// command-list = PKT-LINE(command NUL capability-list)
///              *PKT-LINE(command)
///              flush-pkt
/// ```
pub(super) fn command_list(input: &[u8]) -> IResult<&[u8], Vec<RefModification>> {
    // The first command line must include a NUL and a capability list.
    let (input, first_line) = take_data_line(input)?;
    if !first_line.contains("\0") {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
    }
    let (cmd_part, _) = first_line
        .split_once('\0')
        .ok_or_else(|| nom::Err::Error(Error::new(input, ErrorKind::Tag)))?;
    let (_, first_cmd) = command_line_str(cmd_part)
        .map_err(|_| nom::Err::Error(Error::new(input, ErrorKind::Tag)))?;
    // Subsequent command lines do not include capabilities.
    let (input, mut more_cmds) = many0(command_line).parse(input)?;
    let mut cmds = vec![first_cmd];
    cmds.append(&mut more_cmds);
    // Finally, a flush packet must follow.
    let (input, _) = take_flush_pkt(input)?;
    Ok((input, cmds))
}

/// ```text
/// push-cert = PKT-LINE("push-cert" NUL capability-list LF)
///           PKT-LINE("certificate version 0.1" LF)
///           PKT-LINE("pusher" SP ident LF)
///           PKT-LINE("pushee" SP url LF)
///           PKT-LINE("nonce" SP nonce LF)
///           *PKT-LINE("push-option" SP push-option LF)
///           PKT-LINE(LF)
///           *PKT-LINE(command LF)
///           *PKT-LINE(gpg-signature-lines LF)
///           PKT-LINE("push-cert-end" LF)
/// ```
pub(super) fn push_cert(input: &[u8]) -> IResult<&[u8], Vec<RefModification>> {
    // Each of the following PKT-LINEs must end with an LF.
    let (input, header) = take_data_line_expect_lf(input)?;
    if !header.starts_with("push-cert\0") {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
    }
    let (input, cert_version) = take_data_line_expect_lf(input)?;
    if cert_version != "certificate version 0.1" {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
    }
    let (input, pusher) = take_data_line_expect_lf(input)?;
    if !pusher.starts_with("pusher ") || pusher.len() <= "pusher ".len() {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
    }
    let (input, pushee) = take_data_line_expect_lf(input)?;
    if !pushee.starts_with("pushee ") || pushee.len() <= "pushee ".len() {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
    }
    let (input, nonce) = take_data_line_expect_lf(input)?;
    if !nonce.starts_with("nonce ") || nonce.len() <= "nonce ".len() {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
    }

    let mut input = input;
    // Consume any "push-option" lines (which are each terminated by LF).
    while let Ok((rest, line)) = take_data_line_expect_lf(input) {
        if line.starts_with("push-option ") {
            input = rest;
        } else {
            break;
        }
    }
    let (input, blank) = take_data_line_expect_lf(input)?;
    if !blank.is_empty() {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
    }
    let (input, cmds) = many0(command_line_with_lf).parse(input)?;
    // Consume any gpg-signature lines until we reach "push-cert-end".
    let (input, _gpg) = many0(parse_gpg_line_lf).parse(input)?;
    let (input, end_line) = take_data_line_expect_lf(input)?;
    if end_line != "push-cert-end" {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
    }
    Ok((input, cmds))
}

/// Parse one gpg signature line (or any line) that is not "push-cert-end"
/// In the push-cert grammar these lines are terminated by LF.
fn parse_gpg_line_lf(input: &[u8]) -> IResult<&[u8], ()> {
    let (input, line) = take_data_line_expect_lf(input)?;
    if line == "push-cert-end" {
        Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)))
    } else {
        Ok((input, ()))
    }
}

/// Parse a command line (without capabilities) as per:
///
/// ```text
///   create = zero-id SP new-id SP name
///   delete = old-id SP zero-id SP name
///   update = old-id SP new-id SP name
/// ```
pub(super) fn command_line(input: &[u8]) -> IResult<&[u8], RefModification> {
    let (input, line) = take_data_line(input)?;
    let (_, cmd) =
        command_line_str(line).map_err(|_| nom::Err::Error(Error::new(input, ErrorKind::Tag)))?;
    Ok((input, cmd))
}

/// Variant for push-cert: the command line here is terminated by LF.
pub(super) fn command_line_with_lf(input: &[u8]) -> IResult<&[u8], RefModification> {
    let (input, line) = take_data_line_expect_lf(input)?;
    let (_, cmd) =
        command_line_str(line).map_err(|_| nom::Err::Error(Error::new(input, ErrorKind::Tag)))?;
    Ok((input, cmd))
}

pub(super) fn command_line_str(input: &str) -> IResult<&str, RefModification> {
    let (input, token1) = take_while_m_n::<_, &str, nom::error::Error<&str>>(40, 40, |c: char| {
        c.is_ascii_hexdigit()
    })(input)?;
    let (input, _) = char(' ')(input)?;
    let (input, token2) = take_while_m_n::<_, &str, nom::error::Error<&str>>(40, 40, |c: char| {
        c.is_ascii_hexdigit()
    })(input)?;
    let (input, _) = char(' ')(input)?;
    let (input, token3) = rest(input)?;
    if token1 == ZERO_ID {
        Ok((
            input,
            RefModification::Create {
                new_id: token2.to_string(),
                ref_name: token3.to_string(),
            },
        ))
    } else if token2 == ZERO_ID {
        Ok((
            input,
            RefModification::Delete {
                old_id: token1.to_string(),
                ref_name: token3.to_string(),
            },
        ))
    } else {
        Ok((
            input,
            RefModification::Update {
                old_id: token1.to_string(),
                new_id: token2.to_string(),
                ref_name: token3.to_string(),
            },
        ))
    }
}

/// Consume one packet line and expect it to be a Data packet.
/// Returns the line content.
pub(super) fn take_data_line(input: &[u8]) -> IResult<&[u8], &str> {
    // We need at least 4 bytes for the header.
    if input.len() < 4 {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Eof)));
    }
    let header = &input[..4];
    let header_str =
        str::from_utf8(header).map_err(|_| nom::Err::Error(Error::new(input, ErrorKind::Alpha)))?;
    let line_len = usize::from_str_radix(header_str, 16)
        .map_err(|_| nom::Err::Error(Error::new(input, ErrorKind::Digit)))?;
    if line_len == 0 {
        // A flush packet is not acceptable here.
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Eof)));
    }
    if input.len() < line_len {
        return Err(nom::Err::Incomplete(Needed::new(line_len - input.len())));
    }
    // Call gix_packetline::decode to perform the actual decoding.
    let pkt = gix_packetline::decode(input)
        .map_err(|_| nom::Err::Error(Error::new(input, ErrorKind::Tag)))?;
    match pkt {
        gix_packetline::PacketLineRef::Data(data) => {
            let s = str::from_utf8(data)
                .map_err(|_| nom::Err::Error(Error::new(input, ErrorKind::Alpha)))?;
            Ok((&input[line_len..], s))
        }
        gix_packetline::PacketLineRef::Flush => {
            Err(nom::Err::Error(Error::new(input, ErrorKind::Eof)))
        }
        gix_packetline::PacketLineRef::Delimiter | gix_packetline::PacketLineRef::ResponseEnd => {
            Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)))
        }
    }
}

/// Consume one packet line and require that its data ends with a LF.
/// Returns the line content with the trailing LF removed.
///
/// This covers the grammar
///
/// ```text
/// PKT-LINE(... LF)
/// ```
///
/// where `...` is any content.
fn take_data_line_expect_lf(input: &[u8]) -> IResult<&[u8], &str> {
    let (input, line) = take_data_line(input)?;
    if let Some(stripped) = line.strip_suffix('\n') {
        Ok((input, stripped))
    } else {
        Err(nom::Err::Error(Error::new(input, ErrorKind::Char)))
    }
}

/// Consume one packet line and expect it to be a Flush packet.
pub(super) fn take_flush_pkt(input: &[u8]) -> IResult<&[u8], ()> {
    if input.len() < 4 {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Eof)));
    }
    let header = &input[..4];
    let header_str =
        str::from_utf8(header).map_err(|_| nom::Err::Error(Error::new(input, ErrorKind::Alpha)))?;
    let line_len = usize::from_str_radix(header_str, 16)
        .map_err(|_| nom::Err::Error(Error::new(input, ErrorKind::Digit)))?;
    // For a flush packet the header should encode 0.
    if line_len != 0 {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
    }
    Ok((&input[4..], ()))
}
