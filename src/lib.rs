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
//! // Error classification and retry logic
//! let error = TylError::network("Connection timeout");
//! if error.category().is_retriable() {
//!     let delay = error.category().retry_delay(1);
//!     // Retry after delay...
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

#[cfg(test)]
mod tests {
    use super::*;

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
                TylError::NotFound { resource: r1, id: i1 },
                TylError::NotFound { resource: r2, id: i2 },
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
        assert_eq!(TylError::database("test").category().category_name(), "Transient");
        assert_eq!(TylError::network("test").category().category_name(), "Network");
        assert_eq!(TylError::validation("field", "test").category().category_name(), "Validation");
        assert_eq!(TylError::not_found("resource", "id").category().category_name(), "Permanent");
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
        assert_eq!(context.metadata["endpoint"], serde_json::json!("/api/users"));
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
            fn is_retriable(&self) -> bool { true }
            fn retry_delay(&self, attempt: usize) -> Duration {
                Duration::from_secs(attempt as u64 * 2) // Custom backoff
            }
            fn category_name(&self) -> &'static str { "PaymentProcessing" }
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
            fn is_retriable(&self) -> bool { false }
            fn retry_delay(&self, _attempt: usize) -> Duration { Duration::from_millis(0) }
            fn category_name(&self) -> &'static str { "BusinessLogic" }
            fn clone_box(&self) -> Box<dyn ErrorClassifier> {
                Box::new(self.clone())
            }
        }

        let error = TylError::business_logic("Invalid state transition", Box::new(BusinessLogicError));
        
        // When: getting category
        let category = error.category();
        
        // Then: should use custom logic
        assert!(!category.is_retriable());
        assert_eq!(category.category_name(), "BusinessLogic");
    }
}

// Implementation will go here - starting with failing tests (TDD red phase)
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

pub type TylResult<T> = Result<T, TylError>;

/// Trait for defining custom error classification behavior.
///
/// This trait allows users to define domain-specific error categories
/// without modifying the core tyl-errors module.
pub trait ErrorClassifier: std::fmt::Debug + Send + Sync {
    /// Determine if this error category should trigger retries.
    fn is_retriable(&self) -> bool;
    
    /// Calculate the suggested retry delay for this error category.
    fn retry_delay(&self, attempt: usize) -> Duration;
    
    /// Get a human-readable name for this error category.
    fn category_name(&self) -> &'static str;
    
    /// Clone this error classifier (needed for ErrorCategory cloning).
    fn clone_box(&self) -> Box<dyn ErrorClassifier>;
}

impl Clone for Box<dyn ErrorClassifier> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// Default classifier for deserialization fallback
fn default_classifier() -> Box<dyn ErrorClassifier> {
    Box::new(BuiltinCategory::Unknown)
}

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
        classifier: Box<dyn ErrorClassifier> 
    },
}

/// Built-in error categories provided by tyl-errors.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BuiltinCategory {
    Transient,
    Permanent,
    ResourceExhaustion,
    Network,
    Authentication,
    Validation,
    Internal,
    ServiceUnavailable,
    Unknown,
}

/// Extensible error category system.
///
/// Supports both built-in categories and custom user-defined categories.
#[derive(Debug, Clone)]
pub enum ErrorCategory {
    /// Built-in error categories with predefined behavior.
    Builtin(BuiltinCategory),
    /// Custom error categories defined by users.
    Custom(Box<dyn ErrorClassifier>),
}

impl ErrorClassifier for BuiltinCategory {
    fn is_retriable(&self) -> bool {
        matches!(
            self,
            BuiltinCategory::Transient
                | BuiltinCategory::Network
                | BuiltinCategory::ServiceUnavailable
                | BuiltinCategory::ResourceExhaustion
        )
    }
    
    fn retry_delay(&self, attempt: usize) -> Duration {
        let base_delay = match self {
            BuiltinCategory::Transient => Duration::from_millis(100),
            BuiltinCategory::Network => Duration::from_millis(500),
            BuiltinCategory::ServiceUnavailable => Duration::from_secs(1),
            BuiltinCategory::ResourceExhaustion => Duration::from_secs(5),
            _ => Duration::from_millis(100),
        };

        let multiplier = 2_u32.pow(attempt.min(10) as u32);
        base_delay * multiplier.min(60)
    }
    
