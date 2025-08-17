use std::time::Duration;
use tyl_errors::{ErrorClassifier, TylError};

#[test]
fn test_end_to_end_error_handling() {
    // Test complete error flow from creation to retry decision
    let error = TylError::network("Connection timeout");
    let context = error.to_context("user_service_call".to_string());

    assert_eq!(context.operation, "user_service_call");
    assert!(context.category.is_retriable());
    assert!(context.category.retry_delay(1) > Duration::from_millis(0));
}

#[test]
fn test_serialization_roundtrip() {
    let error = TylError::validation("email", "Invalid format");
    let json = serde_json::to_string(&error).unwrap();
    let deserialized: TylError = serde_json::from_str(&json).unwrap();

    assert_eq!(error.to_string(), deserialized.to_string());
}

#[test]
fn test_custom_error_category_integration() {
    #[derive(Debug, Clone)]
    struct TestCategory;

    impl ErrorClassifier for TestCategory {
        fn is_retriable(&self) -> bool {
            true
        }
        fn retry_delay(&self, attempt: usize) -> Duration {
            Duration::from_millis(attempt as u64 * 100)
        }
        fn category_name(&self) -> &'static str {
            "Test"
        }
        fn clone_box(&self) -> Box<dyn ErrorClassifier> {
            Box::new(self.clone())
        }
    }

    let error = TylError::business_logic("Test error", Box::new(TestCategory));
    let category = error.category();

    assert!(category.is_retriable());
    assert_eq!(category.category_name(), "Test");
    assert_eq!(category.retry_delay(3), Duration::from_millis(300));
}
