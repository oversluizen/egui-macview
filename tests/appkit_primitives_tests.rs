//! High-Assurance Invariant Tests for macOS AppKit Core Control Primitives.
//!
//! Grounded in Apple AppKit, macOS HIG, and W3C DTCG design tokens.
//! Verifies:
//! 1. `MacSlider`, `MacThumbnailCell`, `MacPopUpButton`, `MacSearchField`, `MacStepper`, `MacColorWell`, `MacToggle`, `MacSegmentedControl`, `MacButtonCell`.
//! 2. `MacTheme::from_ctx` inheritance when `.theme(...)` is omitted.
//! 3. `.enabled(false)` disabled state invariants.

use egui::{Color32, Context, Event, Key, PointerButton, Pos2, RawInput, Rect, Vec2};
use egui_macview::{
    theme::{MacAppearance, MacTheme},
    tokens::MacDesignTokens,
    MacButtonCell, MacColorWell, MacPanel, MacPopUpButton, MacSearchField, MacSegmentedControl,
    MacSlider, MacStepper, MacThumbnailCell, MacToggle,
};

#[test]
fn test_token_catalog_extended_primitives() {
    let tokens = MacDesignTokens::load_default();

    let slider = tokens.slider_metrics();
    assert_eq!(slider.track_height.points(), 3.0);
    assert_eq!(slider.thumb_size.points(), 16.0);
    assert_eq!(slider.default_width.points(), 120.0);

    let thumb = tokens.thumbnail_metrics();
    assert_eq!(thumb.size.points(), 24.0);
    assert_eq!(thumb.corner_radius.points(), 4.0);

    let outline = tokens.outline_view_metrics();
    assert_eq!(outline.row_height.points(), 24.0);
    assert_eq!(outline.indent_step.points(), 16.0);
    assert_eq!(outline.accessory_size.points(), 16.0);

    let split = tokens.split_view_metrics();
    assert_eq!(split.divider_thickness.points(), 1.0);
    assert_eq!(split.min_pane_size.points(), 50.0);

    let popup = tokens.popup_button_metrics();
    assert_eq!(popup.height.points(), 22.0);
    assert_eq!(popup.corner_radius.points(), 5.0);

    let search = tokens.search_field_metrics();
    assert_eq!(search.height.points(), 22.0);
    assert_eq!(search.corner_radius.points(), 6.0);

    let stepper = tokens.stepper_metrics();
    assert_eq!(stepper.width.points(), 14.0);
    assert_eq!(stepper.height.points(), 22.0);

    let color_well = tokens.color_well_metrics();
    assert_eq!(color_well.width.points(), 44.0);
    assert_eq!(color_well.height.points(), 23.0);
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

    assert_eq!(dark.slider().thumb_size.points(), 16.0);
    assert_eq!(dark.thumbnail().corner_radius.points(), 4.0);
    assert_eq!(dark.split_view().min_pane_size.points(), 50.0);
    assert_eq!(dark.outline_view().indent_step.points(), 16.0);
    assert_eq!(dark.popup_button().chevron_size.points(), 8.0);
    assert_eq!(dark.search_field().icon_size.points(), 12.0);
    assert_eq!(dark.stepper().arrow_size.points(), 6.0);
    assert_eq!(dark.color_well().swatch_inset.points(), 3.0);
}

#[test]
fn test_slider_and_thumbnail_headless_execution() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut val = 42.0f32;
    let mut callback_invoked = false;
    let mut recorded_rect = Rect::NOTHING;

    let mut output = ctx.run_ui(RawInput::default(), |ui| {
        let resp = MacSlider::new(&mut val, 0.0..=100.0)
            .theme(&theme)
            .ticks(5, true)
            .width(150.0)
            .show(ui);
        assert_eq!(resp.rect.width(), 150.0);

        let t_resp = MacThumbnailCell::new()
            .theme(&theme)
            .selected(true)
            .custom_draw(|_painter, inner_rect| {
                callback_invoked = true;
                recorded_rect = inner_rect;
            })
            .show(ui);
        assert_eq!(t_resp.rect.size(), Vec2::splat(24.0));
    });
    output.textures_delta.clear();

    assert_eq!(val, 42.0);
    assert!(callback_invoked);
    assert_eq!(recorded_rect.width(), 22.0);
}

#[test]
fn test_popup_search_and_color_well_headless_execution() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut selected = 1usize;
    let items = ["Plain", "Bold", "Italic"];
    let mut query = "vector graphics".to_string();
    let mut color = Color32::from_rgba_unmultiplied(0, 122, 255, 200);

    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        let p_resp = MacPopUpButton::new(&mut selected, &items)
            .theme(&theme)
            .width(130.0)
            .show(ui);
        assert_eq!(p_resp.rect.size(), Vec2::new(130.0, 22.0));

        let s_resp = MacSearchField::new(&mut query)
            .theme(&theme)
            .placeholder("Search...")
            .width(180.0)
            .show(ui);
        assert_eq!(s_resp.rect.size(), Vec2::new(180.0, 22.0));

        let c_resp = MacColorWell::new(&mut color)
            .theme(&theme)
            .show_alpha(true)
            .show(ui);
        assert_eq!(c_resp.rect.size(), Vec2::new(44.0, 23.0));
    });
    out.textures_delta.clear();

    let srgba = color.to_srgba_unmultiplied();
    assert_eq!(srgba[0], 0);
    assert!((srgba[1] as i32 - 122).abs() <= 1);
    assert_eq!(srgba[2], 255);
}

