//! # TYL Errors
//!
//! A comprehensive error handling library for the TYL (The YAML Life) framework.
//!
//! This crate provides a unified error type system with built-in retry logic, error classification,
//! and rich context information for debugging and monitoring in hexagonal architecture patterns.
//!
//! ## Features
//!
//! - **Type Safety**: Strongly typed errors with clear semantics
//! - **Error Classification**: Automatic categorization for retry decisions
//! - **Retry Logic**: Built-in exponential backoff calculations
//! - **Rich Context**: Error tracking with UUIDs, timestamps, and metadata
//! - **Extensible**: Custom error categories without modifying core library
//! - **Serialization**: Full serde support for all error types
//! - **Zero Configuration**: Works out of the box with sensible defaults
//!
//! ## Environment Variables
//!
//! tyl-errors supports optional environment variables for runtime behavior:
//!
//! | Variable | Default | Description |
//! |----------|---------|-------------|
//! | `TYL_ERROR_BACKTRACE` | `false` | Enable error backtraces (`true`/`false`) |
//! | `TYL_ERROR_MAX_RETRIES` | `3` | Maximum retry attempts for retriable errors |
//! | `TYL_ERROR_LOG_ERRORS` | `true` | Log errors to stderr (`true`/`false`) |
//! | `TYL_ERROR_LOG_LEVEL` | `INFO` | Log level (`ERROR`/`WARN`/`INFO`/`DEBUG`) |
//! | `RUST_BACKTRACE` | - | Standard Rust backtrace (overrides TYL_ERROR_BACKTRACE) |
//!
//! **Example:**
//! ```bash
//! export TYL_ERROR_BACKTRACE=true
//! export TYL_ERROR_MAX_RETRIES=5
//! export TYL_ERROR_LOG_ERRORS=false
//! export TYL_ERROR_LOG_LEVEL=DEBUG
//! cargo run
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use tyl_errors::{TylError, TylResult};
//!
//! fn validate_email(email: &str) -> TylResult<()> {
//!     if !email.contains('@') {
//!         return Err(TylError::validation("email", "Must contain @ symbol"));
//!     }
//!     Ok(())
//! }
//!
//! // Error classification and retry logic with environment configuration
//! let error = TylError::network("Connection timeout");
//!
//! for attempt in 0..TylError::max_retries() {
//!     if error.should_retry(attempt) {
//!         let delay = error.category().retry_delay(attempt + 1);
//!         // Sleep for delay duration, then retry...
//!         break;
//!     }
//! }
//!
//! // Conditional logging based on environment configuration
//! error.log_if_enabled(tyl_errors::LogLevel::Error);
//!
//! // Manual logging check
//! if TylError::log_errors_enabled() && TylError::log_level() >= tyl_errors::LogLevel::Info {
//!     eprintln!("Error occurred: {}", error);
//! }
//! ```
//!
//! ## Custom Error Categories
//!
//! Create domain-specific error categories:
//!
//! ```rust
//! use tyl_errors::{TylError, ErrorClassifier};
//! use std::time::Duration;
//!
//! #[derive(Debug, Clone)]
//! struct PaymentError;
//!
//! impl ErrorClassifier for PaymentError {
//!     fn is_retriable(&self) -> bool { true }
//!     fn retry_delay(&self, attempt: usize) -> Duration {
//!         Duration::from_secs(attempt as u64 * 2)
//!     }
//!     fn category_name(&self) -> &'static str { "Payment" }
//!     fn clone_box(&self) -> Box<dyn ErrorClassifier> {
//!         Box::new(self.clone())
//!     }
//! }
//!
//! let error = TylError::business_logic("Card declined", Box::new(PaymentError));
//! assert!(error.category().is_retriable());
//! ```

// Module declarations
mod category;
mod context;
mod error;
mod retry;
mod settings;

