# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

-   **`schema_path` attribute**: The generated code no longer assumes the Diesel schema module is `crate::schema`. `many_to_one` and `many_to_many` relations now accept an optional `schema_path` attribute pointing at any module.
-   **`join_key_type` attribute**: `many_to_many` relations no longer hardcode `i32` for join-table/related-primary-key lookups. An optional `join_key_type` attribute (defaults to `"i32"`, unchanged) lets the generated queries use `i64`, `uuid::Uuid`, or any other matching Rust type.

### Fixed

-   **Panics replaced with compile errors**: An invalid `model`/`fk`/`join_table`/... identifier or an unparsable `error_type` used to panic the proc-macro (an opaque ICE-style failure) instead of producing a normal `compile_error!`. Both now produce a proper, span-located compiler error.
-   **Stale `trybuild` snapshots**: Three `tests/ui/fail/*.stderr` snapshots expected a secondary `E0412` diagnostic no longer emitted by the current compiler; updated to match actual output.

### Dependencies

-   Removed `diesel` and `diesel-async` from `[dependencies]` — this proc-macro crate never called into them at compile time (they were only referenced as tokens inside generated code). They remain in `[dev-dependencies]` for the test suite. This means building `diesel_linker` itself no longer requires `libpq`/`libmysqlclient` to be installed.

### CI / Tooling

-   Replaced the archived `actions-rs/toolchain` action with `dtolnay/rust-toolchain` and bumped `actions/checkout` to v4 in both workflows.
-   Added a `lint` CI job running `cargo fmt --all -- --check` and `cargo clippy --all-targets --all-features -- -D warnings`; fixed the pre-existing clippy warnings (`unnecessary_map_or`, `useless_conversion`, `too_many_arguments`, `ptr_arg`) and reformatted the whole codebase with `cargo fmt` so the new gate starts green.
-   Added an `audit` CI job (`rustsec/audit-check`) to catch known-vulnerable dependencies.
-   Removed the `sleep 15` "wait for MySQL" step — the `mysql` service's `--health-cmd` already makes GitHub Actions block until the container is healthy, so the extra sleep was dead weight.
-   Added `.github/dependabot.yml` for weekly `cargo` and `github-actions` update PRs.
-   Simplified `generate_relation_code`'s signature from 17 positional arguments to `(struct_name, &RelationAttributes)`, reusing the existing struct instead of re-destructuring it at the call site.

## [1.3.0] - 2025-08-21

### Added

-   **Asynchronous Support**: Introduced support for asynchronous database operations through integration with `diesel-async`. This is enabled via the new `async = true` attribute in the `#[relation]` macro.
-   **Custom Error Types**: Added the `error_type` attribute to allow users to specify a custom error type for the generated methods, enabling better integration with existing error handling patterns.
-   **Comprehensive Documentation**:
    -   Added a `CONTRIBUTING.md` guide with instructions for setting up the development environment.
    -   Created a full documentation website with `mdBook`, located in the `/book` directory, providing detailed explanations and examples for all features.
-   **Continuous Integration (CI)**: Set up a GitHub Actions workflow to automatically run the full test suite against PostgreSQL and MySQL, ensuring the stability and reliability of all supported backends.
-   **License Files**: Added `LICENSE-MIT` and `LICENSE-APACHE` files to the repository to comply with open-source best practices.

### Fixed

-   **Attribute Parsing Rewrite**: Modernized the `#[relation]` attribute-parsing code to stop relying on `syn`'s deprecated `AttributeArgs`/`NestedMeta` types, parsing a `Punctuated<Meta, Token![,]>` directly instead. Note: the crate still depends on `syn` 1.0 (not 2.0) as of this release.
-   **CI Test Failures**: Fixed several issues in the test suite that caused failures in the CI environment, including database connection problems and race conditions in parallel test execution.
-   **UI Test Snapshots**: Updated outdated UI test snapshots to match new compiler error messages after adding new attributes.
