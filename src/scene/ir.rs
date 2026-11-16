//! Layer 2: Strongly Typed `MacSceneIR`, `SpatialSemanticNormalizer`, `SceneSearchIndex`,
//! `VisibilityRule`, and `SceneContractVerifier` (`SPEC-013 §3`).

use crate::controls::MacControlSize;
use crate::scene::constraints::{CompiledConstraintSchedule, ValueTransformer};
use crate::scene::normalizer::run_spatial_semantic_normalizer;
pub use crate::scene::normalizer::{
    normalize_nib_alignment_rect, ContractAuditReport, CustomClassMapper, HostContractSchema, SceneFilterMask,
};
use egui::{Pos2, Rect, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Arena node identifier inside [`MacSceneIR`].
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
}

impl VisibilityRule {
    /// Evaluates whether the rule permits visibility under `workspace_flags`.
    #[inline]
    pub const fn is_visible(&self, workspace_flags: u64) -> bool {
        match self {
            Self::Always | Self::WhenFlagTrue(_) => true,
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
    /// Numeric input field with stepping and inline arithmetic evaluation (`GSSteppingTextField` / `NSNumberFormatter`).
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
    /// `3x3` 9-point transform origin matrix (`GSOriginPointControl`).
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
}

impl SceneNode {
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

/// Canonical Typed Intermediate Representation (`MacSceneIR`, `SPEC-013 §3`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacSceneIR {
    pub(crate) root: NodeId,
    pub(crate) design_size: [f32; 2],
    pub(crate) nodes: Vec<SceneNode>,
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
        let mut ir =
            Self { root, design_size, nodes, labeled_pairs, field_rows, semantic_sections, constraints, search_index: BTreeMap::new() };
        ir.rebuild_search_index();
        ir
    }

    /// Returns the root `NodeId`.
    pub fn root_id(&self) -> NodeId {
        self.root
    }

    /// Returns all nodes in the scene arena.
    pub fn nodes(&self) -> &[SceneNode] {
        &self.nodes
    }

    /// Returns the inferred `LabeledFieldPair` relationships.
    pub fn labeled_pairs(&self) -> &[LabeledFieldPair] {
        &self.labeled_pairs
    }

    /// Returns the inferred `SemanticSectionGroup` blocks.
    pub fn semantic_sections(&self) -> &[SemanticSectionGroup] {
        &self.semantic_sections
    }

    /// Returns the ordered set of all interactive `BindingId`s in the scene.
    pub fn interactive_binding_ids(&self) -> Vec<BindingId> {
        let mut set = BTreeSet::new();
        for n in &self.nodes {
            if let Some(b) = n.value_binding {
                set.insert(b);
            }
        }
        set.into_iter().collect()
    }

    /// Rebuilds the lowercase `SceneSearchIndex` (`REQ-013-02` §3.3).
    pub fn rebuild_search_index(&mut self) {
        self.search_index.clear();
        let mut label_text_by_control = BTreeMap::new();
        for pair in &self.labeled_pairs {
            if let Some(lbl) = self.nodes.iter().find(|n| n.id == pair.label_node) {
                label_text_by_control.insert(pair.control_node, lbl.title.to_lowercase());
            }
        }
        for n in &self.nodes {
            let mut tokens = format!(
                "{} {} {}",
                n.title.to_lowercase(),
                n.tooltip.to_lowercase(),
                n.binding_path.as_deref().unwrap_or("").to_lowercase()
            );
            if let Some(paired_lbl) = label_text_by_control.get(&n.id) {
                tokens.push(' ');
                tokens.push_str(paired_lbl);
            }
            self.search_index.insert(n.id, tokens);
        }
    }

    /// Queries the built-in `SceneSearchIndex` (`< 5 us`).
    pub fn filter_by_search(&self, query: &str) -> SceneFilterMask {
        let q = query.trim().to_lowercase();
        let mut mask = SceneFilterMask::default();
        if q.is_empty() {
            for n in &self.nodes {
                mask.matched_nodes.insert(n.id);
            }
            return mask;
        }
        for (id, text) in &self.search_index {
            if text.contains(&q) {
                mask.matched_nodes.insert(*id);
            }
        }
        for pair in &self.labeled_pairs {
            if mask.matched_nodes.contains(&pair.label_node) {
                mask.matched_nodes.insert(pair.control_node);
            } else if mask.matched_nodes.contains(&pair.control_node) {
                mask.matched_nodes.insert(pair.label_node);
            }
        }
        mask
    }

    /// Dynamically mounts a child `MacSceneIR` under `parent_or_slot` (`addSubview:` equivalent).
    pub fn mount_subscene(&mut self, parent_or_slot: NodeId, child_scene: &MacSceneIR) {
        let id_offset = self.nodes.iter().map(|n| n.id.0).max().unwrap_or(0) + 100;
        let child_root_new = NodeId(child_scene.root.0 + id_offset);
        if let Some(p) = self.nodes.iter_mut().find(|n| n.id == parent_or_slot) {
            p.children.push(child_root_new);
        }
        for cn in &child_scene.nodes {
            let mut cloned = cn.clone();
            cloned.id = NodeId(cn.id.0 + id_offset);
            cloned.parent = cn.parent.map(|p| NodeId(p.0 + id_offset)).or(Some(parent_or_slot));
            cloned.children = cn.children.iter().map(|c| NodeId(c.0 + id_offset)).collect();
            self.nodes.push(cloned);
        }
        self.rebuild_search_index();
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

    /// Reverse Z-order canvas hit-testing for Studio selection (`REQ-013-02I`).
    pub fn hit_test(&self, pos: Pos2, solved_rects: &BTreeMap<NodeId, Rect>) -> Option<NodeId> {
        for n in self.nodes.iter().rev() {
            if matches!(n.kind, SceneNodeKind::Container { .. }) {
                continue;
            }
            if let Some(rect) = solved_rects.get(&n.id) {
                if rect.contains(pos) {
                    return Some(n.id);
                }
            }
        }
        None
    }

    /// Updates a node's `nib_frame` and live-renormalizes `alignment_rect`, `SemanticSectionGroup`s, and `SceneSearchIndex` (`REQ-013-02I`).
    pub fn set_node_nib_frame(&mut self, id: NodeId, new_nib_frame: [f32; 4]) -> bool {
        let h_parent = self.design_size[1];
        let root_flipped = self.nodes.first().is_some_and(|n| matches!(n.kind, SceneNodeKind::Container { is_flipped: true }));
        let Some(node) = self.nodes.iter_mut().find(|n| n.id == id) else {
            return false;
        };
        node.nib_frame = new_nib_frame;
        node.alignment_rect = normalize_nib_alignment_rect(&node.kind, node.mac_control_size(), new_nib_frame, h_parent, root_flipped);
        self.recompute_semantics();
        true
    }

    /// Inserts a new node under `parent` and re-normalizes semantic groups (`REQ-013-02I`).
    pub fn insert_node(&mut self, parent: NodeId, mut node: SceneNode) -> NodeId {
        let root_flipped = self.nodes.first().is_some_and(|n| matches!(n.kind, SceneNodeKind::Container { is_flipped: true }));
        node.parent = Some(parent);
        node.alignment_rect = normalize_nib_alignment_rect(&node.kind, node.mac_control_size(), node.nib_frame, self.design_size[1], root_flipped);
        let id = node.id;
        if let Some(p) = self.nodes.iter_mut().find(|n| n.id == parent) {
            p.children.push(id);
        }
        self.nodes.push(node);
        self.recompute_semantics();
        id
    }

    /// Removes a node from the scene and re-normalizes semantic groups (`REQ-013-02I`).
    pub fn remove_node(&mut self, id: NodeId) -> bool {
        let len_before = self.nodes.len();
        self.nodes.retain(|n| n.id != id);
        if self.nodes.len() == len_before {
            return false;
        }
        for n in &mut self.nodes {
            n.children.retain(|c| *c != id);
        }
        self.recompute_semantics();
        true
    }

    fn recompute_semantics(&mut self) {
        for n in &mut self.nodes {
            n.labeled_by = None;
        }
        let (pairs, rows, sections) = run_spatial_semantic_normalizer(&mut self.nodes);
        self.labeled_pairs = pairs;
        self.field_rows = rows;
        self.semantic_sections = sections;
        self.rebuild_search_index();
    }
}
