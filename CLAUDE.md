# CLAUDE.md - tyl-errors

## 📋 **Module Context**

**tyl-errors** is the error handling module for the TYL framework. It established the gold standard pattern that all TYL modules follow.

## 🏗️ **Architecture**

### **Port (Interface)**
```rust
trait ErrorClassifier {
    fn is_retriable(&self) -> bool;
    fn retry_delay(&self, attempt: usize) -> Duration;
    fn category_name(&self) -> &'static str;
    fn clone_box(&self) -> Box<dyn ErrorClassifier>;
}
```

### **Adapters (Implementations)**
- `BuiltinCategory` - Built-in categories (Network, Database, Validation, etc.)
- `ErrorCategory::Custom` - Wrapper for user-defined categories

### **Core Types**
- `TylError` - Main enum with variants for each error type
- `TylResult<T>` - Type alias for `Result<T, TylError>`
- `ErrorContext` - Rich context with UUID, timestamps, metadata

## 🧪 **Testing**

### **Running Tests**
```bash
cargo test -p tyl-errors                    # Unit tests
cargo test --doc -p tyl-errors              # Doc tests
cargo test --package tyl-errors --test integration_tests  # Integration tests
```

### **Examples**
```bash
cargo run --example basic_usage -p tyl-errors
cargo run --example extensible_categories -p tyl-errors
cargo run --example error_context -p tyl-errors
cargo run --example retry_logic -p tyl-errors
```

## 📂 **File Structure**

```
tyl-errors/
├── src/lib.rs                 # Core implementation (600+ lines)
├── examples/                  # 4 working examples
├── tests/integration_tests.rs # Integration tests
├── .github/workflows/         # CI/CD with GitHub Actions
├── README.md                  # Main documentation
├── CHANGELOG.md               # Version history
└── Cargo.toml                 # Package metadata
```

## 🔧 **How to Extend**

### **Adding New Custom Category**
```rust
#[derive(Debug, Clone)]
struct MyCustomCategory;

impl ErrorClassifier for MyCustomCategory {
    fn is_retriable(&self) -> bool { true }
    fn retry_delay(&self, attempt: usize) -> Duration {
        Duration::from_millis(attempt as u64 * 100)
    }
    fn category_name(&self) -> &'static str { "MyCustom" }
    fn clone_box(&self) -> Box<dyn ErrorClassifier> {
        Box::new(self.clone())
    }
}

// Usage
let error = TylError::business_logic("Custom error", Box::new(MyCustomCategory));
```

### **Adding New Built-in Error Type**
1. Add variant to `TylError` enum
2. Add pattern in `TylError::category()`
3. Add constructor method
4. Write tests

## 🛠️ **Useful Commands**

```bash
# Desarrollo
cargo clippy -p tyl-errors                 # Linting
cargo fmt -p tyl-errors                    # Formatting
cargo doc --no-deps -p tyl-errors --open   # Generar docs

# Testing completo
cargo test -p tyl-errors --all-features --verbose

# Check antes de commit
cargo clippy -p tyl-errors -- -D warnings
cargo fmt -p tyl-errors --check
```

## 📦 **Dependencies**

### **Runtime**
- `thiserror` - Error derivation
- `serde` - Serialization
- `chrono` - Timestamps
- `uuid` - Unique identifiers

### **Development**
- `serde_json` - JSON serialization (tests/examples)

## 🎯 **Principios de Diseño**

1. **Extensibilidad** - Users pueden agregar categorías sin modificar core
2. **Type Safety** - Strong typing para prevenir errores
3. **Rich Context** - Información suficiente para debugging
4. **Retry Logic** - Built-in smart retry con exponential backoff
5. **Serializable** - Serde support para logging/monitoring

## ⚠️ **Limitaciones Conocidas**

- Custom categories no son completamente serializables (se skip en serde)
- ErrorContext en serialización pierde la category (se usa default)
- Trait objects requieren `clone_box()` manual

## 🔗 **Repository**

- **GitHub**: https://github.com/the-yaml-life/tyl-errors
- **Docs**: https://docs.rs/tyl-errors
- **CI Status**: Branch protection + GitHub Actions

## 📝 **Notas para Contributors**

- Seguir TDD: tests primero, implementación después
- Todos los públicos APIs necesitan doc comments con ejemplos
- Mantener backwards compatibility
- Tests deben pasar en stable, beta, y nightly Rust