//! Tests for the parsing functions in the `parse_commands` module.
#![cfg(test)]

use std::str;

use nom::error::ErrorKind;

use super::parse_commands::*;
use super::pkt_line_string;

// Test fixtures
//
//

/// Create a flush packet ("0000") as a String.
fn flush_pkt() -> String {
    "0000".to_string()
}

/// Build a push-cert input given optional push-option lines,
/// command lines, and GPG signature lines.
fn build_push_cert_input(
    push_option_lines: &[&str],
    command_lines: &[&str],
    gpg_lines: &[&str],
) -> Vec<u8> {
    let mut s = String::new();
    s.push_str(&pkt_line_string("push-cert\0cap1 cap2\n"));
    s.push_str(&pkt_line_string("certificate version 0.1\n"));
    s.push_str(&pkt_line_string("pusher someone@example.com\n"));
    s.push_str(&pkt_line_string("pushee https://example.com/repo.git\n"));
    s.push_str(&pkt_line_string("nonce 12345\n"));
    for opt in push_option_lines {
        s.push_str(&pkt_line_string(&format!("push-option {opt}\n")));
    }
    // Required blank line.
    s.push_str(&pkt_line_string("\n"));
    for cmd in command_lines {
        s.push_str(&pkt_line_string(&(cmd.to_string() + "\n")));
    }
    for gpg in gpg_lines {
        s.push_str(&pkt_line_string(&(gpg.to_string() + "\n")));
    }
    s.push_str(&pkt_line_string("push-cert-end\n"));
    s.into_bytes()
}

/// Returns a valid create command line.
fn create_command_line(ref_name: &str, new_id: &str) -> String {
    format!("{ZERO_ID} {new_id} {ref_name}")
}

/// Returns a valid update command line.
fn update_command_line(ref_name: &str, old_id: &str, new_id: &str) -> String {
    format!("{old_id} {new_id} {ref_name}")
}

// Tests for shallow_parser
//
//

#[test]
fn test_shallow_parser_valid() {
    let obj_id = "0123456789abcdef0123456789abcdef01234567";
    let shallow_line = format!("shallow {obj_id}");
    let input = pkt_line_string(&shallow_line);
    let res = shallow(input.as_bytes());
    assert!(
        res.is_ok(),
        "Valid shallow line must be parsed successfully"
    );
    let (remaining, line) = res.unwrap();
    assert!(remaining.is_empty(), "All input should be consumed");
    assert_eq!(line, shallow_line);
}

#[test]
fn test_shallow_parser_rejects_extra_characters() {
    let obj_id = "0123456789abcdef0123456789abcdef01234567";
    let payload = format!("shallow {obj_id}EXTRA");
    let full = pkt_line_string(&payload);
    let res = shallow(full.as_bytes());
    assert!(
        res.is_err(),
        "Extra characters after a valid object id should be rejected"
    );
}

#[test]
fn test_shallow_parser_invalid_objid() {
    let obj_id = "0123456789abcdef0123456789abcdef0123456G"; // 'G' is not valid hex
    let shallow_line = format!("shallow {obj_id}");
    let input = pkt_line_string(&shallow_line);
    assert!(
        shallow(input.as_bytes()).is_err(),
        "Non-hex characters in object id should error"
    );
}

// Tests for command_line_str
//
//

#[test]
fn test_command_line_str_create() {
    let new_id = "1234567890abcdef1234567890abcdef12345678";
    let ref_name = "refs/heads/feature";
    let cmd = create_command_line(ref_name, new_id);
    let res = command_line_str(&cmd);
    assert!(res.is_ok(), "create command should be parsed successfully");
    let (_remaining, modification) = res.unwrap();
    assert_eq!(
        modification,
        RefModification::Create {
            new_id: new_id.to_string(),
            ref_name: ref_name.to_string()
        }
    );
}

#[test]
fn test_command_line_str_delete() {
    let old_id = "1234567890abcdef1234567890abcdef12345678";
    let ref_name = "refs/heads/oldbranch";
    let cmd = format!("{old_id} {ZERO_ID} {ref_name}");
    let res = command_line_str(&cmd);
    assert!(res.is_ok(), "delete command should be parsed successfully");
    let (_remaining, modification) = res.unwrap();
    assert_eq!(
        modification,
        RefModification::Delete {
            old_id: old_id.to_string(),
            ref_name: ref_name.to_string()
        }
    );
}

#[test]
fn test_command_line_str_update() {
    let old_id = "1234567890abcdef1234567890abcdef12345678";
    let new_id = "abcdef1234567890abcdef1234567890abcdef12";
    let ref_name = "refs/heads/master";
    let cmd = format!("{old_id} {new_id} {ref_name}");
    let res = command_line_str(&cmd);
    assert!(res.is_ok(), "update command should be parsed successfully");
    let (_remaining, modification) = res.unwrap();
    assert_eq!(
        modification,
        RefModification::Update {
            old_id: old_id.to_string(),
            new_id: new_id.to_string(),
            ref_name: ref_name.to_string()
        }
    );
}

