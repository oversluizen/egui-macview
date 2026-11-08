//! Authentic macOS AppKit Path Control (`NSPathControl` equivalent, SPEC-011).

use crate::symbols::MacSymbol;
use crate::theme::MacTheme;
use egui::{
    Align2, Color32, CornerRadius, FontId, Id, Painter, Pos2, Rect, Response, Sense, Stroke,
    StrokeKind, Ui, Vec2,
};
use std::fmt::Debug;
use std::hash::Hash;

/// Single breadcrumb component inside [`MacPathControl`].
#[derive(Debug, Clone)]
pub struct MacPathComponent<'a> {
    /// Display title of the path segment.
    pub title: &'a str,
    /// True if this segment represents a folder/directory; false for a leaf document/file.
    pub is_folder: bool,
    /// Optional explicit [`MacSymbol`] override.
    pub symbol: Option<MacSymbol>,
}

impl<'a> MacPathComponent<'a> {
    /// Creates a folder path component.
    pub fn folder(title: &'a str) -> Self {
        Self {
            title,
            is_folder: true,
            symbol: None,
        }
    }

    /// Creates a leaf file/document path component.
    pub fn file(title: &'a str) -> Self {
        Self {
            title,
            is_folder: false,
            symbol: None,
        }
    }

    /// Overrides the leading symbol with a [`MacSymbol`].
    pub fn with_symbol(mut self, symbol: MacSymbol) -> Self {
        self.symbol = Some(symbol);
        self
    }
}

/// Interaction output returned by [`MacPathControl::show`].
#[derive(Debug, Clone)]
pub struct MacPathControlResponse {
    /// Overall widget interaction response.
    pub response: Response,
    /// Index of the path segment clicked during this frame, if any.
    pub clicked_segment: Option<usize>,
    /// True if the optional trailing `Choose...` button was clicked during this frame.
    pub choose_clicked: bool,
}

/// Authentic macOS AppKit breadcrumb path bar (`NSPathControl` equivalent).
pub struct MacPathControl<'a> {
    id: Id,
    components: Vec<MacPathComponent<'a>>,
    show_choose_button: bool,
    choose_label: &'a str,
    width: Option<f32>,
    enabled: bool,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacPathControl<'a> {
    /// Creates a new `MacPathControl` from a slice of string segment names (all folders except last).
    pub fn from_segments(id_source: impl Hash + Debug, segments: &'a [&'a str]) -> Self {
        let n = segments.len();
        let comps: Vec<_> = segments
            .iter()
            .enumerate()
            .map(|(i, &title)| MacPathComponent {
                title,
                is_folder: i + 1 < n,
                symbol: None,
            })
            .collect();
        Self::new(id_source, &comps)
    }

    /// Creates a new `MacPathControl` from explicit [`MacPathComponent`] items.
    pub fn new(id_source: impl Hash + Debug, components: &[MacPathComponent<'a>]) -> Self {
        #[allow(deprecated)]
        let id = Id::new(id_source);
        Self {
            id,
            components: components.to_vec(),
            show_choose_button: false,
            choose_label: "Choose...",
            width: None,
            enabled: true,
            theme: None,
        }
    }

    /// Shows or hides the trailing `Choose...` button.
    pub fn show_choose_button(mut self, show: bool) -> Self {
        self.show_choose_button = show;
        self
    }

    /// Sets custom label for the trailing choose button.
    pub fn choose_label(mut self, label: &'a str) -> Self {
        self.choose_label = label;
        self
    }

    /// Overrides total width in points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Enables or disables interaction.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Injects dynamic [`MacTheme`].
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Renders the path control bar in `ui`.
    pub fn show(self, ui: &mut Ui) -> MacPathControlResponse {
        let default_theme = MacTheme::from_ctx(ui.ctx());
        let theme = self.theme.unwrap_or(&default_theme);
        let pcm = theme.path_control();

        let width = self.width.unwrap_or_else(|| ui.available_width());
        let height = pcm.height.points();
        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), sense);

        let painter = ui.painter_at(rect);
        let corner_r = pcm.corner_radius.points();
        let border_w = pcm.border_width.points();
        let border_stroke = Stroke::new(border_w, theme.separator());
        painter.rect(
            rect,
            CornerRadius::same(corner_r as u8),
            theme.control_background(),
            border_stroke,
            StrokeKind::Inside,
        );

        let font_id = FontId::proportional(pcm.font_size.points());
        let pad_h = pcm.padding_horizontal.points();
        let seg_pad_h = pcm.segment_padding_horizontal.points();
        let seg_spacing = pcm.segment_spacing.points();
        let icon_sz = pcm.icon_size.points();
        let icon_gap = pcm.icon_spacing.points();
        let chev_sz = pcm.chevron_size.points();
        let seg_r = CornerRadius::same(pcm.segment_corner_radius.points() as u8);

        let mut choose_clicked = false;
        let right_limit = if self.show_choose_button {
            let lbl = self.choose_label.to_owned();
            let choose_text_w = ui.fonts_mut(|f| {
                f.layout_no_wrap(lbl, font_id.clone(), theme.label())
                    .size()
                    .x
            });
            let btn_w = choose_text_w + pcm.choose_button_padding.points() * 2.0;
            let btn_h = pcm.choose_button_height.points();
            let btn_pos = Pos2::new(
                rect.right() - pad_h - btn_w,
                rect.top() + (height - btn_h) * 0.5,
            );
            let btn_rect = Rect::from_min_size(btn_pos, Vec2::new(btn_w, btn_h));
            let btn_resp = ui.interact(btn_rect, self.id.with("choose_btn"), sense);
            if btn_resp.clicked() && self.enabled {
                choose_clicked = true;
            }
            let down = btn_resp.is_pointer_button_down_on();
            let btn_bg = if down {
                theme.selected_control()
            } else if btn_resp.hovered() {
                theme.control()
            } else {
                theme.window_background()
            };
            let btn_fg = if down {
                theme
                    .selected_control_background_style()
                    .primary_text_color(theme)
            } else if self.enabled {
                theme.label()
            } else {
                theme.placeholder_text()
            };
            painter.rect(btn_rect, seg_r, btn_bg, border_stroke, StrokeKind::Inside);
            painter.text(
                btn_rect.center(),
                Align2::CENTER_CENTER,
                self.choose_label,
                font_id.clone(),
                btn_fg,
            );
            btn_rect.left() - pad_h
        } else {
            rect.right() - pad_h
        };

        let mut cursor_x = rect.left() + pad_h;
        let mut clicked_segment = None;
        let count = self.components.len();
        let text_color = if self.enabled {
            theme.label()
        } else {
            theme.placeholder_text()
        };

        for (idx, comp) in self.components.iter().enumerate() {
            if cursor_x >= right_limit {
                break;
            }
            let title_str = comp.title.to_owned();
            let title_w = ui.fonts_mut(|f| {
                f.layout_no_wrap(title_str, font_id.clone(), theme.label())
                    .size()
                    .x
            });
            let seg_w = (seg_pad_h * 2.0 + icon_sz + icon_gap + title_w)
                .min((right_limit - cursor_x).max(0.0));
            let seg_rect = Rect::from_min_size(
                Pos2::new(cursor_x, rect.top() + border_w),
                Vec2::new(seg_w, height - border_w * 2.0),
            );
            let seg_resp = ui.interact(seg_rect, self.id.with(("seg", idx)), sense);
            if seg_resp.clicked() && self.enabled {
                clicked_segment = Some(idx);
            }
            if seg_resp.hovered() && self.enabled {
                painter.rect_filled(seg_rect, seg_r, theme.control());
            }

            let seg_clip = seg_rect.intersect(Rect::from_min_max(
                rect.min,
                Pos2::new(right_limit, rect.max.y),
            ));
            let seg_painter = painter.with_clip_rect(seg_clip);
            let icon_center =
                Pos2::new(seg_rect.left() + seg_pad_h + icon_sz * 0.5, rect.center().y);
            let icon_rect = Rect::from_center_size(icon_center, Vec2::splat(icon_sz));
            paint_component_icon(&seg_painter, theme, comp, icon_rect);

            let text_pos = Pos2::new(icon_rect.right() + icon_gap, rect.center().y);
            seg_painter.text(
                text_pos,
                Align2::LEFT_CENTER,
                comp.title,
                font_id.clone(),
                text_color,
            );

            cursor_x = seg_rect.right() + seg_spacing;
            if idx + 1 < count && cursor_x + chev_sz < right_limit {
                let chev_pos = Pos2::new(cursor_x + chev_sz * 0.5, rect.center().y);
                let chev_font = FontId::proportional(chev_sz);
                painter.text(
                    chev_pos,
                    Align2::CENTER_CENTER,
                    MacSymbol::ChevronRight.as_str(),
                    chev_font,
                    theme.secondary_label(),
                );
                cursor_x += chev_sz + seg_spacing;
            }
        }

        MacPathControlResponse {
            response,
            clicked_segment,
            choose_clicked,
        }
    }
}

