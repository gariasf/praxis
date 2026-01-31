//! Error handling utilities and patterns.
//!
//! This module provides convenient re-exports and utilities for working with
//! `color-eyre` errors throughout the Praxis engine. It centralizes error
//! handling patterns and provides extension traits for adding context to errors.
//!
//! # Core Types
//!
//! ## `Result<T>`
//!
//! Throughout the engine, `Result<T>` is an alias for `Result<T, color_eyre::Report>`.
//! This provides automatic error conversion and context propagation:
//!
//! ```rust
//! use praxis_utils::Result;
//!
//! fn load_config() -> Result<Config> {
//!     // std::io::Error automatically converts to Report
//!     let data = std::fs::read_to_string("config.toml")?;
//!     
//!     // serde errors also convert automatically
//!     let config: Config = toml::from_str(&data)?;
//!     
//!     Ok(config)
//! }
//! ```
//!
//! ## `Report`
//!
//! `color_eyre::Report` is the error type that can represent any error.
//! It provides:
//! - Error chain display (shows the full causal chain)
//! - Optional backtraces (enable with `RUST_BACKTRACE=1`)
//! - Suggestions and help text
//! - Colorized output for better readability
//!
//! # Adding Context
//!
//! ## `WrapErr` Trait
//!
//! The `WrapErr` trait adds `.wrap_err()` and `.wrap_err_with()` methods to
//! `Result` types, allowing you to add context when propagating errors:
//!
//! ```rust
//! use praxis_utils::{Result, WrapErr};
//!
//! fn load_shader(path: &str) -> Result<Vec<u8>> {
//!     // Add static context
//!     std::fs::read(path)
//!         .wrap_err("Failed to read shader file")?;
//!     
//!     Ok(vec![])
//! }
//!
//! fn load_shader_with_path(path: &str) -> Result<Vec<u8>> {
//!     // Add dynamic context (only evaluated if error occurs)
//!     std::fs::read(path)
//!         .wrap_err_with(|| format!("Failed to read shader: {}", path))?;
//!     
//!     Ok(vec![])
//! }
//! ```
//!
//! ## `Context` Trait
//!
//! The `Context` trait provides `.context()` methods similar to `anyhow`:
//!
//! ```rust
//! use praxis_utils::{Result, Context};
//!
//! fn parse_config() -> Result<Config> {
//!     let data = std::fs::read_to_string("config.toml")
//!         .context("Failed to read configuration file")?;
//!     
//!     let config: Config = toml::from_str(&data)
//!         .context("Failed to parse TOML configuration")?;
//!     
//!     Ok(config)
//! }
//! ```
//!
//! # Creating Errors
//!
//! ## `bail!` Macro
//!
//! Early return with an error:
//!
//! ```rust
//! use praxis_utils::{Result, bail};
//!
//! fn validate_positive(value: i32) -> Result<()> {
//!     if value <= 0 {
//!         bail!("Value must be positive, got {}", value);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## `ensure!` Macro
//!
//! Assert a condition or return an error:
//!
//! ```rust
//! use praxis_utils::{Result, ensure};
//!
//! fn divide(a: f32, b: f32) -> Result<f32> {
//!     ensure!(b != 0.0, "Division by zero");
//!     Ok(a / b)
//! }
//! ```
//!
//! ## `eyre!` Macro
//!
//! Create an error without returning:
//!
//! ```rust
//! use praxis_utils::eyre;
//!
//! fn try_operation() -> Option<praxis_utils::Report> {
//!     if something_wrong() {
//!         return Some(eyre::eyre!("Operation failed"));
//!     }
//!     None
//! }
//! # fn something_wrong() -> bool { false }
//! ```
//!
//! # Common Patterns
//!
//! ## Pattern 1: Layered Context
//!
//! Build up context as errors propagate up the call stack:
//!
//! ```rust,ignore
//! // Low-level: file I/O error
//! fn read_file(path: &Path) -> Result<Vec<u8>> {
//!     std::fs::read(path)
//!         .wrap_err_with(|| format!("Failed to read file: {}", path.display()))
//! }
//!
//! // Mid-level: parsing error
//! fn parse_mesh(path: &Path) -> Result<Mesh> {
//!     let data = read_file(path)
//!         .wrap_err("Failed to load mesh data")?;
//!     
//!     parse_obj(&data)
//!         .wrap_err("Failed to parse OBJ format")
//! }
//!
//! // High-level: user-facing error
//! fn load_model(name: &str) -> Result<Model> {
//!     parse_mesh(Path::new(name))
//!         .wrap_err_with(|| format!("Failed to load model '{}'", name))
//! }
//! ```
//!
//! This produces error chains like:
//! ```text
//! Error: Failed to load model 'spaceship.obj'
//!
//! Caused by:
//!    0: Failed to load mesh data
//!    1: Failed to read file: assets/models/spaceship.obj
//!    2: No such file or directory (os error 2)
//! ```
//!
//! ## Pattern 2: Validation with ensure!
//!
//! ```rust
//! use praxis_utils::{Result, ensure};
//!
//! fn create_texture(width: u32, height: u32) -> Result<Texture> {
//!     ensure!(width > 0, "Texture width must be positive");
//!     ensure!(height > 0, "Texture height must be positive");
//!     ensure!(width <= 8192, "Texture width exceeds maximum (8192)");
//!     ensure!(height <= 8192, "Texture height exceeds maximum (8192)");
//!     
//!     // Create texture...
//!     # Ok(Texture)
//! }
//! # struct Texture;
//! ```
//!
//! ## Pattern 3: Option to Result Conversion
//!
//! ```rust
//! use praxis_utils::{Result, eyre};
//!
//! fn get_component(entity: Entity) -> Result<&Component> {
//!     world.get(entity)
//!         .ok_or_else(|| eyre::eyre!("Entity {:?} not found", entity))
//! }
//! # struct Entity;
//! # struct Component;
//! # struct World;
//! # impl World {
//! #     fn get(&self, _: Entity) -> Option<&Component> { None }
//! # }
//! # let world = World;
//! ```
//!
//! ## Pattern 4: Suggestions
//!
//! Provide helpful hints for fixing errors:
//!
//! ```rust,ignore
//! use praxis_utils::{Result, WrapErr};
//! use color_eyre::Help;
//!
//! fn init_vulkan() -> Result<Instance> {
//!     create_instance()
//!         .wrap_err("Failed to create Vulkan instance")?
//!         .suggestion("Ensure Vulkan drivers are installed")
//! }
//! ```
//!
//! ## Pattern 5: Error Logging
//!
//! Log errors while propagating them:
//!
//! ```rust,ignore
//! use praxis_utils::{Result, error};
//!
//! fn critical_operation() -> Result<()> {
//!     perform_operation().map_err(|e| {
//!         error!("Critical operation failed: {}", e);
//!         e // Propagate error to caller
//!     })
//! }
//! ```
//!
//! # Best Practices
//!
//! ## Do's
//!
//! - ✅ Add context at each layer of the call stack
//! - ✅ Use `wrap_err_with()` for expensive formatting (lazy evaluation)
//! - ✅ Provide suggestions for common errors
//! - ✅ Use `ensure!` for validation instead of `if` + `bail!`
//! - ✅ Keep error messages concise but informative
//!
//! ## Don'ts
//!
//! - ❌ Don't add redundant context (let lower layers handle details)
//! - ❌ Don't use `unwrap()` or `expect()` in production code
//! - ❌ Don't lose error context by creating new errors without wrapping
//! - ❌ Don't include secrets or sensitive data in error messages
//! - ❌ Don't format strings unnecessarily (use `wrap_err_with` for lazy eval)
//!
//! # Performance Considerations
//!
//! - **Error creation is cheap**: `Report` uses `Arc` internally
//! - **Context is lazy**: `wrap_err_with()` closures only run if error occurs
//! - **Zero cost on success path**: Error handling has no overhead when operations succeed
//!
//! # See Also
//!
//! - [`color-eyre` documentation](https://docs.rs/color-eyre)
//! - [`eyre` documentation](https://docs.rs/eyre)
//! - [Error Handling in Rust](https://doc.rust-lang.org/book/ch09-00-error-handling.html)

