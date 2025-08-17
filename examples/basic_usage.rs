use tyl_errors::{TylError, TylResult};

fn main() {
    println!("TYL Errors - Basic Usage Examples");

    basic_error_creation();

    match validate_email("invalid-email") {
        Ok(()) => println!("Email is valid"),
        Err(e) => println!("Validation error: {}", e),
    }

    match validate_email("user@example.com") {
        Ok(()) => println!("Email is valid"),
        Err(e) => println!("Validation error: {}", e),
    }

    error_categorization_example();
}

fn basic_error_creation() {
    println!("\n=== Basic Error Creation ===");

    let db_error = TylError::database("Connection timeout");
    println!("Database error: {}", db_error);

    let validation_error = TylError::validation("email", "Invalid email format");
    println!("Validation error: {}", validation_error);

    let not_found_error = TylError::not_found("user", "123");
    println!("Not found error: {}", not_found_error);

    let network_error = TylError::network("Connection refused");
    println!("Network error: {}", network_error);
}

fn validate_email(email: &str) -> TylResult<()> {
    if email.is_empty() {
        return Err(TylError::validation("email", "Email cannot be empty"));
    }

    if !email.contains('@') {
        return Err(TylError::validation("email", "Must contain @ symbol"));
    }

    if !email.contains('.') {
        return Err(TylError::validation("email", "Must contain domain"));
    }

    Ok(())
}

fn error_categorization_example() {
    println!("\n=== Error Categorization ===");

    let errors = vec![
        TylError::database("Connection failed"),
        TylError::network("Timeout"),
        TylError::validation("field", "Invalid"),
        TylError::not_found("resource", "id"),
        TylError::internal("System error"),
    ];

    for error in errors {
        let category = error.category();
        println!(
            "Error: {} | Category: {:?} | Retriable: {}",
            error,
            category,
            category.is_retriable()
        );
    }
}
