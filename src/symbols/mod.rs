//! AppKit Vector Symbols, Open-Source Inter Typography & CupertinoIcons Engine (SPEC-008 & v0.6.5).
//!
//! Provides monochrome, resolution-independent AppKit vector iconography powered by
//! the embedded MIT-licensed `CupertinoIcons` TrueType font alongside the open-source
//! SIL Open Font License 1.1 `Inter` (`Inter-Regular` and `Inter-SemiBold`) typeface.

use std::fmt;

/// Embedded raw bytes of the `CupertinoIcons.ttf` TrueType font asset (MIT License).
pub const CUPERTINO_ICONS_FONT_BYTES: &[u8] =
    include_bytes!("../../assets/fonts/CupertinoIcons.ttf");

/// Name key used to register CupertinoIcons in `egui::FontDefinitions`.
pub const CUPERTINO_ICONS_FONT_NAME: &str = "cupertino_icons";

/// Embedded raw bytes of the open-source `Inter-Regular.ttf` TrueType font asset (SIL OFL 1.1).
pub const INTER_REGULAR_FONT_BYTES: &[u8] =
    include_bytes!("../../assets/fonts/Inter-Regular.ttf");

/// Name key used to register `Inter-Regular` in `egui::FontDefinitions`.
pub const INTER_REGULAR_FONT_NAME: &str = "inter_regular";

/// Embedded raw bytes of the open-source `Inter-SemiBold.ttf` TrueType font asset (SIL OFL 1.1).
pub const INTER_SEMIBOLD_FONT_BYTES: &[u8] =
    include_bytes!("../../assets/fonts/Inter-SemiBold.ttf");

/// Name key used to register `Inter-SemiBold` in `egui::FontDefinitions` and `FontFamily::Name`.
pub const INTER_SEMIBOLD_FONT_NAME: &str = "Inter-SemiBold";

/// Enumeration of monochrome AppKit vector symbols mapped to CupertinoIcons font codepoints.
///
/// # Example
/// ```rust
/// use egui_macview::{MacSymbol, setup_symbols_font};
///
/// let mut fonts = egui::FontDefinitions::default();
/// setup_symbols_font(&mut fonts);
/// assert_eq!(MacSymbol::Search.as_str(), "\u{f4a5}");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MacSymbol {
    /// Magnifying glass search icon (`\u{f4a5}`).
    Search,
    /// Right-pointing navigation chevron (`\u{f3d3}`).
    ChevronRight,
    /// Left-pointing navigation chevron (`\u{f3d2}`).
    ChevronLeft,
    /// Downward-pointing disclosure chevron (`\u{f5d5}`).
    ChevronDown,
    /// Upward-pointing navigation chevron (`\u{f5e5}`).
    ChevronUp,
    /// Sidebar toggle / leading sidebar icon (`\u{f7d6}`).
    SidebarLeading,
    /// Settings gear icon (`\u{f43c}`).
    Gear,
    /// Horizontal sliders / filters icon (`\u{f7dc}`).
    Sliders,
    /// Checkmark verification icon (`\u{f3fd}`).
    Checkmark,
    /// Close / cancel 'x' mark icon (`\u{f404}`).
    Xmark,
    /// Add / plus icon (`\u{f489}`).
    Plus,
    /// Remove / minus icon (`\u{f70f}`).
    Minus,
    /// Clockwise reload / refresh icon (`\u{f49a}`).
    ArrowClockwise,
    /// Counter-clockwise undo / restart icon (`\u{f21c}`).
    ArrowCounterClockwise,
}

impl MacSymbol {
    /// Array of all 14 `MacSymbol` variants in canonical order.
    pub const ALL: [Self; 14] = [
        Self::Search,
        Self::ChevronRight,
        Self::ChevronLeft,
        Self::ChevronDown,
        Self::ChevronUp,
        Self::SidebarLeading,
        Self::Gear,
        Self::Sliders,
        Self::Checkmark,
        Self::Xmark,
        Self::Plus,
        Self::Minus,
        Self::ArrowClockwise,
        Self::ArrowCounterClockwise,
    ];

    /// Returns the character representing the symbol in the CupertinoIcons font.
    #[inline]
    pub const fn as_char(&self) -> char {
        match self {
            Self::Search => '\u{f4a5}',
            Self::ChevronRight => '\u{f3d3}',
            Self::ChevronLeft => '\u{f3d2}',
            Self::ChevronDown => '\u{f5d5}',
            Self::ChevronUp => '\u{f5e5}',
            Self::SidebarLeading => '\u{f7d6}',
            Self::Gear => '\u{f43c}',
            Self::Sliders => '\u{f7dc}',
            Self::Checkmark => '\u{f3fd}',
            Self::Xmark => '\u{f404}',
            Self::Plus => '\u{f489}',
            Self::Minus => '\u{f70f}',
            Self::ArrowClockwise => '\u{f49a}',
            Self::ArrowCounterClockwise => '\u{f21c}',
        }
    }

