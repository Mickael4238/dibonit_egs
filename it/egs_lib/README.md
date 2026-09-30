# EGS Library (egs_lib)

**egs_lib** is a foundational Rust library for the Dibonit Entreprise Global Softwares (EGS) ecosystem. It provides core utilities, data structures, and common functionality used across all EGS components.

## Purpose

The egs_lib serves as the common foundation for all EGS applications and services, providing:

- **Core data types** used throughout the EGS ecosystem
- **Common utilities** for error handling, logging, and serialization
- **Shared configurations** and constants
- **Base traits and interfaces** for consistent API design

## Features

### Core Data Types

- **Message types** for ESB communication
- **Identifier types** for enterprise objects
- **Timestamp and date utilities**
- **Configuration structures**

### Utilities

- **Error handling** with custom error types
- **Logging macros** and utilities
- **Serialization/deserialization** helpers
- **Validation utilities**

## Usage

Add egs_lib as a dependency in your Cargo.toml:

```toml
[dependencies]
 egc_lib = { path = "../it/egs_lib" }
```

Then import and use the library:

```rust
use egs_lib::prelude::*;

fn main() {
    // Use egs_lib functionality
}
```

## Project Structure

```
 egc_lib/
├── Cargo.toml          # Library configuration
├── README.md           # This document
├── src/
│   ├── lib.rs          # Library root
│   ├── prelude.rs      # Prelude module
│   ├── types/          # Core data types
│   │   ├── mod.rs
│   │   ├── message.rs  # Message types
│   │   ├── id.rs       # Identifier types
│   │   └── time.rs     # Time utilities
│   ├── error/          # Error handling
│   │   ├── mod.rs
│   │   └── error.rs    # Custom error types
│   ├── utils/          # Utilities
│   │   ├── mod.rs
│   │   ├── log.rs      # Logging utilities
│   │   └── validate.rs # Validation utilities
│   └── config/         # Configuration
│       ├── mod.rs
│       └── settings.rs # Configuration structures
└── tests/              # Unit tests
    ├── types/          # Type tests
    └── utils/          # Utility tests
```

## Getting Started

### Prerequisites

- Rust 1.70+
- Cargo

### Building

```bash
cd it/egs_lib
cargo build
```

### Testing

```bash
cargo test
```

### Documentation

```bash
cargo doc --open
```

## API Documentation

### Main Modules

- `types`: Core data types used throughout EGS
- `error`: Custom error types and error handling
- `utils`: Utility functions and macros
- `config`: Configuration structures and helpers

### Prelude

The `prelude` module exports commonly used items for convenient access:

```rust
use egs_lib::prelude::*;
```

## Examples

### Creating a Message

```rust
use egs_lib::types::Message;

let msg = Message::new("test.message", "payload");
```

### Error Handling

```rust
use egs_lib::error::EgsError;

fn process() -> Result<(), EgsError> {
    // Your code here
    Ok(())
}
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Run clippy: `cargo clippy`
6. Submit a pull request

## License

This project is proprietary software. All rights reserved.
