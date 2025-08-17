//! Error categorization and classification system.
//!
//! This module provides the extensible error category system that allows both
//! built-in error classifications and custom user-defined categories.

use serde::{Deserialize, Serialize};
use std::time::Duration;

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

/// Default classifier for deserialization fallback.
pub fn default_classifier() -> Box<dyn ErrorClassifier> {
    Box::new(BuiltinCategory::Unknown)
}

/// Built-in error categories provided by tyl-errors.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BuiltinCategory {
    /// Temporary failures that should be retried (database timeouts, etc.).
    Transient,
    /// Permanent failures that should not be retried (validation errors, etc.).
    Permanent,
    /// Resource exhaustion that may be retried after longer delays.
    ResourceExhaustion,
    /// Network-related errors with specific retry behavior.
    Network,
    /// Authentication and authorization failures.
    Authentication,
    /// Input validation and parsing errors.
    Validation,
    /// Internal system errors.
    Internal,
    /// Service unavailable (503-style errors).
    ServiceUnavailable,
    /// Unknown error category (fallback).
    Unknown,
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

impl ErrorCategory {
    // === Built-in Category Constructors ===

    /// Create a transient error category (retriable with short delays).
    pub fn transient() -> Self {
        Self::Builtin(BuiltinCategory::Transient)
    }

    /// Create a permanent error category (not retriable).
    pub fn permanent() -> Self {
        Self::Builtin(BuiltinCategory::Permanent)
    }

    /// Create a resource exhaustion category (retriable with longer delays).
    pub fn resource_exhaustion() -> Self {
        Self::Builtin(BuiltinCategory::ResourceExhaustion)
    }

    /// Create a network error category (retriable with exponential backoff).
    pub fn network() -> Self {
        Self::Builtin(BuiltinCategory::Network)
    }

    /// Create an authentication error category (not retriable).
    pub fn authentication() -> Self {
        Self::Builtin(BuiltinCategory::Authentication)
    }

    /// Create a validation error category (not retriable).
    pub fn validation() -> Self {
        Self::Builtin(BuiltinCategory::Validation)
    }

    /// Create an internal error category (not retriable).
    pub fn internal() -> Self {
        Self::Builtin(BuiltinCategory::Internal)
    }

    /// Create a service unavailable category (retriable with longer delays).
    pub fn service_unavailable() -> Self {
        Self::Builtin(BuiltinCategory::ServiceUnavailable)
    }

    /// Create an unknown error category (not retriable).
    pub fn unknown() -> Self {
        Self::Builtin(BuiltinCategory::Unknown)
    }

    // === Delegation Methods ===

    /// Check if this error category supports retries.
    pub fn is_retriable(&self) -> bool {
        match self {
            ErrorCategory::Builtin(builtin) => builtin.is_retriable(),
            ErrorCategory::Custom(custom) => custom.is_retriable(),
        }
    }

    /// Calculate the suggested retry delay for this category and attempt number.
    pub fn retry_delay(&self, attempt: usize) -> Duration {
        match self {
            ErrorCategory::Builtin(builtin) => builtin.retry_delay(attempt),
            ErrorCategory::Custom(custom) => custom.retry_delay(attempt),
        }
    }

    /// Get the human-readable name of this error category.
    pub fn category_name(&self) -> &str {
        match self {
            ErrorCategory::Builtin(builtin) => builtin.category_name(),
            ErrorCategory::Custom(custom) => custom.category_name(),
        }
    }
}