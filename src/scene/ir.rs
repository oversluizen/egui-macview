//! Layer 2: Strongly Typed `MacSceneIR`, `SpatialSemanticNormalizer`, `SceneSearchIndex`,
//! `VisibilityRule`, and `SceneContractVerifier` (`SPEC-013 §3`).

use crate::controls::{MacControlSize, MacFontWeight, MacLabelColor};
use crate::scene::constraints::{CompiledConstraintSchedule, ConstraintAttr, LayoutConstraintSpec, ValueTransformer};
use crate::scene::normalizer::run_spatial_semantic_normalizer;
pub use crate::scene::normalizer::{
    normalize_nib_alignment_rect, ContractAuditReport, CustomClassMapper, HostContractSchema, SceneFilterMask,
};
use egui::{Rect, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Arena node, binding, and action identifiers inside [`MacSceneIR`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u32);
/// Pre-interned binding handle for zero-allocation `MacSceneDataSource` lookups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BindingId(pub u32);
/// Pre-interned action selector handle for zero-allocation `MacSceneDataSource` dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ActionId(pub u32);

/// Expected value type of a state binding in [`HostContractSchema`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindingValueType {
    /// Boolean state (`Checkbox`, `RadioButton`, `Switch`, `enabled`).
    Bool,
    /// Floating-point state (`NumberField`, `Slider`).
    F64,
    /// Integer index or counter state (`PopUpButton`, `SegmentedControl`, `Stepper`).
    I64,
    /// Text string state (`TextField`, `TextView`, `SearchField`, `ComboBox`).
    String,
}

/// Declarative context and workspace visibility rule (`REQ-013-02` §3.4).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VisibilityRule {
    /// Node is always visible (unless filtered out by an active search query).
    #[default]
    Always,
    /// Node is visible only when `(active_workspace_flags & mask) != 0`.
    WhenWorkspaceFlagsContain(u64),
    /// Node is visible only when a boolean binding evaluates to `true`.
    WhenFlagTrue(BindingId),
    /// Node is visible only when its parent `TabView` (`tab_view`) has `active_tab == tab_index`.
    WhenTabSelected {
        /// Enclosing `TabView` node ID.
        tab_view: NodeId,
        /// Zero-based `<tabViewItem>` index.
        tab_index: usize,
    },
}

impl VisibilityRule {
    /// Evaluates whether the rule permits visibility under `workspace_flags`.
    #[inline]
    pub const fn is_visible(&self, workspace_flags: u64) -> bool {
        match self {
            Self::Always | Self::WhenFlagTrue(_) | Self::WhenTabSelected { .. } => true,
            Self::WhenWorkspaceFlagsContain(mask) => (workspace_flags & *mask) != 0,
        }
    }
}

/// Dual-mode layout policy (`REQ-013-03` §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutPolicy {
    /// Mode A: 100% pixel-exact AppKit `.nib` coordinates, `NSFrame -> alignmentRect` insets, and 6-bit `autoresizingMask`.
    ExactAppleNib,
    /// Mode B: Modern overhaul mode with grouped section cards, automatic toolbar gap closing, and live search collapse.
    AdaptiveSemanticReflow,
}