// Tests for command_line_packet and command_list_parser
//
//

#[test]
fn test_command_line_packet() {
    let old_id = "1234567890abcdef1234567890abcdef12345678";
    let new_id = "abcdef1234567890abcdef1234567890abcdef12";
    let ref_name = "refs/heads/develop";
    let cmd = format!("{old_id} {new_id} {ref_name}");
    let pkt = pkt_line_string(&cmd);
    let res = command_line(pkt.as_bytes());
    assert!(res.is_ok(), "A valid command line packet should be parsed");
    let (remaining, modification) = res.unwrap();
    assert!(
        remaining.is_empty(),
        "The parser should consume the entire packet"
    );
    assert_eq!(
        modification,
        RefModification::Update {
            old_id: old_id.to_string(),
            new_id: new_id.to_string(),
            ref_name: ref_name.to_string()
        }
    );
}

#[test]
fn test_command_list_parser_success() {
    // First command: create (with capabilities in the first packet)
    let new_id = "1111111111111111111111111111111111111111";
    let ref_name1 = "refs/heads/feature1";
    let cmd1 = create_command_line(ref_name1, new_id);
    let first_line = format!("{}{}\0{}", "", cmd1, "cap1 cap2");
    let first_pkt = pkt_line_string(&first_line);

    // Second command: update.
    let old_id = "2222222222222222222222222222222222222222";
    let new_id2 = "3333333333333333333333333333333333333333";
    let ref_name2 = "refs/heads/feature2";
    let cmd2 = format!("{old_id} {new_id2} {ref_name2}");
    let second_pkt = pkt_line_string(&cmd2);

    let mut payload = String::new();
    payload.push_str(&first_pkt);
    payload.push_str(&second_pkt);
    payload.push_str(&flush_pkt());

    let res = command_list(payload.as_bytes());
    assert!(
        res.is_ok(),
        "Valid command list must be parsed successfully"
    );
    let (remaining, cmds) = res.unwrap();
    assert!(remaining.is_empty(), "All input should be consumed");
    assert_eq!(cmds.len(), 2, "Exactly two commands are expected");

    // Check first command (create)
    assert_eq!(
        cmds[0],
        RefModification::Create {
            new_id: new_id.to_string(),
            ref_name: ref_name1.to_string()
        }
    );
    // Check second command (update)
    assert_eq!(
        cmds[1],
        RefModification::Update {
            old_id: old_id.to_string(),
            new_id: new_id2.to_string(),
            ref_name: ref_name2.to_string()
        }
    );
}

#[test]
fn test_command_list_parser_failure_missing_nul() {
    // A command line without the required NUL separator should fail.
    let old_id = "2222222222222222222222222222222222222222";
    let new_id = "3333333333333333333333333333333333333333";
    let ref_name = "refs/heads/feature";
    let cmd = format!("{old_id} {new_id} {ref_name}");
    let first_pkt = pkt_line_string(&cmd);
    let mut payload = String::new();
    payload.push_str(&first_pkt);
    payload.push_str(&flush_pkt());
    assert!(
        command_list(payload.as_bytes()).is_err(),
        "Missing NUL separator should cause a parser error"
    );
}

// Tests for push_cert_parser
//
//

