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
egui-macview = "0.6.4"
```

To run the interactive showcase gallery included in this repository:

```bash
cargo run --example macview_gallery
```

---

## Your First App in 60 Seconds (`eframe`)

Here is a complete, runnable desktop application (`src/main.rs`) with a macOS window toolbar, a resizable sidebar split view, and a two-column settings form:

```rust,no_run
use eframe::egui;
use egui_macview::{
    MacAppearance, MacBox, MacButton, MacForm, MacInputCell, MacSegmentedControl,
    MacSplitOrientation, MacSplitView, MacTheme, MacToggle, MacToolbar, WindowControlsStyle,
    setup_symbols_font,
};

struct MyMacApp {
    appearance: MacAppearance,
    sidebar_width: f32,
    project_name: String,
    auto_save: bool,
    view_mode: usize,
}

impl Default for MyMacApp {
    fn default() -> Self {
        Self {
            appearance: MacAppearance::DarkAqua,
            sidebar_width: 200.0,
            project_name: String::from("Untitled Studio Project"),
            auto_save: true,
            view_mode: 0,
        }
    }
}

impl eframe::App for MyMacApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let theme = MacTheme::new(self.appearance);
        theme.apply_to_egui(ui.ctx());

        // Top window toolbar
        MacToolbar::new("My Studio App")
            .window_controls_style(WindowControlsStyle::AppleTrafficLights)
            .theme(&theme)
            .show(ui, |toolbar_ui| {
                let modes = ["Editor", "Inspector"];
                MacSegmentedControl::new(&modes, self.view_mode)
                    .theme(&theme)
                    .show(toolbar_ui, |idx| self.view_mode = idx);

                let label = match self.appearance {
                    MacAppearance::Aqua => "Switch to Dark",
                    _ => "Switch to Light",
                };
                if toolbar_ui.add(MacButton::new(label).theme(&theme)).clicked() {
                    self.appearance = match self.appearance {
                        MacAppearance::Aqua => MacAppearance::DarkAqua,
                        _ => MacAppearance::Aqua,
                    };
                }
            });

        // Main split view with sidebar and inspector form
        MacSplitView::with_pos(
            "workspace_split",
            MacSplitOrientation::Horizontal,
            &mut self.sidebar_width,
        )
        .min_first(150.0)
        .max_first(320.0)
        .theme(&theme)
        .show(
            ui,
            |sidebar_ui| {
                sidebar_ui.heading("Project Files");
                sidebar_ui.label("• main.rs");
                sidebar_ui.label("• Cargo.toml");
            },
            |content_ui| {
                MacBox::new("Project Preferences")
                    .theme(&theme)
                    .show(content_ui, |box_ui| {
                        MacForm::new("prefs_form").theme(&theme).show(box_ui, |form| {
                            form.row_label("Project Name:", |ui| {
                                MacInputCell::new("proj_name", &mut self.project_name)
                                    .theme(&theme)
                                    .show(ui);
                            });
                            form.row_label("Auto-Save:", |ui| {
                                ui.add(MacToggle::new(&mut self.auto_save).theme(&theme));
                            });
                        });
                    });
            },
        );
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "My Studio App",
        options,
        Box::new(|cc| {
            // Register the embedded Cupertino vector icon font once at startup:
            let mut fonts = egui::FontDefinitions::default();
            setup_symbols_font(&mut fonts);
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(MyMacApp::default()))
        }),
    )
}
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
                MacInputCell::new("project_name_input", &mut project_name)
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
| [`MacSplitView`] | `NSSplitView` | Resizable two-pane split container with min/max size constraints. |
| [`MacStackView`] | `NSStackView` | Horizontal or vertical linear stack with consistent spacing and optional dividers. |
| [`MacScrollView`] | `NSScrollView` | Scroll container with overlay scrollbars that fade out when idle. |
| [`MacForm`] | `NSGridView` / Form | Two-column form layout with right-aligned labels and left-aligned controls. |
| [`MacRow`] | Auto-Layout Row | Single-line horizontal row that distributes width cleanly across child cells. |
| [`MacFixedView`] | Retained Frame | Fixed-size coordinate container that prevents child widgets from overflowing. |
| [`MacBox`] | `NSBox` | Grouped border box with an optional title and standard horizontal separator. |
| [`MacTabView`] | `NSTabView` | Tabbed container supporting top segmented tabs (`TopTabsBezelBorder`) or borderless pages. |
| [`MacDisclosureGroup`] | `NSButton` Disclosure | Collapsible section with an animated rotating disclosure triangle (`CASpring`). |