/// Strongly typed contemporary AppKit widget and container taxonomy (`REQ-013-02` §3.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SceneNodeKind {
    /// Root or intermediate container view (`NSView`).
    Container {
        /// Whether $y=0$ is at the top (`true`) or bottom (`false`).
        is_flipped: bool,
    },
    /// Grouping box container (`NSBox`).
    BoxContainer,
    /// Horizontal or vertical hairline separator (`NSBoxSeparator`).
    Separator,
    /// Non-editable static text label (`NSTextField` label).
    Label,
    /// Standard or primary push button (`NSButton`).
    PushButton {
        /// True when `NSKeyEquivalent == "\r"`.
        is_default: bool,
        /// True when `NSKeyEquivalent == "\x1b"`.
        is_cancel: bool,
    },
    /// Square AppKit checkbox (`NSButton[NSSwitchButton]`).
    Checkbox,
    /// Circular AppKit radio button (`NSButton[NSRadioButton]`).
    RadioButton,
    /// Pill toggle switch (`NSSwitch`).
    Switch,
    /// Pop-up dropdown menu button (`NSPopUpButton`).
    PopUpButton {
        /// Static option items extracted from `NSMenu`.
        options: Vec<String>,
    },
    /// Editable combo box (`NSComboBox`).
    ComboBox {
        /// Static suggestion items.
        options: Vec<String>,
    },
    /// Segmented control (`NSSegmentedControl`).
    SegmentedControl {
        /// Segment labels or icon names.
        segments: Vec<String>,
    },
    /// Single-line editable text field (`NSTextField`).
    TextField,
    /// Numeric input field with stepping and inline arithmetic evaluation (`NSNumberFormatter`).
    NumberField {
        /// Arrow-key step increment.
        step: f64,
        /// Shift+arrow step increment.
        shift_step: f64,
        /// Optional minimum value.
        min: Option<f64>,
        /// Optional maximum value.
        max: Option<f64>,
    },
    /// Multi-line scrollable text or code editor (`NSTextView`).
    TextView {
        /// Whether monospace typography is used.
        monospace: bool,
        /// Whether a line-number gutter is displayed.
        line_numbers: bool,
    },
    /// Search input field (`NSSearchField`).
    SearchField,
    /// Linear slider (`NSSlider`).
    Slider {
        /// Minimum value.
        min: f64,
        /// Maximum value.
        max: f64,
        /// Tick mark count (`0` for smooth continuous slider).
        tick_marks: usize,
    },
    /// Numeric stepper (`NSStepper`).
    Stepper {
        /// Step increment.
        step: f64,
    },
    /// Color swatch well (`NSColorWell`).
    ColorWell,
    /// Circular `?` help button (`NSBezelStyleHelpButton`).
    HelpButton,
    /// Compact inline action badge button (`+`, `-`, `i`, `⚙`).
    InlineActionButton,
    /// `3x3` 9-point transform origin matrix (`OriginPointControl`).
    OriginMatrix,
    /// Finder / AppKit tag color swatch picker (`ColorLabelView`).
    TagColorPicker,
    /// `[+ | -]` table action bar (`NSBezelStyleSmallSquare`).
    TableActionBar,
    /// Collapsible section disclosure header (`NSDisclosureGroup`).
    DisclosureGroup {
        /// Whether a trailing `+` action button is rendered.
        trailing_add_button: bool,
    },
    /// Date and time picker (`NSDatePicker`).
    DatePicker {
        /// True when `style == "clockAndCalendar"`.
        clock_and_calendar: bool,
    },
    /// Tabbed container (`NSTabView`).
    TabView {
        /// Tab item labels.
        tabs: Vec<String>,
    },
    /// Predicate rule editor (`NSRuleEditor`).
    RuleEditor,
    /// Multi-column table view (`NSTableView` unwrapped from `NSScrollView`).
    TableView {
        /// Column headers.
        columns: Vec<(String, f32)>,
    },
    /// Image or template symbol view (`NSImageView`).
    ImageView {
        /// Resource or symbol name.
        icon_name: String,
        /// Whether the image is a monochrome `*Template` image.
        is_template: bool,
    },
    /// Custom application canvas slot (`NSClassSwapper` view).
    CustomCanvasSlot {
        /// Custom class identifier.
        class_name: String,
    },
}

/// AppKit `NSTextAlignment` (`REQ-013-10.1` §3.8.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MacTextAlignment {
    /// Left-aligned (`NSTextAlignmentLeft = 0`).
    #[default]
    Left,
    /// Right-aligned (`NSTextAlignmentRight = 1`).
    Right,
    /// Center-aligned (`NSTextAlignmentCenter = 2`).
    Center,
    /// Justified (`NSTextAlignmentJustified = 3`).
    Justified,
    /// Natural reading-direction alignment (`NSTextAlignmentNatural = 4`).
    Natural,
}

