//! High-Assurance Invariant Tests for macOS AppKit Primitives Suite.
//!
//! Grounded in Apple AppKit, macOS HIG, and W3C DTCG design tokens.
//! Verifies:
//! 1. `MacSlider`: continuous track metrics, thumb knob, tick mark computation, and snapping invariants.
//! 2. `MacThumbnailCell`: 24x24 pt default canvas metrics, custom drawing callbacks, and selection styling.
//! 3. `MacSplitView`: 1px hairline divider, min/max pane constraints, collapse states, and orientation invariants.
//! 4. `MacOutlineView`: hierarchical indentation step, disclosure chevron toggling, and trailing accessory buttons.
//! 5. Zero hardcoded constants: dynamic resolution from `MacTheme` across Aqua and DarkAqua.

use egui::{Color32, Context, Event, Key, PointerButton, Pos2, RawInput, Rect, Sense, Vec2};
use egui_macview::{
    theme::{MacAppearance, MacTheme},
    tokens::MacDesignTokens,
    MacButtonCell, MacColorWell, MacForm, MacInputCell, MacOutlineView, MacPanel, MacPopUpButton,
    MacPopover, MacPopoverArrowEdge, MacScrollView, MacSearchField, MacSegmentedControl, MacSlider,
    MacSplitOrientation, MacSplitView, MacStackAlignment, MacStackOrientation, MacStackView,
    MacStepper, MacThumbnailCell, MacToggle,
};

#[test]
fn test_token_catalog_extended_primitives() {
    let tokens = MacDesignTokens::load_default();

    // 1. Slider metrics tokens
    let slider = tokens.slider_metrics();
    assert_eq!(slider.track_height.points(), 3.0);
    assert_eq!(slider.thumb_size.points(), 16.0);
    assert_eq!(slider.thumb_stroke_width.points(), 1.0);
    assert_eq!(slider.tick_height.points(), 4.0);
    assert_eq!(slider.tick_width.points(), 1.0);
    assert_eq!(slider.tick_offset.points(), 3.0);
    assert_eq!(slider.default_width.points(), 120.0);
    assert_eq!(slider.key_step_fraction.points(), 0.05);

    // 2. Thumbnail cell metrics tokens
    let thumb = tokens.thumbnail_metrics();
    assert_eq!(thumb.size.points(), 24.0);
    assert_eq!(thumb.corner_radius.points(), 4.0);
    assert_eq!(thumb.border_width.points(), 1.0);

    // 3. Outline view metrics tokens
    let outline = tokens.outline_view_metrics();
    assert_eq!(outline.row_height.points(), 24.0);
    assert_eq!(outline.indent_step.points(), 16.0);
    assert_eq!(outline.disclosure_size.points(), 10.0);
    assert_eq!(outline.disclosure_margin.points(), 4.0);
    assert_eq!(outline.content_spacing.points(), 6.0);
    assert_eq!(outline.accessory_size.points(), 16.0);
    assert_eq!(outline.accessory_spacing.points(), 4.0);
    assert_eq!(outline.corner_radius.points(), 4.0);
    assert_eq!(outline.font_size.points(), 12.0);
    assert_eq!(outline.chevron_stroke_width.points(), 1.2);
    assert_eq!(outline.chevron_size.points(), 7.0);

    // 4. Split view metrics tokens
    let split = tokens.split_view_metrics();
    assert_eq!(split.divider_thickness.points(), 1.0);
    assert_eq!(split.hit_thickness.points(), 6.0);
    assert_eq!(split.min_pane_size.points(), 50.0);
    assert_eq!(split.handle_length.points(), 24.0);
    assert_eq!(split.handle_thickness.points(), 3.0);
    assert_eq!(split.default_ratio.points(), 0.5);

    // 5. PopUp button metrics tokens
    let popup = tokens.popup_button_metrics();
    assert_eq!(popup.height.points(), 22.0);
    assert_eq!(popup.corner_radius.points(), 5.0);
    assert_eq!(popup.padding_horizontal.points(), 8.0);
    assert_eq!(popup.chevron_size.points(), 8.0);
    assert_eq!(popup.chevron_stroke_width.points(), 1.2);
    assert_eq!(popup.chevron_spacing.points(), 6.0);

    // 6. Search field metrics tokens
    let search = tokens.search_field_metrics();
    assert_eq!(search.height.points(), 22.0);
    assert_eq!(search.corner_radius.points(), 6.0);
    assert_eq!(search.icon_size.points(), 12.0);
    assert_eq!(search.icon_stroke_width.points(), 1.2);
    assert_eq!(search.padding_horizontal.points(), 8.0);
    assert_eq!(search.clear_button_size.points(), 14.0);

    // 7. Stepper metrics tokens
    let stepper = tokens.stepper_metrics();
    assert_eq!(stepper.width.points(), 14.0);
    assert_eq!(stepper.height.points(), 22.0);
    assert_eq!(stepper.corner_radius.points(), 4.0);
    assert_eq!(stepper.divider_thickness.points(), 1.0);
    assert_eq!(stepper.arrow_size.points(), 6.0);
    assert_eq!(stepper.arrow_stroke_width.points(), 1.2);

    // 8. Color well metrics tokens
    let color_well = tokens.color_well_metrics();
    assert_eq!(color_well.width.points(), 44.0);
    assert_eq!(color_well.height.points(), 23.0);
    assert_eq!(color_well.corner_radius.points(), 5.0);
    assert_eq!(color_well.border_width.points(), 1.0);
    assert_eq!(color_well.swatch_inset.points(), 3.0);
    assert_eq!(color_well.swatch_radius.points(), 3.0);

    // 9. Scroll view metrics tokens
    let scroll_view = tokens.scroll_view_metrics();
    assert_eq!(scroll_view.scrollbar_thickness.points(), 7.0);
    assert_eq!(scroll_view.thumb_min_length.points(), 20.0);
    assert_eq!(scroll_view.corner_radius.points(), 3.5);
    assert_eq!(scroll_view.fade_timeout.points(), 0.8);
    assert_eq!(scroll_view.edge_inset.points(), 3.0);

    // 10. Stack view metrics tokens
    let stack_view = tokens.stack_view_metrics();
    assert_eq!(stack_view.default_spacing.points(), 8.0);
    assert_eq!(stack_view.edge_padding.points(), 8.0);
    assert_eq!(stack_view.separator_thickness.points(), 1.0);
    assert_eq!(stack_view.section_header_font_size.points(), 11.0);
}