#[test]
fn test_stepper_headless_execution_clamping_and_precision_wrapping() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut val = 0.9f64;
    let mut out0 = ctx.run_ui(RawInput::default(), |ui| {
        let resp = MacStepper::new(&mut val, 0.0..=1.0, 0.1)
            .theme(&theme)
            .wraps(true)
            .show(ui);
        assert_eq!(resp.rect.size(), Vec2::new(14.0, 22.0));
        resp.request_focus();
    });
    out0.textures_delta.clear();

    let mut input_up = RawInput::default();
    input_up.events.push(Event::Key {
        key: Key::ArrowUp,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::default(),
    });
    let mut out1 = ctx.run_ui(input_up.clone(), |ui| {
        MacStepper::new(&mut val, 0.0..=1.0, 0.1)
            .theme(&theme)
            .wraps(true)
            .show(ui);
    });
    out1.textures_delta.clear();
    assert!((val - 1.0).abs() < 1e-9);

    let mut out2 = ctx.run_ui(input_up, |ui| {
        MacStepper::new(&mut val, 0.0..=1.0, 0.1)
            .theme(&theme)
            .wraps(true)
            .show(ui);
    });
    out2.textures_delta.clear();
    assert!((val - 0.0).abs() < 1e-9);
}

#[test]
fn test_popup_and_color_well_keyboard_activation_and_escape() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut selected = 0usize;
    let items = ["First", "Second", "Third"];

    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        MacPopUpButton::new(&mut selected, &items)
            .theme(&theme)
            .show(ui)
            .request_focus();
    });
    out1.textures_delta.clear();

    let mut input_space = RawInput::default();
    input_space.events.push(Event::Key {
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
        let is_open = ui.data(|d| {
            d.get_temp::<bool>(resp.id.with("mac_popup_is_open"))
                .unwrap_or(false)
        });
        assert!(is_open, "Pressing Space when focused must open popup");
    });
    out2.textures_delta.clear();

    let mut input_esc = RawInput::default();
    input_esc.events.push(Event::Key {
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
        let is_open = ui.data(|d| {
            d.get_temp::<bool>(resp.id.with("mac_popup_is_open"))
                .unwrap_or(false)
        });
        assert!(!is_open, "Pressing Escape must close popup");
    });
    out3.textures_delta.clear();
}

#[test]
fn test_mac_toggle_headless_interaction_and_disabled_guard() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut toggle_val = false;
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let resp = ui.add(MacToggle::new(&mut toggle_val).theme(&theme));
        assert_eq!(resp.rect.width(), theme.toggle().width.points());
    });
    out1.textures_delta.clear();

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
    let mut out2 = ctx.run_ui(click_input.clone(), |ui| {
        ui.add(MacToggle::new(&mut toggle_val).theme(&theme));
    });
    out2.textures_delta.clear();

    let mut release_input = RawInput::default();
    release_input.events.push(Event::PointerButton {
        pos: click_pos,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::default(),
    });
    let mut out3 = ctx.run_ui(release_input.clone(), |ui| {
        ui.add(MacToggle::new(&mut toggle_val).theme(&theme));
    });
    out3.textures_delta.clear();
    assert!(
        toggle_val,
        "Clicking enabled toggle must invert value to true"
    );

    let mut out4 = ctx.run_ui(click_input, |ui| {
        ui.add(MacToggle::new(&mut toggle_val).theme(&theme).enabled(false));
    });
    out4.textures_delta.clear();
    let mut out5 = ctx.run_ui(release_input, |ui| {
        ui.add(MacToggle::new(&mut toggle_val).theme(&theme).enabled(false));
    });
    out5.textures_delta.clear();
    assert!(toggle_val, "Disabled toggle must not mutate value on click");
}

#[test]
fn test_button_segmented_from_ctx_inheritance_and_centering() {
    let ctx = Context::default();
    let dark_theme = MacTheme::new(MacAppearance::DarkAqua);
    dark_theme.apply_to_egui(&ctx);

    assert_eq!(MacTheme::from_ctx(&ctx).appearance, MacAppearance::DarkAqua);

    let container_size = Vec2::new(380.0, 100.0);
    let mut selected = 0;
    let mut control_rect = Rect::NOTHING;

    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        let btn_resp = ui.add(MacButtonCell::new("Test Button").enabled(false));
        assert!(btn_resp.rect.width() > 0.0);

        ui.allocate_ui_with_layout(
            container_size,
            egui::Layout::top_down(egui::Align::Min),
            |p_ui| {
                p_ui.vertical_centered(|c_ui| {
                    let resp =
                        MacSegmentedControl::new(&["Attributes", "Metrics", "Matrix"], selected)
                            .height(24.0)
                            .show(c_ui, |idx| {
                                selected = idx;
                            });
                    control_rect = resp.rect;
                });
            },
        );

        let (panel_resp, _) = MacPanel::new().title("Dark Panel").show(
            ui,
            Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 60.0)),
            |pui| {
                pui.label("Child");
            },
        );
        assert_eq!(panel_resp.rect.width(), 100.0);
    });
    out.textures_delta.clear();

    let left_margin = control_rect.min.x;
    let right_margin = container_size.x - control_rect.max.x;
    assert!((left_margin - right_margin).abs() <= 1.0);
}