/// Lowered scene node in [`MacSceneIR`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneNode {
    /// Arena node identifier.
    pub id: NodeId,
    /// Interface Builder object identifier (`"142"` or `"_NS:27"`).
    pub ib_object_id: String,
    /// Parent node identifier.
    pub parent: Option<NodeId>,
    /// Ordered child node identifiers.
    pub children: Vec<NodeId>,
    /// Strongly typed widget or container kind.
    pub kind: SceneNodeKind,
    /// Visible title, label, or placeholder text.
    pub title: String,
    /// Optional tooltip (`ibShadowedToolTip` / `.help(...)`).
    pub tooltip: String,
    /// Control size (`Regular`, `Small`, `Mini`).
    pub control_size: String,
    /// Raw NIB frame `[x, y, w, h]` in parent coordinates.
    pub nib_frame: [f32; 4],
    /// Normalized visual alignment rect `[x, y, w, h]` in top-left parent coordinates.
    pub alignment_rect: [f32; 4],
    /// 6-bit `NSAutoresizingMaskOptions` bitfield (`0..=63`).
    pub autoresizing_mask: u8,
    /// Whether the control is enabled by default.
    pub enabled: bool,
    /// Optional primary value binding.
    pub value_binding: Option<BindingId>,
    /// Optional binding key path string (`"values.gridSpacing"`).
    pub binding_path: Option<String>,
    /// Value transformer attached to `value_binding`.
    pub value_transformer: ValueTransformer,
    /// Optional target-action selector handle.
    pub action: Option<ActionId>,
    /// Optional action selector name (`"exportNow:"`).
    pub action_selector: Option<String>,
    /// Optional icon resource name.
    pub icon_name: Option<String>,
    /// Declarative workspace/context visibility rule.
    pub visibility: VisibilityRule,
    /// Paired semantic label node inferred by `SpatialSemanticNormalizer`.
    pub labeled_by: Option<NodeId>,
    /// Strongly typed AppKit text alignment (`NSCellFlags2` bits `26..=28` / `alignment="..."`).
    #[serde(default)]
    pub text_alignment: MacTextAlignment,
    /// AppKit `NSBezelStyle` / `NSTextBezelStyle` (`0 = None`, `1 = Rounded/Pill`, `10 = SmallSquare`, `11 = Recessed`, etc.).
    #[serde(default)]
    pub bezel_style: u8,
    /// Whether the cell draws its background fill (`NSDrawsBackground`).
    #[serde(default)]
    pub draws_background: bool,
    /// Whether the cell draws a 1pt border (`NSCellFlags` bit 23).
    #[serde(default)]
    pub is_bordered: bool,
    /// Whether the cell is bezeled (`NSCellFlags` bit 22).
    #[serde(default)]
    pub is_bezeled: bool,
    /// Whether the control is editable (`NSCellFlags` bit 19).
    #[serde(default)]
    pub is_editable: bool,
    /// Whether the control text is selectable (`NSCellFlags` bit 18).
    #[serde(default)]
    pub is_selectable: bool,
    /// Explicit point size from `NSSupport` (`NSFont.NSSize`), if archived.
    #[serde(default)]
    pub font_size_pt: Option<f32>,
    /// Explicit PostScript / family name from `NSSupport` (`NSFont.NSName`) or `.xib` `<font name="...">`, if archived.
    #[serde(default)]
    pub font_name: Option<String>,
    /// Font weight tier decoded from `NSSupport` / `NSFont` (`Light`, `Regular`, `Medium`, `Semibold`, `Bold`).
    #[serde(default)]
    pub font_weight: MacFontWeight,
    /// Semantic label text color tier decoded from `NSTextColor` (`Primary`, `Secondary`, `Tertiary`).
    #[serde(default)]
    pub label_color: MacLabelColor,
    /// Custom `NSView` subclass name (`NSClassName` / `customClass`), if any.
    #[serde(default)]
    pub custom_class: Option<String>,
    /// Custom `NSCell` subclass name (`NSCell.NSClassName` / `<*Cell customClass="...">`), if any.
    #[serde(default)]
    pub cell_class: Option<String>,
    /// Whether the view is hidden (`NSvFlags` bit 31 / `hidden="YES"`).
    #[serde(default)]
    pub is_hidden: bool,
    /// Optional dynamic `hidden` binding handle (`NSNibBindingConnector` `NSBinding == "hidden"` / `<binding name="hidden">`).
    #[serde(default)]
    pub hidden_binding: Option<BindingId>,
    /// Optional dynamic `hidden` binding key path (`NSKeyPath` / `keyPath`).
    #[serde(default)]
    pub hidden_path: Option<String>,
    /// Optional dynamic `enabled` binding handle (`NSNibBindingConnector` `NSBinding == "enabled"`, `REQ-013-10.9`).
    #[serde(default)]
    pub enabled_binding: Option<BindingId>,
    /// Optional dynamic `enabled` binding key path (`NSKeyPath` / `keyPath`, `REQ-013-10.9`).
    #[serde(default)]
    pub enabled_path: Option<String>,
    /// Value transformer attached to `enabled_binding` (`REQ-013-10.9`).
    #[serde(default)]
    pub enabled_transformer: ValueTransformer,
    /// Optional `NSStackView` orientation (`0 = Horizontal`, `1 = Vertical`, `REQ-013-10.1` §3.8.4).
    #[serde(default)]
    pub stack_orientation: Option<u8>,
    /// Optional `NSStackView` inter-item spacing in points (`NSStackViewSpacing`, default `2.0`).
    #[serde(default)]
    pub stack_spacing: f32,
    /// Whether an `NSStackView` detaches hidden arranged subviews from layout (`detachesHiddenViews`).
    #[serde(default)]
    pub detaches_hidden_views: bool,
    /// Optional AppKit `NSDisplayPattern` format string (e.g. `"%{value1}@ selected %{value2}@/%{value3}@ glyphs"`, `REQ-013-10.10`).
    #[serde(default)]
    pub display_pattern: Option<String>,
    /// Multi-slot `(slot_index 1..=9, BindingId, normalized_key_path)` bindings for `displayPatternValue1..9` (`REQ-013-10.10`).
    #[serde(default)]
    pub display_pattern_bindings: Vec<(u8, BindingId, String)>,
    /// AppKit `NSImageScaling` (`NSScale`: `0 = ProportionallyDown`, `1 = AxesIndependently`, `2 = None`, `3 = ProportionallyUpOrDown`, `REQ-013-10.10`).
    #[serde(default)]
    pub image_scaling: u8,
    /// Optional SF Symbol point size from `IBNSShadowedSymbolConfiguration` / `NSImageSymbolConfiguration` (`NSPointSize`, `REQ-013-10.10`).
    #[serde(default)]
    pub symbol_point_size: Option<f32>,
}

