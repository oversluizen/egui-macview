//! Shared enums, font setup, and metadata constants for `macview_gallery`.

use egui::Color32;
use egui_macview::{setup_symbols_font, MacSymbol};

/// Returns `(width_scale, height_scale, radius_scale, unit_suffix)` for unit index (`0=pt, 1=px(2x Retina), 2=%`).
pub const fn unit_scale_and_suffix(unit_idx: usize) -> (f32, f32, f32, &'static str) {
    match unit_idx {
        1 => (2.0, 2.0, 2.0, "px"),
        2 => (100.0 / 260.0, 100.0 / 150.0, 100.0 / 40.0, "%"),
        _ => (1.0, 1.0, 1.0, "pt"),
    }
}

/// Returns `(width_label, height_label, radius_label)` static strings for unit index (`0=pt, 1=px, 2=%`).
pub const fn unit_row_labels(unit_idx: usize) -> (&'static str, &'static str, &'static str) {
    match unit_idx {
        1 => ("Width (px):", "Height (px):", "Radius (px):"),
        2 => ("Width (%):", "Height (%):", "Radius (%):"),
        _ => ("Width (pt):", "Height (pt):", "Radius (pt):"),
    }
}

/// Blends `top` over `base` using Porter-Duff / Quartz blend mode (`0=Normal, 1=Multiply, 2=Screen, 3=Overlay`).
pub fn apply_blend_mode(base: Color32, top: Color32, mode_idx: usize) -> Color32 {
    let (br, bg, bb) = (
        base.r() as f32 / 255.0,
        base.g() as f32 / 255.0,
        base.b() as f32 / 255.0,
    );
    let (tr, tg, tb) = (
        top.r() as f32 / 255.0,
        top.g() as f32 / 255.0,
        top.b() as f32 / 255.0,
    );
    let blend_ch = |b: f32, t: f32| -> f32 {
        match mode_idx {
            1 => b * t,
            2 => 1.0 - (1.0 - b) * (1.0 - t),
            3 if b < 0.5 => 2.0 * b * t,
            3 => 1.0 - 2.0 * (1.0 - b) * (1.0 - t),
            _ => t,
        }
    };
    Color32::from_rgba_unmultiplied(
        (blend_ch(br, tr).clamp(0.0, 1.0) * 255.0) as u8,
        (blend_ch(bg, tg).clamp(0.0, 1.0) * 255.0) as u8,
        (blend_ch(bb, tb).clamp(0.0, 1.0) * 255.0) as u8,
        top.a(),
    )
}

/// Formats uppercase section headers with optical tracking without intermediate `Vec` allocations.
pub fn format_tracked_small_caps(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 3);
    for (wi, word) in text.split_whitespace().enumerate() {
        if wi > 0 {
            out.push_str("   ");
        }
        for (ci, ch) in word.chars().enumerate() {
            if ci > 0 {
                out.push('\u{2009}');
            }
            out.extend(ch.to_uppercase());
        }
    }
    out
}

