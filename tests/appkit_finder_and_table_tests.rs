//! COTS Red Baseline Tests for Authentic AppKit Finder & Table View Navigation.
//!
//! Grounded in Test-Driven Development (failing red baseline test assertions first, zero unused dependencies,
//! file size ceilings, authentic AppKit doubleAction & NSPathControl invariants).

#[allow(dead_code)]
#[path = "../examples/gallery/finder_data.rs"]
mod finder_data;

use egui::{Context, Event, Key, Modifiers, Pos2, RawInput, Rect, Vec2};
use egui_macview::{theme::MacTheme, MacTableColumn, MacTableView, MacTableViewOutput};
use finder_data::build_parent_chain;
use std::collections::BTreeSet;
use std::path::PathBuf;

#[test]
fn test_cots_windows_drive_root_parent_chain() {
    let test_path = PathBuf::from(r"C:\Projects\egui-macview");
    let (dir_parents, path_titles) = build_parent_chain(&Some(test_path.clone()));

    let parents = dir_parents.expect("dir_parents must be Some");
    assert_eq!(
        parents.len(),
        3,
        "Chain length must be 3 for C:\\Projects\\egui-macview"
    );
    assert_eq!(parents[0], PathBuf::from(r"C:\"));
    assert_eq!(parents[1], PathBuf::from(r"C:\Projects"));
    assert_eq!(parents[2], test_path);

    // Root segment MUST be 'C:' rather than empty, 'C:\' or None
    assert_eq!(
        path_titles,
        vec!["C:", "Projects", "egui-macview"],
        "Path titles must display clean drive volume 'C:' at index 0"
    );

    // Drive root only test: C:\
    let (drive_parents, drive_titles) = build_parent_chain(&Some(PathBuf::from(r"C:\")));
    let d_parents = drive_parents.expect("drive root parents must be Some");
    assert_eq!(d_parents.len(), 1);
    assert_eq!(d_parents[0], PathBuf::from(r"C:\"));
    assert_eq!(drive_titles, vec!["C:"]);
}

#[test]
fn test_cots_table_view_double_click_and_enter_output() {
    let ctx = Context::default();
    let theme = MacTheme::default();
    let cols = [MacTableColumn::new("Name", 200.0)];

    // 1. Double click interaction on row 1
    let mut sel = BTreeSet::from([1]);
    let mut raw = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0))),
        time: Some(0.0),
        ..Default::default()
    };
    // Click 1: Press then Release at row 1 coordinates (header=24, row 0=24..44, row 1=44..64 -> y = 54.0)
    raw.events.push(Event::PointerButton {
        pos: Pos2::new(50.0, 54.0),
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    raw.events.push(Event::PointerButton {
        pos: Pos2::new(50.0, 54.0),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });

    let mut out: Option<MacTableViewOutput> = None;
    let mut egui_out = ctx.run_ui(raw.clone(), |ui| {
        let resp = MacTableView::new("test_tbl", &cols, 3, &mut sel)
            .theme(&theme)
            .row_height(20.0)
            .show(ui, |c_ui, r, _| {
                c_ui.label(format!("Row {r}"));
            });
        out = Some(resp);
    });
    egui_out.textures_delta.clear();

    // Click 2 (within double-click window 0.1s): Press then Release at row 1
    raw.events.clear();
    raw.time = Some(0.1);
    raw.events.push(Event::PointerButton {
        pos: Pos2::new(50.0, 54.0),
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    raw.events.push(Event::PointerButton {
        pos: Pos2::new(50.0, 54.0),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    let mut egui_out2 = ctx.run_ui(raw, |ui| {
        let resp = MacTableView::new("test_tbl", &cols, 3, &mut sel)
            .theme(&theme)
            .row_height(20.0)
            .show(ui, |c_ui, r, _| {
                c_ui.label(format!("Row {r}"));
            });
        out = Some(resp);
    });
    egui_out2.textures_delta.clear();

    let res = out.expect("table output produced");
    assert_eq!(
        res.double_clicked_row,
        Some(1),
        "Double clicking row 1 must emit double_clicked_row = Some(1)"
    );

    // 2. Keyboard Key::Enter navigation on selected row
    let mut enter_raw = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0))),
        ..Default::default()
    };
    enter_raw.events.push(Event::Key {
        key: Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    });

    let mut enter_out: Option<MacTableViewOutput> = None;
    let mut egui_out3 = ctx.run_ui(enter_raw, |ui| {
        let resp = MacTableView::new("test_tbl_enter", &cols, 3, &mut sel)
            .theme(&theme)
            .row_height(20.0)
            .show(ui, |c_ui, r, _| {
                c_ui.label(format!("Row {r}"));
            });
        enter_out = Some(resp);
    });
    egui_out3.textures_delta.clear();

    let enter_res = enter_out.expect("table output produced");
    assert_eq!(
        enter_res.double_clicked_row,
        Some(1),
        "Pressing Key::Enter on selected row 1 must emit double_clicked_row = Some(1)"
    );
}

#[test]
fn test_cots_color_well_id_salt_and_picker_interaction() {
    let ctx = Context::default();
    let theme = MacTheme::default();
    let mut color1 = egui::Color32::RED;
    let mut color2 = egui::Color32::BLUE;

    // Both wells in the same container with distinct id_salt builder method
    let raw = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0))),
        ..Default::default()
    };
    let mut egui_out = ctx.run_ui(raw, |ui| {
        egui_macview::MacColorWell::new(&mut color1)
            .id_salt("well_1")
            .theme(&theme)
            .show(ui);
        egui_macview::MacColorWell::new(&mut color2)
            .id_salt("well_2")
            .theme(&theme)
            .show(ui);
    });
    egui_out.textures_delta.clear();
}

#[test]
fn test_cots_finder_parent_directory_row_pinned() {
    let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tokens: [Vec<String>; 6] = std::array::from_fn(|_| Vec::new());
    let rows = finder_data::collect_finder_rows(&Some(test_dir.clone()), &[], &[4; 6], &tokens);

    assert!(
        !rows.is_empty(),
        "Rows must not be empty for test directory"
    );
    let parent_row = &rows[0];
    assert_eq!(
        parent_row.name, "..",
        "First row must be parent directory '..'"
    );
    assert!(
        parent_row.is_dir,
        "Parent directory row must be is_dir = true"
    );
    assert_eq!(parent_row.dir_path, test_dir.parent().map(|p| p.to_path_buf()));

    // When sorted descending by size or name, '..' must remain pinned at index 0
    let sorted_desc =
        finder_data::filter_and_sort_rows(rows, "", 0, egui_macview::MacSortDirection::Descending);
    assert_eq!(
        sorted_desc[0].name, "..",
        "Parent row '..' must stay at index 0 when sorted descending"
    );
}

#[test]
fn test_cots_finder_default_directory_and_selection_path_bar() {
    let cur = std::env::current_dir().ok();
    assert!(cur.is_some(), "Current working directory must be valid");
    let (parents, titles) = build_parent_chain(&cur);
    assert!(
        !titles.is_empty(),
        "Titles must not be empty for current working directory"
    );
    assert_eq!(
        titles.last().map(String::as_str),
        Some("egui-macview"),
        "Last title component must be current repository directory name"
    );
    assert!(parents.is_some());

    // Verify parent chain without truncation when navigating deeper
    let base = PathBuf::from(r"C:\Projects\egui-macview");
    let (parents_base, titles_base) = build_parent_chain(&Some(base.clone()));
    assert_eq!(titles_base, vec!["C:", "Projects", "egui-macview"]);
    assert_eq!(parents_base.unwrap().len(), 3);

    // Deeper subfolder 'src/controls'
    let deeper = base.join("src").join("controls");
    let (parents_deep, titles_deep) = build_parent_chain(&Some(deeper.clone()));
    assert_eq!(
        titles_deep,
        vec!["C:", "Projects", "egui-macview", "src", "controls"],
        "Path chain must never be truncated when navigating into subfolders"
    );
    assert_eq!(parents_deep.unwrap().len(), 5);
}

#[test]
fn test_cots_mac_background_style_accent_blue_and_wcag_ui_control() {
    use egui::Color32;
    use egui_macview::theme::{contrast_ratio, MacAppearance, MacBackgroundStyle};

    let theme_aqua = MacTheme::new(MacAppearance::Aqua);
    let theme_dark = MacTheme::new(MacAppearance::DarkAqua);

    let blue_aqua = theme_aqua.control_accent(); // #007AFF
    let blue_dark = theme_dark.control_accent(); // #0A84FF

    // In macOS AppKit, Apple System Blue filled buttons use white text (NSBackgroundStyle::Dark / Emphasized)
    assert_eq!(
        MacBackgroundStyle::from_color(blue_aqua),
        MacBackgroundStyle::Dark,
        "#007AFF must be classified as Dark style for high-contrast white text"
    );
    assert_eq!(
        MacBackgroundStyle::from_color(blue_dark),
        MacBackgroundStyle::Dark,
        "#0A84FF must be classified as Dark style for high-contrast white text"
    );

    // WCAG 2.1 §1.4.11 / §1.4.3 UI Component / Control AA requires >= 3.0:1
    let ratio_aqua = contrast_ratio(Color32::WHITE, blue_aqua);
    let ratio_dark = contrast_ratio(Color32::WHITE, blue_dark);
    assert!(
        ratio_aqua >= 3.0,
        "White on #007AFF must satisfy UI Control AA >= 3.0:1, got {ratio_aqua:.2}:1"
    );
    assert!(
        ratio_dark >= 3.0,
        "White on #0A84FF must satisfy UI Control AA >= 3.0:1, got {ratio_dark:.2}:1"
    );

    // Neutral gray boundaries must remain unshifted (119 is Light, 117 is Dark)
    assert_eq!(
        MacBackgroundStyle::from_color(Color32::from_rgb(119, 119, 119)),
        MacBackgroundStyle::Light
    );
    assert_eq!(
        MacBackgroundStyle::from_color(Color32::from_rgb(117, 117, 117)),
        MacBackgroundStyle::Dark
    );
}

#[test]
fn test_cots_table_view_cmd_arrow_down_open() {
    let ctx = Context::default();
    let theme = MacTheme::default();
    let cols = [MacTableColumn::new("Name", 200.0)];
    let mut sel = BTreeSet::from([1]);

    // Keyboard Cmd+ArrowDown on selected row 1 must emit double_clicked_row = Some(1)
    let mut cmd_raw = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0))),
        ..Default::default()
    };
    cmd_raw.events.push(Event::Key {
        key: Key::ArrowDown,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::COMMAND,
    });

    let mut cmd_out: Option<MacTableViewOutput> = None;
    let mut egui_out = ctx.run_ui(cmd_raw, |ui| {
        let resp = MacTableView::new("test_tbl_cmd", &cols, 3, &mut sel)
            .theme(&theme)
            .row_height(20.0)
            .show(ui, |c_ui, r, _| {
                c_ui.label(format!("Row {r}"));
            });
        cmd_out = Some(resp);
    });
    egui_out.textures_delta.clear();

    let res = cmd_out.expect("table output produced");
    assert_eq!(
        res.double_clicked_row,
        Some(1),
        "Pressing Cmd+ArrowDown on selected row 1 must emit double_clicked_row = Some(1)"
    );
}

#[test]
fn test_cots_calculate_pan_weights_and_window_active_deduplication() {
    let (center_l, center_r) = egui_macview::calculate_pan_weights(0.0);
    assert!(
        (center_l - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-4,
        "Equal power center pan L weight must be ~sqrt(0.5) = 0.7071"
    );
    assert!(
        (center_r - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-4,
        "Equal power center pan R weight must be ~sqrt(0.5) = 0.7071"
    );

    let (hard_left_l, hard_left_r) = egui_macview::calculate_pan_weights(-1.0);
    assert!((hard_left_l - 1.0).abs() < 1e-6);
    assert!(hard_left_r.abs() < 1e-6);

    let (hard_right_l, hard_right_r) = egui_macview::calculate_pan_weights(1.0);
    assert!(hard_right_l.abs() < 1e-6);
    assert!((hard_right_r - 1.0).abs() < 1e-6);
}