impl SceneNode {
    /// Constructs a new [`SceneNode`] with default AppKit cell/view flags.
    pub fn new(id: NodeId, kind: SceneNodeKind) -> Self {
        let mut n = crate::scene::nib_frontend::mk_node(
            id.0,
            "",
            None,
            vec![],
            kind,
            "",
            "",
            "regular",
            [0.0, 0.0, 100.0, 22.0],
            0,
            None,
            None,
            None,
            VisibilityRule::Always,
        );
        n.ib_object_id.clear();
        n
    }

    /// Returns the parsed [`MacControlSize`] of this node.
    pub fn mac_control_size(&self) -> MacControlSize {
        match self.control_size.as_str() {
            "small" | "Small" => MacControlSize::Small,
            "mini" | "Mini" => MacControlSize::Mini,
            "compact" | "Compact" => MacControlSize::CompactInfobox,
            "large" | "Large" => MacControlSize::Large,
            _ => MacControlSize::Regular,
        }
    }

    /// Returns the AppKit `NSVisualEffectMaterial` code (`7 = Sidebar`, `18 = ContentBackground`, etc.)
    /// when this container represents an `NSVisualEffectView` (`REQ-013-10.3`).
    pub fn visual_effect_material(&self) -> Option<u32> {
        (matches!(self.kind, SceneNodeKind::Container { .. })
            && self.custom_class.as_deref().is_some_and(|c| c.contains("VisualEffectView")))
        .then(|| {
            let mat = self.bezel_style & 0x7F;
            if mat > 0 {
                u32::from(mat)
            } else {
                7
            }
        })
    }

