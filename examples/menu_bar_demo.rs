//! MenuBar demonstration (placeholder).
//!
//! This example is currently a placeholder. The full menu bar demo
//! requires window and GUI APIs that are not yet fully integrated.
//!
//! See the `praxis_editor` module for the `EditorState` and menu bar APIs.
//!
//! # Intended Features (not yet implemented in demo)
//!
//! - File menu (New, Open, Save, Save As, Exit)
//! - Edit menu (Undo, Redo, Copy, Paste, Duplicate)
//! - Entity menu (Create Empty, Create Primitives, Delete)
//! - View menu (Toggle Panels)
//! - Help menu (About, Documentation)
//! - Standard keyboard shortcuts

#[cfg(feature = "editor")]
fn main() {
    println!("=== Menu Bar Demo (Placeholder) ===\n");
    println!("This example is currently a placeholder.");
    println!("The full menu bar demo requires window and GUI APIs.");
    println!();
    println!("See the `praxis_editor` module for the `EditorState` and menu bar APIs.");
    println!();
    println!("# Keyboard Shortcuts (design spec):");
    println!();
    println!("  File Menu:");
    println!("    Ctrl+N: New Scene");
    println!("    Ctrl+O: Open Scene");
    println!("    Ctrl+S: Save Scene");
    println!("    Ctrl+Shift+S: Save Scene As");
    println!("    Alt+F4: Exit");
    println!();
    println!("  Edit Menu:");
    println!("    Ctrl+Z: Undo");
    println!("    Ctrl+Y: Redo");
    println!("    Ctrl+C: Copy");
    println!("    Ctrl+V: Paste");
    println!("    Ctrl+D: Duplicate");
    println!();
    println!("  Entity Menu:");
    println!("    Delete: Delete Entity");
    println!();
    println!("  Help Menu:");
    println!("    F1: Documentation");
}

#[cfg(not(feature = "editor"))]
fn main() {
    eprintln!("This example requires the 'editor' feature to be enabled.");
    eprintln!("Run with: cargo run --example menu_bar_demo --features editor");
    std::process::exit(1);
}