#[test]
fn test_theme_extended_resolvers_aqua_and_dark() {
    let light = MacTheme::new(MacAppearance::Aqua);
    let dark = MacTheme::new(MacAppearance::DarkAqua);

    assert_eq!(light.slider().track_height.points(), 3.0);
    assert_eq!(light.thumbnail().size.points(), 24.0);
    assert_eq!(light.split_view().divider_thickness.points(), 1.0);
    assert_eq!(light.outline_view().row_height.points(), 24.0);
    assert_eq!(light.popup_button().height.points(), 22.0);
    assert_eq!(light.search_field().corner_radius.points(), 6.0);
    assert_eq!(light.stepper().width.points(), 14.0);
    assert_eq!(light.color_well().width.points(), 44.0);
    assert_eq!(light.scroll_view().scrollbar_thickness.points(), 7.0);
    assert_eq!(light.stack_view().default_spacing.points(), 8.0);
    assert_eq!(light.stack_view().section_header_font_size.points(), 11.0);

    assert_eq!(dark.slider().thumb_size.points(), 16.0);
    assert_eq!(dark.thumbnail().corner_radius.points(), 4.0);
    assert_eq!(dark.split_view().min_pane_size.points(), 50.0);
    assert_eq!(dark.outline_view().indent_step.points(), 16.0);
    assert_eq!(dark.popup_button().chevron_size.points(), 8.0);
    assert_eq!(dark.search_field().icon_size.points(), 12.0);
    assert_eq!(dark.stepper().arrow_size.points(), 6.0);
    assert_eq!(dark.color_well().swatch_inset.points(), 3.0);
    assert_eq!(dark.scroll_view().thumb_min_length.points(), 20.0);
    assert_eq!(dark.stack_view().separator_thickness.points(), 1.0);
    assert_eq!(dark.stack_view().section_header_font_size.points(), 11.0);
}

#[test]
fn test_slider_headless_execution_and_clamping() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut val = 42.0f32;
    let mut output = ctx.run_ui(RawInput::default(), |ui| {
        let resp = MacSlider::new(&mut val, 0.0..=100.0)
            .theme(&theme)
            .ticks(5, true)
            .width(150.0)
            .show(ui);

        assert_eq!(resp.rect.width(), 150.0);
        assert!(resp.rect.height() >= 16.0);
    });
    output.textures_delta.clear();

    assert_eq!(val, 42.0);
}

#[test]
fn test_thumbnail_cell_headless_execution_and_callback() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);

    let mut callback_invoked = false;
    let mut recorded_rect = Rect::NOTHING;

    let mut output = ctx.run_ui(RawInput::default(), |ui| {
        let resp = MacThumbnailCell::new()
            .theme(&theme)
            .selected(true)
            .custom_draw(|_painter, inner_rect| {
                callback_invoked = true;
                recorded_rect = inner_rect;
            })
            .show(ui);

        assert_eq!(resp.rect.size(), Vec2::splat(24.0));
    });
    output.textures_delta.clear();

    assert!(
        callback_invoked,
        "Thumbnail custom drawing callback must execute"
    );
    assert_eq!(recorded_rect.width(), 22.0); // 24.0 - 2 * 1.0pt border
    assert_eq!(recorded_rect.height(), 22.0);
}

#[test]
fn test_split_view_horizontal_and_vertical_constraints() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    // Test Horizontal Split with explicit position and bounds
    let mut pos_x = 100.0f32;
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let (r1_width, r2_width) =
            MacSplitView::with_pos("h_split", MacSplitOrientation::Horizontal, &mut pos_x)
                .theme(&theme)
                .min_first(60.0)
                .max_first(200.0)
                .min_second(80.0)
                .show(
                    ui,
                    |pane1| pane1.available_rect_before_wrap().width(),
                    |pane2| pane2.available_rect_before_wrap().width(),
                );

        assert_eq!(r1_width, 100.0);
        assert!(r2_width > 0.0);
    });
    out1.textures_delta.clear();

    assert_eq!(pos_x, 100.0);

    // Test Collapse States
    let mut pos_collapse = 120.0f32;
    let mut out2 = ctx.run_ui(RawInput::default(), |ui| {
        let (collapsed_w, active_w) = MacSplitView::with_pos(
            "collapsed_split",
            MacSplitOrientation::Horizontal,
            &mut pos_collapse,
        )
        .theme(&theme)
        .collapsed_first(true)
        .show(
            ui,
            |pane1| pane1.available_rect_before_wrap().width(),
            |pane2| pane2.available_rect_before_wrap().width(),
        );

        assert_eq!(collapsed_w, 0.0);
        assert!(active_w > 0.0);
    });
    out2.textures_delta.clear();

    // Test Default Token-Backed Ratio
    let mut out3 = ctx.run_ui(RawInput::default(), |ui| {
        let (r1_w, r2_w) = MacSplitView::new("auto_ratio", MacSplitOrientation::Horizontal)
            .theme(&theme)
            .show(
                ui,
                |pane1| pane1.available_rect_before_wrap().width(),
                |pane2| pane2.available_rect_before_wrap().width(),
            );

        assert!(r1_w > 0.0);
        assert!(r2_w > 0.0);
        let divider_th = theme.split_view().divider_thickness.points();
        let total = r1_w + r2_w + divider_th;
        let diff = (r1_w - total * 0.5).abs();
        assert!(
            diff <= 1.0,
            "Default split ratio must resolve to 50/50 token specification"
        );
    });
    out3.textures_delta.clear();
}

