//! Behavioral Tests for Foreground Click & Scroll Isolation across `MacPopUpButton`, `MacPanel`, `MacPopover`, and `MacFloatingInfoBox`.
//!
//! Verifies:
//! 1. `MacPopUpButton` open dropdown menu intercepts pointer clicks and does NOT leak clicks to underlying canvas widgets.
//! 2. `MacPanel`, `MacPopover`, and `MacFloatingInfoBox` block clicks and scroll-wheel (`smooth_scroll_delta`) events from leaking to an underlying canvas.

use egui::{Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Sense, Vec2};
use egui_macview::{
    theme::{MacAppearance, MacTheme},
    MacFloatingInfoBox, MacPanel, MacPopUpButton, MacPopover, MacPopoverArrowEdge,
};

#[test]
fn test_popup_button_open_dropdown_click_isolation_over_canvas() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);

    let items = ["Default NIB", "Compact HUD", "Expanded Pro"];
    let mut selected = 0usize;
    let mut popup_rect = Rect::NOTHING;

    // Frame 1: Open popup dropdown and run initial sizing pass
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let popup_id = egui::Id::NULL.with("iso_popup");
        let is_open_id = popup_id.with("mac_popup_is_open");
        ui.data_mut(|d| d.insert_temp(is_open_id, true));
        let canvas_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0));
        let _ = ui.interact(
            canvas_rect,
            egui::Id::NULL.with("under_canvas"),
            Sense::click(),
        );
        let resp = MacPopUpButton::new(&mut selected, &items)
            .id("iso_popup")
            .theme(&theme)
            .width(140.0)
            .show(ui);
        popup_rect = resp.rect;
    });
    out1.textures_delta.clear();

    // Frame 2: Let the Foreground Area establish its geometry
    let mut out2 = ctx.run_ui(RawInput::default(), |ui| {
        let canvas_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0));
        let _ = ui.interact(
            canvas_rect,
            egui::Id::NULL.with("under_canvas"),
            Sense::click(),
        );
        let resp = MacPopUpButton::new(&mut selected, &items)
            .id("iso_popup")
            .theme(&theme)
            .width(140.0)
            .show(ui);
        popup_rect = resp.rect;
    });
    out2.textures_delta.clear();

    // Target item 1 ("Compact HUD") inside the open dropdown menu below popup_rect
    // popup_gap = 4.0, item_margin = 8.0, item_height = 24.0 -> item 1 center Y = bottom + 4 + 8 + 24 + 12 = bottom + 48.0
    let dropdown_item_1_pos = Pos2::new(popup_rect.center().x, popup_rect.bottom() + 48.0);

    // Frame 3: Pointer press on dropdown item 1
    let mut press_input = RawInput::default();
    press_input
        .events
        .push(Event::PointerMoved(dropdown_item_1_pos));
    press_input.events.push(Event::PointerButton {
        pos: dropdown_item_1_pos,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    let mut out3 = ctx.run_ui(press_input, |ui| {
        let canvas_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0));
        let _ = ui.interact(
            canvas_rect,
            egui::Id::NULL.with("under_canvas"),
            Sense::click(),
        );
        MacPopUpButton::new(&mut selected, &items)
            .id("iso_popup")
            .theme(&theme)
            .width(140.0)
            .show(ui);
    });
    out3.textures_delta.clear();

    // Frame 4: Pointer release on dropdown item 1
    let mut release_input = RawInput::default();
    release_input
        .events
        .push(Event::PointerMoved(dropdown_item_1_pos));
    release_input.events.push(Event::PointerButton {
        pos: dropdown_item_1_pos,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    let mut canvas_clicked = false;
    let mut out4 = ctx.run_ui(release_input, |ui| {
        let canvas_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0));
        let canvas_resp = ui.interact(
            canvas_rect,
            egui::Id::NULL.with("under_canvas"),
            Sense::click(),
        );
        canvas_clicked = canvas_resp.clicked();
        MacPopUpButton::new(&mut selected, &items)
            .id("iso_popup")
            .theme(&theme)
            .width(140.0)
            .show(ui);
    });
    out4.textures_delta.clear();

    assert_eq!(selected, 1, "Clicking dropdown item 1 must select index 1");
    assert!(
        !canvas_clicked,
        "Clicking open MacPopUpButton dropdown item must NOT pass through to the underlying canvas!"
    );
}

