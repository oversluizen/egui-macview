//! Authentic macOS AppKit Stack View (NSStackView equivalent).
//!
//! Grounded in macOS HIG and W3C DTCG tokens via `MacTheme`.
//! Fully data-driven: zero hardcoded dimensions or colors.
//! Features automatic horizontal/vertical component stacking,
//! configurable token-driven spacing and padding, section grouping,
//! and optional hairline divider lines.

use crate::layout::subpixel_snap;
use crate::theme::MacTheme;
use egui::{FontId, Pos2, Rect, Stroke, Ui, UiBuilder, Vec2};

/// Orientation of a [`MacStackView`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacStackOrientation {
    /// Vertical auto-stacking (column layout, default).
    #[default]
    Vertical,
    /// Horizontal auto-stacking (row layout).
    Horizontal,
}

/// Alignment along the cross-axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacStackAlignment {
    /// Align items to the leading edge (left for vertical, top for horizontal).
    #[default]
    Leading,
    /// Center items along the cross-axis.
    Center,
    /// Align items to the trailing edge.
    Trailing,
}

/// Authentic macOS AppKit linear stacking container (`NSStackView` equivalent).
///
/// Supports automatic horizontal or vertical component stacking,
/// configurable token-driven spacing and padding, grouped sections with header labels,
/// and optional hairline divider lines between items.
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacStackView, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
///
/// MacStackView::vertical()
///     .theme(&theme)
///     .spacing(12.0)
///     .show_separators(true)
///     .show(ui, |stack| {
///         stack.add(|ui| {
///             ui.label("First item");
///         });
///         stack.add_section(Some("Section 2"), |ui| {
///             ui.label("Section content");
///         });
///     });
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacStackView<'a> {
    orientation: MacStackOrientation,
    alignment: MacStackAlignment,
    theme: Option<&'a MacTheme>,
    spacing: Option<f32>,
    padding: Option<f32>,
    show_separators: bool,
    width: Option<f32>,
    height: Option<f32>,
}

/// Interactive stacking context passed to closures in [`MacStackView::show`].
pub struct MacStackUi<'a> {
    /// Underlying egui `Ui` instance for embedding widgets inside the stack.
    pub ui: &'a mut Ui,

    theme: &'a MacTheme,
    orientation: MacStackOrientation,
    spacing: f32,
    separator_thickness: f32,
    show_separators: bool,
    container_height: Option<f32>,
    padding: f32,
    item_count: usize,
    section_count: usize,
}

/// Output returned by [`MacStackView::show`].
#[derive(Debug, Clone)]
pub struct MacStackViewOutput<R> {
    /// Return value of the inner closure.
    pub inner: R,
    /// Bounding rectangle enclosing the stacked items and padding.
    pub rect: Rect,
    /// Total count of items added to the stack.
    pub item_count: usize,
    /// Total count of sections added to the stack.
    pub section_count: usize,
}

impl<'a> MacStackUi<'a> {
    /// Adds an item closure to the stack with automatic spacing and separators.
    pub fn add<R>(&mut self, item: impl FnOnce(&mut Ui) -> R) -> R {
        if self.item_count > 0 {
            if self.show_separators {
                self.draw_separator();
            } else {
                self.advance_spacing();
            }
        }
        self.item_count += 1;
        item(self.ui)
    }