    fn category_name(&self) -> &'static str {
        match self {
            BuiltinCategory::Transient => "Transient",
            BuiltinCategory::Permanent => "Permanent", 
            BuiltinCategory::ResourceExhaustion => "ResourceExhaustion",
            BuiltinCategory::Network => "Network",
            BuiltinCategory::Authentication => "Authentication",
            BuiltinCategory::Validation => "Validation",
            BuiltinCategory::Internal => "Internal",
            BuiltinCategory::ServiceUnavailable => "ServiceUnavailable",
            BuiltinCategory::Unknown => "Unknown",
        }
    }
    
    fn clone_box(&self) -> Box<dyn ErrorClassifier> {
        Box::new(self.clone())
    }
}

// Convenience constructors for built-in categories
impl ErrorCategory {
    pub fn transient() -> Self { Self::Builtin(BuiltinCategory::Transient) }
    pub fn permanent() -> Self { Self::Builtin(BuiltinCategory::Permanent) }
    pub fn resource_exhaustion() -> Self { Self::Builtin(BuiltinCategory::ResourceExhaustion) }
    pub fn network() -> Self { Self::Builtin(BuiltinCategory::Network) }
    pub fn authentication() -> Self { Self::Builtin(BuiltinCategory::Authentication) }
    pub fn validation() -> Self { Self::Builtin(BuiltinCategory::Validation) }
    pub fn internal() -> Self { Self::Builtin(BuiltinCategory::Internal) }
    pub fn service_unavailable() -> Self { Self::Builtin(BuiltinCategory::ServiceUnavailable) }
    pub fn unknown() -> Self { Self::Builtin(BuiltinCategory::Unknown) }
    
    // Delegate methods to the classifier
    pub fn is_retriable(&self) -> bool {
        match self {
            ErrorCategory::Builtin(builtin) => builtin.is_retriable(),
            ErrorCategory::Custom(custom) => custom.is_retriable(),
        }
    }
    
    pub fn retry_delay(&self, attempt: usize) -> Duration {
        match self {
            ErrorCategory::Builtin(builtin) => builtin.retry_delay(attempt),
            ErrorCategory::Custom(custom) => custom.retry_delay(attempt),
        }
    }
    
    pub fn category_name(&self) -> &str {
        match self {
            ErrorCategory::Builtin(builtin) => builtin.category_name(),
            ErrorCategory::Custom(custom) => custom.category_name(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub error_id: Uuid,
    pub operation: String,
    #[serde(skip)]
    #[serde(default = "ErrorCategory::unknown")]
    pub category: ErrorCategory,
    pub message: String,
    pub occurred_at: DateTime<Utc>,
    pub attempt_count: usize,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TylError {
    pub fn database<S: Into<String>>(message: S) -> Self {
        Self::Database {
            message: message.into(),
        }
    }
    
    pub fn network<S: Into<String>>(message: S) -> Self {
        Self::Network {
            message: message.into(),
        }
    }
    
    pub fn validation<F: Into<String>, M: Into<String>>(field: F, message: M) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }
    
    pub fn not_found<R: Into<String>, I: Into<String>>(resource: R, id: I) -> Self {
        Self::NotFound {
            resource: resource.into(),
            id: id.into(),
        }
    }
    
    pub fn conflict<S: Into<String>>(message: S) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }
    
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
    
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }
    
    pub fn not_implemented<S: Into<String>>(feature: S) -> Self {
        Self::NotImplemented {
            feature: feature.into(),
        }
    }
    
    pub fn business_logic<S: Into<String>>(message: S, classifier: Box<dyn ErrorClassifier>) -> Self {
        Self::Custom {
            message: message.into(),
            classifier,
        }
    }
    
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
    
    pub fn to_context(&self, operation: String) -> ErrorContext {
        ErrorContext::new(operation, self.category(), self.to_string())
    }
    
    // Convenience methods
    pub fn parsing<S: Into<String>>(message: S) -> Self {
        Self::Validation {
            field: "parsing".to_string(),
            message: message.into(),
        }
    }
    
    pub fn serialization<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: format!("Serialization error: {}", message.into()),
        }
    }
    
    pub fn connection<S: Into<String>>(message: S) -> Self {
        Self::Network {
            message: format!("Connection error: {}", message.into()),
        }
    }
    
    pub fn initialization<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: format!("Initialization error: {}", message.into()),
        }
    }
}

// From implementations for common error types
impl From<serde_json::Error> for TylError {
    fn from(err: serde_json::Error) -> Self {
        Self::Internal {
            message: format!("JSON serialization error: {}", err),
        }
    }
}


impl ErrorContext {
    pub fn new(operation: String, category: ErrorCategory, message: String) -> Self {
        Self {
            error_id: Uuid::new_v4(),
            operation,
            category,
            message,
            occurred_at: Utc::now(),
            attempt_count: 1,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    pub fn increment_attempt(&mut self) {
        self.attempt_count += 1;
    }
}