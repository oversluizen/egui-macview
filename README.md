# `egui-macview`

**Native-style macOS AppKit controls, layouts, and smooth spring animations for [`egui`](https://github.com/emilk/egui) on Windows, macOS, Linux, and Web.**

[![Crates.io](https://img.shields.io/crates/v/egui-macview.svg)](https://crates.io/crates/egui-macview)
[![Documentation](https://docs.rs/egui-macview/badge.svg)](https://docs.rs/egui-macview)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)

---

## Why `egui-macview`?

Standard `egui` is great for quick tools and game overlays, but building a desktop application that feels like a native macOS app (such as Xcode, Final Cut Pro, Finder, or System Settings) requires predictable widget dimensions, crisp 1px borders, keyboard navigation, and smooth animations.

`egui-macview` gives you a ready-to-use library of **35+ macOS AppKit and AVKit controls** built directly on `egui 0.37`:

- **Measured on real macOS hardware**: Control heights, padding insets, corner radii, and colors are measured directly from macOS Sonoma (`AppKit` and `AVKit`) so your UI looks right out of the box.
- **Light, Dark, and Anthracite themes**: Switch seamlessly between `MacAppearance::Aqua` (Light), `MacAppearance::DarkAqua` (Dark), and `MacAppearance::Anthracite`, with automatic WCAG contrast handling for custom accent colors.
- **Battery-friendly by design**: Zero background repaints when idle, automatic animation pausing when the window is unfocused or minimized, and 2D viewport culling for large tables and grids.
- **Cross-platform vector icons**: Includes an embedded `CupertinoIcons` TrueType font so symbols look identical on Windows, macOS, Linux, and Web without relying on OS emojis.

---

## Installation

Add `egui-macview` to your `Cargo.toml`:

```toml
[dependencies]
egui = "0.37.0"
egui-macview = "0.6.1"
```

To run the interactive showcase gallery included in this repository:

```bash
cargo run --example macview_gallery
```

---

## Quick Start & Copy-Paste Examples

Every control in `egui-macview` works with standard `egui::Ui` and optionally accepts a [`MacTheme`]. If you call `theme.apply_to_egui(ctx)` once when your app starts (or when the theme changes), widgets will automatically pick up your active theme even if you omit `.theme(&theme)`.

### 1. Theme Setup, Buttons, Toggles & Segmented Controls

```rust
use egui_macview::{
    MacAppearance, MacButton, MacSegmentedControl, MacTheme, MacToggle, setup_symbols_font,
};

# let ctx = egui::Context::default();
// Call once at startup to register the monochrome Cupertino vector icon font:
let mut fonts = egui::FontDefinitions::default();
setup_symbols_font(&mut fonts);
ctx.set_fonts(fonts);

// Create a macOS theme (Aqua for Light mode, DarkAqua for Dark mode)
let theme = MacTheme::new(MacAppearance::DarkAqua);
theme.apply_to_egui(&ctx);

# let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
let mut auto_save = true;
let mut view_mode = 0usize;

ui.horizontal(|ui| {
    // macOS pill toggle switch (NSSwitch)
    ui.label("Auto-Save:");
    ui.add(MacToggle::new(&mut auto_save).theme(&theme));

    // Segmented pill switcher (NSSegmentedControl)
    let modes = ["Grid", "List", "Columns"];
    MacSegmentedControl::new(&modes, view_mode)
        .theme(&theme)
        .show(ui, |new_idx| view_mode = new_idx);

    // Push button (NSButton)
    if ui.add(MacButton::new("Apply").selected(true).theme(&theme)).clicked() {
        // Handle click
    }
});
# });
# output.textures_delta.clear();
```

---

### 2. Forms, Text Inputs, Search Fields, Sliders & Steppers

Use [`MacForm`] for clean two-column macOS preference panes with right-aligned labels and left-aligned controls:

```rust
use egui_macview::{
    MacAppearance, MacForm, MacInputCell, MacSearchField, MacSlider, MacStepper, MacTheme,
};

# let ctx = egui::Context::default();
# let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
let theme = MacTheme::new(MacAppearance::Aqua);
let mut project_name = String::from("Untitled Project");
let mut search_query = String::new();
let mut zoom_percent = 100.0f64;
let mut export_quality = 80.0f32;

// Search bar with magnifying glass icon and instant clear (×) button:
MacSearchField::new(&mut search_query)
    .placeholder("Filter settings...")
    .theme(&theme)
    .show(ui);

// Two-column macOS Settings form:
MacForm::new("project_settings_form")
    .theme(&theme)
    .show(ui, |form| {
        form.row(
            |ui| { ui.label("Project Name:"); },
            |ui| {
                MacInputCell::with_id(egui::Id::new("project_name_input"), &mut project_name)
                    .theme(&theme)
                    .show(ui);
            },
        );

        form.row(
            |ui| { ui.label("Canvas Zoom:"); },
            |ui| {
                ui.horizontal(|ui| {
                    MacStepper::new(&mut zoom_percent, 25.0..=400.0, 25.0)
                        .theme(&theme)
                        .show(ui);
                    ui.label(format!("{zoom_percent:.0}%"));
                });
            },
        );

        form.row(
            |ui| { ui.label("Export Quality:"); },
            |ui| {
                MacSlider::new(&mut export_quality, 0.0..=100.0)
                    .ticks(5, true)
                    .theme(&theme)
                    .show(ui);
            },
        );
    });
# });
# output.textures_delta.clear();
```

---

### 3. Dropdowns & Searchable Combo Boxes (`MacComboBox` & `MacPopUpButton`)

Use [`MacPopUpButton`] when the user picks from a fixed list, or [`MacComboBox`] when they can both type to filter and pick from a dropdown list:

```rust
use egui_macview::{MacAppearance, MacComboBox, MacPopUpButton, MacTheme};

# let ctx = egui::Context::default();
# let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
let theme = MacTheme::new(MacAppearance::Aqua);

// 1. Fixed selection menu (NSPopUpButton)
let formats = ["PNG (Lossless)", "JPEG (Compressed)", "SVG (Vector)", "PDF (Print)"];
let mut selected_format = 0usize;
MacPopUpButton::new(&mut selected_format, &formats)
    .theme(&theme)
    .show(ui);

// 2. Editable + filterable combo box (NSComboBox)
let mut selected_font = String::from("SF Pro Text");
let fonts = ["Helvetica Neue", "Menlo", "New York", "SF Mono", "SF Pro Display", "SF Pro Text"];
MacComboBox::new("font_family_combo", &mut selected_font, &fonts)
    .width(200.0)
    .theme(&theme)
    .show(ui);
# });
# output.textures_delta.clear();
```

---

### 4. Date & Time Picker (`MacDatePicker`)

[`MacDatePicker`] supports both a compact text field with an integrated stepper (`TextFieldAndStepper`, where users can click year/month/day segments and use arrow keys) and a visual month calendar grid with an analog clock (`ClockAndCalendar`):

```rust
use egui_macview::{
    MacAppearance, MacDate, MacDatePicker, MacDatePickerElementFlags, MacDatePickerStyle, MacTheme,
};

# let ctx = egui::Context::default();
# let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
let theme = MacTheme::new(MacAppearance::Aqua);
let mut scheduled_at = MacDate {
    year: 2026,
    month: 11,
    day: 10,
    hour: 14,
    minute: 30,
    second: 0,
};

// Compact date & time field with up/down stepper arrows:
ui.add(
    MacDatePicker::new()
        .date(&mut scheduled_at)
        .style(MacDatePickerStyle::TextFieldAndStepper)
        .elements(MacDatePickerElementFlags::DateAndTime)
        .theme(&theme),
);

// Visual month calendar grid + 12-hour analog clock:
ui.add(
    MacDatePicker::new()
        .date(&mut scheduled_at)
        .style(MacDatePickerStyle::ClockAndCalendar)
        .elements(MacDatePickerElementFlags::DateAndTime)
        .theme(&theme),
);
# });
# output.textures_delta.clear();
```

---

### 5. Layout Containers: Tabs, Collapsible Disclosure Groups, Boxes & Split Views

Organize complex inspector panels and multi-pane windows with [`MacTabView`], [`MacDisclosureGroup`], [`MacBox`], and [`MacSplitView`]:

```rust
use egui_macview::{
    MacAppearance, MacBox, MacDisclosureGroup, MacSplitOrientation, MacSplitView, MacTabView,
    MacTheme,
};

# let ctx = egui::Context::default();
# let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
let theme = MacTheme::new(MacAppearance::DarkAqua);
let mut active_tab = 0usize;
let mut show_advanced = true;
let mut sidebar_width = 200.0f32;

// 1. Tabbed container (NSTabView)
MacTabView::new(&mut active_tab)
    .theme(&theme)
    .tab("General", |ui| {
        ui.label("General application settings");
    })
    .tab("Shortcuts", |ui| {
        ui.label("Keyboard shortcut bindings");
    })
    .show(ui);

// 2. Collapsible section with animated disclosure triangle
MacDisclosureGroup::new("advanced_options", "Advanced Export Options", &mut show_advanced)
    .theme(&theme)
    .show(ui, |ui| {
        MacBox::new("Color Profile")
            .theme(&theme)
            .show(ui, |ui| {
                ui.label("Display P3 Wide Gamut");
            });
    });

// 3. Resizable two-pane split view (NSSplitView)
MacSplitView::with_pos("main_split", MacSplitOrientation::Horizontal, &mut sidebar_width)
    .min_first(140.0)
    .max_first(360.0)
    .theme(&theme)
    .show(
        ui,
        |left_ui| {
            left_ui.label("Sidebar Navigation");
        },
        |right_ui| {
            right_ui.label("Main Editor Canvas");
        },
    );
# });
# output.textures_delta.clear();
```

---

### 6. Data Views: Multi-Column Tables, 2D Grids, Breadcrumb Path Bars & Rule Editors

Render large datasets smoothly with [`MacTableView`] and [`MacCollectionView`] (both use viewport culling so off-screen rows/tiles cost zero CPU time):

```rust
use std::collections::BTreeSet;
use egui_macview::{
    MacAppearance, MacCollectionView, MacCollectionViewFlowLayout, MacPathComponent,
    MacPathControl, MacRuleEditor, MacRuleRow, MacSortDirection, MacTableColumn, MacTableView,
    MacTheme, MacTokenField,
};

# let ctx = egui::Context::default();
# let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
let theme = MacTheme::new(MacAppearance::Aqua);

// 1. Finder-style breadcrumb bar (NSPathControl)
let path = [
    MacPathComponent::folder("Projects"),
    MacPathComponent::folder("egui-macview"),
    MacPathComponent::file("Cargo.toml"),
];
MacPathControl::new("project_path", &path)
    .theme(&theme)
    .show(ui);

// 2. Multi-column sortable table (NSTableView)
let mut columns = vec![
    MacTableColumn::new("Name", 180.0).sort(Some(MacSortDirection::Ascending)),
    MacTableColumn::new("Kind", 100.0),
];
let rows = [("README.md", "Markdown"), ("Cargo.toml", "TOML")];
let mut selected_rows = BTreeSet::from([0usize]);

MacTableView::new("files_table", &mut columns, rows.len(), &mut selected_rows)
    .theme(&theme)
    .show(ui, |cell_ui, row_idx, col_idx| {
        let (name, kind) = rows[row_idx];
        cell_ui.label(if col_idx == 0 { name } else { kind });
    });

// 3. 2D Grid Collection View (NSCollectionView) with viewport culling & marquee selection
let glyphs = ["A", "B", "C", "D", "E", "F"];
let mut selected_tiles = BTreeSet::from([0usize]);
MacCollectionView::new(&glyphs, &mut selected_tiles)
    .layout(MacCollectionViewFlowLayout::default())
    .theme(&theme)
    .show_with_response(ui, |tile_ui, glyph| {
        tile_ui.centered_and_justified(|ui| {
            ui.label(*glyph);
        });
    });

// 4. Tag pill input (NSTokenField) & Predicate filter builder (NSRuleEditor)
let mut tags = vec!["rust".to_string(), "gui".to_string()];
let mut tag_draft = String::new();
MacTokenField::new("topic_tags", &mut tags, &mut tag_draft)
    .placeholder("Add tag...")
    .theme(&theme)
    .show(ui);

let mut rules = vec![MacRuleRow::new(vec![0, 0, 0])];
let criteria: &[&[&str]] = &[
    &["Name", "Extension", "Modified Date"],
    &["contains", "begins with", "is"],
    &["rs", "toml", "md"],
];
MacRuleEditor::new("smart_filter", &mut rules, criteria)
    .theme(&theme)
    .show(ui);
# });
# output.textures_delta.clear();
```

---

### 7. Media Player Overlay, Timeline Scrubber, Audio Meters & Rotary Dials

Build audio/video players and studio interfaces with [`MacPlayerView`], [`MacTrimScrubber`], [`MacLevelIndicator`], [`MacCircularSlider`], and [`MacRatingIndicator`]:

```rust
use egui_macview::{
    MacAppearance, MacCircularSlider, MacLevelIndicator, MacLevelIndicatorStyle,
    MacPlayerControlsStyle, MacPlayerView, MacRatingIndicator, MacTheme, MacTrimScrubber,
};

# let ctx = egui::Context::default();
# let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
let theme = MacTheme::new(MacAppearance::DarkAqua);
let mut is_playing = false;
let mut progress = 0.25f32;
let mut volume = 0.8f32;
let mut playhead = 5.0f64;
let mut trim_in = 2.0f64;
let mut trim_out = 18.0f64;
let mut pan_dial = 0.0f32;
let mut rating = 4u32;

// 1. QuickTime floating HUD media player overlay (AVPlayerView)
ui.add(
    MacPlayerView::new()
        .controls_style(MacPlayerControlsStyle::Floating)
        .is_playing_mut(&mut is_playing)
        .progress_mut(&mut progress)
        .volume_mut(&mut volume)
        .duration_secs(60.0)
        .theme(&theme),
);

// 2. QuickTime yellow trim bar with waveform envelope (MacTrimScrubber)
let waveform = [0.2, 0.5, 0.8, 0.6, 0.9, 0.4, 0.7, 0.3];
MacTrimScrubber::new("clip_trim", 20.0, &mut trim_in, &mut trim_out, &mut playhead)
    .waveform(&waveform)
    .theme(&theme)
    .show(ui);

// 3. Rotary pan knob, VU level meter & 5-star rating indicator
ui.horizontal(|ui| {
    ui.add(MacCircularSlider::new(&mut pan_dial, -1.0..=1.0).theme(&theme));
    MacLevelIndicator::new(0.72, 0.0..=1.0)
        .style(MacLevelIndicatorStyle::Discrete)
        .peak_hold(0.88)
        .theme(&theme)
        .show(ui);
    ui.add(MacRatingIndicator::new(&mut rating, 5).theme(&theme));
});
# });
# output.textures_delta.clear();
```

---

## Component Reference

### Layout Containers (`src/layout/` & `src/controls/`)

| Component | macOS AppKit Equivalent | What It Does |
| :--- | :--- | :--- |
| [`MacSplitView`](src/layout/split_view.rs) | `NSSplitView` | Resizable two-pane split container with min/max size constraints. |
| [`MacStackView`](src/layout/stack_view.rs) | `NSStackView` | Horizontal or vertical linear stack with consistent spacing and optional dividers. |
| [`MacScrollView`](src/layout/scroll_view.rs) | `NSScrollView` | Scroll container with overlay scrollbars that fade out when idle. |
| [`MacForm`](src/layout/form.rs) | `NSGridView` / Form | Two-column form layout with right-aligned labels and left-aligned controls. |
| [`MacRow`](src/layout/row.rs) | Auto-Layout Row | Single-line horizontal row that distributes width cleanly across child cells. |
| [`MacFixedView`](src/layout/fixed_view.rs) | Retained Frame | Fixed-size coordinate container that prevents child widgets from overflowing. |
| [`MacBox`](src/controls/box_view.rs) | `NSBox` | Grouped border box with an optional title and standard horizontal separator. |
| [`MacTabView`](src/controls/tab_view.rs) | `NSTabView` | Tabbed container supporting top segmented tabs (`TopTabsBezelBorder`) or borderless pages. |
| [`MacDisclosureGroup`](src/controls/disclosure_group.rs) | `NSButton` Disclosure | Collapsible section with an animated rotating disclosure triangle (`CASpring`). |

### Controls & Data Views (`src/controls/`)

| Component | macOS AppKit Equivalent | What It Does |
| :--- | :--- | :--- |
| [`MacButton`](src/controls/button_cell.rs) / [`MacButtonCell`](src/controls/button_cell.rs) | `NSButton` | Push buttons, default action blue buttons, checkboxes, radio buttons, and icon buttons. |
| [`MacToggle`](src/controls/toggle.rs) | `NSSwitch` | Animated pill switch with system green active fill. |
| [`MacSegmentedControl`](src/controls/segmented.rs) | `NSSegmentedControl` | Multi-segment pill switcher with a sliding selection thumb. |
| [`MacInputCell`](src/controls/input_cell.rs) | `NSTextField` | Single-line text and numeric input with focus ring, `Tab` cycling, `Enter` commit, and `Escape` cancel. |
| [`MacSearchField`](src/controls/search_field.rs) | `NSSearchField` | Search input with leading magnifying glass icon and trailing clear (`×`) button. |
| [`MacComboBox`](src/controls/combo_box.rs) | `NSComboBox` | Editable text field combined with a filterable dropdown list (`Alt+Down`, arrow keys, `Enter`). |
| [`MacPopUpButton`](src/controls/popup_button.rs) | `NSPopUpButton` | Dropdown menu button with double-chevron indicator and screen-clamped popover. |
| [`MacSlider`](src/controls/slider.rs) | `NSSlider` (Linear) | Horizontal slider with optional tick marks and keyboard arrow stepping. |
| [`MacCircularSlider`](src/controls/circular_slider.rs) | `NSSlider` (Circular) | Rotary dial knob with drag tracking, angle indicator, and spring feedback. |
| [`MacStepper`](src/controls/stepper.rs) | `NSStepper` | Up/down numeric increment buttons with range clamping. |
| [`MacDatePicker`](src/controls/date_picker/mod.rs) | `NSDatePicker` | Date and time selector supporting both inline stepper fields and a month calendar + analog clock. |
| [`MacColorWell`](src/controls/color_well.rs) | `NSColorWell` | Color swatch button with alpha checkerboard preview and popover color picker. |
| [`MacProgressIndicator`](src/controls/progress_indicator.rs) | `NSProgressIndicator` | Determinate/indeterminate progress bars and 12-spoke spinning activity indicators. |
| [`MacLevelIndicator`](src/controls/level_indicator.rs) | `NSLevelIndicator` | Continuous or discrete audio/capacity meter with green/yellow/red thresholds and peak hold. |
| [`MacRatingIndicator`](src/controls/rating_indicator.rs) | `NSLevelIndicator` (Rating) | Interactive 5-star rating widget with optional half-star precision. |
| [`MacTokenField`](src/controls/token_field.rs) | `NSTokenField` | Tag input that turns comma- or Enter-separated words into removable capsule pills. |
| [`MacPathControl`](src/controls/path_control.rs) | `NSPathControl` | Interactive folder/file breadcrumb bar with clickable segments. |
| [`MacOutlineView`](src/controls/outline_view.rs) | `NSOutlineView` | Collapsible tree view for sidebars and hierarchical lists. |
| [`MacTableView`](src/controls/table_view/mod.rs) | `NSTableView` | Multi-column sortable table with alternating row fills, multi-selection, and viewport culling. |
| [`MacCollectionView`](src/controls/collection_view.rs) | `NSCollectionView` | 2D grid view with flow layout, rubber-band marquee selection, arrow-key navigation, and viewport culling. |
| [`MacRuleEditor`](src/controls/rule_editor.rs) | `NSRuleEditor` | Multi-row predicate filter builder with criterion pills and `[+]` / `[-]` row buttons. |
| [`MacPlayerView`](src/controls/player_view/mod.rs) | `AVPlayerView` | Media player transport overlay supporting Floating HUD pod, Inline bar, and Minimal styles. |
| [`MacTrimScrubber`](src/controls/trim_scrubber/mod.rs) | `AVPlayerView` Trim Bar | QuickTime-style timeline scrubber with waveform preview, yellow In/Out trim handles, and timecode badge. |

### Navigation, Surfaces & State (`src/navigation/`, `src/surfaces/`, `src/state/`)

| Component | macOS AppKit Equivalent | What It Does |
| :--- | :--- | :--- |
| [`MacToolbar`](src/navigation/toolbar.rs) | `NSToolbar` | Window titlebar with traffic light buttons (or Windows caption buttons), drag-to-move, and double-click maximize. |
| [`MacMenuBar`](src/navigation/menu.rs) | `NSMenu` | Top menu bar with cascading submenus and keyboard shortcut handling. |
| [`MacCommandPalette`](src/navigation/command_palette.rs) | Spotlight Overlay | `⌘K` / `Ctrl+K` modal command search bar with fuzzy matching. |
| [`MacCanvasFooter`](src/navigation/footer.rs) | Status Bar | `24 pt` bottom status bar with breadcrumbs, cursor coordinates, and zoom presets. |
| [`MacContextMenu`](src/feedback/context_menu.rs) | Context Menu | Right-click popup menu clamped to the window bounds. |
| [`MacToastManager`](src/feedback/toast.rs) | HUD Notification | Non-blocking toast banners with spring entry/exit animations. |
| [`MacPanel`](src/surfaces/mod.rs) / [`MacPopover`](src/surfaces/mod.rs) | `NSPanel` / `NSPopover` | Floating translucent utility panels, inspector cards ([`MacCard`](src/surfaces/card.rs)), and compact HUD boxes ([`MacFloatingInfoBox`](src/surfaces/floating_info_box.rs)). |
| [`NSUndoManager`](src/state/undo.rs) | `NSUndoManager` | Bounded undo/redo history stack with duplicate-state filtering. |

---

## Quality & Testing

Run the full test suite and linter with:

```bash
cargo test --all-targets
cargo test --doc
cargo clippy --all-targets -- -D warnings
```

- **22 integration test suites** and **30+ documentation tests** verify widget layout, keyboard navigation, undo/redo roundtrips, WCAG contrast ratios, and zero idle CPU usage.
- **Design token architecture**: All dimensions, spacing, corner radii, and semantic colors live in [`tokens/design_tokens.json`](tokens/design_tokens.json) and are resolved through [`MacTheme`].

---

## License

Dual-licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
