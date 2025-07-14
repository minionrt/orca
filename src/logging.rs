use tracing_subscriber::{fmt, EnvFilter};

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

/// Initialize simple console-only logging (useful for tests)
pub fn init_simple_logging() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::{info, warn, error, debug};

    #[test]
    fn test_simple_logging_initialization() {
        // This test verifies that our logging setup doesn't panic
        init_simple_logging();
        
        debug!("This is a debug message");
        info!("This is an info message");
        warn!("This is a warning message");
        error!("This is an error message");
        
        // If we get here without panicking, logging is working
        assert!(true);
    }
}
