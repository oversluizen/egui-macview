//! Behavioral Tests for `MacInputCell`, `MacSearchField` Paint Z-Order, and `MacOutlineView` Accessory Click Isolation.
//!
//! Verifies:
//! 1. `MacInputCell` `.control_size(MacControlSize)` and `.enabled(false)`.
//! 2. `MacSearchField` background bezel shape index < `TextEdit` galley shape index in `FullOutput.shapes`.
//! 3. `MacOutlineView` trailing `accessory_custom` click fires callback while `row_resp.clicked == false`.

use egui::{Context, Event, Id, Key, Modifiers, PointerButton, Pos2, RawInput, Rect};
use egui_macview::{
    theme::{MacAppearance, MacTheme},
    MacControlSize, MacInputCell, MacOutlineView, MacPopUpButton, MacSearchField,
};

fn click_raw(pos: Pos2, pressed: bool) -> RawInput {
    RawInput {
        events: vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            },
        ],
        ..Default::default()
    }
}

fn find_shape_indices(
    shapes: &[egui::epaint::ClippedShape],
    target_text: &str,
    search_rect: Rect,
) -> (Option<usize>, Option<usize>) {
    let mut last_bg_rect_idx = None;
    let mut first_text_idx = None;

    for (idx, clipped) in shapes.iter().enumerate() {
        inspect_shape(
            &clipped.shape,
            idx,
            target_text,
            search_rect,
            &mut last_bg_rect_idx,
            &mut first_text_idx,
        );
    }

    (last_bg_rect_idx, first_text_idx)
}

fn inspect_shape(
    shape: &egui::epaint::Shape,
    top_idx: usize,
    target_text: &str,
    search_rect: Rect,
    last_bg_rect_idx: &mut Option<usize>,
    first_text_idx: &mut Option<usize>,
) {
    match shape {
        egui::epaint::Shape::Rect(r) => {
            if (r.rect.width() - search_rect.width()).abs() < 2.0
                && (r.rect.height() - search_rect.height()).abs() < 2.0
            {
                *last_bg_rect_idx = Some(top_idx);
            }
        }
        egui::epaint::Shape::Text(t) => {
            if t.galley.text().contains(target_text) && first_text_idx.is_none() {
                *first_text_idx = Some(top_idx);
            }
        }
        egui::epaint::Shape::Vec(vec) => {
            for s in vec {
                inspect_shape(
                    s,
                    top_idx,
                    target_text,
                    search_rect,
                    last_bg_rect_idx,
                    first_text_idx,
                );
            }
        }
        _ => {}
    }
}

#[test]
fn test_search_field_paint_order_background_before_text_galley() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);
    let mut query = "VisibleQueryText".to_string();
    let mut search_rect = Rect::NOTHING;

    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        let resp = MacSearchField::new(&mut query)
            .theme(&theme)
            .width(180.0)
            .show(ui);
        search_rect = resp.rect;
    });
    out.textures_delta.clear();

    let (bg_idx, text_idx) = find_shape_indices(&out.shapes, "VisibleQueryText", search_rect);
    let bg_i = bg_idx.expect("Search field pill background rect shape must exist");
    let text_i = text_idx.expect("Search field TextEdit galley shape must exist");

    assert!(
        bg_i < text_i,
        "CRITICAL Z-ORDER BUG: SearchField background rect (shape index {bg_i}) is painted AFTER TextEdit galley (shape index {text_i}), obscuring typed text!"
    );
}