    /// Returns `true` when this `NSVisualEffectView` uses `NSVisualEffectBlendingModeBehindWindow`.
    pub fn visual_effect_is_behind_window(&self) -> bool {
        self.visual_effect_material().is_some() && (self.bezel_style & 0x80) != 0
    }
}

/// Semantic `(Label, Control)` pair reconstructed by `SpatialSemanticNormalizer` (`REQ-013-02` §3.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabeledFieldPair {
    /// Static `Label` node ID.
    pub label_node: NodeId,
    /// Paired interactive control node ID.
    pub control_node: NodeId,
}

/// Multi-field horizontal row group (e.g. `L: [input] R: [input]`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldRowGroup {
    /// Ordered nodes participating in the horizontal row.
    pub nodes: Vec<NodeId>,
}

/// Semantic section block inferred from `NSBox` enclosures or `NSBoxSeparator` dividers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticSectionGroup {
    /// Section title (derived from `NSBox` title or leading bold/header label).
    pub title: String,
    /// Ordered interactive and static child nodes inside this section.
    pub nodes: Vec<NodeId>,
    /// Paired `(Label, Control)` rows inside this section.
    pub pairs: Vec<LabeledFieldPair>,
}

/// Compilation error emitted by Layer 1 frontends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneCompileError(pub String);
impl std::fmt::Display for SceneCompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SceneCompileError: {}", self.0)
    }
}
impl std::error::Error for SceneCompileError {}

/// Attached `NSMenu` entry `(ib_object_id, title, value_binding, binding_path)` decoded from `NSNibOutletConnector` (`NSLabel = "menu"`).
pub type AttachedButtonMenuItem = (String, String, Option<BindingId>, Option<String>);

/// Map from button [`NodeId`] to its attached `NSMenu` entries.
pub type ButtonMenuMap = BTreeMap<NodeId, Vec<AttachedButtonMenuItem>>;

/// Canonical Typed Intermediate Representation (`MacSceneIR`, `SPEC-013 §3`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacSceneIR {
    pub(crate) root: NodeId,
    pub(crate) design_size: [f32; 2],
    pub(crate) nodes: Vec<SceneNode>,
    #[serde(default)]
    pub(crate) outlets: BTreeMap<String, NodeId>,
    #[serde(default)]
    pub(crate) button_menus: ButtonMenuMap,
    #[serde(default)]
    pub(crate) nib_constraints: Vec<LayoutConstraintSpec>,
    pub(crate) labeled_pairs: Vec<LabeledFieldPair>,
    pub(crate) field_rows: Vec<FieldRowGroup>,
    pub(crate) semantic_sections: Vec<SemanticSectionGroup>,
    pub(crate) constraints: CompiledConstraintSchedule,
    pub(crate) search_index: BTreeMap<NodeId, String>,
}

