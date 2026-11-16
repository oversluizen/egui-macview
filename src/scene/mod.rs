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
pub use xib_strings::{StringsCatalog, XibXmlFrontend};
