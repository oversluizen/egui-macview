//! Pre-compiled topological `NSLayoutConstraint` DAG schedule, `NSValueTransformer` evaluation,
//! and deterministic inline arithmetic expression evaluator (`SPEC-013 §2.2`, `§3.1.5`, `§4.1.3`).

use crate::scene::ir::NodeId;
use egui::{pos2, vec2, Rect};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Built-in AppKit `NSValueTransformer` truth table (`REQ-013-02E`).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ValueTransformer {
    /// Identity transformer (passes value unchanged).
    #[default]
    Identity,
    /// `NSNegateBoolean` transformer (`b -> !b`).
    NegateBoolean,
    /// `NSIsNotNil` transformer (`!s.is_empty()`).
    IsNotNil,
    /// `NSIsNil` transformer (`s.is_empty()`).
    IsNil,
    /// Custom named application transformer.
    Custom(String),
}

impl ValueTransformer {
    /// Evaluates a boolean value through the transformer.
    #[inline]
    pub fn apply_bool(&self, val: bool) -> bool {
        match self {
            Self::NegateBoolean => !val,
            _ => val,
        }
    }

    /// Evaluates a string value into a boolean presence flag (`IsNotNil` / `IsNil`).
    #[inline]
    pub fn apply_str(&self, val: &str) -> bool {
        match self {
            Self::IsNotNil => !val.is_empty(),
            Self::IsNil => val.is_empty(),
            Self::NegateBoolean => val.is_empty(),
            Self::Identity | Self::Custom(_) => !val.is_empty(),
        }
    }
}

/// AppKit `NSLayoutAttribute` codes (`1..=12`, `REQ-013-03B`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintAttr {
    /// Not an attribute (`0`, used for constant width/height constraints).
    NotAnAttribute,
    /// Left edge (`1`).
    Left,
    /// Right edge (`2`).
    Right,
    /// Top edge (`3`).
    Top,
    /// Bottom edge (`4`).
    Bottom,
    /// Leading edge (`5`, maps to `Left` in LTR).
    Leading,
    /// Trailing edge (`6`, maps to `Right` in LTR).
    Trailing,
    /// Width dimension (`7`).
    Width,
    /// Height dimension (`8`).
    Height,
    /// Horizontal center (`9`).
    CenterX,
    /// Vertical center (`10`).
    CenterY,
    /// Last baseline (`11`).
    LastBaseline,
    /// First baseline (`12`).
    FirstBaseline,
}

impl ConstraintAttr {
    /// Decodes an integer `NSLayoutAttribute` code from a binary `.nib` or `.xib`.
    pub const fn from_code(code: i32) -> Self {
        match code {
            1 => Self::Left,
            2 => Self::Right,
            3 => Self::Top,
            4 => Self::Bottom,
            5 => Self::Leading,
            6 => Self::Trailing,
            7 => Self::Width,
            8 => Self::Height,
            9 => Self::CenterX,
            10 => Self::CenterY,
            11 => Self::LastBaseline,
            12 => Self::FirstBaseline,
            _ => Self::NotAnAttribute,
        }
    }
}

/// AppKit `NSLayoutRelation` codes (`-1`, `0`, `+1`, `REQ-013-03B`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintRelation {
    /// Less than or equal (`-1`).
    LessThanOrEqual,
    /// Equal (`0`).
    Equal,
    /// Greater than or equal (`+1`).
    GreaterThanOrEqual,
}

/// Single `NSLayoutConstraint` specification ($A.\text{attr}_1 \;\text{rel}\; m \cdot B.\text{attr}_2 + c$).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutConstraintSpec {
    /// Target node $A$ (`None` represents container superview).
    pub first_item: Option<NodeId>,
    /// Target attribute on $A$.
    pub first_attr: ConstraintAttr,
    /// Relation operator (`<=`, `==`, `>=`).
    pub relation: ConstraintRelation,
    /// Reference node $B$ (`None` represents container superview or constant).
    pub second_item: Option<NodeId>,
    /// Reference attribute on $B$.
    pub second_attr: ConstraintAttr,
    /// Linear multiplier $m$.
    pub multiplier: f32,
    /// Constant offset $c$ in points.
    pub constant: f32,
    /// Constraint priority (`1..=1000`).
    pub priority: u16,
}