pub use color_eyre::eyre::{bail, ensure};

/// Extension trait for adding context to `Result` types.
///
/// This trait is automatically implemented for all `Result<T, E>` where `E`
/// can be converted to `color_eyre::Report`.
///
/// # Methods
///
/// - `wrap_err(msg)`: Add a static string as context
/// - `wrap_err_with(f)`: Add dynamically generated context (lazy evaluation)
///
/// # Examples
///
/// ```rust
/// use praxis_utils::{Result, WrapErr};
///
/// fn read_config() -> Result<String> {
///     std::fs::read_to_string("config.toml")
///         .wrap_err("Failed to read configuration file")
/// }
///
/// fn read_config_with_path(path: &str) -> Result<String> {
///     std::fs::read_to_string(path)
///         .wrap_err_with(|| format!("Failed to read config: {}", path))
/// }
/// ```
pub trait WrapErr<T, E> {
    /// Wraps the error with additional context.
    ///
    /// # Arguments
    ///
    /// * `msg` - Static context message to add to the error
    ///
    /// # Returns
    ///
    /// `Ok(T)` if the result is `Ok`, or an error with added context if `Err`.
    ///
    /// # Errors
    ///
    /// Returns an error with the added context if the original result was `Err`.
    fn wrap_err<D>(self, msg: D) -> crate::Result<T>
    where
        D: std::fmt::Display + Send + Sync + 'static;