// Re-export main types and traits
pub use error::{TylError, TylResult};
pub use category::{ErrorCategory, ErrorClassifier, BuiltinCategory};
pub use context::ErrorContext;
pub use retry::{RetryableError, RetryPolicy, RetryResult};
pub use settings::{ErrorSettings, LogLevel};

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_tyl_error_creation_should_create_different_error_types() {
        // Given: different error creation methods
        // When: creating various error types
        let db_error = TylError::database("Connection failed");
        let validation_error = TylError::validation("email", "Invalid format");
        let not_found_error = TylError::not_found("user", "123");

        // Then: errors should match expected types
        assert!(matches!(db_error, TylError::Database { .. }));
        assert!(matches!(validation_error, TylError::Validation { .. }));
        assert!(matches!(not_found_error, TylError::NotFound { .. }));
    }

    #[test]
    fn test_error_display_should_format_correctly() {
        // Given: a validation error with field and message
        // When: converting to string
        let error = TylError::validation("email", "Must be valid email address");

        // Then: should format as expected
        assert_eq!(
            error.to_string(),
            "Validation error: email: Must be valid email address"
        );
    }

    #[test]
    fn test_error_serialization_should_preserve_data() {
        // Given: a not found error
        // When: serializing and deserializing
        let error = TylError::not_found("memory", "abc-123");
        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: TylError = serde_json::from_str(&serialized).unwrap();

        // Then: data should be preserved
        match (error, deserialized) {
            (
                TylError::NotFound {
                    resource: r1,
                    id: i1,
                },
                TylError::NotFound {
                    resource: r2,
                    id: i2,
                },
            ) => {
                assert_eq!(r1, r2);
                assert_eq!(i1, i2);
            }
            _ => panic!("Serialization failed"),
        }
    }

    #[test]
    fn test_error_categorization_should_classify_correctly() {
        // Given: different error types
        // When: getting their categories
        // Then: should match expected categories
        assert_eq!(
            TylError::database("test").category().category_name(),
            "Transient"
        );
        assert_eq!(
            TylError::network("test").category().category_name(),
            "Network"
        );
        assert_eq!(
            TylError::validation("field", "test")
                .category()
                .category_name(),
            "Validation"
        );
        assert_eq!(
            TylError::not_found("resource", "id")
                .category()
                .category_name(),
            "Permanent"
        );
    }

    #[test]
    fn test_retry_logic_should_identify_retriable_errors() {
        // Given: different error categories
        // When: checking if retriable
        // Then: should match expected behavior
        assert!(ErrorCategory::transient().is_retriable());
        assert!(ErrorCategory::network().is_retriable());
        assert!(!ErrorCategory::permanent().is_retriable());
        assert!(!ErrorCategory::validation().is_retriable());
    }

    #[test]
    fn test_retry_delays_should_use_exponential_backoff() {
        // Given: network error category
        let network = ErrorCategory::network();

        // When: calculating retry delays
        let delay1 = network.retry_delay(1);
        let delay2 = network.retry_delay(2);
        let delay3 = network.retry_delay(3);

        // Then: should use exponential backoff
        assert!(delay2 > delay1);
        assert!(delay3 > delay2);

        // And: should cap at reasonable maximum
        let delay_high = network.retry_delay(20);
        assert!(delay_high.as_secs() <= 60 * 500 / 1000);
    }

    #[test]
    fn test_error_context_should_track_operations() {
        // Given: a network error
        // When: converting to context
        let error = TylError::network("Connection failed");
        let context = error.to_context("test_operation".to_string());

        // Then: should preserve operation details
        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.category.category_name(), "Network");
        assert_eq!(context.attempt_count, 1);
        assert!(context.error_id != uuid::Uuid::nil());
    }

    #[test]
    fn test_error_context_metadata_should_support_builder_pattern() {
        // Given: error context creation
        // When: adding metadata with builder pattern
        let context = ErrorContext::new(
            "api_call".to_string(),
            ErrorCategory::network(),
            "Timeout".to_string(),
        )
        .with_metadata("endpoint".to_string(), serde_json::json!("/api/users"))
        .with_metadata("timeout_ms".to_string(), serde_json::json!(5000));

        // Then: metadata should be added correctly
        assert_eq!(context.metadata.len(), 2);
        assert_eq!(
            context.metadata["endpoint"],
            serde_json::json!("/api/users")
        );
        assert_eq!(context.metadata["timeout_ms"], serde_json::json!(5000));
    }

    #[test]
    fn test_error_context_retry_tracking_should_increment() {
        // Given: error context
        let mut context = ErrorContext::new(
            "retry_operation".to_string(),
            ErrorCategory::network(),
            "Connection failed".to_string(),
        );

        // When: incrementing attempts
        assert_eq!(context.attempt_count, 1);
        context.increment_attempt();

        // Then: count should increment
        assert_eq!(context.attempt_count, 2);
    }

    #[test]
    fn test_custom_error_category_should_be_extensible() {
        // Given: a custom domain-specific error category
        use std::time::Duration;

        #[derive(Debug, Clone)]
        struct PaymentProcessingError;

        impl ErrorClassifier for PaymentProcessingError {
            fn is_retriable(&self) -> bool {
                true
            }
            fn retry_delay(&self, attempt: usize) -> Duration {
                Duration::from_secs(attempt as u64 * 2) // Custom backoff
            }
            fn category_name(&self) -> &'static str {
                "PaymentProcessing"
            }
            fn clone_box(&self) -> Box<dyn ErrorClassifier> {
                Box::new(self.clone())
            }
        }

        // When: creating custom category
        let custom_category = ErrorCategory::Custom(Box::new(PaymentProcessingError));

        // Then: should behave according to custom logic
        assert!(custom_category.is_retriable());
        assert_eq!(custom_category.retry_delay(1), Duration::from_secs(2));
        assert_eq!(custom_category.retry_delay(2), Duration::from_secs(4));
        assert_eq!(custom_category.category_name(), "PaymentProcessing");
    }

    #[test]
    fn test_builtin_categories_should_work_as_before() {
        // Given: builtin categories
        let network = ErrorCategory::Builtin(BuiltinCategory::Network);
        let validation = ErrorCategory::Builtin(BuiltinCategory::Validation);

        // When/Then: should maintain existing behavior
        assert!(network.is_retriable());
        assert!(!validation.is_retriable());
        assert_eq!(network.category_name(), "Network");
        assert_eq!(validation.category_name(), "Validation");
    }

    #[test]
    fn test_tyl_error_with_custom_categories_should_work() {
        // Given: TylError that can return custom categories
        #[derive(Debug, Clone)]
        struct BusinessLogicError;

        impl ErrorClassifier for BusinessLogicError {
            fn is_retriable(&self) -> bool {
                false
            }
            fn retry_delay(&self, _attempt: usize) -> Duration {
                Duration::from_millis(0)
            }
            fn category_name(&self) -> &'static str {
                "BusinessLogic"
            }
            fn clone_box(&self) -> Box<dyn ErrorClassifier> {
                Box::new(self.clone())
            }
        }

        let error =
            TylError::business_logic("Invalid state transition", Box::new(BusinessLogicError));

        // When: getting category
        let category = error.category();

        // Then: should use custom logic
        assert!(!category.is_retriable());
        assert_eq!(category.category_name(), "BusinessLogic");
    }
}

