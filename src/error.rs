//! Core TylError type and constructor methods.
//!
//! This module defines the main TylError enum that represents all error types
//! in the TYL framework, along with convenient constructor methods.

use crate::category::{default_classifier, ErrorCategory, ErrorClassifier};
use crate::context::ErrorContext;
use crate::settings::ErrorSettings;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type alias for TYL framework operations.
pub type TylResult<T> = Result<T, TylError>;

/// Main error type for the TYL framework.
///
/// Provides a comprehensive set of error variants covering common error scenarios
/// in hexagonal architecture patterns, with built-in retry logic and error classification.
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum TylError {
    #[error("Database error: {message}")]
    Database { message: String },

    #[error("Network error: {message}")]
    Network { message: String },

    #[error("Validation error: {field}: {message}")]
    Validation { field: String, message: String },

    #[error("Not found: {resource} with id {id}")]
    NotFound { resource: String, id: String },

    #[error("Conflict: {message}")]
    Conflict { message: String },

    #[error("Internal error: {message}")]
    Internal { message: String },

    #[error("Configuration error: {message}")]
    Configuration { message: String },

    #[error("Feature not implemented: {feature}")]
    NotImplemented { feature: String },

    #[error("Custom error: {message}")]
    Custom {
        message: String,
        #[serde(skip)]
        #[serde(default = "default_classifier")]
        classifier: Box<dyn ErrorClassifier>,
    },
}

impl TylError {
    // === Primary Constructors ===

    /// Create a database-related error.
    pub fn database<S: Into<String>>(message: S) -> Self {
        Self::Database {
            message: message.into(),
        }
    }

    /// Create a network-related error.
    pub fn network<S: Into<String>>(message: S) -> Self {
        Self::Network {
            message: message.into(),
        }
    }

    /// Create a validation error for a specific field.
    pub fn validation<F: Into<String>, M: Into<String>>(field: F, message: M) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a "not found" error for a specific resource.
    pub fn not_found<R: Into<String>, I: Into<String>>(resource: R, id: I) -> Self {
        Self::NotFound {
            resource: resource.into(),
            id: id.into(),
        }
    }

    /// Create a conflict error (e.g., duplicate resources, constraint violations).
    pub fn conflict<S: Into<String>>(message: S) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }

    /// Create an internal system error.
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Create a configuration error.
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Create a "not implemented" error for missing features.
    pub fn not_implemented<S: Into<String>>(feature: S) -> Self {
        Self::NotImplemented {
            feature: feature.into(),
        }
    }

    /// Create a custom error with domain-specific classification.
    pub fn business_logic<S: Into<String>>(
        message: S,
        classifier: Box<dyn ErrorClassifier>,
    ) -> Self {
        Self::Custom {
            message: message.into(),
            classifier,
        }
    }

    // === Convenience Constructors ===

    /// Create a parsing error (specialized validation error).
    pub fn parsing<S: Into<String>>(message: S) -> Self {
        Self::Validation {
            field: "parsing".to_string(),
            message: message.into(),
        }
    }

    /// Create a serialization error (specialized internal error).
    pub fn serialization<S: Into<String>>(message: S) -> Self {
        let msg = message.into();
        Self::Internal {
            message: format!("Serialization error: {msg}"),
        }
    }

    /// Create a connection error (specialized network error).
    pub fn connection<S: Into<String>>(message: S) -> Self {
        let msg = message.into();
        Self::Network {
            message: format!("Connection error: {msg}"),
        }
    }

    /// Create an initialization error (specialized internal error).
    pub fn initialization<S: Into<String>>(message: S) -> Self {
        let msg = message.into();
        Self::Internal {
            message: format!("Initialization error: {msg}"),
        }
    }

    // === Error Category and Classification ===

    /// Get the error category for this error type.
    pub fn category(&self) -> ErrorCategory {
        match self {
            TylError::Database { .. } => ErrorCategory::transient(),
            TylError::Network { .. } => ErrorCategory::network(),
            TylError::Validation { .. } => ErrorCategory::validation(),
            TylError::NotFound { .. } => ErrorCategory::permanent(),
            TylError::Conflict { .. } => ErrorCategory::permanent(),
            TylError::Internal { .. } => ErrorCategory::internal(),
            TylError::Configuration { .. } => ErrorCategory::permanent(),
            TylError::NotImplemented { .. } => ErrorCategory::permanent(),
            TylError::Custom { classifier, .. } => ErrorCategory::Custom(classifier.clone()),
        }
    }

    /// Convert this error to an ErrorContext for tracking operations.
    pub fn to_context(&self, operation: String) -> ErrorContext {
        ErrorContext::new(operation, self.category(), self.to_string())
    }

    // === Environment-based Configuration ===

    /// Check if backtraces are enabled via environment variables.
    ///
    /// Checks TYL_ERROR_BACKTRACE first, falls back to RUST_BACKTRACE.
    pub fn backtrace_enabled() -> bool {
        ErrorSettings::global().backtrace_enabled
    }

    /// Get maximum retry attempts from TYL_ERROR_MAX_RETRIES (default: 3).
    pub fn max_retries() -> usize {
        ErrorSettings::global().max_retries
    }

    /// Check if error logging is enabled via TYL_ERROR_LOG_ERRORS (default: true).
    pub fn log_errors_enabled() -> bool {
        ErrorSettings::global().log_errors
    }

    /// Get current log level from TYL_ERROR_LOG_LEVEL (default: INFO).
    pub fn log_level() -> crate::settings::LogLevel {
        ErrorSettings::global().log_level
    }

    /// Check if this error should be retried based on attempt count and max retries.
    pub fn should_retry(&self, attempt: usize) -> bool {
        self.category().is_retriable() && attempt < Self::max_retries()
    }

    /// Log error if logging is enabled and meets log level criteria.
    pub fn log_if_enabled(&self, level: crate::settings::LogLevel) {
        if Self::log_errors_enabled() && level <= Self::log_level() {
            eprintln!(
                "[{}] {}",
                match level {
                    crate::settings::LogLevel::Error => "ERROR",
                    crate::settings::LogLevel::Warn => "WARN",
                    crate::settings::LogLevel::Info => "INFO",
                    crate::settings::LogLevel::Debug => "DEBUG",
                },
                self
            );
        }
    }
}

// === Standard Library Integrations ===

/// Convert serde_json errors to TylError.
impl From<serde_json::Error> for TylError {
    fn from(err: serde_json::Error) -> Self {
        Self::Internal {
            message: format!("JSON serialization error: {err}"),
        }
    }
}
