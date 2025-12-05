//! State management types for the annotator application.
//!
//! This module contains all state-related types including:
//! - Dataset state and entries
//! - View state (pan/zoom)
//! - Drawing and resize states
//! - Undo/redo history
//! - Stored annotation format

mod types;
mod dataset;

pub use types::*;
pub use dataset::*;