impl MacSceneIR {
    /// Constructs a normalized `MacSceneIR` from a lowered node arena and runs `SpatialSemanticNormalizer`.
    pub fn from_nodes(root: NodeId, design_size: [f32; 2], mut nodes: Vec<SceneNode>, constraints: CompiledConstraintSchedule) -> Self {
        let root_flipped = nodes.first().is_some_and(|n| matches!(n.kind, SceneNodeKind::Container { is_flipped: true }));
        for node in &mut nodes {
            let flip = if matches!(node.kind, SceneNodeKind::Container { .. }) { true } else { root_flipped };
            node.alignment_rect = normalize_nib_alignment_rect(&node.kind, node.mac_control_size(), node.nib_frame, design_size[1], flip);
        }
        let (labeled_pairs, field_rows, semantic_sections) = run_spatial_semantic_normalizer(&mut nodes);
        let nib_constraints = constraints.steps().to_vec();
        let mut ir = Self { root, design_size, nodes, outlets: BTreeMap::new(), button_menus: BTreeMap::new(), nib_constraints, labeled_pairs, field_rows, semantic_sections, constraints, search_index: BTreeMap::new() };
        ir.rebuild_search_index();
        ir
    }

    /// Returns the attached `NSMenu` items `(ib_object_id, title, value_binding, binding_path)` decoded from `NSNibOutletConnector` (`NSLabel = "menu"`).
    pub fn button_menus(&self) -> &ButtonMenuMap {
        &self.button_menus
    }

