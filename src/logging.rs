use tracing_subscriber::EnvFilter;

/// Initialize the tracing subscriber with console-only logging
pub fn init_logging() {
    // Create a filter from environment variables (default to INFO level)
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Set up console logging with colors and pretty formatting
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(true)
        .init();
}