    /// Returns the string representation of the symbol.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Search => "\u{f4a5}",
            Self::ChevronRight => "\u{f3d3}",
            Self::ChevronLeft => "\u{f3d2}",
            Self::ChevronDown => "\u{f5d5}",
            Self::ChevronUp => "\u{f5e5}",
            Self::SidebarLeading => "\u{f7d6}",
            Self::Gear => "\u{f43c}",
            Self::Sliders => "\u{f7dc}",
            Self::Checkmark => "\u{f3fd}",
            Self::Xmark => "\u{f404}",
            Self::Plus => "\u{f489}",
            Self::Minus => "\u{f70f}",
            Self::ArrowClockwise => "\u{f49a}",
            Self::ArrowCounterClockwise => "\u{f21c}",
        }
    }

    /// Returns the 32-bit Unicode scalar codepoint of the symbol.
    #[inline]
    pub const fn codepoint(&self) -> u32 {
        self.as_char() as u32
    }
}

impl fmt::Display for MacSymbol {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for MacSymbol {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Registers the open-source `Inter` (`Inter-Regular` and `Inter-SemiBold`) and `CupertinoIcons`
/// TrueType fonts with the provided [`egui::FontDefinitions`].
///
/// - Installs `inter_regular` at the front of `FontFamily::Proportional` so all default proportional UI
///   text renders with `Inter` (matching `SF Pro Text` metrics across Windows, Linux, macOS, and Web).
/// - Installs `Inter-SemiBold` under `FontFamily::Name("Inter-SemiBold".into())` for genuine Semibold/Bold
///   rendering in [`crate::MacLabel`] and [`crate::MacDisclosureGroup`] headers.
/// - Installs `cupertino_icons` in `FontFamily::Name("cupertino_icons".into())`, `FontFamily::Proportional`,
///   and `FontFamily::Monospace`.
///
/// This function is idempotent: calling it multiple times will not insert duplicate entries.
pub fn setup_symbols_font(fonts: &mut egui::FontDefinitions) {
    fonts.font_data.insert(
        INTER_REGULAR_FONT_NAME.to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(INTER_REGULAR_FONT_BYTES)),
    );
    fonts.font_data.insert(
        INTER_SEMIBOLD_FONT_NAME.to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(INTER_SEMIBOLD_FONT_BYTES)),
    );
    fonts.font_data.insert(
        CUPERTINO_ICONS_FONT_NAME.to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(CUPERTINO_ICONS_FONT_BYTES)),
    );

    let prop_family = fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default();
    if !prop_family.iter().any(|f| f == INTER_REGULAR_FONT_NAME) {
        prop_family.insert(0, INTER_REGULAR_FONT_NAME.to_owned());
    }
    if !prop_family.iter().any(|f| f == CUPERTINO_ICONS_FONT_NAME) {
        prop_family.push(CUPERTINO_ICONS_FONT_NAME.to_owned());
    }

    let mono_family = fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default();
    if !mono_family.iter().any(|f| f == CUPERTINO_ICONS_FONT_NAME) {
        mono_family.push(CUPERTINO_ICONS_FONT_NAME.to_owned());
    }

    let semibold_family = fonts
        .families
        .entry(egui::FontFamily::Name(INTER_SEMIBOLD_FONT_NAME.into()))
        .or_default();
    if !semibold_family.iter().any(|f| f == INTER_SEMIBOLD_FONT_NAME) {
        semibold_family.insert(0, INTER_SEMIBOLD_FONT_NAME.to_owned());
    }
    if !semibold_family.iter().any(|f| f == CUPERTINO_ICONS_FONT_NAME) {
        semibold_family.push(CUPERTINO_ICONS_FONT_NAME.to_owned());
    }

    let icon_family = fonts
        .families
        .entry(egui::FontFamily::Name(CUPERTINO_ICONS_FONT_NAME.into()))
        .or_default();
    if !icon_family.iter().any(|f| f == CUPERTINO_ICONS_FONT_NAME) {
        icon_family.insert(0, CUPERTINO_ICONS_FONT_NAME.to_owned());
    }
}

/// Convenience alias for [`setup_symbols_font`] registering both open-source `Inter` and `CupertinoIcons`.
pub fn setup_macview_fonts(fonts: &mut egui::FontDefinitions) {
    setup_symbols_font(fonts);
}
