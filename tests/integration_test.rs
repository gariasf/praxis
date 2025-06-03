//! Integration tests for Praxis engine components.
//!
//! These tests verify that different crates work together correctly,
//! focusing on initialization flows and error handling.

use praxis_utils::init;

/// Test that the tracing system initializes correctly and doesn't interfere
/// with other components.
#[test]
fn test_tracing_initialization() {
    // Note: Tracing can only be initialized once globally, so we just test
    // that calling init doesn't panic and handles the case gracefully
    let result = std::panic::catch_unwind(|| {
        let _ = init();
    });
    assert!(result.is_ok(), "Tracing initialization should not panic");
}