### Controls & Data Views (`src/controls/`)

| Component | macOS AppKit Equivalent | What It Does |
| :--- | :--- | :--- |
| [`MacButton`] / [`MacButtonCell`] | `NSButton` | Push buttons, default action blue buttons, checkboxes, radio buttons, and icon buttons. |
| [`MacToggle`] | `NSSwitch` | Animated pill switch with system green active fill. |
| [`MacSegmentedControl`] | `NSSegmentedControl` | Multi-segment pill switcher with a sliding selection thumb. |
| [`MacInputCell`] | `NSTextField` | Single-line text and numeric input with focus ring, `Tab` cycling, `Enter` commit, and `Escape` cancel. |
| [`MacSearchField`] | `NSSearchField` | Search input with leading magnifying glass icon and trailing clear (`×`) button. |
| [`MacComboBox`] | `NSComboBox` | Editable text field combined with a filterable dropdown list (`Alt+Down`, arrow keys, `Enter`). |
| [`MacPopUpButton`] | `NSPopUpButton` | Dropdown menu button with double-chevron indicator and screen-clamped popover. |
| [`MacSlider`] | `NSSlider` (Linear) | Horizontal slider with optional tick marks and keyboard arrow stepping. |
| [`MacCircularSlider`] | `NSSlider` (Circular) | Rotary dial knob with drag tracking, angle indicator, and spring feedback. |
| [`MacStepper`] | `NSStepper` | Up/down numeric increment buttons with range clamping. |
| [`MacDatePicker`] | `NSDatePicker` | Date and time selector supporting both inline stepper fields and a month calendar + analog clock. |
| [`MacColorWell`] | `NSColorWell` | Color swatch button with alpha checkerboard preview and popover color picker. |
| [`MacProgressIndicator`] | `NSProgressIndicator` | Determinate/indeterminate progress bars and 12-spoke spinning activity indicators. |
| [`MacLevelIndicator`] | `NSLevelIndicator` | Continuous or discrete audio/capacity meter with green/yellow/red thresholds and peak hold. |
| [`MacRatingIndicator`] | `NSLevelIndicator` (Rating) | Interactive 5-star rating widget with optional half-star precision. |
| [`MacTokenField`] | `NSTokenField` | Tag input that turns comma- or Enter-separated words into removable capsule pills. |
| [`MacPathControl`] | `NSPathControl` | Interactive folder/file breadcrumb bar with clickable segments. |
| [`MacOutlineView`] | `NSOutlineView` | Collapsible tree view for sidebars and hierarchical lists. |
| [`MacTableView`] | `NSTableView` | Multi-column sortable table with alternating row fills, multi-selection, and viewport culling. |
| [`MacCollectionView`] | `NSCollectionView` | 2D grid view with flow layout, rubber-band marquee selection, arrow-key navigation, and viewport culling. |
| [`MacRuleEditor`] | `NSRuleEditor` | Multi-row predicate filter builder with criterion pills and `[+]` / `[-]` row buttons. |
| [`MacPlayerView`] | `AVPlayerView` | Media player transport overlay supporting Floating HUD pod, Inline bar, and Minimal styles. |
| [`MacTrimScrubber`] | `AVPlayerView` Trim Bar | QuickTime-style timeline scrubber with waveform preview, yellow In/Out trim handles, and timecode badge. |

### Navigation, Surfaces & State (`src/navigation/`, `src/surfaces/`, `src/state/`)

