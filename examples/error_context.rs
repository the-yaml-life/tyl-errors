use serde_json::json;
use tyl_errors::{ErrorCategory, ErrorContext, TylError};

fn main() {
    println!("TYL Errors - Error Context Example");

    basic_context_example();
    context_with_metadata_example();
    error_to_context_example();
}

fn basic_context_example() {
    println!("\n=== Basic Error Context ===");

    let context = ErrorContext::new(
        "fetch_user_data".to_string(),
        ErrorCategory::transient(),
        "Connection failed to primary database".to_string(),
    );

    println!("Error ID: {}", context.error_id);
    println!("Operation: {}", context.operation);
    println!("Category: {}", context.category.category_name());
    println!("Message: {}", context.message);
    println!("Occurred at: {}", context.occurred_at);
    println!("Attempt count: {}", context.attempt_count);
    println!("Metadata: {:?}", context.metadata);
}

fn context_with_metadata_example() {
    println!("\n=== Error Context with Metadata ===");

    let mut context = ErrorContext::new(
        "search_data".to_string(),
        ErrorCategory::network(),
        "API request timeout".to_string(),
    )
    .with_metadata("query".to_string(), json!("search term"))
    .with_metadata("user_id".to_string(), json!("user-123"))
    .with_metadata("timeout_ms".to_string(), json!(5000))
    .with_metadata("endpoint".to_string(), json!("/api/v1/search"));

    println!("Operation: {}", context.operation);
    println!("Message: {}", context.message);
    println!("Metadata:");
    for (key, value) in &context.metadata {
        println!("  {key}: {value}");
    }

    // Simulate retry
    context.increment_attempt();
    println!("After retry - Attempt count: {}", context.attempt_count);

    context.increment_attempt();
    println!(
        "After another retry - Attempt count: {}",
        context.attempt_count
    );
}

fn error_to_context_example() {
    println!("\n=== Converting Errors to Context ===");

    let errors = vec![
        TylError::database("Connection pool exhausted"),
        TylError::validation("email", "Invalid email format"),
        TylError::not_found("user", "user-456"),
        TylError::network("Request timeout"),
    ];

    for (i, error) in errors.iter().enumerate() {
        let operation = format!("operation_{}", i + 1);
        let context = error.to_context(operation.clone());

        println!("\nOriginal error: {error}");
        println!("Context operation: {}", context.operation);
        println!("Context category: {}", context.category.category_name());
        println!("Context error_id: {}", context.error_id);
        println!("Is retriable: {}", context.category.is_retriable());
    }
}
