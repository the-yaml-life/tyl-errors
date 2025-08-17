use std::time::Duration;
use tyl_errors::{ErrorCategory, TylError, TylResult};

fn main() {
    println!("TYL Errors - Retry Logic Example");

    retry_delay_example();
    simulate_retry_logic();
}

fn retry_delay_example() {
    println!("\n=== Retry Delay Calculation ===");

    let categories = vec![
        ErrorCategory::transient(),
        ErrorCategory::network(),
        ErrorCategory::service_unavailable(),
        ErrorCategory::resource_exhaustion(),
    ];

    for category in categories {
        println!("\nCategory: {}", category.category_name());
        for attempt in 1..=5 {
            let delay = category.retry_delay(attempt);
            println!("  Attempt {attempt}: {delay:?}");
        }
    }
}

fn simulate_retry_logic() {
    println!("\n=== Simulated Retry Logic ===");

    let mut attempt_count = 0;
    let max_attempts = 4;

    for attempt in 1..=max_attempts {
        attempt_count += 1;

        match simulate_network_call(attempt_count) {
            Ok(result) => {
                println!("Success on attempt {attempt}: {result}");
                break;
            }
            Err(error) => {
                let category = error.category();

                if !category.is_retriable() {
                    println!("Non-retriable error: {error}");
                    break;
                }

                if attempt == max_attempts {
                    println!("Max attempts reached. Last error: {error}");
                    break;
                }

                let delay = category.retry_delay(attempt);
                println!("Attempt {attempt} failed: {error} (will retry after {delay:?})");

                // In real code, you'd use tokio::time::sleep(delay).await
                std::thread::sleep(Duration::from_millis(10)); // Short delay for demo
            }
        }
    }
}

fn simulate_network_call(attempt: usize) -> TylResult<String> {
    match attempt {
        1 => Err(TylError::network("Connection timeout")),
        2 => Err(TylError::network("Connection refused")),
        3 => Err(TylError::network("Service temporarily unavailable")),
        _ => Ok("Data retrieved successfully".to_string()),
    }
}