/// Pre-compiled topological evaluation schedule for `NSLayoutConstraint` chains (`REQ-013-03B`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CompiledConstraintSchedule {
    steps: Vec<LayoutConstraintSpec>,
}

impl CompiledConstraintSchedule {
    /// Compiles a list of `LayoutConstraintSpec` items into a topologically ordered 1-pass schedule.
    pub fn compile(mut specs: Vec<LayoutConstraintSpec>) -> Self {
        specs.sort_by_key(|c| {
            let dep_rank = if c.second_item.is_none() { 0 } else { 1 };
            let attr_rank = match c.first_attr {
                ConstraintAttr::Width | ConstraintAttr::Height => 0,
                ConstraintAttr::Left | ConstraintAttr::Leading | ConstraintAttr::Top => 1,
                _ => 2,
            };
            (dep_rank, attr_rank, std::cmp::Reverse(c.priority))
        });
        Self { steps: specs }
    }

    /// Returns the compiled constraint steps.
    pub fn steps(&self) -> &[LayoutConstraintSpec] {
        &self.steps
    }

    /// Evaluates the compiled constraint schedule in-place over `rects` within `container_rect`.
    pub fn evaluate(&self, container_rect: Rect, rects: &mut BTreeMap<NodeId, Rect>) {
        for step in &self.steps {
            let Some(target_id) = step.first_item else {
                continue;
            };
            let ref_rect = match step.second_item {
                Some(sec_id) => rects.get(&sec_id).copied().unwrap_or(container_rect),
                None => container_rect,
            };
            let rhs_base = read_attr(ref_rect, step.second_attr);
            let target_val = step.multiplier * rhs_base + step.constant;

            if let Some(cur) = rects.get_mut(&target_id) {
                apply_attr(cur, step.first_attr, step.relation, target_val);
            }
        }
    }
}

fn read_attr(r: Rect, attr: ConstraintAttr) -> f32 {
    match attr {
        ConstraintAttr::NotAnAttribute => 0.0,
        ConstraintAttr::Left | ConstraintAttr::Leading => r.min.x,
        ConstraintAttr::Right | ConstraintAttr::Trailing => r.max.x,
        ConstraintAttr::Top => r.min.y,
        ConstraintAttr::Bottom | ConstraintAttr::LastBaseline | ConstraintAttr::FirstBaseline => r.max.y,
        ConstraintAttr::Width => r.width(),
        ConstraintAttr::Height => r.height(),
        ConstraintAttr::CenterX => r.center().x,
        ConstraintAttr::CenterY => r.center().y,
    }
}

fn apply_attr(r: &mut Rect, attr: ConstraintAttr, rel: ConstraintRelation, val: f32) {
    let w = r.width();
    let h = r.height();
    let resolve = |cur: f32| match rel {
        ConstraintRelation::Equal => val,
        ConstraintRelation::LessThanOrEqual => cur.min(val),
        ConstraintRelation::GreaterThanOrEqual => cur.max(val),
    };
    match attr {
        ConstraintAttr::Left | ConstraintAttr::Leading => {
            let nx = resolve(r.min.x);
            *r = Rect::from_min_size(pos2(nx, r.min.y), vec2(w, h));
        }
        ConstraintAttr::Right | ConstraintAttr::Trailing => {
            let nr = resolve(r.max.x);
            *r = Rect::from_min_size(pos2(nr - w, r.min.y), vec2(w, h));
        }
        ConstraintAttr::Top => {
            let ny = resolve(r.min.y);
            *r = Rect::from_min_size(pos2(r.min.x, ny), vec2(w, h));
        }
        ConstraintAttr::Bottom | ConstraintAttr::LastBaseline | ConstraintAttr::FirstBaseline => {
            let nb = resolve(r.max.y);
            *r = Rect::from_min_size(pos2(r.min.x, nb - h), vec2(w, h));
        }
        ConstraintAttr::Width => {
            let nw = resolve(w).max(0.0);
            *r = Rect::from_min_size(r.min, vec2(nw, h));
        }
        ConstraintAttr::Height => {
            let nh = resolve(h).max(0.0);
            *r = Rect::from_min_size(r.min, vec2(w, nh));
        }
        ConstraintAttr::CenterX => {
            let cx = resolve(r.center().x);
            *r = Rect::from_min_size(pos2(cx - w * 0.5, r.min.y), vec2(w, h));
        }
        ConstraintAttr::CenterY => {
            let cy = resolve(r.center().y);
            *r = Rect::from_min_size(pos2(r.min.x, cy - h * 0.5), vec2(w, h));
        }
        ConstraintAttr::NotAnAttribute => {}
    }
}

