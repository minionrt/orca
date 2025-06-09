use teamprojekt_agents::llm::Completion;
use teamprojekt_agents::openai::ToolCall;

#[test]
fn parses_only_toolcalls_to_completionkind() {
    //example response body for tools
    let json = r#"
        [
            {
                "id": "call_abc123",
                "type": "function",
                "function": {
                    "name": "get_current_weather",
                    "arguments": "{\"location\": \"Boston, MA\"}"
                }
            }
        ]
        "#;

    let tool_calls: Vec<ToolCall> = serde_json::from_str(json).unwrap();
    let completion_kind = Completion::ToolCalls(tool_calls.clone());
    // run cargo test -- --nocapture to see actual parsing.
    println!("{:?}", completion_kind);

    // checking if it was parsed right
    match completion_kind {
        Completion::ToolCalls(tc) => {
            assert_eq!(tc[0].function.name, "get_current_weather");
            assert_eq!(tc[0].function.arguments, "{\"location\": \"Boston, MA\"}");
        }
        _ => panic!("Did not parse ToolCalls correctly!"),
    }
}
