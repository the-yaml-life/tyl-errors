# tyl-errors

Error handling library for TYL framework with extensible classification and retry logic.

## Features

- Type-safe error handling
- Extensible error categories  
- Built-in retry logic with exponential backoff
- Rich error context with metadata
- Serialization support

## Quick Start

```rust
use tyl_errors::{TylError, TylResult};

fn validate_data(input: &str) -> TylResult<()> {
    if input.is_empty() {
        return Err(TylError::validation("input", "Cannot be empty"));
    }
    Ok(())
}
```

## Error Types

- Database - Database operations
- Network - Network failures  
- Validation - Input validation
- NotFound - Resource not found
- Internal - System errors
- Custom - Extensible custom types

## Examples

Run examples:
```bash
cargo run --example basic_usage
cargo run --example extensible_categories
```

## Testing

```bash
cargo test
```