use mockito::Server;
use std::env;
use teamprojekt_agents::agent_actions::ask_user::ask_user;

#[test]
fn test_ask_user_success() {
    // Create mock server
    let mut server = Server::new();
    let server_url = server.url();

    // Set the vars to fit the URL of the Mock-Server and use anything for MINION_API_TOKEN.
    unsafe {
        env::set_var("MINION_API_BASE_URL", server_url);
        env::set_var("MINION_API_TOKEN", "testtoken");
    }

    // Create mock endpoint
    let _mock = server
        .mock("POST", "/agent/inquiry")
        .match_header("authorization", "Bearer testtoken")
        .match_header("content-type", "application/json")
        .match_body(r#"{"inquiry":"Testinquiry"}"#)
        .with_status(200)
        .with_body("Mock-Server response")
        .create();
    // Check whether response contains "Mock-Server response", since its the definded outcome of the Mock-Server.
    let response = ask_user("Testinquiry");
    assert_eq!(response, "Mock-Server response");
}

#[test]
fn test_ask_user_error() {
    // Create mock server
    let mut server = Server::new();
    let server_url = server.url();

    // Set the vars to fit the URL of the Mock-Server and use anything for MINION_API_TOKEN.
    unsafe {
        env::set_var("MINION_API_BASE_URL", server_url);
        env::set_var("MINION_API_TOKEN", "testtoken");
    }

    // Mock endpoint returns error
    let _mock = server
        .mock("POST", "/agent/inquiry")
        .with_status(500)
        .create();
    // Check whether response contains [ERROR], since its part of the expected outcome. 
    let response = ask_user("Failure");
    assert!(response.contains("[ERROR]"));
}
