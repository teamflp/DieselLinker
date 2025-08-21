# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

-   **Dependency Compatibility**: Migrated the codebase to be compatible with `syn` version 2.0, resolving critical build failures and ensuring compatibility with the modern Rust ecosystem.
-   **CI Test Failures**: Fixed several issues in the test suite that caused failures in the CI environment, including database connection problems and race conditions in parallel test execution.
-   **UI Test Snapshots**: Updated outdated UI test snapshots to match new compiler error messages after adding new attributes.
