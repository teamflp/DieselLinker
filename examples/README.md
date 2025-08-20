# DieselLinker Examples

This directory contains runnable examples for the `diesel_linker` crate.

## How to Run

You can run each example using the `cargo run --example` command, followed by the name of the example file (without the `.rs` extension).

For example, to run the `basic_usage` example:

```sh
cargo run --example basic_usage
```

Make sure you have the required dependencies in your `Cargo.toml` file, as specified in the main `README.md` of this project. For the examples here, you will need at least:

```toml
[dependencies]
diesel = { version = "2.1.0", features = ["sqlite"] }
diesel_linker = { path = ".." } # Assuming you are running from the root of the project
```