#[test]
fn test_mac_panel_and_popover_click_and_scroll_isolation() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);

    let panel_rect = Rect::from_min_size(Pos2::new(50.0, 50.0), Vec2::new(200.0, 120.0));
    let other_rect = Rect::from_min_size(Pos2::new(280.0, 50.0), Vec2::new(200.0, 120.0));
    let inside_pos = panel_rect.center();

    // Frame 1: Initial layout pass + verify default ID uniqueness without explicit .id(...)
    let mut panel_id_a = egui::Id::NULL;
    let mut panel_id_b = egui::Id::NULL;
    let mut pop_id_a = egui::Id::NULL;
    let mut pop_id_b = egui::Id::NULL;
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let canvas_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 400.0));
        let _ = ui.interact(
            canvas_rect,
            egui::Id::NULL.with("bg_canvas"),
            Sense::click(),
        );
        panel_id_a = MacPanel::new()
            .title("Inspector A")
            .theme(&theme)
            .show(ui, panel_rect, |pui| {
                pui.label("Panel A");
            })
            .0
            .id;
        panel_id_b = MacPanel::new()
            .title("Inspector B")
            .theme(&theme)
            .show(ui, other_rect, |pui| {
                pui.label("Panel B");
            })
            .0
            .id;
        pop_id_a = MacPopover::new()
            .theme(&theme)
            .show(ui, panel_rect, |pui| {
                pui.label("Pop A");
            })
            .0
            .id;
        pop_id_b = MacPopover::new()
            .theme(&theme)
            .show(ui, other_rect, |pui| {
                pui.label("Pop B");
            })
            .0
            .id;
    });
    out1.textures_delta.clear();
    assert_ne!(
        panel_id_a, panel_id_b,
        "Default MacPanel IDs must not collide across distinct titles/rects"
    );
    assert_ne!(
        pop_id_a, pop_id_b,
        "Default MacPopover IDs must not collide across distinct anchor rects"
    );

    // Frame 2: Hover + Scroll + Click press inside MacPanel over canvas
    let mut scroll_input = RawInput::default();
    scroll_input.events.push(Event::PointerMoved(inside_pos));
    scroll_input.events.push(Event::PointerButton {
        pos: inside_pos,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });

    let mut remaining_smooth = Vec2::ZERO;
    let mut out2 = ctx.run_ui(scroll_input, |ui| {
        let canvas_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 400.0));
        let _ = ui.interact(
            canvas_rect,
            egui::Id::NULL.with("bg_canvas"),
            Sense::click(),
        );
        ui.input_mut(|i| {
            i.smooth_scroll_delta = Vec2::new(0.0, -40.0);
        });
        MacPanel::new()
            .title("Inspector A")
            .theme(&theme)
            .show(ui, panel_rect, |pui| {
                pui.label("Panel A");
            });
        remaining_smooth = ui.input(|i| i.smooth_scroll_delta);
    });
    out2.textures_delta.clear();

    assert_eq!(
        remaining_smooth,
        Vec2::ZERO,
        "MacPanel must consume smooth_scroll_delta when hovered!"
    );

    // Frame 2b: Release click inside MacPanel -> underlying canvas must NOT be clicked
    let mut release_panel = RawInput::default();
    release_panel.events.push(Event::PointerMoved(inside_pos));
    release_panel.events.push(Event::PointerButton {
        pos: inside_pos,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    let mut canvas_clicked_under_panel = false;
    let mut out2b = ctx.run_ui(release_panel, |ui| {
        let canvas_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 400.0));
        let c_resp = ui.interact(
            canvas_rect,
            egui::Id::NULL.with("bg_canvas"),
            Sense::click(),
        );
        canvas_clicked_under_panel = c_resp.clicked();
        MacPanel::new()
            .title("Inspector A")
            .theme(&theme)
            .show(ui, panel_rect, |pui| {
                pui.label("Panel A");
            });
    });
    out2b.textures_delta.clear();
    assert!(
        !canvas_clicked_under_panel,
        "Clicking inside MacPanel must NOT leak click to underlying canvas!"
    );

    // Frame 3: Hover + Scroll inside MacPopover
    let mut pop_scroll_input = RawInput::default();
    pop_scroll_input
        .events
        .push(Event::PointerMoved(inside_pos));

    let mut pop_remaining_smooth = Vec2::ZERO;
    let mut out3 = ctx.run_ui(pop_scroll_input, |ui| {
        ui.input_mut(|i| {
            i.smooth_scroll_delta = Vec2::new(0.0, 30.0);
        });
        MacPopover::new()
            .id("iso_popover")
            .theme(&theme)
            .arrow_edge(MacPopoverArrowEdge::Top)
            .show(ui, panel_rect, |pui| {
                pui.label("Popover Content");
            });
        pop_remaining_smooth = ui.input(|i| i.smooth_scroll_delta);
    });
    out3.textures_delta.clear();

    assert_eq!(
        pop_remaining_smooth,
        Vec2::ZERO,
        "MacPopover must consume smooth_scroll_delta when hovered!"
    );
}

