#[cfg(test)]
mod tests {
    //use super::*; -> this imports parent modules

    use teamprojekt_agents::memory::Memory;
    use teamprojekt_agents::models::Model;
    use url::Url;

    #[test]
    fn test_memory_set_history() {
        // setup
        let expected_history = "Test history.".to_string();
        let memory = Memory::new(
            "test".to_string(),
            &Url::parse("https://www.test.io/").expect("Invalid URL"),
        );
        let memory = memory.with_model(Model::TestModel); //to prevent prompting a real model

        // execute
        let new_memory = memory.with_history(&expected_history);
        let actual_history = new_memory.read();

        // assert
        assert_eq!(
            expected_history.clone(),
            actual_history.clone(),
            "2 + 3 should equal 5"
        );
    }
}