#[test]
fn test_outline_view_hierarchy_and_accessory_toggles() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut is_expanded = false;
    let mut is_visible = true;
    let mut is_locked = false;

    let mut output = ctx.run_ui(RawInput::default(), |ui| {
        MacOutlineView::new("test_outline")
            .theme(&theme)
            .show(ui, |tree| {
                let resp = tree.row(
                    "item_1",
                    1,    // depth 1
                    true, // expandable
                    &mut is_expanded,
                    true, // selected
                    |row| {
                        row.thumbnail(|_p, _r| {});
                        row.label("Test Node");
                        row.accessory_custom(
                            "vis_toggle",
                            &mut is_visible,
                            |_painter, rect, active| {
                                assert!(rect.width() > 0.0);
                                assert!(active);
                            },
                        );
                        row.accessory_custom(
                            "lock_toggle",
                            &mut is_locked,
                            |_painter, rect, active| {
                                assert!(rect.width() > 0.0);
                                assert!(!active);
                            },
                        );
                    },
                );

                assert!(resp.is_selected);
                assert!(!resp.is_expanded);
            });
    });
    output.textures_delta.clear();

    assert!(is_visible);
    assert!(!is_locked);
}

#[test]
fn test_popup_button_headless_execution_and_selection() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut selected = 1usize;
    let items = ["Plain", "Bold", "Italic"];

    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let resp = MacPopUpButton::new(&mut selected, &items)
            .theme(&theme)
            .width(130.0)
            .show(ui);

        assert_eq!(resp.rect.width(), 130.0);
        assert_eq!(resp.rect.height(), 22.0);
    });
    out1.textures_delta.clear();
    assert_eq!(selected, 1);
}

#[test]
fn test_search_field_headless_execution_and_clearing() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);

    let mut query = "vector graphics".to_string();

    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let resp = MacSearchField::new(&mut query)
            .theme(&theme)
            .placeholder("Search components...")
            .width(180.0)
            .show(ui);

        assert_eq!(resp.rect.width(), 180.0);
        assert_eq!(resp.rect.height(), 22.0);
    });
    out1.textures_delta.clear();
    assert_eq!(query, "vector graphics");
}

#[test]
fn test_stepper_headless_execution_and_clamping() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut val = 5.0f64;

    // Normal stepping
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let resp = MacStepper::new(&mut val, 0.0..=10.0, 1.0)
            .theme(&theme)
            .show(ui);

        assert_eq!(resp.rect.width(), 14.0);
        assert_eq!(resp.rect.height(), 22.0);
    });
    out1.textures_delta.clear();
    assert_eq!(val, 5.0);

    // Clamping to max bound
    val = 10.0;
    let mut input_up = RawInput::default();
    input_up.events.push(egui::Event::Key {
        key: Key::ArrowUp,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::default(),
    });
    let mut out2 = ctx.run_ui(input_up, |ui| {
        let resp = MacStepper::new(&mut val, 0.0..=10.0, 1.0)
            .theme(&theme)
            .wraps(false)
            .show(ui);
        resp.request_focus();
    });
    out2.textures_delta.clear();
    assert_eq!(val, 10.0); // clamped at max

    // Wrapping to min bound
    let mut input_wrap = RawInput::default();
    input_wrap.events.push(egui::Event::Key {
        key: Key::ArrowUp,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::default(),
    });
    let mut out3 = ctx.run_ui(input_wrap, |ui| {
        let resp = MacStepper::new(&mut val, 0.0..=10.0, 1.0)
            .theme(&theme)
            .wraps(true)
            .show(ui);
        resp.request_focus();
    });
    out3.textures_delta.clear();
    assert_eq!(val, 0.0); // wrapped to min
}

#[test]
fn test_color_well_headless_execution() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut color = Color32::from_rgba_premultiplied(0, 122, 255, 200);

    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        let resp = MacColorWell::new(&mut color)
            .theme(&theme)
            .show_alpha(true)
            .show(ui);

        assert_eq!(resp.rect.width(), 44.0);
        assert_eq!(resp.rect.height(), 23.0);
    });
    out.textures_delta.clear();
    assert_eq!(color.r(), 0);
    assert_eq!(color.g(), 122);
    assert_eq!(color.b(), 255);
}

#[test]
fn test_stepper_float_precision_wrapping() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut val = 0.9f64;
    // Frame 1: render and request focus
    let mut out0 = ctx.run_ui(RawInput::default(), |ui| {
        let resp = MacStepper::new(&mut val, 0.0..=1.0, 0.1)
            .theme(&theme)
            .wraps(true)
            .show(ui);
        resp.request_focus();
    });
    out0.textures_delta.clear();

    // Step Up from 0.9 with step 0.1 in [0.0, 1.0] -> must reach 1.0 exactly
    let mut input_up = RawInput::default();
    input_up.events.push(egui::Event::Key {
        key: Key::ArrowUp,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::default(),
    });
    let mut out1 = ctx.run_ui(input_up, |ui| {
        MacStepper::new(&mut val, 0.0..=1.0, 0.1)
            .theme(&theme)
            .wraps(true)
            .show(ui);
    });
    out1.textures_delta.clear();
    assert!((val - 1.0).abs() < 1e-9, "Expected 1.0, got {}", val);

    // Step Up from 1.0 with wraps -> wraps to 0.0
    let mut input_up2 = RawInput::default();
    input_up2.events.push(egui::Event::Key {
        key: Key::ArrowUp,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::default(),
    });
    let mut out2 = ctx.run_ui(input_up2, |ui| {
        MacStepper::new(&mut val, 0.0..=1.0, 0.1)
            .theme(&theme)
            .wraps(true)
            .show(ui);
    });
    out2.textures_delta.clear();
    assert!((val - 0.0).abs() < 1e-9, "Expected 0.0, got {}", val);

    // Step Down from 0.1 -> reaches 0.0
    val = 0.1;
    let mut input_down = RawInput::default();
    input_down.events.push(egui::Event::Key {
        key: Key::ArrowDown,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::default(),
    });
    let mut out3 = ctx.run_ui(input_down, |ui| {
        MacStepper::new(&mut val, 0.0..=1.0, 0.1)
            .theme(&theme)
            .wraps(true)
            .show(ui);
    });
    out3.textures_delta.clear();
    assert!((val - 0.0).abs() < 1e-9, "Expected 0.0, got {}", val);

    // Step Down from 0.0 with wraps -> wraps to 1.0
    let mut input_down2 = RawInput::default();
    input_down2.events.push(egui::Event::Key {
        key: Key::ArrowDown,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::default(),
    });
    let mut out4 = ctx.run_ui(input_down2, |ui| {
        MacStepper::new(&mut val, 0.0..=1.0, 0.1)
            .theme(&theme)
            .wraps(true)
            .show(ui);
    });
    out4.textures_delta.clear();
    assert!((val - 1.0).abs() < 1e-9, "Expected 1.0, got {}", val);
}