#[test]
fn test_mac_floating_infobox_hud_geometry_and_scroll_isolation() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);

    let mut hud_pos = Pos2::new(80.0, 80.0);
    let mut x_str = "128.0".to_string();
    let mut y_str = "256.0".to_string();
    let mut w_str = "280.0".to_string();
    let mut h_str = "180.0".to_string();

    // Frame 1: Render MacFloatingInfoBox over canvas
    let mut hud_rect = Rect::NOTHING;
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let canvas_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 400.0));
        let _ = ui.interact(
            canvas_rect,
            egui::Id::NULL.with("hud_bg_canvas"),
            Sense::click(),
        );
        let resp = MacFloatingInfoBox::new("test_hud", &mut hud_pos)
            .theme(&theme)
            .title("Compact HUD")
            .show_xywh(ui.ctx(), &mut x_str, &mut y_str, &mut w_str, &mut h_str);
        hud_rect = resp.rect;
    });
    out1.textures_delta.clear();

    assert_eq!(
        hud_rect.width(),
        178.0,
        "SPEC-006 §2.2 requires 178.0pt HUD width"
    );
    assert_eq!(
        hud_rect.height(),
        63.0,
        "SPEC-006 §2.2 requires 63.0pt HUD height"
    );

    // Frame 2: Scroll inside MacFloatingInfoBox -> scroll delta must be consumed (and Area completes steady pass)
    let hud_center = Pos2::new(hud_rect.left() + 12.0, hud_rect.top() + 8.0);
    let mut scroll_input = RawInput::default();
    scroll_input.events.push(Event::PointerMoved(hud_center));

    let mut remaining_smooth = Vec2::ZERO;
    let mut out2 = ctx.run_ui(scroll_input, |ui| {
        let canvas_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 400.0));
        let _ = ui.interact(
            canvas_rect,
            egui::Id::NULL.with("hud_bg_canvas"),
            Sense::click(),
        );
        ui.input_mut(|i| {
            i.smooth_scroll_delta = Vec2::new(0.0, -25.0);
        });
        MacFloatingInfoBox::new("test_hud", &mut hud_pos)
            .theme(&theme)
            .title("Compact HUD")
            .show_xywh(ui.ctx(), &mut x_str, &mut y_str, &mut w_str, &mut h_str);
        remaining_smooth = ui.input(|i| i.smooth_scroll_delta);
    });
    out2.textures_delta.clear();

    assert_eq!(
        remaining_smooth,
        Vec2::ZERO,
        "MacFloatingInfoBox must isolate smooth_scroll_delta!"
    );

    // Frame 3: Press click on HUD header over canvas
    let mut press_hud = RawInput::default();
    press_hud.events.push(Event::PointerMoved(hud_center));
    press_hud.events.push(Event::PointerButton {
        pos: hud_center,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    let mut out3 = ctx.run_ui(press_hud, |ui| {
        let canvas_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 400.0));
        let _ = ui.interact(
            canvas_rect,
            egui::Id::NULL.with("hud_bg_canvas"),
            Sense::click(),
        );
        MacFloatingInfoBox::new("test_hud", &mut hud_pos)
            .theme(&theme)
            .title("Compact HUD")
            .show_xywh(ui.ctx(), &mut x_str, &mut y_str, &mut w_str, &mut h_str);
    });
    out3.textures_delta.clear();

    // Frame 4: Release click on HUD header -> underlying canvas must NOT be clicked
    let mut release_hud = RawInput::default();
    release_hud.events.push(Event::PointerMoved(hud_center));
    release_hud.events.push(Event::PointerButton {
        pos: hud_center,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    let mut canvas_clicked = false;
    let mut out4 = ctx.run_ui(release_hud, |ui| {
        let canvas_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 400.0));
        let c_resp = ui.interact(
            canvas_rect,
            egui::Id::NULL.with("hud_bg_canvas"),
            Sense::click(),
        );
        canvas_clicked = c_resp.clicked();
        MacFloatingInfoBox::new("test_hud", &mut hud_pos)
            .theme(&theme)
            .title("Compact HUD")
            .show_xywh(ui.ctx(), &mut x_str, &mut y_str, &mut w_str, &mut h_str);
    });
    out4.textures_delta.clear();
    assert!(
        !canvas_clicked,
        "Clicking MacFloatingInfoBox must NOT leak click to underlying canvas!"
    );
}
