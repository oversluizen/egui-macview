//! Authentic macOS AppKit Rule Editor (`NSRuleEditor` equivalent, REQ-012-07).
//!
//! Grounded in Apple AppKit `NSRuleEditor` with customizable predicate rows,
//! slice criteria pop-up pills, and right-aligned [+] / [-] action buttons.

use crate::theme::MacTheme;
use egui::{CornerRadius, Id, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget};
use std::hash::Hash;

/// Single predicate rule row descriptor for `MacRuleEditor`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacRuleRow {
    /// Selected index for each criterion slice in this row.
    pub slices: Vec<usize>,
}

impl MacRuleRow {
    /// Creates a new rule row with initial slice selections.
    pub fn new(slices: impl Into<Vec<usize>>) -> Self {
        Self {
            slices: slices.into(),
        }
    }
}

/// Authentic macOS AppKit Rule Editor (`NSRuleEditor` equivalent).
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacRuleEditor, MacRuleRow, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::Aqua);
/// let mut rows = vec![MacRuleRow::new(vec![0, 0, 0])];
/// let criteria: &[&[&str]] = &[
///     &["Name", "Kind", "Size"],
///     &["contains", "is", "begins with"],
///     &["Document", "Image", "Audio"],
/// ];
///
/// MacRuleEditor::new("filter_rules", &mut rows, criteria)
///     .theme(&theme)
///     .show(ui);
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacRuleEditor<'a> {
    id_salt: Id,
    rows: &'a mut Vec<MacRuleRow>,
    criteria: &'a [&'a [&'a str]],
    theme: Option<&'a MacTheme>,
}

impl<'a> MacRuleEditor<'a> {
    /// Creates a new rule editor widget bound to a mutable row list and slice options.
    pub fn new(
        id_salt: impl Hash + std::fmt::Debug,
        rows: &'a mut Vec<MacRuleRow>,
        criteria: &'a [&'a [&'a str]],
    ) -> Self {
        Self {
            id_salt: Id::NULL.with(id_salt),
            rows,
            criteria,
            theme: None,
        }
    }

    /// Injects dynamic theme.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Resolves predicate row height in points ($32.0\,\text{pt}$).
    pub fn row_height(&self) -> f32 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        theme_cow.rule_editor().row_height.points()
    }