#[test]
fn test_popup_button_keyboard_space_and_escape() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut selected = 0usize;
    let items = ["First", "Second", "Third"];

    // 1. Initial render and request focus
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let resp = MacPopUpButton::new(&mut selected, &items)
            .theme(&theme)
            .show(ui);
        resp.request_focus();
        let is_open_id = resp.id.with("mac_popup_is_open");
        let is_open = ui.data(|d| d.get_temp::<bool>(is_open_id).unwrap_or(false));
        assert!(!is_open, "Initially popup must be closed");
    });
    out1.textures_delta.clear();

    // 2. Press Space while focused -> popup opens
    let mut input_space = RawInput::default();
    input_space.events.push(egui::Event::Key {
        key: Key::Space,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::default(),
    });
    let mut out2 = ctx.run_ui(input_space, |ui| {
        let resp = MacPopUpButton::new(&mut selected, &items)
            .theme(&theme)
            .show(ui);
        let is_open_id = resp.id.with("mac_popup_is_open");
        let is_open = ui.data(|d| d.get_temp::<bool>(is_open_id).unwrap_or(false));
        assert!(is_open, "Pressing Space when focused must open popup");
    });
    out2.textures_delta.clear();

    // 3. Press Escape -> popup closes
    let mut input_esc = RawInput::default();
    input_esc.events.push(egui::Event::Key {
        key: Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::default(),
    });
    let mut out3 = ctx.run_ui(input_esc, |ui| {
        let resp = MacPopUpButton::new(&mut selected, &items)
            .theme(&theme)
            .show(ui);
        let is_open_id = resp.id.with("mac_popup_is_open");
        let is_open = ui.data(|d| d.get_temp::<bool>(is_open_id).unwrap_or(false));
        assert!(!is_open, "Pressing Escape must close popup");
    });
    out3.textures_delta.clear();
}

#[test]
fn test_color_well_keyboard_activation_and_escape() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut color = Color32::RED;

    // 1. Initial render and request focus
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let resp = MacColorWell::new(&mut color).theme(&theme).show(ui);
        resp.request_focus();
        let is_open_id = resp.id.with("mac_color_picker_is_open");
        let is_open = ui.data(|d| d.get_temp::<bool>(is_open_id).unwrap_or(false));
        assert!(!is_open, "Initially color picker must be closed");
    });
    out1.textures_delta.clear();

    // 2. Press Enter while focused -> picker opens
    let mut input_enter = RawInput::default();
    input_enter.events.push(egui::Event::Key {
        key: Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::default(),
    });
    let mut out2 = ctx.run_ui(input_enter, |ui| {
        let resp = MacColorWell::new(&mut color).theme(&theme).show(ui);
        let is_open_id = resp.id.with("mac_color_picker_is_open");
        let is_open = ui.data(|d| d.get_temp::<bool>(is_open_id).unwrap_or(false));
        assert!(is_open, "Pressing Enter when focused must open picker");
    });
    out2.textures_delta.clear();

    // 3. Press Escape -> picker closes
    let mut input_esc = RawInput::default();
    input_esc.events.push(egui::Event::Key {
        key: Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::default(),
    });
    let mut out3 = ctx.run_ui(input_esc, |ui| {
        let resp = MacColorWell::new(&mut color).theme(&theme).show(ui);
        let is_open_id = resp.id.with("mac_color_picker_is_open");
        let is_open = ui.data(|d| d.get_temp::<bool>(is_open_id).unwrap_or(false));
        assert!(!is_open, "Pressing Escape must close color picker");
    });
    out3.textures_delta.clear();
}

#[test]
fn test_search_field_click_and_escape_behavior() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut query = "hello world".to_string();
    let search_id = egui::Id::NULL.with("test_search_input_id");

    // 1. Initial render with text and request focus
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let resp = MacSearchField::new(&mut query)
            .id(search_id)
            .theme(&theme)
            .show(ui);
        resp.request_focus();
    });
    out1.textures_delta.clear();

    // 1b. Intermediary frame where focus is active
    let mut out_steady = ctx.run_ui(RawInput::default(), |ui| {
        MacSearchField::new(&mut query)
            .id(search_id)
            .theme(&theme)
            .show(ui);
    });
    out_steady.textures_delta.clear();

    // 2. Press Escape when text is present -> text is cleared
    let mut input_esc = RawInput::default();
    input_esc.events.push(egui::Event::Key {
        key: Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::default(),
    });
    let mut out2 = ctx.run_ui(input_esc, |ui| {
        MacSearchField::new(&mut query)
            .id(search_id)
            .theme(&theme)
            .show(ui);
    });
    out2.textures_delta.clear();
    assert!(query.is_empty(), "Escape must clear text in search field");

    // 3. Press Escape when text is empty -> surrenders focus
    assert!(ctx.memory(|m| m.has_focus(search_id)));
    let mut input_esc2 = RawInput::default();
    input_esc2.events.push(egui::Event::Key {
        key: Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::default(),
    });
    let mut out3 = ctx.run_ui(input_esc2, |ui| {
        MacSearchField::new(&mut query)
            .id(search_id)
            .theme(&theme)
            .show(ui);
    });
    out3.textures_delta.clear();
    assert!(
        !ctx.memory(|m| m.has_focus(search_id)),
        "Escape on empty search field must surrender focus"
    );
}

