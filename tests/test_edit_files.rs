use std::fs;
use std::path::Path;
use teamprojekt_agents::agent_actions::edit_files::{edit_file, edit_file_from_to};


const TEST_FILE: &str = "test.txt";

fn cleanup() {
    let _ = fs::remove_file(TEST_FILE);
}

#[test]
fn test_edit_file_creates_and_writes() {
    cleanup();
    let content = "Hello, world!";
    edit_file(TEST_FILE.to_string(), content.to_string());
    let read = fs::read_to_string(TEST_FILE).unwrap();
    assert_eq!(read, content);
    cleanup();
}

#[test]
fn test_edit_file_overwrites() {
    cleanup();
    fs::write(TEST_FILE, "Old content").unwrap();
    let new_content = "New content";
    edit_file(TEST_FILE.to_string(), new_content.to_string());
    let read = fs::read_to_string(TEST_FILE).unwrap();
    assert_eq!(read, new_content);
    cleanup();
}

#[test]
fn test_edit_file_from_to_middle() {
    cleanup();
    fs::write(TEST_FILE, "abcdefg").unwrap();
    // Replace "cd" (indices 2..4) with "XY"
    edit_file_from_to(TEST_FILE.to_string(), "XY".to_string(), 2, 4);
    let read = fs::read_to_string(TEST_FILE).unwrap();
    assert_eq!(read, "abXYefg");
    cleanup();
}

#[test]
fn test_edit_file_from_to_out_of_bounds() {
    cleanup();
    fs::write(TEST_FILE, "abc").unwrap();
    // Try to replace from 1 to 10 with "XYZ"
    edit_file_from_to(TEST_FILE.to_string(), "XYZ".to_string(), 1, 10);
    let read = fs::read_to_string(TEST_FILE).unwrap();
    assert_eq!(read, "aXYZ");
    cleanup();
}

#[test]
fn test_edit_file_from_to_on_new_file() {
    cleanup();
    // File does not exist, should create with content
    edit_file_from_to(TEST_FILE.to_string(), "Hello".to_string(), 0, 0);
    let read = fs::read_to_string(TEST_FILE).unwrap();
    assert_eq!(read, "Hello");
    cleanup();
}