#[test]
fn test_push_cert_parser_success_standard() {
    let create_cmd = create_command_line(
        "refs/heads/master",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    );
    let input = build_push_cert_input(&["opt1"], &[&create_cmd], &["gpgsig abcdef"]);
    let res = push_cert(&input);
    assert!(
        res.is_ok(),
        "Valid push-cert input should be parsed successfully"
    );
    let (remaining, cmds) = res.unwrap();
    assert!(remaining.is_empty(), "No data should remain after parsing");
    assert_eq!(cmds.len(), 1, "One command is expected");
    match &cmds[0] {
        RefModification::Create { new_id, ref_name } => {
            assert_eq!(new_id, "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
            assert_eq!(ref_name, "refs/heads/master");
        }
        _ => panic!("Expected a Create command"),
    }
}

#[test]
fn test_push_cert_parser_fail_wrong_header() {
    let mut s = String::new();
    s.push_str(&pkt_line_string("not-push-cert\0cap1 cap2\n"));
    s.push_str(&pkt_line_string("certificate version 0.1\n"));
    s.push_str(&pkt_line_string("pusher someone@example.com\n"));
    s.push_str(&pkt_line_string("pushee https://example.com/repo.git\n"));
    s.push_str(&pkt_line_string("nonce 12345\n"));
    s.push_str(&pkt_line_string("\n"));
    s.push_str(&pkt_line_string(
        &(create_command_line(
            "refs/heads/master",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        ) + "\n"),
    ));
    s.push_str(&pkt_line_string("push-cert-end\n"));

    let input = s.into_bytes();
    assert!(
        push_cert(&input).is_err(),
        "A wrong push-cert header must result in an error"
    );
}

#[test]
fn test_push_cert_parser_fail_bad_cert_version() {
    let mut s = String::new();
    s.push_str(&pkt_line_string("push-cert\0cap1 cap2\n"));
    s.push_str(&pkt_line_string("certificate version 9.9\n")); // incorrect version
    s.push_str(&pkt_line_string("pusher someone@example.com\n"));
    s.push_str(&pkt_line_string("pushee https://example.com/repo.git\n"));
    s.push_str(&pkt_line_string("nonce 12345\n"));
    s.push_str(&pkt_line_string("\n"));
    s.push_str(&pkt_line_string(
        &(create_command_line(
            "refs/heads/master",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        ) + "\n"),
    ));
    s.push_str(&pkt_line_string("push-cert-end\n"));

    let input = s.into_bytes();
    assert!(
        push_cert(&input).is_err(),
        "An invalid certificate version should be rejected"
    );
}

#[test]
fn test_push_cert_parser_fail_missing_blank_line() {
    let mut s = String::new();
    s.push_str(&pkt_line_string("push-cert\0cap1 cap2\n"));
    s.push_str(&pkt_line_string("certificate version 0.1\n"));
    s.push_str(&pkt_line_string("pusher someone@example.com\n"));
    s.push_str(&pkt_line_string("pushee https://example.com/repo.git\n"));
    s.push_str(&pkt_line_string("nonce 12345\n"));
    // Missing blank line: directly a command line.
    s.push_str(&pkt_line_string(
        &(create_command_line(
            "refs/heads/master",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        ) + "\n"),
    ));
    s.push_str(&pkt_line_string("push-cert-end\n"));

    let input = s.into_bytes();
    assert!(
        push_cert(&input).is_err(),
        "Missing blank line after push-option lines should fail"
    );
}

#[test]
fn test_push_cert_parser_fail_missing_push_cert_end() {
    let mut s = String::new();
    s.push_str(&pkt_line_string("push-cert\0cap1 cap2\n"));
    s.push_str(&pkt_line_string("certificate version 0.1\n"));
    s.push_str(&pkt_line_string("pusher someone@example.com\n"));
    s.push_str(&pkt_line_string("pushee https://example.com/repo.git\n"));
    s.push_str(&pkt_line_string("nonce 12345\n"));
    s.push_str(&pkt_line_string("push-option opt1\n"));
    s.push_str(&pkt_line_string("\n"));
    s.push_str(&pkt_line_string(
        &(create_command_line(
            "refs/heads/master",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        ) + "\n"),
    ));
    s.push_str(&pkt_line_string("gpgsig abcdef\n"));
    // Missing push-cert-end
    let input = s.into_bytes();
    assert!(
        push_cert(&input).is_err(),
        "Missing push-cert-end should cause an error"
    );
}

#[test]
fn test_push_cert_success_minimal() {
    let input = build_push_cert_input(&[], &[], &[]);
    let res = push_cert(&input);
    assert!(
        res.is_ok(),
        "A minimal push-cert input should parse successfully"
    );
    let (remaining, cmds) = res.unwrap();
    assert!(remaining.is_empty(), "All input should be consumed");
    assert_eq!(
        cmds.len(),
        0,
        "No commands should be present in minimal input"
    );
}

#[test]
fn test_push_cert_success_multiple_commands() {
    let create_cmd = create_command_line(
        "refs/heads/feature",
        "cccccccccccccccccccccccccccccccccccccccc",
    );
    let update_cmd = update_command_line(
        "refs/heads/bugfix",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "dddddddddddddddddddddddddddddddddddddddd",
    );
    let input = build_push_cert_input(&["opt1", "opt2"], &[&create_cmd, &update_cmd], &[]);
    let res = push_cert(&input);
    assert!(res.is_ok(), "Multiple commands must be parsed correctly");
    let (remaining, cmds) = res.unwrap();
    assert!(remaining.is_empty());
    assert_eq!(cmds.len(), 2, "Expected two commands");
    match &cmds[0] {
        RefModification::Create { new_id, ref_name } => {
            assert_eq!(new_id, "cccccccccccccccccccccccccccccccccccccccc");
            assert_eq!(ref_name, "refs/heads/feature");
        }
        _ => panic!("Expected first command to be Create"),
    }
    match &cmds[1] {
        RefModification::Update {
            old_id,
            new_id,
            ref_name,
        } => {
            assert_eq!(old_id, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            assert_eq!(new_id, "dddddddddddddddddddddddddddddddddddddddd");
            assert_eq!(ref_name, "refs/heads/bugfix");
        }
        _ => panic!("Expected second command to be Update"),
    }
}

#[test]
fn test_push_cert_success_multiple_gpg_lines() {
    let create_cmd = create_command_line(
        "refs/heads/master",
        "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
    );
    let input = build_push_cert_input(
        &["opt1"],
        &[&create_cmd],
        &["gpgsig line1", "gpgsig line2", "gpgsig line3"],
    );
    let res = push_cert(&input);
    assert!(
        res.is_ok(),
        "Multiple GPG lines should not interfere with command parsing"
    );
    let (remaining, cmds) = res.unwrap();
    assert!(remaining.is_empty());
    assert_eq!(cmds.len(), 1, "Only one command should be present");
    match &cmds[0] {
        RefModification::Create { new_id, ref_name } => {
            assert_eq!(new_id, "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
            assert_eq!(ref_name, "refs/heads/master");
        }
        _ => panic!("Expected a Create command"),
    }
}

// Tests for low-level functions
//
//

#[test]
fn test_take_data_line_incomplete_header() {
    let input = b"00";
    let res = take_data_line(input);
    assert!(res.is_err(), "Input with less than 4 bytes should error");
    if let Err(nom::Err::Error(e)) = res {
        assert_eq!(e.code, ErrorKind::Eof);
    } else {
        panic!("Expected an Eof error");
    }
}

#[test]
fn test_take_flush_pkt_invalid() {
    let non_flush = pkt_line_string("some data");
    let res = take_flush_pkt(non_flush.as_bytes());
    assert!(res.is_err(), "Non-flush packet should be rejected");
}

#[test]
fn test_command_line_str_invalid_hex_token1() {
    let invalid_token1 = "g1234567890abcdef1234567890abcdef1234567"; // invalid hex char 'g'
    let token2 = "abcdef1234567890abcdef1234567890abcdef12";
    let ref_name = "refs/heads/master";
    let line = format!("{invalid_token1} {token2} {ref_name}");
    assert!(
        command_line_str(&line).is_err(),
        "Invalid hex in token1 should cause an error"
    );
}

#[test]
fn test_command_line_with_lf_missing_lf() {
    let old_id = "1234567890abcdef1234567890abcdef12345678";
    let new_id = "abcdef1234567890abcdef1234567890abcdef12";
    let ref_name = "refs/heads/develop";
    let cmd = format!("{old_id} {new_id} {ref_name}");
    // Our pkt_line_string uses the string as provided so it will not add an LF if not present.
    let pkt = pkt_line_string(&cmd);
    assert!(
        command_line_with_lf(pkt.as_bytes()).is_err(),
        "Missing LF should cause an error"
    );
}

#[test]
fn test_update_requests_parser_empty_input() {
    let input = b"";
    assert!(
        update_requests(input).is_err(),
        "Empty input should cause an error"
    );
}

#[test]
fn test_push_cert_parser_missing_nonce() {
    let mut s = String::new();
    s.push_str(&pkt_line_string("push-cert\0cap1 cap2\n"));
    s.push_str(&pkt_line_string("certificate version 0.1\n"));
    s.push_str(&pkt_line_string("pusher someone@example.com\n"));
    s.push_str(&pkt_line_string("pushee https://example.com/repo.git\n"));
    // Omit the nonce line.
    s.push_str(&pkt_line_string("push-option opt1\n"));
    s.push_str(&pkt_line_string("\n"));
    s.push_str(&pkt_line_string(
        &(create_command_line(
            "refs/heads/master",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        ) + "\n"),
    ));
    s.push_str(&pkt_line_string("push-cert-end\n"));
    let input = s.into_bytes();
    assert!(
        push_cert(&input).is_err(),
        "Missing nonce line should cause an error"
    );
}

#[test]
fn test_push_cert_parser_bad_pusher_line() {
    let mut s = String::new();
    s.push_str(&pkt_line_string("push-cert\0cap1 cap2\n"));
    s.push_str(&pkt_line_string("certificate version 0.1\n"));
    // pusher line without required space after the keyword.
    s.push_str(&pkt_line_string("pusher\n"));
    s.push_str(&pkt_line_string("pushee https://example.com/repo.git\n"));
    s.push_str(&pkt_line_string("nonce 12345\n"));
    s.push_str(&pkt_line_string("\n"));
    s.push_str(&pkt_line_string(
        &(create_command_line(
            "refs/heads/master",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        ) + "\n"),
    ));
    s.push_str(&pkt_line_string("push-cert-end\n"));
    let input = s.into_bytes();
    assert!(
        push_cert(&input).is_err(),
        "Bad pusher line should result in an error"
    );
}