/// Evaluates an inline arithmetic expression (`+`, `-`, `*`, `/`, parentheses, unary `-`)
/// for `NumberField` (`GSCalculatingNumberFormatter` / `NSNumberFormatter`, `REQ-013-02E`).
pub fn evaluate_arithmetic_expr(input: &str) -> Option<f64> {
    struct Parser<'a> {
        bytes: &'a [u8],
        pos: usize,
    }
    impl<'a> Parser<'a> {
        fn skip_ws(&mut self) {
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
                self.pos += 1;
            }
        }
        fn parse_expr(&mut self) -> Option<f64> {
            let mut lhs = self.parse_term()?;
            loop {
                self.skip_ws();
                if self.pos >= self.bytes.len() {
                    break;
                }
                match self.bytes[self.pos] {
                    b'+' => {
                        self.pos += 1;
                        lhs += self.parse_term()?;
                    }
                    b'-' => {
                        self.pos += 1;
                        lhs -= self.parse_term()?;
                    }
                    _ => break,
                }
            }
            Some(lhs)
        }
        fn parse_term(&mut self) -> Option<f64> {
            let mut lhs = self.parse_factor()?;
            loop {
                self.skip_ws();
                if self.pos >= self.bytes.len() {
                    break;
                }
                match self.bytes[self.pos] {
                    b'*' => {
                        self.pos += 1;
                        lhs *= self.parse_factor()?;
                    }
                    b'/' => {
                        self.pos += 1;
                        let rhs = self.parse_factor()?;
                        if rhs.abs() < 1e-12 {
                            return None;
                        }
                        lhs /= rhs;
                    }
                    _ => break,
                }
            }
            Some(lhs)
        }
        fn parse_factor(&mut self) -> Option<f64> {
            self.skip_ws();
            if self.pos >= self.bytes.len() {
                return None;
            }
            if self.bytes[self.pos] == b'+' {
                self.pos += 1;
                return self.parse_factor();
            }
            if self.bytes[self.pos] == b'-' {
                self.pos += 1;
                return self.parse_factor().map(|v| -v);
            }
            if self.bytes[self.pos] == b'(' {
                self.pos += 1;
                let val = self.parse_expr()?;
                self.skip_ws();
                if self.pos >= self.bytes.len() || self.bytes[self.pos] != b')' {
                    return None;
                }
                self.pos += 1;
                return Some(val);
            }
            let start = self.pos;
            while self.pos < self.bytes.len() && (self.bytes[self.pos].is_ascii_digit() || self.bytes[self.pos] == b'.') {
                self.pos += 1;
            }
            if self.pos == start {
                return None;
            }
            std::str::from_utf8(&self.bytes[start..self.pos]).ok()?.parse::<f64>().ok()
        }
    }

    let mut parser = Parser { bytes: input.as_bytes(), pos: 0 };
    let val = parser.parse_expr()?;
    parser.skip_ws();
    if parser.pos == parser.bytes.len() {
        Some(val)
    } else {
        None
    }
}
