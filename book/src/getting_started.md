# Getting Started

To get started with `DieselLinker`, you need to add it to your `Cargo.toml` file along with `diesel`.

### Prerequisites

*   Rust and Cargo installed on your system.
*   `diesel` and `diesel_linker` added to your `Cargo.toml`.

```toml
[dependencies]
diesel = { version = "2.2.2", features = ["postgres", "sqlite", "mysql"] } # Enable features for your database
diesel_linker = "1.3.0" # Use the latest version
```

### Basic Usage

1.  Add `diesel_linker` to your `Cargo.toml`.
2.  Define your Diesel models and schema as usual.
3.  Add the `#[relation]` attribute to your model structs to define the relationships.

In the following chapters, we will explore the `#[relation]` attribute in detail.