    /// Adds a grouped section with an optional section header label.
    pub fn add_section<R>(
        &mut self,
        header: Option<&str>,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> R {
        if self.item_count > 0 || self.section_count > 0 {
            self.draw_separator();
        }
        self.section_count += 1;

        if let Some(title) = header {
            let label_color = self.theme.secondary_label();
            let font_size = self.theme.stack_view().section_header_font_size.points();
            let font_id = FontId::proportional(font_size);
            self.ui.add(
                egui::Label::new(egui::RichText::new(title).font(font_id).color(label_color))
                    .wrap_mode(egui::TextWrapMode::Extend),
            );
            self.advance_spacing();
        }

        self.item_count += 1;
        content(self.ui)
    }

    /// Explicitly inserts a hairline divider line spanning the cross-axis.
    pub fn separator(&mut self) {
        self.draw_separator();
    }

    /// Explicitly inserts custom spacing in points.
    pub fn spacing(&mut self, pts: f32) {
        self.ui.add_space(pts);
    }

    fn advance_spacing(&mut self) {
        self.ui.add_space(self.spacing);
    }

    fn draw_separator(&mut self) {
        self.ui.add_space(self.spacing * 0.5);
        let scale = self.ui.pixels_per_point();
        let th = self.separator_thickness;
        let color = self.theme.separator();

        match self.orientation {
            MacStackOrientation::Vertical => {
                let avail_w = self.ui.available_width();
                let (rect, _) = self
                    .ui
                    .allocate_exact_size(Vec2::new(avail_w, th), egui::Sense::hover());
                let y = subpixel_snap(rect.center().y, scale);
                self.ui.painter().line_segment(
                    [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                    Stroke::new(th, color),
                );
            }
            MacStackOrientation::Horizontal => {
                let avail_h = if let Some(ch) = self.container_height {
                    (ch - 2.0 * self.padding).max(1.0)
                } else {
                    let current_h = self.ui.min_rect().height();
                    if current_h > 0.0 {
                        current_h
                    } else {
                        self.theme.stack_view().default_item_height.points()
                    }
                };
                let (rect, _) = self
                    .ui
                    .allocate_exact_size(Vec2::new(th, avail_h), egui::Sense::hover());
                let x = subpixel_snap(rect.center().x, scale);
                self.ui.painter().line_segment(
                    [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                    Stroke::new(th, color),
                );
            }
        }
        self.ui.add_space(self.spacing * 0.5);
    }
}

impl<'a> MacStackView<'a> {
    /// Creates a new stack view with the specified orientation.
    pub fn new(orientation: MacStackOrientation) -> Self {
        Self {
            orientation,
            alignment: MacStackAlignment::Leading,
            theme: None,
            spacing: None,
            padding: None,
            show_separators: false,
            width: None,
            height: None,
        }
    }

    /// Creates a vertical column stack view (default).
    pub fn vertical() -> Self {
        Self::new(MacStackOrientation::Vertical)
    }

    /// Creates a horizontal row stack view.
    pub fn horizontal() -> Self {
        Self::new(MacStackOrientation::Horizontal)
    }

    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Sets cross-axis alignment.
    pub fn alignment(mut self, alignment: MacStackAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Customizes spacing between adjacent elements in points.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = Some(spacing);
        self
    }

    /// Customizes outer edge padding around all stacked elements in points.
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Enables automatic hairline separators between adjacent items and sections.
    pub fn show_separators(mut self, show: bool) -> Self {
        self.show_separators = show;
        self
    }

    /// Sets container width constraint in points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets container height constraint in points.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Renders the linear stack and executes the items closure.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut MacStackUi<'_>) -> R,
    ) -> MacStackViewOutput<R> {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;

        let metrics = theme.stack_view();
        let spacing = self
            .spacing
            .unwrap_or_else(|| metrics.default_spacing.points());
        let padding = self
            .padding
            .unwrap_or_else(|| metrics.edge_padding.points());
        let separator_thickness = metrics.separator_thickness.points();

        let available = ui.available_rect_before_wrap();
        let target_w = self.width.unwrap_or(available.width());
        let target_h = self.height.unwrap_or(available.height());
        let container_rect = Rect::from_min_size(available.min, Vec2::new(target_w, target_h));

        let inner_rect = container_rect.shrink(padding);

        let stack_id = ui.next_auto_id();
        let mut child_ui = match self.orientation {
            MacStackOrientation::Vertical => {
                let layout = egui::Layout::top_down(match self.alignment {
                    MacStackAlignment::Leading => egui::Align::Min,
                    MacStackAlignment::Center => egui::Align::Center,
                    MacStackAlignment::Trailing => egui::Align::Max,
                });
                ui.new_child(
                    UiBuilder::new()
                        .id_salt((stack_id, "stack_v"))
                        .max_rect(inner_rect)
                        .layout(layout),
                )
            }
            MacStackOrientation::Horizontal => {
                let layout = egui::Layout::left_to_right(match self.alignment {
                    MacStackAlignment::Leading => egui::Align::Min,
                    MacStackAlignment::Center => egui::Align::Center,
                    MacStackAlignment::Trailing => egui::Align::Max,
                });
                ui.new_child(
                    UiBuilder::new()
                        .id_salt((stack_id, "stack_h"))
                        .max_rect(inner_rect)
                        .layout(layout),
                )
            }
        };

        let mut stack_ui = MacStackUi {
            ui: &mut child_ui,
            theme,
            orientation: self.orientation,
            spacing,
            separator_thickness,
            show_separators: self.show_separators,
            container_height: self.height,
            padding,
            item_count: 0,
            section_count: 0,
        };

        let inner = add_contents(&mut stack_ui);
        let items_rect = stack_ui.ui.min_rect();
        let content_w = if items_rect.is_positive() {
            items_rect.width() + padding * 2.0
        } else {
            padding * 2.0
        };
        let content_h = if items_rect.is_positive() {
            items_rect.height() + padding * 2.0
        } else {
            padding * 2.0
        };
        let final_w = self.width.unwrap_or(content_w).max(content_w);
        let final_h = self.height.unwrap_or(content_h).max(content_h);
        let final_rect = Rect::from_min_size(container_rect.min, Vec2::new(final_w, final_h));

        ui.allocate_rect(final_rect, egui::Sense::hover());

        MacStackViewOutput {
            inner,
            rect: final_rect,
            item_count: stack_ui.item_count,
            section_count: stack_ui.section_count,
        }
    }
}