/// Paints the leading folder, file, or override symbol for a path segment.
fn paint_component_icon(
    painter: &Painter,
    theme: &MacTheme,
    comp: &MacPathComponent<'_>,
    icon_rect: Rect,
) {
    let pcm = theme.path_control();
    let border_w = pcm.border_width.points();
    let r = CornerRadius::same((border_w * 2.0) as u8);
    if let Some(sym) = comp.symbol {
        let font = FontId::proportional(pcm.font_size.points());
        painter.text(
            icon_rect.center(),
            Align2::CENTER_CENTER,
            sym.as_str(),
            font,
            theme.control_accent(),
        );
    } else if comp.is_folder {
        let tab_w = pcm.folder_tab_width.points();
        let tab_h = pcm.folder_tab_height.points();
        let body_min = Pos2::new(
            icon_rect.left() + border_w,
            icon_rect.top() + tab_h + border_w,
        );
        let body_max = Pos2::new(
            icon_rect.right() - border_w,
            icon_rect.bottom() - border_w * 2.0,
        );
        let body_rect = Rect::from_min_max(body_min, body_max);
        let tab_rect = Rect::from_min_size(
            Pos2::new(body_rect.left(), body_rect.top() - tab_h),
            Vec2::new(tab_w, tab_h + border_w),
        );
        painter.rect_filled(
            tab_rect,
            CornerRadius::same(border_w as u8),
            theme.control_accent(),
        );
        painter.rect_filled(body_rect, r, theme.control_accent());
    } else {
        let inset_x = border_w * 2.0 + border_w;
        let inset_y = border_w * 2.0;
        let doc_min = Pos2::new(icon_rect.left() + inset_x, icon_rect.top() + inset_y);
        let doc_max = Pos2::new(icon_rect.right() - inset_x, icon_rect.bottom() - inset_y);
        let doc_rect = Rect::from_min_max(doc_min, doc_max);
        painter.rect(
            doc_rect,
            r,
            Color32::TRANSPARENT,
            Stroke::new(border_w, theme.secondary_label()),
            StrokeKind::Inside,
        );
    }
}
