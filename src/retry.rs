//! Retry policies and utilities for error handling.
//!
//! This module provides abstractions for retry logic, policies, and utilities
//! for implementing robust retry mechanisms in error-prone operations.

use crate::category::ErrorCategory;
use crate::error::TylError;
use std::time::Duration;

/// Trait for errors that support retry logic.
///
/// This trait provides a standard interface for determining if an error
/// should be retried and calculating appropriate delays.
pub trait RetryableError {
    /// Check if this error should be retried for the given attempt number.
    fn should_retry(&self, attempt: usize) -> bool;

    /// Get the suggested delay before the next retry attempt.
    fn retry_delay(&self, attempt: usize) -> Duration;

    /// Get the maximum number of retry attempts allowed.
    fn max_retries(&self) -> usize;
}

/// Implementation of RetryableError for TylError.
impl RetryableError for TylError {
    fn should_retry(&self, attempt: usize) -> bool {
        self.should_retry(attempt)
    }

    fn retry_delay(&self, attempt: usize) -> Duration {
        self.category().retry_delay(attempt)
    }

    fn max_retries(&self) -> usize {
        TylError::max_retries()
    }
}

/// Configurable retry policy for operations.
///
/// Provides a flexible way to define retry behavior that can be customized
/// per operation or error type.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts.
    pub max_attempts: usize,
    /// Base delay for exponential backoff.
    pub base_delay: Duration,
    /// Maximum delay cap to prevent excessive wait times.
    pub max_delay: Duration,
    /// Multiplier for exponential backoff.
    pub backoff_multiplier: f64,
    /// Whether to add jitter to delays.
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of retry attempts.
    pub fn with_max_attempts(mut self, max_attempts: usize) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Set the base delay for exponential backoff.
    pub fn with_base_delay(mut self, base_delay: Duration) -> Self {
        self.base_delay = base_delay;
        self
    }

    /// Set the maximum delay cap.
    pub fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }

    /// Set the backoff multiplier.
    pub fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// Enable or disable jitter.
    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }

    /// Calculate the delay for a given attempt number.
    ///
    /// # Arguments
    /// * `attempt` - The attempt number (1-based)
    ///
    /// # Returns
    /// The calculated delay duration.
    pub fn calculate_delay(&self, attempt: usize) -> Duration {
        if attempt == 0 {
            return Duration::from_millis(0);
        }

        let exponential_delay = self.base_delay.as_millis() as f64
            * self.backoff_multiplier.powi((attempt - 1) as i32);

        let mut delay = Duration::from_millis(exponential_delay as u64);

        // Apply maximum delay cap
        if delay > self.max_delay {
            delay = self.max_delay;
        }

        // Apply jitter if enabled
        if self.jitter {
            delay = self.add_jitter(delay);
        }

        delay
    }

    /// Check if a retry should be attempted for the given attempt number.
    ///
    /// # Arguments
    /// * `attempt` - The current attempt number (0-based)
    ///
    /// # Returns
    /// True if retry should be attempted.
    pub fn should_retry(&self, attempt: usize) -> bool {
        attempt < self.max_attempts
    }

    /// Add jitter to a delay duration.
    ///
    /// Adds up to ±25% jitter to prevent thundering herd problems.
    fn add_jitter(&self, delay: Duration) -> Duration {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        std::thread::current().id().hash(&mut hasher);
        std::time::SystemTime::now().hash(&mut hasher);

        let hash = hasher.finish();
        let jitter_factor = (hash % 50) as f64 / 100.0 + 0.75; // 0.75 to 1.25

        let jittered_millis = (delay.as_millis() as f64 * jitter_factor) as u64;
        Duration::from_millis(jittered_millis)
    }
}

/// Predefined retry policies for common scenarios.
impl RetryPolicy {
    /// Fast retry policy for quick operations.
    pub fn fast() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 1.5,
            jitter: true,
        }
    }

    /// Standard retry policy for most operations.
    pub fn standard() -> Self {
        Self::default()
    }

    /// Slow retry policy for expensive operations.
    pub fn slow() -> Self {
        Self {
            max_attempts: 5,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }

    /// Network retry policy optimized for network operations.
    pub fn network() -> Self {
        Self {
            max_attempts: 4,
            base_delay: Duration::from_millis(250),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }

    /// Database retry policy optimized for database operations.
    pub fn database() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Retry result indicating the outcome of a retry operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryResult<T, E> {
    /// Operation succeeded with the given value.
    Success(T),
    /// Operation failed but should be retried.
    Retry(E),
    /// Operation failed and should not be retried.
    Failed(E),
}

/// Utility function to determine if an error category is retriable.
///
/// # Arguments
/// * `category` - The error category to check
///
/// # Returns
/// True if the category supports retries.
#[allow(dead_code)]
pub fn is_retriable(category: &ErrorCategory) -> bool {
    category.is_retriable()
}

/// Utility function to calculate retry delay for an error category.
///
/// # Arguments
/// * `category` - The error category
/// * `attempt` - The attempt number (1-based)
///
/// # Returns
/// The calculated delay duration.
#[allow(dead_code)]
pub fn calculate_retry_delay(category: &ErrorCategory, attempt: usize) -> Duration {
    category.retry_delay(attempt)
}