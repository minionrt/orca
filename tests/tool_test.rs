use orca::llm::Completion;
use orca::openai::ToolCall;

#[test]
fn parses_only_toolcalls_to_completionkind() {
    // Example response body for tools
    let json = r#"
        [
            {
                "id": "call_abc123",
                "type": "function",
                "function": {
                    "name": "get_current_weather",
                    "arguments": { "location": "Boston, MA" }
                }
            }
        ]
        "#;

    let tool_calls: Vec<ToolCall> = serde_json::from_str(json).unwrap();
    let completion_kind = Completion::ToolCalls(tool_calls.clone());
    // Run cargo test -- --nocapture to see actual parsing.
    println!("{completion_kind:?}");

    // Checking if it was parsed right
    match completion_kind {
        Completion::ToolCalls(tc) => {
            assert_eq!(tc[0].function.name, "get_current_weather");
            assert_eq!(
                tc[0].function.arguments["location"].as_str(),
                Some("Boston, MA")
            );
            assert_eq!(tc[0].tool_type, "function");
        }
        _ => panic!("Did not parse ToolCalls correctly!"),
    }
}
