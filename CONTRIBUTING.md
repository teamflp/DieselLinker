# Contributing to Diesel Linker

First off, thank you for considering contributing to Diesel Linker! It's people like you that make the open source community such a great place.

We welcome any type of contribution, not only code. You can help with:
*   **Reporting a bug**
*   **Discussing the current state of the code**
*   **Submitting a fix**
*   **Proposing new features**
*   **Becoming a maintainer**

## Getting Started

### Prerequisites

*   [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
*   [Cargo](https://doc.rust-lang.org/cargo/) (comes with Rust)

### Fork & Clone

1.  Fork the repository on GitHub.
2.  Clone your fork locally:
    ```sh
    git clone https://github.com/your-username/DieselLinker.git
    cd DieselLinker
    ```

## Setting up the Development Environment

To run the full test suite, you will need to have client libraries for PostgreSQL and MySQL installed on your system.

### Installing Database Dependencies

#### Linux (Debian/Ubuntu)

```sh
sudo apt-get update
sudo apt-get install libpq-dev libmysqlclient-dev
```

#### macOS

Using [Homebrew](https://brew.sh/):

```sh
brew install postgresql mysql
```

#### Windows

Using [Chocolatey](https://chocolatey.org/) or [winget](https://docs.microsoft.com/en-us/windows/package-manager/winget/):

```sh
# Using Chocolatey
choco install libpq mysql-connector-c

# Using winget
winget install PostgreSQL.PostgreSQL
winget install MySQL.ConnectorC
```
*Note: The exact package names might vary. Please refer to the documentation of your package manager.*

### Running the Tests

Once you have the dependencies installed, you can run the test suite:

```sh
cargo test
```

Please note that the test suite includes integration tests that may require running instances of PostgreSQL and MySQL databases. The tests expect to be able to connect to these databases using default credentials. If you do not have these databases running, you can still run the tests that do not depend on them. The SQLite-based tests (including the new async tests) will run without any external database setup.

## Submitting a Pull Request

1.  Create a new branch for your changes:
    ```sh
    git checkout -b my-feature-branch
    ```
2.  Make your changes and commit them with a descriptive message.
3.  Push your branch to your fork:
    ```sh
    git push origin my-feature-branch
    ```
4.  Open a pull request on the original repository.

Please make sure your PR description clearly describes the problem and solution. Include the relevant issue number if applicable.

Thank you for your contribution!