#[test]
fn test_search_field_click_and_escape_consumes_key() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut query = "hello world".to_string();
    let search_id = Id::NULL.with("test_search_input_id");

    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let resp = MacSearchField::new(&mut query)
            .id(search_id)
            .theme(&theme)
            .show(ui);
        resp.request_focus();
    });
    out1.textures_delta.clear();

    let mut out_steady = ctx.run_ui(RawInput::default(), |ui| {
        MacSearchField::new(&mut query)
            .id(search_id)
            .theme(&theme)
            .show(ui);
    });
    out_steady.textures_delta.clear();

    let mut input_esc = RawInput::default();
    input_esc.events.push(Event::Key {
        key: Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    let mut escape_leaked = false;
    let mut out2 = ctx.run_ui(input_esc, |ui| {
        MacSearchField::new(&mut query)
            .id(search_id)
            .theme(&theme)
            .show(ui);
        escape_leaked = ui.input(|i| i.key_pressed(Key::Escape));
    });
    out2.textures_delta.clear();
    assert!(query.is_empty(), "Escape must clear text in search field");
    assert!(
        !escape_leaked,
        "MacSearchField must consume Key::Escape so parent modals do not close"
    );
}

#[test]
fn test_outline_view_trailing_accessory_click_isolation() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);

    let mut is_expanded = false;
    let mut is_visible = true;
    let mut acc_rect = Rect::NOTHING;

    // Frame 1: Render outline row and capture accessory button rect
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        MacOutlineView::new("outline_acc_isolation")
            .theme(&theme)
            .show(ui, |tree| {
                tree.row("row_1", 0, true, &mut is_expanded, false, |row| {
                    row.label("Layer 1");
                    row.accessory_custom("vis_btn", &mut is_visible, |_painter, rect, _active| {
                        acc_rect = rect;
                    });
                });
            });
    });
    out1.textures_delta.clear();
    assert!(acc_rect.width() > 0.0, "Accessory rect must be allocated");

    // Frame 2 & 3: Press and release pointer on trailing accessory button (completes click)
    let click_pos = acc_rect.center();
    let mut row_was_clicked = false;
    for pressed in [true, false] {
        let mut out_step = ctx.run_ui(click_raw(click_pos, pressed), |ui| {
            MacOutlineView::new("outline_acc_isolation")
                .theme(&theme)
                .show(ui, |tree| {
                    let resp = tree.row("row_1", 0, true, &mut is_expanded, false, |row| {
                        row.label("Layer 1");
                        row.accessory_custom("vis_btn", &mut is_visible, |_p, _r, _a| {});
                    });
                    if !pressed {
                        row_was_clicked = resp.clicked;
                    }
                });
        });
        out_step.textures_delta.clear();
    }

    assert!(
        !is_visible,
        "Clicking trailing accessory button must toggle accessory state to false"
    );
    assert!(
        !row_was_clicked,
        "Clicking trailing accessory button must NOT set row_resp.clicked to true!"
    );
}

#[test]
fn test_input_cell_control_sizes_and_disabled_state() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut val = "128.0".to_string();
    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        let r_reg = MacInputCell::new("in_reg", &mut val)
            .theme(&theme)
            .control_size(MacControlSize::Regular)
            .show(ui);
        let r_sm = MacInputCell::new("in_sm", &mut val)
            .theme(&theme)
            .control_size(MacControlSize::Small)
            .show(ui);
        let r_mini = MacInputCell::new("in_mini", &mut val)
            .theme(&theme)
            .control_size(MacControlSize::Mini)
            .show(ui);
        let r_hud = MacInputCell::new("in_hud", &mut val)
            .theme(&theme)
            .control_size(MacControlSize::CompactInfobox)
            .enabled(false)
            .show(ui);

        assert_eq!(r_reg.rect.height(), 22.0);
        assert_eq!(r_sm.rect.height(), 19.0);
        assert_eq!(r_mini.rect.height(), 15.0);
        assert_eq!(r_hud.rect.height(), 18.0);
    });
    out.textures_delta.clear();
}

