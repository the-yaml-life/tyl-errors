use std::time::Duration;
use tyl_errors::{ErrorCategory, ErrorClassifier, TylError, TylResult};

// Custom domain-specific error categories
#[derive(Debug, Clone)]
struct PaymentProcessingError;

impl ErrorClassifier for PaymentProcessingError {
    fn is_retriable(&self) -> bool {
        true // Payment failures might be temporary
    }

    fn retry_delay(&self, attempt: usize) -> Duration {
        // Custom backoff for payment processing: longer delays
        Duration::from_secs(attempt as u64 * 5)
    }

    fn category_name(&self) -> &'static str {
        "PaymentProcessing"
    }

    fn clone_box(&self) -> Box<dyn ErrorClassifier> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
struct BusinessRuleViolation;

impl ErrorClassifier for BusinessRuleViolation {
    fn is_retriable(&self) -> bool {
        false // Business rule violations are permanent
    }

    fn retry_delay(&self, _attempt: usize) -> Duration {
        Duration::from_millis(0) // No retry
    }

    fn category_name(&self) -> &'static str {
        "BusinessRuleViolation"
    }

    fn clone_box(&self) -> Box<dyn ErrorClassifier> {
        Box::new(self.clone())
    }
}

fn main() {
    println!("TYL Errors - Extensible Categories Example");

    // Example 1: Using built-in categories
    builtin_categories_example();

    // Example 2: Using custom categories
    custom_categories_example();

    // Example 3: Domain-specific error handling
    domain_specific_example();
}

fn builtin_categories_example() {
    println!("\n=== Built-in Categories ===");

    let errors = vec![
        TylError::database("Connection timeout"),
        TylError::network("Service unavailable"),
        TylError::validation("email", "Invalid format"),
        TylError::not_found("user", "123"),
    ];

    for error in errors {
        let category = error.category();
        println!(
            "Error: {} | Category: {} | Retriable: {} | Delay: {:?}",
            error,
            category.category_name(),
            category.is_retriable(),
            category.retry_delay(1)
        );
    }
}

fn custom_categories_example() {
    println!("\n=== Custom Categories ===");

    // Payment processing error with custom retry logic
    let payment_error =
        TylError::business_logic("Credit card declined", Box::new(PaymentProcessingError));

    // Business rule violation - not retriable
    let business_error = TylError::business_logic(
        "Account balance insufficient for premium features",
        Box::new(BusinessRuleViolation),
    );

    let custom_errors = vec![payment_error, business_error];

    for error in custom_errors {
        let category = error.category();
        println!(
            "Error: {} | Category: {} | Retriable: {} | Delay: {:?}",
            error,
            category.category_name(),
            category.is_retriable(),
            category.retry_delay(1)
        );
    }
}

fn process_payment(amount: f64) -> TylResult<String> {
    if amount <= 0.0 {
        return Err(TylError::business_logic(
            "Amount must be positive",
            Box::new(BusinessRuleViolation),
        ));
    }

    if amount > 10000.0 {
        return Err(TylError::business_logic(
            "Amount exceeds daily limit",
            Box::new(BusinessRuleViolation),
        ));
    }

    // Simulate payment gateway failure
    if amount == 99.99 {
        return Err(TylError::business_logic(
            "Payment gateway temporarily unavailable",
            Box::new(PaymentProcessingError),
        ));
    }

    Ok(format!("Payment of ${:.2} processed successfully", amount))
}

fn domain_specific_example() {
    println!("\n=== Domain-specific Error Handling ===");

    let test_amounts = vec![-10.0, 50.0, 99.99, 15000.0, 100.0];

    for amount in test_amounts {
        match process_payment(amount) {
            Ok(result) => println!("✓ {}", result),
            Err(error) => {
                let category = error.category();
                println!(
                    "✗ Payment ${:.2} failed: {} [{}{}]",
                    amount,
                    error,
                    category.category_name(),
                    if category.is_retriable() {
                        " - Can retry"
                    } else {
                        " - Permanent"
                    }
                );

                if category.is_retriable() {
                    println!("  → Suggested retry delay: {:?}", category.retry_delay(1));
                }
            }
        }
    }
}
