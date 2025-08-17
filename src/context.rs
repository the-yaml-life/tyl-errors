//! Error context tracking and metadata management.
//!
//! This module provides the ErrorContext struct for tracking error operations,
//! retry attempts, and associated metadata for debugging and monitoring.

use crate::category::ErrorCategory;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Context information for error tracking and monitoring.
///
/// Provides rich metadata about error occurrences including operation context,
/// retry tracking, and arbitrary metadata for debugging and monitoring systems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Unique identifier for this error occurrence.
    pub error_id: Uuid,
    /// Name of the operation that failed.
    pub operation: String,
    /// Error category for classification and retry decisions.
    #[serde(skip)]
    #[serde(default = "ErrorCategory::unknown")]
    pub category: ErrorCategory,
    /// Human-readable error message.
    pub message: String,
    /// Timestamp when the error occurred.
    pub occurred_at: DateTime<Utc>,
    /// Number of attempts for this operation (starts at 1).
    pub attempt_count: usize,
    /// Additional metadata for debugging and monitoring.
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ErrorContext {
    /// Create a new error context.
    ///
    /// # Arguments
    /// * `operation` - Name of the operation that failed
    /// * `category` - Error category for classification
    /// * `message` - Human-readable error message
    ///
    /// # Returns
    /// A new ErrorContext with generated UUID, current timestamp, and empty metadata.
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

    /// Add metadata to this error context using builder pattern.
    ///
    /// # Arguments
    /// * `key` - Metadata key
    /// * `value` - Serializable value to store
    ///
    /// # Returns
    /// Self for method chaining.
    ///
    /// # Example
    /// ```rust
    /// use tyl_errors::{ErrorContext, ErrorCategory};
    ///
    /// let context = ErrorContext::new(
    ///     "api_call".to_string(),
    ///     ErrorCategory::network(),
    ///     "Timeout".to_string(),
    /// )
    /// .with_metadata("endpoint".to_string(), serde_json::json!("/api/users"))
    /// .with_metadata("timeout_ms".to_string(), serde_json::json!(5000));
    /// ```
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Increment the attempt count for retry tracking.
    ///
    /// This should be called each time an operation is retried to maintain
    /// accurate retry attempt tracking.
    pub fn increment_attempt(&mut self) {
        self.attempt_count += 1;
    }

    /// Add or update metadata entry.
    ///
    /// # Arguments
    /// * `key` - Metadata key
    /// * `value` - Serializable value to store
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    /// Get metadata value by key.
    ///
    /// # Arguments
    /// * `key` - Metadata key to look up
    ///
    /// # Returns
    /// Optional reference to the stored value.
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Check if this context has metadata for the given key.
    ///
    /// # Arguments
    /// * `key` - Metadata key to check
    ///
    /// # Returns
    /// True if the key exists in metadata.
    pub fn has_metadata(&self, key: &str) -> bool {
        self.metadata.contains_key(key)
    }

    /// Clear all metadata from this context.
    pub fn clear_metadata(&mut self) {
        self.metadata.clear();
    }

    /// Get the number of metadata entries.
    pub fn metadata_count(&self) -> usize {
        self.metadata.len()
    }
}