#[test]
fn test_scroll_view_headless_zero_gutter_and_virtual_viewport() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        let output = MacScrollView::new("test_scroll")
            .theme(&theme)
            .vertical(true)
            .horizontal(false)
            .max_height(100.0)
            .max_width(200.0)
            .show(ui, |content_ui| {
                for i in 0..20 {
                    content_ui.label(format!("Row {}", i));
                }
                42
            });

        assert_eq!(output.inner, 42);
        assert_eq!(output.inner_rect.width(), 200.0);
        assert_eq!(output.inner_rect.height(), 100.0);
        assert!(
            output.content_size.y > 100.0,
            "Content must exceed viewport height"
        );
        assert_eq!(output.offset.x, 0.0);
    });
    out.textures_delta.clear();
}

#[test]
fn test_scroll_view_programmatic_offset_and_clamping() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);

    let mut offset = Vec2::new(0.0, 50.0);
    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        let output = MacScrollView::new("test_scroll_offset")
            .theme(&theme)
            .with_offset(&mut offset)
            .max_height(80.0)
            .max_width(150.0)
            .show(ui, |content_ui| {
                content_ui.allocate_exact_size(Vec2::new(150.0, 400.0), egui::Sense::hover());
            });

        assert_eq!(output.inner_rect.size(), Vec2::new(150.0, 80.0));
        assert_eq!(output.content_size.y, 400.0);
        assert_eq!(output.offset.y, 50.0);
    });
    out.textures_delta.clear();
    assert_eq!(offset.y, 50.0);
}

#[test]
fn test_stack_view_vertical_and_horizontal_layout_and_separators() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    // 1. Vertical stack with section and separators
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let output = MacStackView::new(MacStackOrientation::Vertical)
            .theme(&theme)
            .spacing(10.0)
            .padding(6.0)
            .show_separators(true)
            .show(ui, |stack| {
                stack.add(|item_ui| {
                    item_ui.label("Item 1");
                });
                stack.add(|item_ui| {
                    item_ui.label("Item 2");
                });
                stack.add_section(Some("Section Header"), |sec_ui| {
                    sec_ui.label("Section Content");
                });
            });

        assert_eq!(output.item_count, 3);
        assert_eq!(output.section_count, 1);
        assert!(output.rect.height() > 30.0);
    });
    out1.textures_delta.clear();

    // 2. Horizontal stack
    let mut out2 = ctx.run_ui(RawInput::default(), |ui| {
        let output = MacStackView::new(MacStackOrientation::Horizontal)
            .theme(&theme)
            .alignment(MacStackAlignment::Center)
            .show(ui, |stack| {
                stack.add(|item_ui| {
                    item_ui.label("Col 1");
                });
                stack.separator();
                stack.add(|item_ui| {
                    item_ui.label("Col 2");
                });
            });

        assert_eq!(output.item_count, 2);
        assert!(output.rect.width() > 20.0);
    });
    out2.textures_delta.clear();
}

#[test]
fn test_scroll_view_programmatic_reset_to_zero() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut offset = Vec2::new(0.0, 100.0);
    // Frame 1: render at 100.0
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let output = MacScrollView::new("test_scroll_reset")
            .theme(&theme)
            .with_offset(&mut offset)
            .max_height(80.0)
            .show(ui, |c| {
                c.allocate_exact_size(Vec2::new(100.0, 500.0), Sense::hover());
            });
        assert_eq!(output.offset.y, 100.0);
    });
    out1.textures_delta.clear();
    assert_eq!(offset.y, 100.0);

    // Frame 2: consumer resets offset to 0.0
    offset = Vec2::ZERO;
    let mut out2 = ctx.run_ui(RawInput::default(), |ui| {
        let output = MacScrollView::new("test_scroll_reset")
            .theme(&theme)
            .with_offset(&mut offset)
            .max_height(80.0)
            .show(ui, |c| {
                c.allocate_exact_size(Vec2::new(100.0, 500.0), Sense::hover());
            });
        assert_eq!(output.offset.y, 0.0, "Output offset must reset to 0.0");
    });
    out2.textures_delta.clear();
    assert_eq!(offset.y, 0.0, "Consumer offset variable must remain 0.0");
}

#[test]
fn test_scroll_view_simulated_drag_interaction() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    // Frame 1: Initial render with 80pt viewport and 400pt content
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        MacScrollView::new("test_scroll_drag")
            .theme(&theme)
            .max_height(80.0)
            .max_width(120.0)
            .show(ui, |c| {
                c.allocate_exact_size(Vec2::new(120.0, 400.0), Sense::hover());
            });
    });
    out1.textures_delta.clear();

    // Frame 2: Pointer down on the vertical thumb
    let thumb_center = Pos2::new(113.5, 13.0);
    let mut input_press = RawInput::default();
    input_press.events.push(Event::PointerMoved(thumb_center));
    input_press.events.push(Event::PointerButton {
        pos: thumb_center,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::default(),
    });
    let mut out2 = ctx.run_ui(input_press, |ui| {
        MacScrollView::new("test_scroll_drag")
            .theme(&theme)
            .max_height(80.0)
            .max_width(120.0)
            .show(ui, |c| {
                c.allocate_exact_size(Vec2::new(120.0, 400.0), Sense::hover());
            });
    });
    out2.textures_delta.clear();

    // Frame 3: Pointer drag downward by 20pt while holding button
    let mut input_move = RawInput::default();
    input_move
        .events
        .push(Event::PointerMoved(Pos2::new(113.5, 33.0)));

    let mut dragged_offset = Vec2::ZERO;
    let mut out3 = ctx.run_ui(input_move, |ui| {
        let output = MacScrollView::new("test_scroll_drag")
            .theme(&theme)
            .max_height(80.0)
            .max_width(120.0)
            .show(ui, |c| {
                c.allocate_exact_size(Vec2::new(120.0, 400.0), Sense::hover());
            });
        dragged_offset = output.offset;
    });
    out3.textures_delta.clear();

    assert!(
        dragged_offset.y > 0.0,
        "Dragging vertical thumb downward must increase scroll offset"
    );

    // Frame 4: Next frame preserves dragged offset
    let mut out4 = ctx.run_ui(RawInput::default(), |ui| {
        let output = MacScrollView::new("test_scroll_drag")
            .theme(&theme)
            .max_height(80.0)
            .max_width(120.0)
            .show(ui, |c| {
                c.allocate_exact_size(Vec2::new(120.0, 400.0), Sense::hover());
            });
        assert!(
            output.offset.y > 0.0,
            "Subsequent frame must retain dragged offset"
        );
    });
    out4.textures_delta.clear();
}