| Component | macOS AppKit Equivalent | What It Does |
| :--- | :--- | :--- |
| [`MacToolbar`] | `NSToolbar` | Window titlebar with traffic light buttons (or Windows caption buttons), drag-to-move, and double-click maximize. |
| [`MacMenuBar`] | `NSMenu` | Top menu bar with cascading submenus and keyboard shortcut handling. |
| [`MacCommandPalette`] | Spotlight Overlay | `⌘K` / `Ctrl+K` modal command search bar with fuzzy matching. |
| [`MacCanvasFooter`] | Status Bar | `24 pt` bottom status bar with breadcrumbs, cursor coordinates, and zoom presets. |
| [`MacContextMenu`] | Context Menu | Right-click popup menu clamped to the window bounds. |
| [`MacToastManager`] | HUD Notification | Non-blocking toast banners with spring entry/exit animations. |
| [`MacPanel`] / [`MacPopover`] | `NSPanel` / `NSPopover` | Floating translucent utility panels, inspector cards ([`MacCard`]), and compact HUD boxes ([`MacFloatingInfoBox`]). |
| [`NSUndoManager`] | `NSUndoManager` | Bounded undo/redo history stack with duplicate-state filtering. |

---

## When to Use Which Control (Decision Guide)

Not sure which widget fits your screen best? Use these guidelines modeled after Apple's macOS Human Interface Guidelines:

### 1. Choosing a Selection Control

| Control | Best For | Avoid When |
| :--- | :--- | :--- |
| **[`MacSegmentedControl`]** | Switching between **2 to 5 closely related views or modes** (e.g., `Grid \| List \| Columns`) where all options should be visible at a glance. | You have more than 5 options, or the labels are long phrases. Use [`MacPopUpButton`] instead. |
| **[`MacPopUpButton`]** | Picking **one option from a fixed list of 5 to 20 items** (e.g., file export format or color profile) where the user does not need to type custom values. | The list has dozens of items that need text filtering, or the user can enter a custom value. Use [`MacComboBox`] instead. |
| **[`MacComboBox`]** | Picking from a **searchable list OR typing a custom value** (e.g., choosing a font family or entering a custom zoom preset). | The user must only pick from a small, strictly fixed set of options. Use [`MacPopUpButton`] instead. |

### 2. Choosing a Data View

| Control | Best For | Avoid When |
| :--- | :--- | :--- |
| **[`MacTableView`]** | Displaying **structured records with multiple attributes** (e.g., Name, Date Modified, Size) that users sort by column header or navigate row-by-row. | Items are primarily visual (thumbnails, icons, color swatches). Use [`MacCollectionView`] instead. |
| **[`MacOutlineView`]** | Navigating **hierarchical trees** (e.g., folder trees, document outlines, scene graphs, or sidebar navigation) with collapsible parent nodes. | Your data is flat and has no parent-child hierarchy. Use [`MacTableView`] instead. |
| **[`MacCollectionView`]** | Browsing **visual 2D grids of tiles** (e.g., photo galleries, icon pickers, asset browsers) with rubber-band marquee selection and 2D arrow-key navigation. | Users need to compare text or numbers across multiple columns. Use [`MacTableView`] instead. |

### 3. Choosing a Layout Container

| Container | Best For | Avoid When |
| :--- | :--- | :--- |
| **[`MacSplitView`]** | Dividing a window into **resizable workspace panes** (e.g., a left navigation sidebar, a center canvas, and a right inspector). | Grouping small related controls inside a single pane. Use [`MacBox`] or [`MacForm`] instead. |
| **[`MacTabView`]** | Switching an entire pane between **peer pages** (e.g., `General`, `Shortcuts`, `Export` preference tabs) where only one page is visible at a time. | Hiding optional advanced settings on the same page. Use [`MacDisclosureGroup`] instead. |
| **[`MacDisclosureGroup`]** | Toggling **optional or advanced details** inline so the user can expand multiple sections simultaneously. | Top-level window navigation. Use [`MacTabView`] or [`MacSplitView`] instead. |
| **[`MacBox`]** | Visually **grouping a cluster of related fields** under a subtle border and title label. | Aligning two-column `Label: Control` pairs. Put a [`MacForm`] inside the [`MacBox`] for alignment. |

### 4. Choosing a Numeric Input

| Control | Best For |
| :--- | :--- |
| **[`MacSlider`]** | Adjusting a continuous value within a bounded range (e.g., Opacity `0%–100%` or Volume) where visual proportion matters more than exact digits. |
| **[`MacStepper`]** | Nudging an exact numeric value up or down by fixed increments (e.g., `±1 px` border width or `±25%` zoom) next to a readout or text field. |
| **[`MacCircularSlider`]** | Compact rotary dials in audio mixers, pan pots, or angle/rotation inspectors where vertical space is tight. |
| **[`MacInputCell::numeric`]** | Typing an exact number with keyboard arrow stepping (`Up`/`Down` for `±1×`, `Shift` for `±10×`, `Cmd`/`Ctrl` for `±100×`) and `Escape` rollback. |

