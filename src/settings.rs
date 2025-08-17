//! Global error configuration and environment variable handling.
//!
//! This module provides zero-configuration error settings that can be customized
//! via environment variables, avoiding circular dependencies while maintaining
//! flexible configuration options.

/// Log level for error output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
}

impl LogLevel {
    /// Parse log level from environment variable string.
    ///
    /// # Returns
    /// Some(LogLevel) if the string is recognized, None otherwise.
    fn from_env() -> Option<Self> {
        std::env::var("TYL_ERROR_LOG_LEVEL").ok().and_then(|level| {
            match level.to_uppercase().as_str() {
                "ERROR" => Some(LogLevel::Error),
                "WARN" | "WARNING" => Some(LogLevel::Warn),
                "INFO" => Some(LogLevel::Info),
                "DEBUG" => Some(LogLevel::Debug),
                _ => None,
            }
        })
    }
}

/// Global error configuration from environment variables.
///
/// Zero-config with sensible defaults to avoid circular dependencies.
/// Settings are loaded once and cached for the lifetime of the application.
pub struct ErrorSettings {
    /// Whether backtraces are enabled for errors.
    pub backtrace_enabled: bool,
    /// Maximum number of retry attempts for retriable errors.
    pub max_retries: usize,
    /// Whether to automatically log errors to stderr.
    pub log_errors: bool,
    /// Minimum log level for error output.
    pub log_level: LogLevel,
}

impl ErrorSettings {
    /// Get global error settings from environment variables.
    ///
    /// Settings are loaded once on first access and cached using OnceLock.
    /// This ensures consistent behavior throughout the application lifetime
    /// and avoids repeated environment variable parsing.
    ///
    /// # Environment Variables
    ///
    /// | Variable | Default | Description |
    /// |----------|---------|-------------|
    /// | `TYL_ERROR_BACKTRACE` | `false` | Enable error backtraces (`true`/`false`) |
    /// | `TYL_ERROR_MAX_RETRIES` | `3` | Maximum retry attempts for retriable errors |
    /// | `TYL_ERROR_LOG_ERRORS` | `true` | Log errors to stderr (`true`/`false`) |
    /// | `TYL_ERROR_LOG_LEVEL` | `INFO` | Log level (`ERROR`/`WARN`/`INFO`/`DEBUG`) |
    /// | `RUST_BACKTRACE` | - | Standard Rust backtrace (overrides TYL_ERROR_BACKTRACE) |
    ///
    /// # Returns
    /// A static reference to the global ErrorSettings instance.
    pub fn global() -> &'static Self {
        use std::sync::OnceLock;
        static SETTINGS: OnceLock<ErrorSettings> = OnceLock::new();

        SETTINGS.get_or_init(|| {
            let backtrace_enabled = std::env::var("TYL_ERROR_BACKTRACE")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or_else(|_| {
                    // Fall back to RUST_BACKTRACE if TYL_ERROR_BACKTRACE not set
                    std::env::var("RUST_BACKTRACE").is_ok()
                });

            let max_retries = std::env::var("TYL_ERROR_MAX_RETRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3);

            let log_errors = std::env::var("TYL_ERROR_LOG_ERRORS")
                .map(|v| v.to_lowercase() != "false")
                .unwrap_or(true);

            let log_level = LogLevel::from_env().unwrap_or(LogLevel::Info);

            ErrorSettings {
                backtrace_enabled,
                max_retries,
                log_errors,
                log_level,
            }
        })
    }

    /// Create ErrorSettings with custom values (primarily for testing).
    ///
    /// # Arguments
    /// * `backtrace_enabled` - Whether to enable backtraces
    /// * `max_retries` - Maximum retry attempts
    /// * `log_errors` - Whether to log errors
    /// * `log_level` - Minimum log level
    ///
    /// # Returns
    /// A new ErrorSettings instance with the specified values.
    pub fn new(
        backtrace_enabled: bool,
        max_retries: usize,
        log_errors: bool,
        log_level: LogLevel,
    ) -> Self {
        Self {
            backtrace_enabled,
            max_retries,
            log_errors,
            log_level,
        }
    }

    /// Create ErrorSettings with default values.
    pub fn default() -> Self {
        Self {
            backtrace_enabled: false,
            max_retries: 3,
            log_errors: true,
            log_level: LogLevel::Info,
        }
    }
}

impl Default for ErrorSettings {
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        // Test that log levels are properly ordered
        assert!(LogLevel::Error < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Debug);
    }

    #[test]
    fn test_log_level_from_env() {
        // Test parsing various log level strings
        std::env::set_var("TYL_ERROR_LOG_LEVEL", "ERROR");
        assert_eq!(LogLevel::from_env(), Some(LogLevel::Error));

        std::env::set_var("TYL_ERROR_LOG_LEVEL", "warn");
        assert_eq!(LogLevel::from_env(), Some(LogLevel::Warn));

        std::env::set_var("TYL_ERROR_LOG_LEVEL", "INFO");
        assert_eq!(LogLevel::from_env(), Some(LogLevel::Info));

        std::env::set_var("TYL_ERROR_LOG_LEVEL", "debug");
        assert_eq!(LogLevel::from_env(), Some(LogLevel::Debug));

        std::env::set_var("TYL_ERROR_LOG_LEVEL", "invalid");
        assert_eq!(LogLevel::from_env(), None);

        std::env::remove_var("TYL_ERROR_LOG_LEVEL");
    }

    #[test]
    fn test_error_settings_default() {
        // Test that default settings have expected values
        let settings = ErrorSettings::default();
        assert!(!settings.backtrace_enabled);
        assert_eq!(settings.max_retries, 3);
        assert!(settings.log_errors);
        assert_eq!(settings.log_level, LogLevel::Info);
    }

    #[test]
    fn test_error_settings_new() {
        // Test creating custom settings
        let settings = ErrorSettings::new(true, 5, false, LogLevel::Debug);
        assert!(settings.backtrace_enabled);
        assert_eq!(settings.max_retries, 5);
        assert!(!settings.log_errors);
        assert_eq!(settings.log_level, LogLevel::Debug);
    }
}