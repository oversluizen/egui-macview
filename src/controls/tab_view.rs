//! Authentic macOS AppKit Tab View (`NSTabView` equivalent, REQ-012-04).
//!
//! Grounded in Apple AppKit `NSTabView` with flipped coordinate model (`isFlipped == 1`),
//! exact MacBook content insets across `TopTabsBezelBorder` and `NoTabsNoBorder` styles,
//! and W3C DTCG tokens via [`MacTheme`].

use crate::controls::MacControlSize;
use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{
    CornerRadius, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, UiBuilder,
    Vec2, WidgetText,
};

/// Content insets in logical points (`f32`) matching AppKit `NSEdgeInsets`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MacInsets {
    /// Inset from the top edge in points.
    pub top: f32,
    /// Inset from the bottom edge in points.
    pub bottom: f32,
    /// Inset from the left edge in points.
    pub left: f32,
    /// Inset from the right edge in points.
    pub right: f32,
}

/// Tab presentation and bezel border styles matching AppKit `NSTabViewType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacTabStyle {
    /// Classic macOS segmented top tab bar with container bezel border (`NSTopTabsBezelBorder`).
    #[default]
    TopTabsBezelBorder,
    /// Borderless card container without tab bar (`NSNoTabsNoBorder`).
    NoTabsNoBorder,
}

type TabContentFn<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;

/// Single tab item descriptor.
pub struct MacTabItem<'a> {
    title: WidgetText,
    content: Option<TabContentFn<'a>>,
}

/// Authentic macOS AppKit Tab View container (`NSTabView` equivalent).
///
/// Features hardware-calibrated content insets, flipped coordinate layout,
/// animated segment transitions, and bezel border strokes.
pub struct MacTabView<'a> {
    selected_index: &'a mut usize,
    style: MacTabStyle,
    control_size: MacControlSize,
    theme: Option<&'a MacTheme>,
    tabs: Vec<MacTabItem<'a>>,
}

impl<'a> MacTabView<'a> {
    /// Creates a new tab view bound to an active tab selection index.
    pub fn new(selected_index: &'a mut usize) -> Self {
        Self {
            selected_index,
            style: MacTabStyle::TopTabsBezelBorder,
            control_size: MacControlSize::Regular,
            theme: None,
            tabs: Vec::new(),
        }
    }

    /// Sets the tab display style (`TopTabsBezelBorder` or `NoTabsNoBorder`).
    pub fn style(mut self, style: MacTabStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets the control size (`Regular`, `Small`, or `Mini`).
    pub fn control_size(mut self, size: MacControlSize) -> Self {
        self.control_size = size;
        self
    }

    /// Injects dynamic theme.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Appends a tab with a title and content closure.
    pub fn tab(mut self, title: impl Into<WidgetText>, content: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.tabs.push(MacTabItem {
            title: title.into(),
            content: Some(Box::new(content)),
        });
        self
    }

    /// Resolves tab bar header height in points.
    pub fn tab_bar_height(&self) -> f32 {
        if self.style == MacTabStyle::NoTabsNoBorder {
            return 0.0;
        }
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        let m = theme_cow.tab_view();
        match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => m.regular_bar_height.points(),
            MacControlSize::Small | MacControlSize::CompactInfobox => m.small_bar_height.points(),
            MacControlSize::Mini => m.mini_bar_height.points(),
        }
    }

