//! State integrity, transaction management, and snapshot history.

pub mod undo;

pub use undo::{NSUndoManager, DEFAULT_UNDO_CAPACITY};
