#[cfg(test)]
mod tests {
    // Use super::*; -> this imports parent modules

    use orca::memory::Memory;
    use orca::models::Model;
    use url::Url;

    #[test]
    fn test_memory_set_history() {
        // Setup
        let history = "Test history.".to_string();
        let expected_history =
            "Summarized history:\nTest history.\n\nRecent interactions:\n".to_string();
        let memory = Memory::new(
            "test",
            &Url::parse("https://www.test.io/").expect("Invalid URL"),
        );
        let memory = memory.with_model(Model::Test); // To prevent prompting a real model

        // Execute
        let new_memory = memory.with_history(&history);
        let actual_history = new_memory.read();

        // Assert
        assert_eq!(
            expected_history.clone(),
            actual_history.clone(),
            "Setting the history didn't work."
        );
    }
}
