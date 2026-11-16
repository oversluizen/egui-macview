//! Live Rust Code Snippet Generator and Interactive Developer / AI Export Tool.

use super::{CardModel, CenterState, GalleryScreen};
use egui_macview::MacSymbol;

/// Lightweight snapshot of studio center state used for live code snippet generation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SnippetContext {
    pub selected_symbol: MacSymbol,
    pub selected_table_row: usize,
    pub media_in_point: f64,
    pub media_out_point: f64,
    pub media_playhead: f64,
    pub media_gain: f32,
    pub audio_level_l: f32,
    pub audio_peak_l: f32,
}

impl Default for SnippetContext {
    fn default() -> Self {
        Self {
            selected_symbol: MacSymbol::Search,
            selected_table_row: 0,
            media_in_point: 4.0,
            media_out_point: 26.0,
            media_playhead: 4.0,
            media_gain: 1.0,
            audio_level_l: 7.4,
            audio_peak_l: 8.2,
        }
    }
}

impl From<&CenterState> for SnippetContext {
    fn from(c: &CenterState) -> Self {
        Self {
            selected_symbol: c.selected_symbol,
            selected_table_row: c.selected_table_rows.iter().next().copied().unwrap_or(0),
            media_in_point: c.media_in_point,
            media_out_point: c.media_out_point,
            media_playhead: c.media_playhead,
            media_gain: c.media_gain,
            audio_level_l: c.audio_level_l,
            audio_peak_l: c.audio_peak_l,
        }
    }
}