    /// Returns the persistent raw `NSLayoutConstraint` specifications (`REQ-013-10.1` §3.8.4).
    pub fn nib_constraints(&self) -> &[LayoutConstraintSpec] {
        &self.nib_constraints
    }
    /// Appends a programmatic `NSLayoutConstraint` specification and re-solves layout (`REQ-013-10.1` §3.8.4).
    pub fn add_layout_constraint(&mut self, spec: LayoutConstraintSpec) {
        self.nib_constraints.push(spec);
        self.solve_autolayout();
    }
    /// Appends a programmatic `first.leading == second.leading` constraint and re-solves layout (`REQ-013-10.1` §3.8.4).
    pub fn add_equal_leading_constraint(&mut self, first_id_or_outlet: &str, second_id_or_outlet: &str) -> bool {
        self.add_anchor_constraint(first_id_or_outlet, ConstraintAttr::Leading, Some(second_id_or_outlet), ConstraintAttr::Leading, 0.0)
    }
    /// Appends a programmatic anchor constraint (`first.attr1 == second.attr2 + constant`) and re-solves layout (`REQ-013-10.1` §3.8.4).
    pub fn add_anchor_constraint(
        &mut self,
        first: &str,
        attr1: ConstraintAttr,
        second: Option<&str>,
        attr2: ConstraintAttr,
        constant: f32,
    ) -> bool {
        crate::scene::normalizer::add_scene_anchor_constraint(self, first, attr1, second, attr2, constant)
    }
    /// Pins `child_id_or_outlet` to its parent container's 4 edges (`Leading`, `Trailing`, `Top`, `Bottom`, `constant = 0.0`).
    pub fn pin_to_superview_edges(&mut self, child_id_or_outlet: &str) -> bool {
        crate::scene::normalizer::pin_scene_node_to_superview_edges(self, child_id_or_outlet)
    }
    /// Updates `node.is_hidden` and triggers `solve_autolayout()` (`REQ-013-10.1` §3.8.4).
    pub fn set_node_hidden(&mut self, id_or_outlet: &str, hidden: bool) -> bool {
        let Some(nid) = self.resolve_node_id(id_or_outlet) else { return false };
        let Some(node) = self.nodes.iter_mut().find(|n| n.id == nid) else { return false };
        node.is_hidden = hidden;
        self.solve_autolayout();
        true
    }
    /// Upgrades `node.kind` during `awakeFromNib` and triggers `solve_autolayout()` (`REQ-013-10.1` §3.8.4).
    pub fn set_node_kind(&mut self, id_or_outlet: &str, kind: SceneNodeKind) -> bool {
        let Some(nid) = self.resolve_node_id(id_or_outlet) else { return false };
        let Some(node) = self.nodes.iter_mut().find(|n| n.id == nid) else { return false };
        node.kind = kind;
        self.solve_autolayout();
        true
    }
    /// Extracts a view subtree identified by `id_or_outlet` into a standalone [`MacSceneIR`] (`REQ-013-10.1` §3.8.4).
    pub fn extract_outlet_subscene(&self, id_or_outlet: &str) -> Option<MacSceneIR> {
        crate::scene::normalizer::extract_scene_outlet_subscene(self, id_or_outlet)
    }
    /// Executes the 2D `alignmentRect`-space Auto Layout and `NSStackView` solver across the scene (`REQ-013-10.1` §3.8.4).
    pub fn solve_autolayout(&mut self) {
        crate::scene::normalizer::solve_scene_autolayout(self);
    }
    /// Returns the controller `IBOutlet` mapping table (`NSLabel -> NodeId`, `REQ-013-10.1` §3.8.2).
    pub fn outlets(&self) -> &BTreeMap<String, NodeId> {
        &self.outlets
    }
    /// Registers a controller `IBOutlet` mapping (`outlet_name -> node_id`).
    pub fn insert_outlet(&mut self, outlet_name: impl Into<String>, node_id: NodeId) {
        self.outlets.insert(outlet_name.into(), node_id);
    }
    /// Resolves a controller `IBOutlet` name to its target [`NodeId`].
    pub fn outlet_node(&self, outlet_name: &str) -> Option<NodeId> {
        self.outlets.get(outlet_name).copied()
    }
    /// Resolves either a controller `IBOutlet` name (e.g. `"productionField"`) or an `ib_object_id` (e.g. `"_NS:376"`) to its [`NodeId`].
    pub fn resolve_node_id(&self, id_or_outlet: &str) -> Option<NodeId> {
        self.outlets.get(id_or_outlet).copied().or_else(|| self.nodes.iter().find(|n| n.ib_object_id == id_or_outlet).map(|n| n.id))
    }
    /// Returns the root `NodeId`.
    pub fn root_id(&self) -> NodeId {
        self.root
    }
    /// Returns all nodes in the scene arena.
    pub fn nodes(&self) -> &[SceneNode] {
        &self.nodes
    }
    /// Solves the 1:1 Apple `.nib` layout at the scene's exact `design_size` (`LayoutPolicy::ExactAppleNib`, `REQ-013-08.1`).
    pub fn solve_exact_nib_layout(&self) -> BTreeMap<NodeId, egui::Rect> {
        let vp = egui::vec2(self.design_size[0].max(1.0), self.design_size[1].max(1.0));
        self.solve_layout(vp, LayoutPolicy::ExactAppleNib, u64::MAX, "")
    }
    /// Returns the inferred `LabeledFieldPair` relationships.
    pub fn labeled_pairs(&self) -> &[LabeledFieldPair] {
        &self.labeled_pairs
    }
    /// Returns the inferred `FieldRowGroup` horizontal rows.
    pub fn field_rows(&self) -> &[FieldRowGroup] {
        &self.field_rows
    }
    /// Returns the inferred `SemanticSectionGroup` blocks.
    pub fn semantic_sections(&self) -> &[SemanticSectionGroup] {
        &self.semantic_sections
    }
    /// Solves node rectangles for `viewport_size` under `policy`, `workspace_flags`, and `search_query`.
    pub fn solve_layout(
        &self,
        viewport_size: Vec2,
        policy: LayoutPolicy,
        workspace_flags: u64,
        search_query: &str,
    ) -> BTreeMap<NodeId, Rect> {
        crate::scene::runtime::solve_scene_layout(self, viewport_size, policy, workspace_flags, search_query)
    }
}
