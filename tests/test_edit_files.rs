use std::fs;
use teamprojekt_agents::agent_actions::edit_files::{edit_file, edit_file_from_to};

fn cleanup(file: &str) {
    let _ = fs::remove_file(file);
}

#[test]
fn test_edit_file_creates_and_writes() {
    let test_file = "test_creates_and_writes.txt";
    cleanup(test_file);
    let content = "Hello, world!";
    edit_file(test_file.to_string(), content.to_string());
    let read = fs::read_to_string(test_file).expect("File should exist after edit_file");
    assert_eq!(read, content, "File content mismatch after create/write");
    cleanup(test_file);
}

#[test]
fn test_edit_file_overwrites() {
    let test_file = "test_overwrites.txt";
    cleanup(test_file);
    fs::write(test_file, "Old content").expect("Failed to write initial content");
    let new_content = "New content";
    edit_file(test_file.to_string(), new_content.to_string());
    let read = fs::read_to_string(test_file).expect("File should exist after overwrite");
    assert_eq!(read, new_content, "File content mismatch after overwrite");
    cleanup(test_file);
}

#[test]
fn test_edit_file_from_to_middle() {
    let test_file = "test_from_to_middle.txt";
    cleanup(test_file);
    fs::write(test_file, "abcdefg").expect("Failed to write initial content");
    edit_file_from_to(test_file.to_string(), "XY".to_string(), 2, 4);
    let read = fs::read_to_string(test_file).expect("File should exist after from_to");
    assert_eq!(
        read, "abXYefg",
        "File content mismatch after from_to_middle"
    );
    cleanup(test_file);
}

#[test]
fn test_edit_file_from_to_out_of_bounds() {
    let test_file = "test_from_to_out_of_bounds.txt";
    cleanup(test_file);
    fs::write(test_file, "abc").expect("Failed to write initial content");
    edit_file_from_to(test_file.to_string(), "XYZ".to_string(), 1, 10);
    let read =
        fs::read_to_string(test_file).expect("File should exist after from_to out of bounds");
    assert_eq!(
        read, "aXYZ",
        "File content mismatch after from_to_out_of_bounds"
    );
    cleanup(test_file);
}

#[test]
fn test_edit_file_from_to_on_new_file() {
    let test_file = "test_from_to_new_file.txt";
    cleanup(test_file);
    edit_file_from_to(test_file.to_string(), "Hello".to_string(), 0, 0);
    let read = fs::read_to_string(test_file).expect("File should exist after from_to on new file");
    assert_eq!(
        read, "Hello",
        "File content mismatch after from_to_on_new_file"
    );
    cleanup(test_file);
}
