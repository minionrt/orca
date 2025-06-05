use std::fs;
use teamprojekt_agents::agent_actions::read_files::read_file;

fn cleanup(file: &str) {
    let _ = fs::remove_file(file);
}

#[test]
fn test_read_file_existing() {
    let test_file = "test_read_existing.txt";
    cleanup(test_file);
    let content = "Test content for reading.";
    fs::write(test_file, content).expect("Failed to write test file");

    let result = read_file(test_file.to_string());
    assert_eq!(result, Some(content.to_string()), "Content mismatch");

    cleanup(test_file);
}

#[test]
fn test_read_file_nonexistent() {
    let test_file = "test_read_nonexistent.txt";
    cleanup(test_file);

    let result = read_file(test_file.to_string());
    assert_eq!(result, None, "Expected None for nonexistent file");
}

#[test]
fn test_read_file_empty() {
    let test_file = "test_read_empty.txt";
    cleanup(test_file);
    fs::write(test_file, "").expect("Failed to create empty file");

    let result = read_file(test_file.to_string());
    assert_eq!(result, Some(String::new()), "Expected empty string");

    cleanup(test_file);
}