/// Loads local SF Pro / Inter / SF Mono / Cascadia fonts if present, plus the embedded vector symbol font.
pub fn setup_system_or_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let dir = std::path::Path::new(&std::env::var("LOCALAPPDATA").unwrap_or_default())
        .join("Microsoft\\Windows\\Fonts");
    for (n, p, fam) in [
        (
            "sf_pro",
            dir.join("SF-Pro-Text-Regular.otf"),
            egui::FontFamily::Proportional,
        ),
        (
            "inter",
            dir.join("Inter-Variable.ttf"),
            egui::FontFamily::Proportional,
        ),
        (
            "sf_mono",
            dir.join("SF-Mono-Regular.otf"),
            egui::FontFamily::Monospace,
        ),
        (
            "cascadia_mono",
            std::path::PathBuf::from("C:\\Windows\\Fonts\\CascadiaMono.ttf"),
            egui::FontFamily::Monospace,
        ),
    ] {
        if let Ok(bytes) = std::fs::read(&p) {
            fonts.font_data.insert(
                n.to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(fam)
                .or_default()
                .insert(0, n.to_owned());
        }
    }
    setup_symbols_font(&mut fonts);
    ctx.set_fonts(fonts);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GalleryScreen {
    #[default]
    CanvasPhysics = 0,
    VectorAssets = 1,
    DesignTokensWcag = 2,
    FinderDataBrowser = 3,
    AvKitMediaStudio = 4,
    AdvancedControls = 5,
}

impl GalleryScreen {
    pub const ALL: [Self; 6] = [
        Self::CanvasPhysics,
        Self::VectorAssets,
        Self::DesignTokensWcag,
        Self::FinderDataBrowser,
        Self::AvKitMediaStudio,
        Self::AdvancedControls,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Self::CanvasPhysics => "v0.1–v0.2: Vector & Inspector Studio",
            Self::VectorAssets => "v0.3: Symbols & Layout Lab",
            Self::DesignTokensWcag => "v0.4–v0.4.1: Command & State Lab",
            Self::FinderDataBrowser => "v0.5: Finder & Publishing Browser",
            Self::AvKitMediaStudio => "v0.5: AVKit Media & Timeline Studio",
            Self::AdvancedControls => "v0.6: Advanced Media, Scheduling & Grid Controls",
        }
    }

    pub fn sidebar_label(&self) -> &'static str {
        match self {
            Self::CanvasPhysics => "1. Vector Studio (v0.2)",
            Self::VectorAssets => "2. Symbols & Layout (v0.3)",
            Self::DesignTokensWcag => "3. Command & State (v0.4)",
            Self::FinderDataBrowser => "4. Finder & Publish (v0.5)",
            Self::AvKitMediaStudio => "5. AVKit Timeline (v0.5)",
            Self::AdvancedControls => "6. Advanced Controls (v0.6)",
        }
    }

    pub fn search_placeholder(&self) -> &'static str {
        match self {
            Self::CanvasPhysics => "Search canvas geometry, transforms, CASpring physics...",
            Self::VectorAssets => "Filter 14 AppKit vector symbols (e.g. search, chevron, gear)...",
            Self::DesignTokensWcag => "Filter W3C design tokens (e.g. button, metric, motion)...",
            Self::FinderDataBrowser => {
                "Filter v0.5 table assets by name, codec, or NSTokenField tag..."
            }
            Self::AvKitMediaStudio => "Search AVKit timeline scenes and audio channels...",
            Self::AdvancedControls => {
                "Filter advanced controls (combo, date, tab, collection, player, box)..."
            }
        }
    }

    pub fn breadcrumb_path(&self) -> [&'static str; 3] {
        match self {
            Self::CanvasPhysics => [
                "macview_gallery",
                "State 1 (v0.2)",
                "Vector & Inspector Studio",
            ],
            Self::VectorAssets => ["macview_gallery", "State 2 (v0.3)", "Symbols & Layout Lab"],
            Self::DesignTokensWcag => {
                ["macview_gallery", "State 3 (v0.4.1)", "Command & State Lab"]
            }
            Self::FinderDataBrowser => ["macview_gallery", "State 4 (v0.5)", "Finder & Publishing"],
            Self::AvKitMediaStudio => ["macview_gallery", "State 5 (v0.5)", "AVKit Media Timeline"],
            Self::AdvancedControls => ["macview_gallery", "State 6 (v0.6)", "Advanced Controls"],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorMode {
    DefaultNib = 0,
    CompactHud = 1,
    ExpandedPro = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpandedProTab {
    Attributes = 0,
    Metrics = 1,
    Matrix = 2,
    CodeSnippet = 3,
}

/// Metadata table for the 14 embedded Cupertino vector symbols.
pub const SYMBOLS_META: [(MacSymbol, &str, &str, &str); 14] = [
    (
        MacSymbol::Search,
        "Search",
        "Search fields, Spotlight",
        "U+F4A5",
    ),
    (
        MacSymbol::ChevronRight,
        "Chevron Right",
        "Navigation forward, disclosure",
        "U+F3D3",
    ),
    (
        MacSymbol::ChevronLeft,
        "Chevron Left",
        "Navigation back",
        "U+F3D2",
    ),
    (
        MacSymbol::ChevronDown,
        "Chevron Down",
        "Popup buttons, dropdown menus",
        "U+F5D5",
    ),
    (
        MacSymbol::ChevronUp,
        "Chevron Up",
        "Steppers, navigation up",
        "U+F5E5",
    ),
    (
        MacSymbol::SidebarLeading,
        "Sidebar Leading",
        "Sidebar toggle, window layout",
        "U+F7D6",
    ),
    (
        MacSymbol::Gear,
        "Gear",
        "Settings, preferences, metrics",
        "U+F43C",
    ),
    (
        MacSymbol::Sliders,
        "Sliders",
        "Filters, display adjustments",
        "U+F7DC",
    ),
    (
        MacSymbol::Checkmark,
        "Checkmark",
        "Menu checkmarks, verification",
        "U+F3FD",
    ),
    (
        MacSymbol::Xmark,
        "Xmark",
        "Close button, cancel, clear",
        "U+F404",
    ),
    (
        MacSymbol::Plus,
        "Plus",
        "Add layer, expand, zoom in",
        "U+F489",
    ),
    (
        MacSymbol::Minus,
        "Minus",
        "Remove layer, collapse, zoom out",
        "U+F70F",
    ),
    (
        MacSymbol::ArrowClockwise,
        "Arrow Clockwise",
        "Reload, refresh, repeat",
        "U+F49A",
    ),
    (
        MacSymbol::ArrowCounterClockwise,
        "Arrow Counter Clockwise",
        "Undo action, reset",
        "U+F21C",
    ),
];

/// Serializable / undoable snapshot for AVKit Studio controls.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AvKitStudioSnapshot {
    pub in_point: f64,
    pub out_point: f64,
    pub playhead: f64,
    pub gain: f32,
    pub pan: f32,
    pub exposure: f32,
    pub contrast: f32,
    pub zoom: f32,
    pub rating: u32,
}

impl AvKitStudioSnapshot {
    /// Compares studio tuning parameters, ignoring continuous playback playhead advancement.
    pub fn params_differ(&self, other: &Self) -> bool {
        (self.in_point - other.in_point).abs() > 1e-4
            || (self.out_point - other.out_point).abs() > 1e-4
            || (self.gain - other.gain).abs() > 1e-4
            || (self.pan - other.pan).abs() > 1e-4
            || (self.exposure - other.exposure).abs() > 1e-4
            || (self.contrast - other.contrast).abs() > 1e-4
            || (self.zoom - other.zoom).abs() > 1e-4
            || self.rating != other.rating
    }
}
