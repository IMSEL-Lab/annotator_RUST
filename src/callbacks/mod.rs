//! Callback handlers for the annotator application.
//!
//! This module contains all UI callback implementations organized by functionality:
//! - `navigation` - Image navigation (next/prev/first/last/random)
//! - `selection` - Annotation selection (select/deselect/select_all/delete)
//! - `drawing` - Drawing operations (bbox, point creation)
//! - `annotation` - Annotation manipulation (delete, classify, undo, redo, copy, paste)
//! - `polygon` - Polygon annotation creation
//! - `resize` - Annotation resizing
//! - `file_ops` - File operations (save, open, new, export)
//! - `auto_resize` - Smart bbox auto-resize using edge detection

pub mod navigation;
pub mod selection;
pub mod drawing;
pub mod annotation;
pub mod polygon;
pub mod resize;
pub mod file_ops;
pub mod auto_resize;
