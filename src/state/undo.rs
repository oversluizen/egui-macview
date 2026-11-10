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
    active_group_initial: Option<T>,
}

impl<T: Clone + PartialEq> Default for NSUndoManager<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + PartialEq> NSUndoManager<T> {
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
            active_group_initial: None,
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

    /// Returns `true` if a coalesced undo group is currently open.
    #[inline]
    pub fn is_grouping(&self) -> bool {
        self.active_group_initial.is_some()
    }

    /// Non-destructively inspects the most recent undo snapshot.
    #[inline]
    pub fn peek_undo(&self) -> Option<&T> {
        self.undo_stack.back()
    }

    /// Non-destructively inspects the most recent redo snapshot.
    #[inline]
    pub fn peek_redo(&self) -> Option<&T> {
        self.redo_stack.back()
    }

    /// Begins a coalesced undo group (such as a continuous slider or canvas drag).
    ///
    /// While a group is active, intermediate [`Self::push_snapshot`] calls are ignored
    /// until [`Self::end_grouping`] commits the net state transition as a single undo step.
    pub fn begin_grouping(&mut self, initial_state: T) {
        if self.active_group_initial.is_none() {
            self.active_group_initial = Some(initial_state);
        }
    }

    /// Ends the active coalesced undo group and records a single undo snapshot
    /// if `final_state` differs from the group's `initial_state`.
    pub fn end_grouping(&mut self, final_state: &T) -> bool {
        if let Some(initial) = self.active_group_initial.take() {
            if &initial != final_state {
                self.redo_stack.clear();
                if self.undo_stack.back() != Some(&initial) {
                    if self.undo_stack.len() >= self.capacity {
                        self.undo_stack.pop_front();
                    }
                    self.undo_stack.push_back(initial);
                }
                return true;
            }
        }
        false
    }

    /// Records a new state snapshot on the undo stack.
    ///
    /// In accordance with SPEC-008 §2.2:
    /// 1. Ignores intermediate snapshots while a coalesced group (`begin_grouping`) is active.
    /// 2. Deduplicates consecutive identical snapshots (`no-op` protection).
    /// 3. Drops the oldest entry if the undo stack exceeds capacity.
    /// 4. Unconditionally clears the redo stack.
    pub fn push_snapshot(&mut self, state: T) {
        if self.active_group_initial.is_some() {
            return;
        }
        self.redo_stack.clear();
        if self.undo_stack.back() == Some(&state) {
            return;
        }
        if self.undo_stack.len() >= self.capacity {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(state);
    }

    /// Performs an undo operation:
    /// - Pushes `*current_state` onto the redo stack.
    /// - Restores `*current_state` from the most recent undo snapshot.
    /// - Returns `true` if state was restored, or `false` if `undo_stack` was empty.
    pub fn undo(&mut self, current_state: &mut T) -> bool {
        if let Some(initial) = self.active_group_initial.take() {
            if *current_state != initial {
                if self.redo_stack.len() >= self.capacity {
                    self.redo_stack.pop_front();
                }
                self.redo_stack.push_back(current_state.clone());
                *current_state = initial;
                return true;
            }
        }
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
        self.active_group_initial = None;
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
        self.active_group_initial = None;
    }

    /// Validates all `NSUndoManager` structural invariants (State-Invariant Roundtrip Verification):
    /// 1. `capacity >= 1`
    /// 2. `undo_stack.len() <= capacity` and `redo_stack.len() <= capacity`
    /// 3. Zero consecutive identical snapshots in `undo_stack`
    pub fn validate_invariants(&self) -> Result<(), String> {
        if self.capacity == 0 {
            return Err("NSUndoManager capacity must be >= 1".to_string());
        }
        if self.undo_stack.len() > self.capacity {
            return Err(format!(
                "undo_stack len ({}) exceeds capacity ({})",
                self.undo_stack.len(),
                self.capacity
            ));
        }
        if self.redo_stack.len() > self.capacity {
            return Err(format!(
                "redo_stack len ({}) exceeds capacity ({})",
                self.redo_stack.len(),
                self.capacity
            ));
        }
        for i in 1..self.undo_stack.len() {
            if self.undo_stack[i - 1] == self.undo_stack[i] {
                return Err(format!(
                    "duplicate consecutive undo snapshot at index {}",
                    i
                ));
            }
        }
        Ok(())
    }
}
