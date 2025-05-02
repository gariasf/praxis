# GitHub Workflows

## Rust CI

The `rust-ci.yml` workflow runs the following Rust code quality checks on pull requests:

- `cargo check`: Verifies that the code compiles without errors
- `cargo fmt --check`: Ensures code follows Rust formatting standards
- `cargo clippy`: Runs the Rust linter with warnings treated as errors

These checks must pass for pull requests to be mergeable.

The workflow uses the `dtolnay/rust-toolchain` action to install Rust with the necessary components (fmt, clippy), ensuring they're available across the entire workspace.
