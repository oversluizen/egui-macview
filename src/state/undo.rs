//! Authentic AppKit `NSUndoManager` transaction state restoration.
//!
//! Normative Reference: SPEC-008 §2.
//! Grounded in bounded ringbuffer mechanics with strict capacity ceilings
//! and property-verified cyclic state preservation.

use std::collections::VecDeque;

/// Default capacity for undo and redo transaction snapshots.
pub const DEFAULT_UNDO_CAPACITY: usize = 50;

/// Bounded ringbuffer undo/redo state manager (`NSUndoManager` equivalent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NSUndoManager<T> {
    undo_stack: VecDeque<T>,
    redo_stack: VecDeque<T>,
    capacity: usize,
}

impl<T: Clone> Default for NSUndoManager<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> NSUndoManager<T> {
    /// Creates a new `NSUndoManager` with default capacity (`DEFAULT_UNDO_CAPACITY = 50`).
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_UNDO_CAPACITY)
    }

    /// Creates a new `NSUndoManager` with a custom maximum capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        let cap = capacity.max(1);
        Self {
            undo_stack: VecDeque::with_capacity(cap),
            redo_stack: VecDeque::with_capacity(cap),
            capacity: cap,
        }
    }

    /// Maximum snapshot capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Number of recorded undo steps available.
    #[inline]
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Number of recorded redo steps available.
    #[inline]
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Returns `true` if an undo operation can be performed.
    #[inline]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns `true` if a redo operation can be performed.
    #[inline]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Records a new state snapshot on the undo stack.
    ///
    /// In accordance with SPEC-008 §2.2:
    /// 1. Drops the oldest entry if the undo stack exceeds capacity.
    /// 2. Unconditionally clears the redo stack.
    pub fn push_snapshot(&mut self, state: T) {
        if self.undo_stack.len() >= self.capacity {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(state);
        self.redo_stack.clear();
    }

    /// Performs an undo operation:
    /// - Pushes `*current_state` onto the redo stack.
    /// - Restores `*current_state` from the most recent undo snapshot.
    /// - Returns `true` if state was restored, or `false` if `undo_stack` was empty.
    pub fn undo(&mut self, current_state: &mut T) -> bool {
        if let Some(prev) = self.undo_stack.pop_back() {
            if self.redo_stack.len() >= self.capacity {
                self.redo_stack.pop_front();
            }
            self.redo_stack.push_back(current_state.clone());
            *current_state = prev;
            true
        } else {
            false
        }
    }

    /// Performs a redo operation:
    /// - Pushes `*current_state` onto the undo stack.
    /// - Restores `*current_state` from the most recent redo snapshot.
    /// - Returns `true` if state was restored, or `false` if `redo_stack` was empty.
    pub fn redo(&mut self, current_state: &mut T) -> bool {
        if let Some(next) = self.redo_stack.pop_back() {
            if self.undo_stack.len() >= self.capacity {
                self.undo_stack.pop_front();
            }
            self.undo_stack.push_back(current_state.clone());
            *current_state = next;
            true
        } else {
            false
        }
    }

    /// Clears all undo and redo history.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}