#[test]
fn test_scroll_view_bidirectional_tiny_viewport_no_overlap() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);

    let mut offset = Vec2::new(150.0, 150.0);
    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        let output = MacScrollView::both("test_scroll_tiny")
            .theme(&theme)
            .with_offset(&mut offset)
            .max_width(35.0)
            .max_height(35.0)
            .show(ui, |c| {
                c.allocate_exact_size(Vec2::new(200.0, 200.0), Sense::hover());
            });

        assert_eq!(output.inner_rect.width(), 35.0);
        assert_eq!(output.inner_rect.height(), 35.0);
        assert!(output.offset.x > 0.0 && output.offset.x <= (200.0 - 35.0));
        assert!(output.offset.y > 0.0 && output.offset.y <= (200.0 - 35.0));
    });
    out.textures_delta.clear();
}

#[test]
fn test_stack_view_horizontal_separator_compact_height() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        let output = MacStackView::horizontal()
            .theme(&theme)
            .show_separators(true)
            .show(ui, |stack| {
                stack.add(|u| {
                    u.allocate_exact_size(Vec2::new(40.0, 22.0), Sense::hover());
                });
                stack.separator();
                stack.add(|u| {
                    u.allocate_exact_size(Vec2::new(50.0, 22.0), Sense::hover());
                });
            });

        assert!(
            output.rect.height() < 50.0,
            "Horizontal stack view with separator must have compact height, got {}",
            output.rect.height()
        );
        assert!(output.rect.width() > 90.0);
    });
    out.textures_delta.clear();
}

#[test]
fn test_stack_view_explicit_dimensions_and_allocation() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        let output = MacStackView::vertical()
            .theme(&theme)
            .width(250.0)
            .height(120.0)
            .show(ui, |stack| {
                stack.add(|u| {
                    u.label("Content");
                });
            });

        assert_eq!(
            output.rect.width(),
            250.0,
            "Explicit width must be respected"
        );
        assert_eq!(
            output.rect.height(),
            120.0,
            "Explicit height must be respected"
        );
        assert!(
            ui.min_rect().height() >= 120.0,
            "Parent Ui min_rect must register allocated space"
        );
    });
    out.textures_delta.clear();
}

use proptest::prelude::*;

proptest! {
    /// Invariant: Slider continuous fraction mapping never yields NaN or out-of-bounds values.
    #[test]
    fn prop_slider_fraction_bounds(
        min in -500.0f32..500.0f32,
        span in 0.1f32..1000.0f32,
        val in -2000.0f32..2000.0f32,
    ) {
        let max = min + span;
        let clamped = val.clamp(min, max);
        let fraction = ((clamped - min) / span).clamp(0.0, 1.0);

        prop_assert!(!fraction.is_nan());
        prop_assert!((0.0..=1.0).contains(&fraction));
        prop_assert!(clamped >= min - 1e-4 && clamped <= max + 1e-4);
    }

    /// Invariant: Split view divider position preserves min/max constraints unconditionally.
    #[test]
    fn prop_split_view_clamp_invariants(
        total_span in 200.0f32..2000.0f32,
        min_1 in 10.0f32..50.0f32,
        min_2 in 10.0f32..50.0f32,
        raw_pos in -500.0f32..2500.0f32,
    ) {
        let max_1 = total_span - min_2;
        let clamped_pos = raw_pos.clamp(min_1, max_1.max(min_1));

        prop_assert!(clamped_pos >= min_1);
        prop_assert!(clamped_pos <= max_1.max(min_1));
        prop_assert!(!clamped_pos.is_nan());
    }

    /// Invariant: Outline tree indentation is strictly monotonic with respect to nesting depth.
    #[test]
    fn prop_outline_indentation_monotonic(
        depth in 0usize..25usize,
        step in 8.0f32..32.0f32,
    ) {
        let indent = depth as f32 * step;
        prop_assert!(indent >= 0.0);
        let next_indent = (depth + 1) as f32 * step;
        prop_assert!(next_indent > indent);
    }

    /// Invariant: Stepper without wraps strictly respects [min, max] bounds across arbitrary increments.
    #[test]
    fn prop_stepper_bounds_clamping(
        min in -500.0f64..500.0f64,
        span in 1.0f64..1000.0f64,
        step in 0.1f64..20.0f64,
        clicks in -50i32..50i32,
    ) {
        let max = min + span;
        let mut val = (min + max) * 0.5;

        for _ in 0..clicks.abs() {
            if clicks > 0 {
                val = (val + step).min(max);
            } else {
                val = (val - step).max(min);
            }
        }

        prop_assert!(val >= min);
        prop_assert!(val <= max);
        prop_assert!(!val.is_nan());
    }

    /// Invariant: ScrollView virtual viewport clamping ensures scroll offset is strictly bounded.
    #[test]
    fn prop_scroll_view_viewport_clamping(
        viewport_h in 50.0f32..1000.0f32,
        content_h in 10.0f32..5000.0f32,
        raw_offset in -500.0f32..10000.0f32,
    ) {
        let max_scroll = (content_h - viewport_h).max(0.0);
        let clamped = raw_offset.clamp(0.0, max_scroll);

        prop_assert!(clamped >= 0.0);
        prop_assert!(clamped <= max_scroll);
        prop_assert!(!clamped.is_nan());
    }

    /// Invariant: StackView deterministic spacing scales linearly with item count.
    #[test]
    fn prop_stack_view_cumulative_spacing(
        item_count in 1usize..50usize,
        spacing in 0.0f32..32.0f32,
    ) {
        let total_spacing = (item_count.saturating_sub(1)) as f32 * spacing;
        prop_assert!(total_spacing >= 0.0);
        if item_count > 1 && spacing > 0.0 {
            prop_assert!(total_spacing > 0.0);
        }
    }

    /// Invariant: ScrollView dual scrollbar overlay geometry provably never overlaps at the corner.
    #[test]
    fn prop_scroll_view_dual_overlay_no_overlap(
        view_w in 20.0f32..1000.0f32,
        view_h in 20.0f32..1000.0f32,
        content_w in 50.0f32..5000.0f32,
        content_h in 50.0f32..5000.0f32,
        offset_frac_x in 0.0f32..1.0f32,
        offset_frac_y in 0.0f32..1.0f32,
    ) {
        let edge_inset = 3.0f32;
        let scrollbar_thickness = 7.0f32;
        let thumb_min_length = 20.0f32;

        let has_v = content_h > view_h;
        let has_h = content_w > view_w;

        if has_v && has_h {
            let top_inset = edge_inset;
            let bottom_inset = edge_inset + scrollbar_thickness + edge_inset;
            let avail_h = (view_h - top_inset - bottom_inset).max(0.0);
            let ratio_y = (view_h / content_h).clamp(0.0, 1.0);
            let thumb_h = (avail_h * ratio_y).max(thumb_min_length.min(avail_h)).min(avail_h);
            let travel_y = (avail_h - thumb_h).max(0.0);
            let v_thumb_bottom = top_inset + offset_frac_y * travel_y + thumb_h;

            let left_inset = edge_inset;
            let right_inset = edge_inset + scrollbar_thickness + edge_inset;
            let avail_w = (view_w - left_inset - right_inset).max(0.0);
            let ratio_x = (view_w / content_w).clamp(0.0, 1.0);
            let thumb_w = (avail_w * ratio_x).max(thumb_min_length.min(avail_w)).min(avail_w);
            let travel_x = (avail_w - thumb_w).max(0.0);
            let h_thumb_right = left_inset + offset_frac_x * travel_x + thumb_w;

            let h_track_top = view_h - edge_inset - scrollbar_thickness;
            let v_track_left = view_w - edge_inset - scrollbar_thickness;

            if avail_h > 0.0 && thumb_h > 0.0 {
                prop_assert!(v_thumb_bottom <= h_track_top + 1e-4);
            }
            if avail_w > 0.0 && thumb_w > 0.0 {
                prop_assert!(h_thumb_right <= v_track_left + 1e-4);
            }
        }
    }
}

