//! Authentic macOS fuzzy subsequence search and ranking algorithm.
//!
//! Normative Reference: SPEC-007 §4.
//! Grounded in Single Responsibility Principle to maintain GL-1015 modularity ceilings.

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