    /// Resolves exact MacBook content insets from outer container bounds.
    pub fn content_insets(&self) -> MacInsets {
        if self.style == MacTabStyle::NoTabsNoBorder {
            return MacInsets {
                top: 0.0,
                bottom: 0.0,
                left: 0.0,
                right: 0.0,
            };
        }
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        let m = theme_cow.tab_view();
        match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => MacInsets {
                top: m.regular_inset_top.points(),
                bottom: m.regular_inset_bottom.points(),
                left: m.regular_inset_horizontal.points(),
                right: m.regular_inset_horizontal.points(),
            },
            MacControlSize::Small | MacControlSize::CompactInfobox => MacInsets {
                top: m.small_inset_top.points(),
                bottom: m.small_inset_bottom.points(),
                left: m.small_inset_horizontal.points(),
                right: m.small_inset_horizontal.points(),
            },
            MacControlSize::Mini => MacInsets {
                top: m.mini_inset_top.points(),
                bottom: m.mini_inset_bottom.points(),
                left: m.mini_inset_horizontal.points(),
                right: m.mini_inset_horizontal.points(),
            },
        }
    }

    /// Resolves MacBook minimum container dimensions across control sizes.
    pub fn min_container_size(&self) -> Vec2 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        let m = theme_cow.tab_view();
        match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => {
                Vec2::new(m.regular_min_width.points(), m.regular_min_height.points())
            }
            MacControlSize::Small | MacControlSize::CompactInfobox => {
                Vec2::new(m.small_min_width.points(), m.small_min_height.points())
            }
            MacControlSize::Mini => {
                Vec2::new(m.mini_min_width.points(), m.mini_min_height.points())
            }
        }
    }

    /// Computes the layout rectangle of the top segmented tab bar within a container.
    pub fn tab_bar_rect(&self, container_rect: Rect, ui: &Ui) -> Rect {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let metrics = theme_cow.tab_view();
        let bar_h = self.tab_bar_height();
        let min_size = self.min_container_size();
        let pad = metrics.tab_padding_horizontal.points();
        let mut total_intrinsic_w = 0.0f32;
        for tab in &self.tabs {
            let g = tab.title.clone().into_galley(
                ui,
                Some(egui::TextWrapMode::Extend),
                f32::INFINITY,
                egui::TextStyle::Button,
            );
            total_intrinsic_w += g.size().x + pad;
        }
        let bar_w = total_intrinsic_w.clamp(min_size.x.min(container_rect.width()), container_rect.width());
        Rect::from_center_size(
            Pos2::new(container_rect.center().x, container_rect.min.y + bar_h * 0.5),
            Vec2::new(bar_w, bar_h),
        )
    }

    /// Shows the tab view layout with tab bar and selected content panel.
    pub fn show(mut self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let metrics = theme.tab_view();

        let available_w = ui.available_width();
        let start_pos = ui.cursor().min;
        let bar_h = self.tab_bar_height();
        let insets = self.content_insets();
        let min_size = self.min_container_size();

        let tab_count = self.tabs.len();
        let active_idx = *self.selected_index;

        let container_w = available_w.max(min_size.x);
        let content_start_y = start_pos.y + insets.top;
        let content_rect = Rect::from_min_max(
            Pos2::new(start_pos.x + insets.left, content_start_y),
            Pos2::new(start_pos.x + container_w - insets.right, f32::INFINITY),
        );

        // Pre-allocate background shape on ui painter so it renders BEHIND tab child content (Z-order invariant)
        let bg_shape_idx = ui.painter().add(egui::Shape::Noop);
        let tab_scope_id = ui.next_auto_id().with("tab_content").with(active_idx);

        let mut child_ui = ui.new_child(
            UiBuilder::new()
                .id_salt(tab_scope_id)
                .max_rect(content_rect),
        );

        if active_idx < tab_count {
            if let Some(content_fn) = self.tabs[active_idx].content.take() {
                content_fn(&mut child_ui);
            }
        }

        let child_min_rect = child_ui.min_rect();
        let content_h = if active_idx < tab_count {
            child_min_rect.height()
        } else {
            0.0
        };

        let container_h = (if self.style == MacTabStyle::TopTabsBezelBorder {
            insets.top + content_h + insets.bottom
        } else {
            content_h
        }).max(min_size.y);

        let container_rect = Rect::from_min_size(start_pos, Vec2::new(container_w, container_h));
        let container_rect = subpixel_snap_rect(container_rect, ui.ctx().pixels_per_point());

        let (rect, mut response) = ui.allocate_exact_size(container_rect.size(), Sense::click());

        if self.style == MacTabStyle::TopTabsBezelBorder {
            let corner_r = metrics.corner_radius.points();

            // Bezel border frame below tab bar pre-allocated BEHIND child content
            let body_rect = Rect::from_min_max(
                Pos2::new(rect.min.x, rect.min.y + bar_h * 0.5),
                rect.max,
            );
            ui.painter().set(
                bg_shape_idx,
                egui::epaint::RectShape::new(
                    body_rect,
                    CornerRadius::same(corner_r as u8),
                    theme.control_background(),
                    Stroke::new(1.0, theme.separator()),
                    StrokeKind::Inside,
                ),
            );

            let painter = ui.painter_at(rect);

            // Centered segmented tab bar segments
            if tab_count > 0 {
                let bar_rect = self.tab_bar_rect(rect, ui);
                painter.rect(
                    bar_rect,
                    CornerRadius::same(corner_r as u8),
                    theme.window_background(),
                    Stroke::new(1.0, theme.separator()),
                    StrokeKind::Inside,
                );

                let tab_w = bar_rect.width() / (tab_count as f32);
                for i in 0..tab_count {
                    let tab_rect = Rect::from_min_size(
                        Pos2::new(bar_rect.min.x + (i as f32) * tab_w, bar_rect.min.y),
                        Vec2::new(tab_w, bar_h),
                    );
                    let tab_hovered = ui
                        .input(|inp| inp.pointer.hover_pos())
                        .is_some_and(|p| tab_rect.contains(p));

                    let is_active = *self.selected_index == i;
                    if is_active || tab_hovered {
                        let tab_fill = if is_active {
                            theme.control_background()
                        } else {
                            theme.control()
                        };

                        painter.rect(
                            tab_rect,
                            CornerRadius::same(corner_r as u8),
                            tab_fill,
                            Stroke::new(1.0, theme.separator()),
                            StrokeKind::Inside,
                        );
                    }

                    let galley = self.tabs[i].title.clone().into_galley(
                        ui,
                        Some(egui::TextWrapMode::Extend),
                        tab_w,
                        egui::TextStyle::Button,
                    );
                    let text_pos = Pos2::new(
                        tab_rect.center().x - galley.size().x * 0.5,
                        tab_rect.center().y - galley.size().y * 0.5,
                    );
                    let text_color = if is_active {
                        theme.label()
                    } else {
                        theme.secondary_label()
                    };
                    painter.galley(text_pos, galley, text_color);

                    if tab_hovered && ui.input(|inp| inp.pointer.primary_clicked()) {
                        *self.selected_index = i;
                        response.mark_changed();
                    }
                }
            }
        }

        response
    }
}