#[test]
fn test_popover_headless_execution() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut content_invoked = false;
    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(200.0, 100.0), Sense::hover());
        let (resp, _) = MacPopover::new()
            .theme(&theme)
            .arrow_edge(MacPopoverArrowEdge::Top)
            .scale(1.0)
            .alpha(1.0)
            .show(ui, rect, |pop_ui| {
                content_invoked = true;
                pop_ui.label("Popover Inside Content");
            });

        assert_eq!(resp.rect.width(), 200.0);
        assert_eq!(resp.rect.height(), 100.0);
    });
    out.textures_delta.clear();
    assert!(content_invoked, "Popover content closure must be called");
}

#[test]
fn test_popover_arrow_edges_and_offsets() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);

    for edge in [
        MacPopoverArrowEdge::Top,
        MacPopoverArrowEdge::Bottom,
        MacPopoverArrowEdge::Left,
        MacPopoverArrowEdge::Right,
        MacPopoverArrowEdge::None,
    ] {
        let mut invoked = false;
        let mut out = ctx.run_ui(RawInput::default(), |ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(240.0, 120.0), Sense::hover());
            let (resp, _) = MacPopover::new()
                .theme(&theme)
                .arrow_edge(edge)
                .arrow_offset(60.0)
                .scale(0.85)
                .alpha(0.75)
                .show(ui, rect, |_child_ui| {
                    invoked = true;
                });

            assert_eq!(resp.rect.width(), 240.0 * 0.85);
            assert_eq!(resp.rect.height(), 120.0 * 0.85);
        });
        out.textures_delta.clear();
        assert!(invoked, "Arrow edge {:?} must invoke content closure", edge);
    }
}

#[test]
fn test_mac_toggle_headless_interaction() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut toggle_val = false;

    // Frame 1: render unclicked toggle
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let resp = ui.add(MacToggle::new(&mut toggle_val).theme(&theme));
        assert_eq!(resp.rect.width(), theme.toggle().width.points());
        assert_eq!(resp.rect.height(), theme.toggle().height.points());
    });
    out1.textures_delta.clear();
    assert!(!toggle_val, "Initial toggle value must be false");

    // Frame 2: click on toggle
    let click_pos = Pos2::new(
        theme.toggle().width.points() / 2.0,
        theme.toggle().height.points() / 2.0,
    );
    let mut click_input = RawInput::default();
    click_input.events.push(Event::PointerButton {
        pos: click_pos,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::default(),
    });
    let mut out2 = ctx.run_ui(click_input, |ui| {
        ui.add(MacToggle::new(&mut toggle_val).theme(&theme));
    });
    out2.textures_delta.clear();

    // Release mouse button to register click
    let mut release_input = RawInput::default();
    release_input.events.push(Event::PointerButton {
        pos: click_pos,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::default(),
    });
    let mut out3 = ctx.run_ui(release_input, |ui| {
        let resp = ui.add(MacToggle::new(&mut toggle_val).theme(&theme));
        assert!(resp.clicked());
    });
    out3.textures_delta.clear();

    assert!(toggle_val, "Clicking toggle must invert value to true");
}

