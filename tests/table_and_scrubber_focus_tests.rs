//! COTS Behavioral Tests for Unfocused Table Keyboard Isolation and Trim Scrubber Playhead Invariants.

use egui::{Context, Event, Key, Modifiers, Pos2, RawInput, Rect, Vec2};
use egui_macview::theme::MacTheme;
use egui_macview::{MacTableColumn, MacTableView, MacTrimScrubber};
use std::collections::BTreeSet;

#[test]
fn test_cots_unfocused_table_ignores_arrow_keys_and_scrubber_preserves_valid_playhead() {
    let ctx = Context::default();
    let theme = MacTheme::default();
    let cols = [MacTableColumn::new("Item", 160.0)];

    let mut sel_a = BTreeSet::from([0usize]);
    let mut sel_b = BTreeSet::from([1usize]);
    let mut rect_a = Rect::NOTHING;
    let mut rect_b = Rect::NOTHING;
    let mut id_b = egui::Id::NULL;

    let base_raw = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
        ..Default::default()
    };
    ctx.run_ui(base_raw.clone(), |ui| {
        rect_a = MacTableView::new("tbl_a", &cols, 5, &mut sel_a)
            .theme(&theme)
            .row_height(20.0)
            .show(ui, |u, r, _| {
                u.label(format!("A{r}"));
            })
            .response
            .rect;
        let resp_b = MacTableView::new("tbl_b", &cols, 5, &mut sel_b)
            .theme(&theme)
            .row_height(20.0)
            .show(ui, |u, r, _| {
                u.label(format!("B{r}"));
            })
            .response;
        rect_b = resp_b.rect;
        id_b = resp_b.id;
    })
    .textures_delta
    .clear();

    let mut arrow_raw = base_raw.clone();
    arrow_raw.events.push(Event::PointerMoved(rect_b.center()));
    arrow_raw.events.push(Event::Key {
        key: Key::ArrowDown,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    ctx.run_ui(arrow_raw, |ui| {
        MacTableView::new("tbl_a", &cols, 5, &mut sel_a)
            .theme(&theme)
            .row_height(20.0)
            .show(ui, |u, r, _| {
                u.label(format!("A{r}"));
            });
        MacTableView::new("tbl_b", &cols, 5, &mut sel_b)
            .theme(&theme)
            .row_height(20.0)
            .show(ui, |u, r, _| {
                u.label(format!("B{r}"));
            });
    })
    .textures_delta
    .clear();

    assert_eq!(sel_a, BTreeSet::from([0]));
    assert_eq!(sel_b, BTreeSet::from([2]));

    // When tbl_b holds keyboard focus and pointer rests over tbl_a, tbl_a must NOT steal ArrowDown
    ctx.memory_mut(|m| m.request_focus(id_b));
    let mut focus_raw = base_raw.clone();
    focus_raw.events.push(Event::PointerMoved(rect_a.center()));
    focus_raw.events.push(Event::Key {
        key: Key::ArrowDown,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    ctx.run_ui(focus_raw, |ui| {
        MacTableView::new("tbl_a", &cols, 5, &mut sel_a)
            .theme(&theme)
            .row_height(20.0)
            .show(ui, |u, r, _| {
                u.label(format!("A{r}"));
            });
        MacTableView::new("tbl_b", &cols, 5, &mut sel_b)
            .theme(&theme)
            .row_height(20.0)
            .show(ui, |u, r, _| {
                u.label(format!("B{r}"));
            });
    })
    .textures_delta
    .clear();

    assert_eq!(
        sel_a,
        BTreeSet::from([0]),
        "Hovered unfocused tbl_a must not steal ArrowDown when tbl_b has keyboard focus"
    );
    assert_eq!(sel_b, BTreeSet::from([3]));

    let mut in_pt = 500.0;
    let mut out_pt = 1000.0;
    let mut playhead = 70.0;
    let mut scrub_resp = None;
    ctx.run_ui(base_raw.clone(), |ui| {
        scrub_resp = Some(
            MacTrimScrubber::new("trim_test", 1200.0, &mut in_pt, &mut out_pt, &mut playhead)
                .clamp_playhead_to_trim(false)
                .theme(&theme)
                .show(ui),
        );
    })
    .textures_delta
    .clear();
    let resp = scrub_resp.expect("scrubber response");
    assert!((playhead - 70.0).abs() < 1e-6);
    assert!(!resp.playhead_dragged && !resp.changed);

    // Dragging the in_point handle when clamp_playhead_to_trim(false) must NOT snap playhead (70.0) to in_point!
    let scrub_rect = resp.response.rect;
    let in_handle_x = scrub_rect.left() + (500.0 / 1200.0) as f32 * scrub_rect.width() + 2.0;
    let drag_to_x = scrub_rect.left() + (600.0 / 1200.0) as f32 * scrub_rect.width();
    let handle_y = scrub_rect.bottom() - 10.0;
    let mut press_in_raw = base_raw.clone();
    press_in_raw
        .events
        .push(Event::PointerMoved(Pos2::new(in_handle_x, handle_y)));
    press_in_raw.events.push(Event::PointerButton {
        pos: Pos2::new(in_handle_x, handle_y),
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    ctx.run_ui(press_in_raw, |ui| {
        MacTrimScrubber::new("trim_test", 1200.0, &mut in_pt, &mut out_pt, &mut playhead)
            .clamp_playhead_to_trim(false)
            .theme(&theme)
            .show(ui);
    })
    .textures_delta
    .clear();

    let mut drag_in_raw = base_raw;
    drag_in_raw
        .events
        .push(Event::PointerMoved(Pos2::new(drag_to_x, handle_y)));
    drag_in_raw.events.push(Event::PointerButton {
        pos: Pos2::new(drag_to_x, handle_y),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    ctx.run_ui(drag_in_raw, |ui| {
        MacTrimScrubber::new("trim_test", 1200.0, &mut in_pt, &mut out_pt, &mut playhead)
            .clamp_playhead_to_trim(false)
            .theme(&theme)
            .show(ui);
    })
    .textures_delta
    .clear();
    assert!(in_pt > 550.0);
    assert!(
        (playhead - 70.0).abs() < 1e-6,
        "Dragging in_point handle with clamp_playhead_to_trim(false) must preserve out-of-trim playhead (70.0), got {playhead}"
    );
}