    /// Resolves action [+] / [-] button size in points ($20.0\,\text{pt}$).
    pub fn button_size(&self) -> f32 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        theme_cow.rule_editor().add_remove_button_size.points()
    }

    /// Resolves horizontal spacing between rule clauses in points ($8.0\,\text{pt}$).
    pub fn row_spacing(&self) -> f32 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        theme_cow.rule_editor().row_horizontal_spacing.points()
    }

    /// Shows the rule editor widget inside `ui`.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl<'a> Widget for MacRuleEditor<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let metrics = theme.rule_editor();

        let row_h = metrics.row_height.points();
        let btn_sz = metrics.add_remove_button_size.points();
        let spacing = metrics.row_horizontal_spacing.points();
        let row_count = self.rows.len();

        let available_w = ui.available_width();
        let total_h = (row_count as f32) * row_h;
        let (rect, mut response) = ui.allocate_exact_size(Vec2::new(available_w, total_h), Sense::hover());

        let mut row_to_add = None;
        let mut row_to_remove = None;

        for r_idx in 0..row_count {
            let row_y = rect.min.y + (r_idx as f32) * row_h;
            let row_rect = Rect::from_min_size(Pos2::new(rect.min.x, row_y), Vec2::new(available_w, row_h));
            let painter = ui.painter_at(row_rect);

            // Draw row separator hairline
            if r_idx + 1 < row_count {
                painter.line_segment(
                    [row_rect.left_bottom(), row_rect.right_bottom()],
                    Stroke::new(1.0, theme.separator()),
                );
            }

            // Right-aligned [-] and [+] action buttons
            let plus_rect = Rect::from_min_size(
                Pos2::new(row_rect.max.x - btn_sz, row_y + (row_h - btn_sz) * 0.5),
                Vec2::new(btn_sz, btn_sz),
            );
            let minus_rect = Rect::from_min_size(
                Pos2::new(plus_rect.min.x - btn_sz - spacing * 0.5, row_y + (row_h - btn_sz) * 0.5),
                Vec2::new(btn_sz, btn_sz),
            );

            // [+] button
            let plus_id = self.id_salt.with("plus").with(r_idx);
            let plus_resp = ui.interact(plus_rect, plus_id, Sense::click());
            let plus_hovered = plus_resp.hovered();
            painter.rect(
                plus_rect,
                CornerRadius::same((btn_sz * 0.5) as u8),
                if plus_hovered { theme.control() } else { theme.control_background() },
                Stroke::new(1.0, theme.separator()),
                StrokeKind::Inside,
            );
            let plus_center = plus_rect.center();
            let arm = (btn_sz * 0.5) * 0.5;
            painter.line_segment([Pos2::new(plus_center.x - arm, plus_center.y), Pos2::new(plus_center.x + arm, plus_center.y)], Stroke::new(1.0, theme.label()));
            painter.line_segment([Pos2::new(plus_center.x, plus_center.y - arm), Pos2::new(plus_center.x, plus_center.y + arm)], Stroke::new(1.0, theme.label()));

            let is_plus_clicked = plus_resp.clicked()
                || (ui.input(|i| i.pointer.primary_clicked())
                    && ui.input(|i| i.pointer.interact_pos()).is_some_and(|pos| plus_rect.contains(pos)));
            if is_plus_clicked {
                row_to_add = Some(r_idx + 1);
                response.mark_changed();
            }

            // [-] button
            let minus_id = self.id_salt.with("minus").with(r_idx);
            let minus_resp = ui.interact(minus_rect, minus_id, Sense::click());
            let minus_hovered = minus_resp.hovered();
            painter.rect(
                minus_rect,
                CornerRadius::same((btn_sz * 0.5) as u8),
                if minus_hovered { theme.control() } else { theme.control_background() },
                Stroke::new(1.0, theme.separator()),
                StrokeKind::Inside,
            );
            let minus_center = minus_rect.center();
            painter.line_segment([Pos2::new(minus_center.x - arm, minus_center.y), Pos2::new(minus_center.x + arm, minus_center.y)], Stroke::new(1.0, theme.label()));

            let is_minus_clicked = minus_resp.clicked()
                || (ui.input(|i| i.pointer.primary_clicked())
                    && ui.input(|i| i.pointer.interact_pos()).is_some_and(|pos| minus_rect.contains(pos)));
            if is_minus_clicked {
                row_to_remove = Some(r_idx);
                response.mark_changed();
            }

            // Criteria clause pills
            let mut cur_x = row_rect.min.x + spacing;
            let row = &mut self.rows[r_idx];
            for (c_idx, &options) in self.criteria.iter().enumerate() {
                if c_idx >= row.slices.len() {
                    row.slices.push(0);
                }
                let sel = row.slices[c_idx].min(options.len().saturating_sub(1));
                let label = options.get(sel).copied().unwrap_or("");

                let text_style = ui.style().text_styles.get(&egui::TextStyle::Button).cloned().unwrap_or_default();
                let galley = painter.layout_no_wrap(label.to_string(), text_style, theme.label());
                let pill_w = galley.size().x + spacing * 2.0;
                let pill_h = btn_sz;
                let pill_rect = Rect::from_min_size(
                    Pos2::new(cur_x, row_y + (row_h - pill_h) * 0.5),
                    Vec2::new(pill_w, pill_h),
                );

                let pill_id = self.id_salt.with("pill").with(r_idx).with(c_idx);
                let pill_resp = ui.interact(pill_rect, pill_id, Sense::click());
                let pill_hovered = pill_resp.hovered();
                painter.rect(
                    pill_rect,
                    CornerRadius::same((pill_h * 0.5) as u8),
                    if pill_hovered { theme.control() } else { theme.control_background() },
                    Stroke::new(1.0, theme.separator()),
                    StrokeKind::Inside,
                );
                painter.galley(
                    Pos2::new(pill_rect.center().x - galley.size().x * 0.5, pill_rect.center().y - galley.size().y * 0.5),
                    galley,
                    theme.label(),
                );

                let is_pill_clicked = pill_resp.clicked()
                    || (ui.input(|i| i.pointer.primary_clicked())
                        && ui.input(|i| i.pointer.interact_pos()).is_some_and(|pos| pill_rect.contains(pos)));
                if is_pill_clicked && !options.is_empty() {
                    row.slices[c_idx] = (sel + 1) % options.len();
                    response.mark_changed();
                }

                cur_x += pill_w + spacing;
            }
        }

        if let Some(ins_idx) = row_to_add {
            let new_slices = if let Some(last) = self.rows.get(ins_idx.saturating_sub(1)) {
                last.slices.clone()
            } else {
                vec![0; self.criteria.len()]
            };
            self.rows.insert(ins_idx.min(self.rows.len()), MacRuleRow::new(new_slices));
        }

        if let Some(del_idx) = row_to_remove {
            if self.rows.len() > 1 && del_idx < self.rows.len() {
                self.rows.remove(del_idx);
            }
        }

        response
    }
}