#[test]
fn test_input_cell_consumes_single_char_keys_and_commits_on_outside_popup_click() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);

    let mut val = "128".to_string();
    let items = ["Points", "Pixels"];
    let mut unit_idx = 0usize;
    let mut popup_pos = Pos2::ZERO;

    // Frame 1: Request focus on MacInputCell
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let r = MacInputCell::new("cell_edit", &mut val)
            .theme(&theme)
            .show(ui);
        r.request_focus();
        let p = MacPopUpButton::new(&mut unit_idx, &items)
            .id("unit_pop")
            .theme(&theme)
            .show(ui);
        popup_pos = p.rect.center();
    });
    out1.textures_delta.clear();

    // Frame 2: Steady focus frame
    let mut out2 = ctx.run_ui(RawInput::default(), |ui| {
        MacInputCell::new("cell_edit", &mut val)
            .theme(&theme)
            .show(ui);
        MacPopUpButton::new(&mut unit_idx, &items)
            .id("unit_pop")
            .theme(&theme)
            .show(ui);
    });
    out2.textures_delta.clear();

    // Frame 3: Type ".5" and press single-char tool shortcut Key::V while focused
    let mut type_input = RawInput::default();
    type_input.events.push(Event::Text(".5".to_string()));
    type_input.events.push(Event::Key {
        key: Key::V,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    let mut v_key_leaked = false;
    let mut out3 = ctx.run_ui(type_input, |ui| {
        MacInputCell::new("cell_edit", &mut val)
            .theme(&theme)
            .show(ui);
        MacPopUpButton::new(&mut unit_idx, &items)
            .id("unit_pop")
            .theme(&theme)
            .show(ui);
        v_key_leaked = ui.input(|i| i.key_pressed(Key::V));
    });
    out3.textures_delta.clear();
    assert!(
        !v_key_leaked,
        "Single-character canvas key Key::V must be consumed while MacInputCell has focus!"
    );
    assert_eq!(val, "128.5");

    // Frame 4 & 5: Click MacPopUpButton outside MacInputCell -> MacInputCell commits on blur & popup opens
    let mut lost_focus_on_click = false;
    let mut popup_is_open = false;
    for pressed in [true, false] {
        let mut out_pop = ctx.run_ui(click_raw(popup_pos, pressed), |ui| {
            let cell_resp = MacInputCell::new("cell_edit", &mut val)
                .theme(&theme)
                .show(ui);
            lost_focus_on_click = cell_resp.lost_focus() || !cell_resp.has_focus();
            MacPopUpButton::new(&mut unit_idx, &items)
                .id("unit_pop")
                .theme(&theme)
                .show(ui);
            let is_open_id = Id::NULL.with("unit_pop").with("mac_popup_is_open");
            popup_is_open = ui.data_mut(|d| d.get_temp::<bool>(is_open_id).unwrap_or(false));
        });
        out_pop.textures_delta.clear();
    }

    assert_eq!(
        val, "128.5",
        "Edited value must remain committed when blurring via outside click"
    );
    assert!(
        lost_focus_on_click,
        "MacInputCell must surrender focus on outside click"
    );
    assert!(
        popup_is_open,
        "Clicking MacPopUpButton while MacInputCell was focused must open the popup immediately"
    );
}

#[test]
fn test_input_cell_numeric_arrow_key_stepping_and_clamping() {
    let (ctx, theme, mut val) = (
        Context::default(),
        MacTheme::new(MacAppearance::Aqua),
        "100.0".to_string(),
    );
    for _ in 0..2 {
        ctx.run_ui(RawInput::default(), |ui| {
            MacInputCell::new("num_cell", &mut val)
                .theme(&theme)
                .numeric(50.0..=250.0, 1.0)
                .show(ui)
                .request_focus();
        })
        .textures_delta
        .clear();
    }
    let cases = [
        (Key::ArrowUp, Modifiers::NONE, "101.0"),
        (Key::ArrowDown, Modifiers::NONE, "100.0"),
        (Key::ArrowUp, Modifiers::SHIFT, "110.0"),
        (Key::ArrowDown, Modifiers::SHIFT, "100.0"),
        (Key::ArrowUp, Modifiers::COMMAND, "200.0"),
        (Key::ArrowUp, Modifiers::COMMAND, "250.0"),
        (Key::ArrowDown, Modifiers::CTRL, "150.0"),
        (Key::ArrowDown, Modifiers::CTRL, "50.0"),
        (Key::ArrowDown, Modifiers::NONE, "50.0"),
    ];
    for (key, modifiers, expected) in cases {
        let ev = Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        };
        let mut changed = false;
        ctx.run_ui(
            RawInput {
                events: vec![ev],
                ..Default::default()
            },
            |ui| {
                changed = MacInputCell::new("num_cell", &mut val)
                    .theme(&theme)
                    .numeric(50.0..=250.0, 1.0)
                    .show(ui)
                    .changed();
            },
        )
        .textures_delta
        .clear();
        assert_eq!(val, expected);
        assert!(changed);
    }

    // Edge cases: non-numeric ("abc"), scientific notation ("1e2"), and Escape rollback
    for (buf_override, key, expected) in [
        ("abc", Key::ArrowUp, "51.0"),
        ("1e2", Key::ArrowUp, "101.0"),
        ("corrupted", Key::Escape, "101.0"),
    ] {
        let ev = Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        ctx.run_ui(
            RawInput {
                events: vec![ev],
                ..Default::default()
            },
            |ui| {
                let edit_id = Id::NULL.with("num_cell").with("mac_input_buffer");
                ui.data_mut(|d| d.insert_temp(edit_id, buf_override.to_string()));
                MacInputCell::new("num_cell", &mut val)
                    .theme(&theme)
                    .numeric(50.0..=250.0, 1.0)
                    .show(ui);
            },
        )
        .textures_delta
        .clear();
        assert_eq!(val, expected);
    }
}
