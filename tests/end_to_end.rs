mod e2e;

use std::sync::Once;
use tracing_subscriber::EnvFilter;

static INIT: Once = Once::new();

fn init_tracing() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .with_test_writer()
            .try_init()
            .ok();
    });
}

#[test]
fn test_simple() -> Result<(), Box<dyn std::error::Error>> {
    use e2e::*;

    init_tracing();

    let test_env = environment::AgentTestEnvironment::new()
        .with_task(&Task::new("Test task"))
        .expect_llm_call(&LLMResponse::Message("do nothing".to_owned()), None);

    let runner = test_env.build("./Containerfile")?.run()?;
    let results = runner.join()?;

    assert!(results.test_failure().is_none());

    Ok(())
}
