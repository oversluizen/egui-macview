//! Three-Layer UI Compiler Architecture (`SPEC-013`, Milestone `v0.7.0`).
//!
//! Provides multi-format ingestion frontends (`NibArchiveFrontend`, `XibXmlFrontend`,
//! `SwiftUiFrontend`, `StringsCatalog`, `MacSceneSpec`), a strongly typed intermediate
//! representation (`MacSceneIR`, `SpatialSemanticNormalizer`, `SceneSearchIndex`,
//! `SceneContractVerifier`, `SceneExporter`), and a dual-mode 120 FPS layout solver
//! (`LayoutPolicy::ExactAppleNib` and `LayoutPolicy::AdaptiveSemanticReflow`).

pub mod constraints;
pub mod exporter;
pub mod ir;
pub mod keyed_archiver;
pub mod nib_frontend;
pub mod nib_ibtool;
pub(crate) mod nib_ibtool_widgets;
pub mod nib_varint;
pub mod normalizer;
pub mod runtime;
pub(crate) mod runtime_tabs;
pub(crate) mod runtime_widgets;
pub mod storyboard;
pub mod swiftui;
pub mod xib_exporter;
pub mod xib_strings;

pub use constraints::{
    evaluate_arithmetic_expr, CompiledConstraintSchedule, ConstraintAttr, ConstraintRelation, LayoutConstraintSpec, ValueTransformer,
};
pub use exporter::{MacSceneSpec, SceneExporter};
pub use ir::{
    ActionId, BindingId, BindingValueType, ContractAuditReport, CustomClassMapper, FieldRowGroup, HostContractSchema, LabeledFieldPair,
    LayoutPolicy, MacSceneIR, NodeId, SceneCompileError, SceneFilterMask, SceneNode, SceneNodeKind, SemanticSectionGroup, VisibilityRule,
};
pub use nib_frontend::NibArchiveFrontend;
pub use runtime::MacSceneDataSource;
pub use storyboard::{MacStoryboardIR, SegueKind, StoryboardFrontend, StoryboardScene, StoryboardSegue};
pub use swiftui::SwiftUiFrontend;
pub use xib_strings::XibXmlFrontend;

/// Apple `<NibName>.strings` localization catalog (`"ObjectID.property" = "Localized";`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StringsCatalog {
    entries: std::collections::BTreeMap<(String, String), String>,
}

impl StringsCatalog {
    /// Parses an Apple `.strings` text file (`/* comments */` and `"ObjectID.prop" = "Value";` lines).
    pub fn from_str(source: &str) -> Result<Self, SceneCompileError> {
        let mut entries = std::collections::BTreeMap::new();
        for raw_line in source.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with("/*") || line.starts_with("//") {
                continue;
            }
            let Some((lhs, rhs)) = line.split_once('=') else {
                continue;
            };
            let key = unquote_strings_token(lhs.trim());
            let val = unquote_strings_token(rhs.trim().trim_end_matches(';').trim());
            if let Some((obj_id, prop)) = key.split_once('.') {
                entries.insert((obj_id.to_string(), prop.to_string()), val);
            }
        }
        Ok(Self { entries })
    }

    /// Parses either a binary/XML `plist` `.strings` file or a UTF-8 text `.strings` file.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SceneCompileError> {
        if let Ok(plist::Value::Dictionary(dict)) = plist::from_bytes::<plist::Value>(bytes) {
            let mut entries = std::collections::BTreeMap::new();
            for (k, v) in dict {
                if let (Some((obj_id, prop)), Some(s)) = (k.split_once('.'), v.as_string()) {
                    entries.insert((obj_id.to_string(), prop.to_string()), s.to_string());
                }
            }
            return Ok(Self { entries });
        }
        let text = std::str::from_utf8(bytes).map_err(|e| SceneCompileError(format!("Invalid UTF-8 in .strings: {e}")))?;
        Self::from_str(text)
    }

    /// Looks up a localized property for `(ib_object_id, property)`.
    pub fn get(&self, ib_object_id: &str, property: &str) -> Option<&str> {
        self.entries.get(&(ib_object_id.to_string(), property.to_string())).map(String::as_str)
    }
}

fn unquote_strings_token(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed[1..trimmed.len() - 1].replace("\\\"", "\"").replace("\\n", "\n")
    } else {
        trimmed.to_string()
    }
}

impl MacSceneIR {
    /// Applies a localized [`StringsCatalog`] to all matching nodes in $O(N)$ time and rebuilds [`MacSceneIR::rebuild_search_index`].
    pub fn apply_strings_catalog(&mut self, catalog: &StringsCatalog) {
        for node in &mut self.nodes {
            if let Some(title) = catalog.get(&node.ib_object_id, "title") {
                node.title = title.to_string();
            }
            if let Some(tip) = catalog.get(&node.ib_object_id, "ibShadowedToolTip") {
                node.tooltip = tip.to_string();
            }
        }
        self.rebuild_search_index();
    }
}
