//! Authentic macOS fuzzy subsequence search, ranking algorithm, and command palette item models.
//!
//! Normative Reference: SPEC-007 §4.
//! Grounded in Single Responsibility Principle to maintain modularity ceilings.

use super::item::MacKeyboardShortcut;

/// Trait for types that can be converted into a shortcut string representation.
pub trait IntoShortcutString {
    /// Converts into a human-readable shortcut string.
    fn into_shortcut_string(self) -> String;
}

impl IntoShortcutString for String {
    fn into_shortcut_string(self) -> String {
        self
    }
}

impl IntoShortcutString for &str {
    fn into_shortcut_string(self) -> String {
        self.to_string()
    }
}

impl IntoShortcutString for MacKeyboardShortcut {
    fn into_shortcut_string(self) -> String {
        self.format_text()
    }
}

/// Individual registered command entry.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandPaletteItem {
    /// Action identifier returned when command is executed.
    pub id: String,
    /// Human-readable command title label.
    pub title: String,
    /// Optional explanatory subtitle.
    pub subtitle: Option<String>,
    /// Optional command group/category name (e.g. "File", "View").
    pub category: Option<String>,
    /// Optional keyboard shortcut display string (e.g. "Ctrl+S").
    pub shortcut: Option<String>,
    /// Keywords for query search matching.
    pub keywords: Vec<String>,
}

impl CommandPaletteItem {
    /// Constructs a basic command palette item.
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: None,
            category: None,
            shortcut: None,
            keywords: Vec::new(),
        }
    }

    /// Sets the subtitle.
    pub fn subtitle(mut self, sub: impl Into<String>) -> Self {
        self.subtitle = Some(sub.into());
        self
    }

    /// Sets the category group.
    pub fn category(mut self, cat: impl Into<String>) -> Self {
        self.category = Some(cat.into());
        self
    }

    /// Sets keyboard shortcut display string or MacKeyboardShortcut.
    pub fn shortcut(mut self, sc: impl IntoShortcutString) -> Self {
        self.shortcut = Some(sc.into_shortcut_string());
        self
    }

    /// Sets search keywords.
    pub fn keywords(mut self, kw: &[impl AsRef<str>]) -> Self {
        self.keywords = kw.iter().map(|s| s.as_ref().to_string()).collect();
        self
    }
}

/// Computes a fuzzy match score for pattern against text.
/// Returns `Some(score)` if pattern is a subsequence of text (case-insensitive), or `None` otherwise.
pub fn fuzzy_match_score(pattern: &str, text: &str) -> Option<i32> {
    if pattern.is_empty() {
        return Some(0);
    }
    let p_chars: Vec<char> = pattern.to_lowercase().chars().collect();
    let t_chars: Vec<char> = text.to_lowercase().chars().collect();

    let mut p_idx = 0;
    let mut score = 0;
    let mut prev_matched_idx: Option<usize> = None;

    for (t_idx, &t_ch) in t_chars.iter().enumerate() {
        if p_idx < p_chars.len() && t_ch == p_chars[p_idx] {
            if let Some(prev) = prev_matched_idx {
                if prev + 1 == t_idx {
                    score += 10;
                }
            }
            if t_idx == 0 || !t_chars[t_idx - 1].is_alphanumeric() {
                score += 15;
            } else {
                score += 1;
            }
            prev_matched_idx = Some(t_idx);
            p_idx += 1;
        }
    }

    if p_idx == p_chars.len() {
        Some(score)
    } else {
        None
    }
}
