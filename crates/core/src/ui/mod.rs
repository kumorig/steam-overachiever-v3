//! Shared UI components for desktop and WASM
//! 
//! This module provides platform-agnostic UI rendering using egui.
//! Platform-specific details (like image loading) are abstracted via traits.

mod stats_panel;

pub use stats_panel::*;