    /// Wraps the error with dynamically generated context.
    ///
    /// The closure is only called if an error occurs, allowing for lazy
    /// evaluation of expensive formatting operations.
    ///
    /// # Arguments
    ///
    /// * `f` - Closure that generates the context message
    ///
    /// # Returns
    ///
    /// `Ok(T)` if the result is `Ok`, or an error with added context if `Err`.
    ///
    /// # Errors
    ///
    /// Returns an error with the added context if the original result was `Err`.
    fn wrap_err_with<D, F>(self, f: F) -> crate::Result<T>
    where
        D: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> D;
}

impl<T, E> WrapErr<T, E> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn wrap_err<D>(self, msg: D) -> crate::Result<T>
    where
        D: std::fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|e| crate::Report::new(e).wrap_err(msg))
    }

    fn wrap_err_with<D, F>(self, f: F) -> crate::Result<T>
    where
        D: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> D,
    {
        self.map_err(|e| crate::Report::new(e).wrap_err(f()))
    }
}

/// Extension trait for adding context to `Option` types.
///
/// This trait is automatically implemented for all `Option<T>`.
///
/// # Methods
///
/// - `context(msg)`: Convert `None` to an error with static context
/// - `with_context(f)`: Convert `None` to an error with dynamic context
///
/// # Examples
///
/// ```rust
/// use praxis_utils::{Result, Context};
///
/// fn find_entity(id: u32) -> Result<Entity> {
///     get_entity(id)
///         .context("Entity not found")
/// }
///
/// fn find_entity_with_id(id: u32) -> Result<Entity> {
///     get_entity(id)
///         .with_context(|| format!("Entity {} not found", id))
/// }
///
/// fn get_entity(_id: u32) -> Option<Entity> {
///     None
/// }
/// # struct Entity;
/// ```
pub trait Context<T> {
    /// Converts `None` to an error with the given context.
    ///
    /// # Arguments
    ///
    /// * `msg` - Static context message for the error
    ///
    /// # Returns
    ///
    /// `Ok(T)` if the option is `Some`, or an error with the context if `None`.
    ///
    /// # Errors
    ///
    /// Returns an error with the provided context if the option is `None`.
    fn context<D>(self, msg: D) -> crate::Result<T>
    where
        D: std::fmt::Display + Send + Sync + 'static;

    /// Converts `None` to an error with dynamically generated context.
    ///
    /// The closure is only called if the option is `None`, allowing for lazy
    /// evaluation of expensive formatting operations.
    ///
    /// # Arguments
    ///
    /// * `f` - Closure that generates the context message
    ///
    /// # Returns
    ///
    /// `Ok(T)` if the option is `Some`, or an error with the context if `None`.
    ///
    /// # Errors
    ///
    /// Returns an error with the provided context if the option is `None`.
    fn with_context<D, F>(self, f: F) -> crate::Result<T>
    where
        D: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> D;
}

impl<T> Context<T> for Option<T> {
    fn context<D>(self, msg: D) -> crate::Result<T>
    where
        D: std::fmt::Display + Send + Sync + 'static,
    {
        self.ok_or_else(|| crate::Report::msg(format!("{msg}")))
    }

    fn with_context<D, F>(self, f: F) -> crate::Result<T>
    where
        D: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> D,
    {
        self.ok_or_else(|| crate::Report::msg(format!("{}", f())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_err() {
        let result: std::result::Result<(), std::io::Error> =
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));

        let wrapped = result.wrap_err("Failed to read file");
        assert!(wrapped.is_err());

        let err = wrapped.unwrap_err();
        let err_string = format!("{}", err);
        assert!(err_string.contains("Failed to read file"));
    }

    #[test]
    fn test_wrap_err_with() {
        let path = "test.txt";
        let result: std::result::Result<(), std::io::Error> =
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));

        let wrapped = result.wrap_err_with(|| format!("Failed to read {}", path));
        assert!(wrapped.is_err());

        let err = wrapped.unwrap_err();
        let err_string = format!("{}", err);
        assert!(err_string.contains("Failed to read test.txt"));
    }

    #[test]
    fn test_option_context() {
        let value: Option<i32> = None;
        let result = value.context("Value not found");
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_string = format!("{}", err);
        assert!(err_string.contains("Value not found"));
    }

    #[test]
    fn test_option_with_context() {
        let id = 42;
        let value: Option<i32> = None;
        let result = value.with_context(|| format!("Item {} not found", id));
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_string = format!("{}", err);
        assert!(err_string.contains("Item 42 not found"));
    }

    #[test]
    fn test_ensure_macro() {
        fn validate(value: i32) -> crate::Result<()> {
            ensure!(value > 0, "Value must be positive");
            Ok(())
        }

        assert!(validate(1).is_ok());
        assert!(validate(-1).is_err());
    }

    #[test]
    fn test_bail_macro() {
        fn fail_early() -> crate::Result<()> {
            bail!("Operation failed");
        }

        let result = fail_early();
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_string = format!("{}", err);
        assert!(err_string.contains("Operation failed"));
    }
}