---

## Troubleshooting & FAQ

### 1. Why do icons or symbols show up as empty boxes (`□`)?
`egui-macview` uses an embedded monochrome vector icon font (`CupertinoIcons.ttf`) so symbols render identically on Windows, macOS, Linux, and Web. You must register this font once on your `egui::Context` at startup:

```rust
use egui_macview::setup_symbols_font;

# let ctx = egui::Context::default();
let mut fonts = egui::FontDefinitions::default();
setup_symbols_font(&mut fonts);
ctx.set_fonts(fonts);
```

### 2. How do I switch dynamically between Light (`Aqua`), Dark (`DarkAqua`), and `Anthracite` themes?
Create a [`MacTheme`] with your chosen [`MacAppearance`] and call `theme.apply_to_egui(ctx)`. Calling `apply_to_egui` updates `egui`'s background visuals and stores the theme inside `egui::Context` so every `egui-macview` widget automatically uses it:

```rust
use egui_macview::{MacAppearance, MacButton, MacTheme};

# let ctx = egui::Context::default();
let theme = MacTheme::new(MacAppearance::Anthracite);
theme.apply_to_egui(&ctx);

# let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
// Because `apply_to_egui` was called, widgets automatically pick up `Anthracite`
// even when you don't pass `.theme(&theme)` explicitly:
ui.add(MacButton::new("Automatically Themed"));
# });
# out.textures_delta.clear();
```

### 3. How do I avoid widget ID collisions when rendering controls in a `for` loop?
Just like standard `egui`, stateful controls (`MacComboBox`, `MacTableView`, `MacDisclosureGroup`, `MacInputCell`, `MacRuleEditor`) use an ID salt to store popup or focus state across frames. When rendering multiple instances in a loop, either pass a unique tuple `(name, index)` as the ID salt or wrap each iteration in `ui.push_id`:

```rust
use egui_macview::MacComboBox;

# let ctx = egui::Context::default();
# let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
let options = ["None", "Fade", "Dissolve"];
let mut row_values = vec![String::from("Fade"), String::from("None")];

for (row_idx, val) in row_values.iter_mut().enumerate() {
    // Pass `("transition_combo", row_idx)` so each row gets a distinct egui::Id:
    MacComboBox::new(("transition_combo", row_idx), val, &options).show(ui);
}
# });
# out.textures_delta.clear();
```

### 4. How does battery-saving idle repaint work?
Immediate-mode GUIs often burn CPU and laptop battery by repainting at 60–120 FPS even when nothing is moving. `egui-macview` avoids this automatically:
- **Spring animations (`CASpring`)** request repaints only while a transition is in flight. As soon as a toggle switch, segmented pill, or disclosure chevron settles at its target position, it stops requesting repaints (`0% CPU` at rest).
- **Floating media HUD (`MacPlayerView`)** schedules a single wakeup timer (`request_repaint_after_secs`) for its 3.0-second auto-hide instead of polling every frame.
- **Unfocused & minimized windows**: Continuous animations (such as indeterminate progress spinners) check `is_window_active(ctx)` and `ui.is_rect_visible(rect)` and pause automatically when your app window is in the background or minimized.

---

## Quality & Testing

Run the full test suite and linter with:

```bash
cargo test --all-targets
cargo test --doc
cargo clippy --all-targets -- -D warnings
```

- **23 integration test suites** and **60+ documentation tests** verify widget layout, keyboard navigation, undo/redo roundtrips, WCAG contrast ratios, and zero idle CPU usage.
- **Design token architecture**: All dimensions, spacing, corner radii, and semantic colors live in `tokens/design_tokens.json` ([`MacDesignTokens`]) and are resolved through [`MacTheme`].

---

## License

Dual-licensed under either of:
- Apache License, Version 2.0 (<https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license (<https://opensource.org/licenses/MIT>)

at your option.
