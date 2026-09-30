//! Logging utilities for EGS
//!
//! Provides logging macros and utilities for consistent logging across EGS components.

use tracing::{debug, error, info, warn, Level};

/// Initializes logging with default configuration
pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter("egs=debug,info")
        .with_target(true)
        .with_line_number(true)
        .init();
}

/// Initializes logging with custom filter
pub fn init_logging_with_filter(filter: impl Into<String>) {
    tracing_subscriber::fmt()
        .with_env_filter(filter.into())
        .with_target(true)
        .with_line_number(true)
        .init();
}

/// Logs an error with context
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        error!($($arg)*)
    };
}

/// Logs a warning with context
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        warn!($($arg)*)
    };
}

/// Logs information with context
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        info!($($arg)*)
    };
}

/// Logs debug information with context
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        debug!($($arg)*)
    };
}

/// Logs a message with a specific level
pub fn log_with_level(level: Level, message: impl Into<String>) {
    match level {
        Level::ERROR => error!("{}", message.into()),
        Level::WARN => warn!("{}", message.into()),
        Level::INFO => info!("{}", message.into()),
        Level::DEBUG => debug!("{}", message.into()),
        Level::TRACE => tracing::trace!("{}", message.into()),
    }
}