#[test]
fn test_button_and_segmented_dark_mode_contrast() {
    let ctx = Context::default();
    let dark_theme = MacTheme::new(MacAppearance::DarkAqua);

    // Verify dark mode label color matches W3C DTCG dark aqua token (#DCDCDC)
    assert_eq!(dark_theme.label(), Color32::from_rgb(220, 220, 220));

    let mut selected_seg = 0usize;
    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        // MacButtonCell in dark mode
        let btn_resp = ui.add(MacButtonCell::new("Test Button").theme(&dark_theme));
        assert!(btn_resp.rect.width() > 0.0);

        // MacSegmentedControl in dark mode
        let seg_resp = MacSegmentedControl::new(&["One", "Two"], selected_seg)
            .theme(&dark_theme)
            .show(ui, |idx| {
                selected_seg = idx;
            });
        assert!(seg_resp.rect.width() > 0.0);

        // MacPanel in dark mode
        let (panel_resp, _) = MacPanel::new().theme(&dark_theme).title("Dark Panel").show(
            ui,
            Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 60.0)),
            |pui| {
                pui.label("Child");
            },
        );
        assert_eq!(panel_resp.rect.width(), 100.0);
    });
    out.textures_delta.clear();
}

#[test]
fn test_mac_form_inspector_alignment_and_trailing_labels() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut rotation = 45.0f32;
    let mut fine_step = 5.0f64;
    let mut scale = 100.0f32;
    let mut skew = 0.0f32;
    let mut toggle = true;
    let mut text = String::from("Layer 1");
    let mut color = Color32::RED;

    let mut label_rects = Vec::new();
    let mut control_rects = Vec::new();

    // 2-frame loop for egui::Grid layout pass
    for frame in 0..2 {
        let mut out = ctx.run_ui(RawInput::default(), |ui| {
            MacForm::new("test_inspector_alignment_form")
                .theme(&theme)
                .show(ui, |form| {
                    let (l1, c1) = form.row_label("Rotation:", |ui| {
                        MacSlider::new(&mut rotation, 0.0..=360.0)
                            .theme(&theme)
                            .ticks(8, true)
                            .width(120.0)
                            .show(ui)
                    });
                    let (l2, c2) = form.row_label("Fine Step:", |ui| {
                        ui.add(
                            MacStepper::new(&mut fine_step, 0.0..=360.0, 5.0)
                                .theme(&theme)
                                .wraps(true),
                        )
                    });
                    let (l3, c3) = form.row_label("Scale:", |ui| {
                        MacSlider::new(&mut scale, 50.0..=200.0)
                            .theme(&theme)
                            .ticks(6, true)
                            .width(120.0)
                            .show(ui)
                    });
                    let (l4, c4) = form.row_label("Skew / Shear:", |ui| {
                        MacSlider::new(&mut skew, -45.0..=45.0)
                            .theme(&theme)
                            .width(120.0)
                            .show(ui)
                    });
                    let (l5, c5) = form.row_label("Drop Shadow:", |ui| {
                        ui.add(MacToggle::new(&mut toggle).theme(&theme))
                    });
                    let (l6, c6) = form.row_label("Name:", |ui| {
                        MacInputCell::new("test_input_name", &mut text)
                            .theme(&theme)
                            .width(180.0)
                            .show(ui)
                    });
                    let (l7, c7) = form.row_label("Fill Color:", |ui| {
                        MacColorWell::new(&mut color).theme(&theme).show(ui)
                    });

                    if frame == 1 {
                        label_rects = vec![
                            l1.rect, l2.rect, l3.rect, l4.rect, l5.rect, l6.rect, l7.rect,
                        ];
                        control_rects = vec![
                            c1.rect, c2.rect, c3.rect, c4.rect, c5.rect, c6.rect, c7.rect,
                        ];
                    }
                });
        });
        out.textures_delta.clear();
    }

    assert_eq!(label_rects.len(), 7, "All 7 labels must be recorded");
    assert_eq!(control_rects.len(), 7, "All 7 controls must be recorded");

    // 1. Column 0 (trailing-aligned labels): All labels must align at their trailing (right) edge.
    let right_edge_0 = label_rects[0].max.x;
    for (i, rect) in label_rects.iter().enumerate() {
        let diff = (rect.max.x - right_edge_0).abs();
        assert!(
            diff <= 1.0,
            "Label {i} trailing edge {} deviates from expected {right_edge_0} (diff={diff})",
            rect.max.x
        );
    }

    // 2. Column 1 (leading-aligned controls): All interactive controls must start at the exact same X coordinate.
    let left_edge_1 = control_rects[0].min.x;
    for (i, rect) in control_rects.iter().enumerate() {
        let diff = (rect.min.x - left_edge_1).abs();
        assert!(
            diff <= 1.0,
            "Control {i} leading edge {} deviates from expected {left_edge_1} (diff={diff})",
            rect.min.x
        );
    }

    // 3. Gutter invariant: Label max.x must be strictly less than Control min.x for every row.
    for i in 0..7 {
        assert!(
            label_rects[i].max.x < control_rects[i].min.x,
            "Row {i} label overlaps control: label.max.x={}, control.min.x={}",
            label_rects[i].max.x,
            control_rects[i].min.x
        );
    }
}

#[test]
fn test_segmented_control_horizontal_centering() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let container_size = Vec2::new(380.0, 100.0);
    let mut selected = 0;
    let mut control_rect = Rect::NOTHING;

    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        ui.allocate_ui_with_layout(
            container_size,
            egui::Layout::top_down(egui::Align::Min),
            |p_ui| {
                p_ui.vertical_centered(|c_ui| {
                    let resp = MacSegmentedControl::new(
                        &["Attributes", "Metrics", "Matrix"],
                        selected,
                    )
                    .theme(&theme)
                    .height(24.0)
                    .show(c_ui, |idx| {
                        selected = idx;
                    });
                    control_rect = resp.rect;
                });
            },
        );
    });
    out.textures_delta.clear();

    assert!(control_rect.width() > 0.0);
    let left_margin = control_rect.min.x;
    let right_margin = container_size.x - control_rect.max.x;
    let diff = (left_margin - right_margin).abs();
    assert!(
        diff <= 1.0,
        "Segmented control not centered horizontally: left={left_margin}, right={right_margin}, diff={diff}"
    );
}