/// Generates idiomatic, ready-to-run Rust code for the active gallery screen and control state.
pub fn generate_rust_snippet(screen: GalleryScreen, card: &CardModel, ctx: &SnippetContext) -> String {
    match screen {
        GalleryScreen::CanvasPhysics => format!(
            r#"// egui-macview v0.5.2: Vector Card & CASpring Physics
use egui_macview::{{theme::MacTheme, CASpring, MacCard, MacSlider}};

let theme = MacTheme::default();

// 1. Analytical CASpring CoreAnimation Physics
let spring = CASpring::from_perceptual(0.35, 0.65);
let displacement = spring.evaluate(ui.input(|i| i.time));

// 2. Interactive AppKit Card Surface
MacCard::new("hero_vector_card")
    .theme(&theme)
    .width({:.1})
    .height({:.1})
    .corner_radius({:.1})
    .shadow(true)
    .show(ui, |card_ui| {{
        card_ui.heading({:?});
        card_ui.label("Rotation: {:.1}° | Scale: {:.0}%");
    }});
"#,
            card.width.parse::<f32>().unwrap_or(260.0),
            card.height.parse::<f32>().unwrap_or(150.0),
            card.radius.parse::<f32>().unwrap_or(10.0),
            card.layer_name,
            card.rotation_deg,
            card.scale_pct,
        ),
        GalleryScreen::VectorAssets => format!(
            r#"// egui-macview v0.5.2: Embedded Cupertino Vector Symbols & Fixed Layout
use egui_macview::{{theme::MacTheme, MacFixedView, MacRow, MacSymbol}};

let theme = MacTheme::default();

// 1. Embedded Cupertino Vector Symbol (Zero Assets, Zero Disk Footprint)
let symbol = MacSymbol::{:?};
ui.label(symbol.as_str());

// 2. Pixel-Perfect MacRow Budget Layout
MacRow::new("studio_symbol_row")
    .theme(&theme)
    .height(28.0)
    .show(ui, |row| {{
        row.label("Active Symbol:");
        row.symbol(symbol);
    }});
"#,
            ctx.selected_symbol
        ),
        GalleryScreen::DesignTokensWcag => r#"// egui-macview v0.5.2: Design Tokens, WCAG 2.1 AA Contrast & NSUndoManager
use egui_macview::{theme::MacTheme, contrast_ratio, MacAppearance, MacSegmentedControl, NSUndoManager};

let theme = MacTheme::new(MacAppearance::Aqua);
let mut undo_mgr: NSUndoManager<f32> = NSUndoManager::new();

// Evaluate authentic WCAG 2.1 AA relative luminance & contrast ratio
let contrast = contrast_ratio(theme.label(), theme.window_background());
assert!(contrast >= 4.5, "WCAG 2.1 AA compliance verified (ratio: {contrast:.2})");
"#
        .to_string(),
        GalleryScreen::FinderDataBrowser => format!(
            r#"// egui-macview v0.5.2: Finder Data Browser (NSPathControl, NSTableView, NSTokenField)
use egui_macview::{{
    theme::MacTheme, MacPathComponent, MacPathControl, MacProgressControlSize,
    MacProgressIndicator, MacTableColumn, MacTableView, MacTokenField,
}};
use std::collections::BTreeSet;

let theme = MacTheme::default();
let mut selected_rows = BTreeSet::from([{}]);
let mut tags = vec!["ProRes 422".to_string(), "Master Deliverable".to_string()];
let mut draft = String::new();

// 1. NSPathControl Breadcrumbs
let path = [MacPathComponent::folder("Projects"), MacPathComponent::file("Master.mov")];
MacPathControl::new("finder_path", &path).theme(&theme).show(ui);

// 2. NSTokenField Metadata Tags
MacTokenField::new("asset_tags", &mut tags, &mut draft).theme(&theme).show(ui);
"#,
            ctx.selected_table_row
        ),
        GalleryScreen::AvKitMediaStudio => format!(
            r#"// egui-macview v0.5.2: AVKit Media & Timeline Studio (MacTrimScrubber, MacCircularSlider & MacLevelIndicator)
use egui_macview::{{
    theme::MacTheme, MacCircularSlider, MacLevelIndicator, MacLevelIndicatorStyle,
    MacLevelOrientation, MacRatingIndicator, MacTimelineZone, MacTrimScrubber,
}};

let theme = MacTheme::default();
let mut in_point = {:.1};
let mut out_point = {:.1};
let mut playhead = {:.1};
let mut gain = {:.2};
let mut rating = 5u32;

// 1. AVPlayerView Inline Trim Scrubber
let zones = [MacTimelineZone::new(18.0, 46.0, "Dialogue")];
MacTrimScrubber::new("avkit_scrubber", 120.0, &mut in_point, &mut out_point, &mut playhead)
    .theme(&theme)
    .zones(&zones)
    .show(ui);

// 2. Rotary Circular Gain Dial (NSSlider Circular)
MacCircularSlider::new(&mut gain, 0.0..=2.0).theme(&theme).diameter(36.0).show(ui);

// 3. Stereo Audio VU Meters with Peak-Hold
MacLevelIndicator::new({:.1}, 0.0..=10.0)
    .theme(&theme)
    .style(MacLevelIndicatorStyle::Discrete)
    .warning_value(6.5)
    .critical_value(8.5)
    .peak_hold({:.1})
    .show(ui);

// 4. Asset Rating Indicator (1..=5 stars)
MacRatingIndicator::new(&mut rating, 5).theme(&theme).show(ui);
"#,
            ctx.media_in_point, ctx.media_out_point, ctx.media_playhead, ctx.media_gain, ctx.audio_level_l, ctx.audio_peak_l,
        ),
        GalleryScreen::AdvancedControls => r#"// Milestone 0.6.8: AppKit NIB Inspector Matrix, Baseline & Surface Vibrancy (SPEC-012)
use egui_macview::{
    MacBox, MacCheckbox, MacComboBox, MacControlSize, MacDisclosureGroup,
    MacFontWeight, MacHelpButton, MacInlineActionButton, MacInputCell,
    MacLabel, MacLabelColor, MacPopUpButton, MacTheme,
};

let theme = MacTheme::default();
let (mut expanded, mut group_enabled, mut checked) = (true, true, true);
let (mut width, mut unit_idx, mut preset_idx) = ("1920".to_string(), 0usize, 0usize);
let mut combo_text = "Apple ProRes 422 HQ".to_string();

MacBox::new("Inspector Group").theme(&theme).show(ui, |ui| {
    MacDisclosureGroup::new("disc", "Media Settings", &mut expanded)
        .theme(&theme)
        .with_trailing_add_button(true)
        .show(ui, |disc_ui| {
            MacCheckbox::new(&mut group_enabled, "Enable Group").theme(&theme).show(disc_ui);
            disc_ui.add_enabled_ui(group_enabled, |ui| {
                MacLabel::new("Width:").weight(MacFontWeight::Semibold).theme(&theme).show(ui);
                MacInputCell::new("w_in", &mut width).width(64.0).theme(&theme).show(ui);
                MacPopUpButton::new(&mut unit_idx, &["pt", "px", "%"]).theme(&theme).show(ui);
                MacPopUpButton::new(&mut preset_idx, &["Sonoma 14.8"]).accent_badge(true).theme(&theme).show(ui);
                MacCheckbox::new(&mut checked, "Enable HDR").control_size(MacControlSize::Regular).theme(&theme).show(ui);
                MacComboBox::new("cb", &mut combo_text, &["ProRes 422", "H.265"]).theme(&theme).show(ui);
                MacLabel::new("WCAG Vibrancy").color(MacLabelColor::Secondary).theme(&theme).show(ui);
                MacInlineActionButton::new("+").theme(&theme).show(ui);
                MacHelpButton::new().theme(&theme).show(ui);
            });
        });
});
"#
        .to_string(),
    }
}
