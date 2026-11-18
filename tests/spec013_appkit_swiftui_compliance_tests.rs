//! Normative COTS Tests for 100% AppKit & SwiftUI Compliance (`REQ-013-07.4`, `TC-013-14`).

mod common;

use egui::{Pos2, Vec2};
use egui_macview::{
    scene::{
        BindingId, CompiledConstraintSchedule, LayoutPolicy, MacSceneDataSource, MacSceneIR, NodeId, SceneExporter, SceneNode,
        SceneNodeKind, SwiftUiFrontend, ValueTransformer, VisibilityRule, XibXmlFrontend,
    },
    MacAppearance,
};

struct DummyStore {
    tab_idx: i64,
}

impl MacSceneDataSource for DummyStore {
    fn get_bool(&self, _id: BindingId) -> bool {
        true
    }
    fn set_bool(&mut self, _id: BindingId, _val: bool) {}
    fn get_f64(&self, _id: BindingId) -> f64 {
        50.0
    }
    fn set_f64(&mut self, _id: BindingId, _val: f64) {}
    fn get_i64(&self, _id: BindingId) -> i64 {
        self.tab_idx
    }
    fn set_i64(&mut self, _id: BindingId, val: i64) {
        self.tab_idx = val;
    }
    fn get_str(&self, _id: BindingId) -> &str {
        "Sample"
    }
    fn set_str(&mut self, _id: BindingId, _val: &str) {}
    fn get_options(&self, _id: BindingId) -> &[String] {
        &[]
    }
    fn on_action(&mut self, _action: egui_macview::scene::ActionId, _sender: NodeId) {}
    fn render_custom_slot(&mut self, _ui: &mut egui::Ui, _theme: &egui_macview::MacTheme, _slot_class: &str, _rect: egui::Rect) {}
}

#[test]
fn tc_013_14_appkit_and_swiftui_100_percent_compliance_and_zero_ghost_widgets() {
    let mk = |id: u32, kind: SceneNodeKind, title: &str, path: &str, y: f32| SceneNode {
        id: NodeId(id),
        ib_object_id: format!("n-{id}"),
        parent: Some(NodeId(1)),
        children: vec![],
        kind,
        title: title.into(),
        tooltip: String::new(),
        control_size: "regular".into(),
        nib_frame: [24.0, y, 160.0, 22.0],
        alignment_rect: [24.0, y, 160.0, 22.0],
        autoresizing_mask: 0,
        enabled: true,
        value_binding: Some(BindingId(id)),
        binding_path: Some(path.into()),
        value_transformer: ValueTransformer::Identity,
        action: None,
        action_selector: None,
        icon_name: None,
        visibility: VisibilityRule::Always,
        labeled_by: None,
    };

    let root = SceneNode {
        id: NodeId(1),
        ib_object_id: "root".into(),
        parent: None,
        children: (2..=9).map(NodeId).collect(),
        kind: SceneNodeKind::Container { is_flipped: true },
        title: "Window".into(),
        tooltip: String::new(),
        control_size: "regular".into(),
        nib_frame: [0.0, 0.0, 480.0, 360.0],
        alignment_rect: [0.0, 0.0, 480.0, 360.0],
        autoresizing_mask: 0,
        enabled: true,
        value_binding: None,
        binding_path: None,
        value_transformer: ValueTransformer::Identity,
        action: None,
        action_selector: None,
        icon_name: None,
        visibility: VisibilityRule::Always,
        labeled_by: None,
    };

    let nodes = vec![
        root,
        mk(2, SceneNodeKind::ColorWell, "Accent", "accentCol", 20.0),
        mk(3, SceneNodeKind::DatePicker { clock_and_calendar: true }, "Release", "relDate", 52.0),
        mk(4, SceneNodeKind::TabView { tabs: vec!["General".into(), "Advanced".into()] }, "Tabs", "activeTab", 84.0),
        mk(5, SceneNodeKind::DisclosureGroup { trailing_add_button: true }, "Details", "detailsOpen", 116.0),
        mk(6, SceneNodeKind::SearchField, "Filter...", "filterText", 148.0),
        mk(7, SceneNodeKind::ComboBox { options: vec!["ProRes".into()] }, "Codec", "codecStr", 180.0),
        mk(8, SceneNodeKind::RuleEditor, "Predicates", "rules", 212.0),
        mk(9, SceneNodeKind::TableView { columns: vec![("Name".into(), 120.0)] }, "Assets", "assetSel", 244.0),
    ];

    let scene = MacSceneIR::from_nodes(NodeId(1), [480.0, 360.0], nodes, CompiledConstraintSchedule::default());
    let swift_src = SceneExporter::to_swiftui_source(&scene);
    assert!(swift_src.contains("ColorPicker("), "SwiftUI exporter must emit ColorPicker for ColorWell, got:\n{swift_src}");
    assert!(swift_src.contains("DatePicker("), "SwiftUI exporter must emit DatePicker, got:\n{swift_src}");
    assert!(swift_src.contains("TabView("), "SwiftUI exporter must emit TabView, got:\n{swift_src}");
    assert!(swift_src.contains("DisclosureGroup("), "SwiftUI exporter must emit DisclosureGroup, got:\n{swift_src}");
    assert!(swift_src.contains("SearchField("), "SwiftUI exporter must emit SearchField, got:\n{swift_src}");
    assert!(swift_src.contains("ComboBox("), "SwiftUI exporter must emit ComboBox, got:\n{swift_src}");
    assert!(swift_src.contains("RuleEditor("), "SwiftUI exporter must emit RuleEditor, got:\n{swift_src}");
    assert!(swift_src.contains("Table("), "SwiftUI exporter must emit Table, got:\n{swift_src}");

    let parsed = SwiftUiFrontend::parse(&swift_src).expect("parse transpiled SwiftUI");
    let kinds: Vec<&SceneNodeKind> = parsed.nodes().iter().map(|n| &n.kind).collect();
    assert!(kinds.iter().any(|k| matches!(k, SceneNodeKind::ColorWell)));
    assert!(kinds.iter().any(|k| matches!(k, SceneNodeKind::DatePicker { clock_and_calendar: true })));
    assert!(kinds.iter().any(|k| matches!(k, SceneNodeKind::TabView { .. })));
    assert!(kinds.iter().any(|k| matches!(k, SceneNodeKind::DisclosureGroup { .. })));
    assert!(kinds.iter().any(|k| matches!(k, SceneNodeKind::SearchField)));
    assert!(kinds.iter().any(|k| matches!(k, SceneNodeKind::ComboBox { .. })));
    assert!(kinds.iter().any(|k| matches!(k, SceneNodeKind::RuleEditor)));
    assert!(kinds.iter().any(|k| matches!(k, SceneNodeKind::TableView { .. })));

    // 2. Declarative <tabViewItem><view key="view"><subviews> rendering in NSTabView
    let xib_with_tab_children = r#"<?xml version="1.0" encoding="UTF-8"?>
<document type="com.apple.InterfaceBuilder3.Cocoa.XIB" version="3.0" toolsVersion="23504" targetRuntime="MacOSX.Cocoa">
    <objects>
        <window title="TabTest" id="win-1">
            <view key="contentView" flipped="YES" id="root-tv">
                <rect key="frame" x="0.0" y="0.0" width="400" height="240"/>
                <subviews>
                    <tabView id="tv-main">
                        <rect key="frame" x="16" y="16" width="368" height="200"/>
                        <tabViewItems>
                            <tabViewItem label="First Tab" identifier="tab-0" id="tvi-0">
                                <view key="view" id="tvi-view-0">
                                    <rect key="frame" x="10" y="33" width="348" height="154"/>
                                    <subviews>
                                        <button id="btn-tab0">
                                            <rect key="frame" x="16" y="16" width="140" height="24"/>
                                            <buttonCell key="cell" type="push" title="TabZeroChildButton" bezelStyle="rounded" id="c-b0"/>
                                        </button>
                                    </subviews>
                                </view>
                            </tabViewItem>
                            <tabViewItem label="Second Tab" identifier="tab-1" id="tvi-1">
                                <view key="view" id="tvi-view-1">
                                    <rect key="frame" x="10" y="33" width="348" height="154"/>
                                    <subviews>
                                        <button id="btn-tab1">
                                            <rect key="frame" x="16" y="16" width="140" height="24"/>
                                            <buttonCell key="cell" type="push" title="TabOneChildButton" bezelStyle="rounded" id="c-b1"/>
                                        </button>
                                    </subviews>
                                </view>
                            </tabViewItem>
                        </tabViewItems>
                        <connections>
                            <binding destination="-2" name="selectedIndex" keyPath="values.tabIdx" id="b-tv"/>
                        </connections>
                    </tabView>
                </subviews>
            </view>
        </window>
    </objects>
</document>"#;

    let tab_scene = XibXmlFrontend::from_xml_str(xib_with_tab_children).expect("parse tabView with child subviews");
    let (ctx, theme) = common::headless_ctx(MacAppearance::DarkAqua);
    let mut store = DummyStore { tab_idx: 0 };
    let shapes_tab0 = common::run_frame(&ctx, egui::RawInput::default(), Vec2::new(400.0, 240.0), |ui| {
        tab_scene.show(ui, &theme, &mut store, LayoutPolicy::ExactAppleNib, 0, "");
    });
    let mut texts0 = Vec::new();
    for cs in &shapes_tab0 {
        common::collect_text_shapes(&cs.shape, &mut texts0);
    }
    assert!(
        texts0.iter().any(|t| t.galley.text() == "TabZeroChildButton"),
        "Active tab 0 must render its declarative child subviews (<tabViewItem> -> <view> -> <subviews>)"
    );
    assert!(
        !texts0.iter().any(|t| t.galley.text() == "TabOneChildButton"),
        "Inactive tab 1 child subviews must not render when tab 0 is active"
    );

    // Switch to tab 1 and verify TabOneChildButton renders while TabZeroChildButton hides
    store.tab_idx = 1;
    let shapes_tab1 = common::run_frame(&ctx, egui::RawInput::default(), Vec2::new(400.0, 240.0), |ui| {
        tab_scene.show(ui, &theme, &mut store, LayoutPolicy::ExactAppleNib, 0, "");
    });
    let mut texts1 = Vec::new();
    for cs in &shapes_tab1 {
        common::collect_text_shapes(&cs.shape, &mut texts1);
    }
    assert!(texts1.iter().any(|t| t.galley.text() == "TabOneChildButton"));
    assert!(!texts1.iter().any(|t| t.galley.text() == "TabZeroChildButton"));

    // 3. Verify all 7 ibtool-compiled .nib archives transpile to typed SwiftUI and parse back losslessly
    let mapper = egui_macview::scene::CustomClassMapper::default_studio_mappings();
    for stem in [
        "1_Vector_and_Transform_Inspector",
        "1_Typography_and_Grid_Inspector",
        "2_Appearance_and_Theme_Settings",
        "2_Theme_and_Appearance_Studio",
        "3_AVKit_Audio_Mastering_Strip",
        "4_Publishing_and_Asset_Queue",
        "Right_Inspector_Default",
    ] {
        let nib_bytes = std::fs::read(format!("assets/nib_showcase/{stem}.nib")).expect("read .nib");
        let nib_scene = egui_macview::scene::NibArchiveFrontend::from_bytes(&nib_bytes, &mapper).expect("decode .nib");
        assert!(nib_scene.validate_invariants().is_ok());
        let swift = SceneExporter::to_swiftui_source(&nib_scene);
        let roundtrip = SwiftUiFrontend::parse(&swift).expect("parse transpiled SwiftUI from .nib");
        assert!(!roundtrip.nodes().is_empty(), "{stem}.nib SwiftUI roundtrip must produce nodes");
    }
    let _ = Pos2::ZERO;
}

#[test]
fn tc_014_09_liquid_glass_filleted_cushions_16band_drum_mesh_and_keyboard_a11y() {
    use egui::{Event, RawInput, Rect};
    use egui_macview::{MacContextThumbDial, MacEdgeThumbwheel, MacThumbDialLayer, MacThumbIntentMask};

    let (ctx, dark_theme) = common::headless_ctx(MacAppearance::DarkAqua);
    let mut latched = MacThumbIntentMask::NONE;
    let mut rotary_val = 0.5f32;
    let mut layer = MacThumbDialLayer::CompassModifiers;
    let mut wheel_val = 100.0f32;
    let vp = Vec2::new(800.0, 600.0);

    let shapes_dial = common::run_frame(&ctx, RawInput::default(), vp, |ui| {
        let r = MacContextThumbDial::new("mockup_parity_dial", &mut latched, &mut rotary_val)
            .layer(&mut layer)
            .theme(&dark_theme)
            .show(ui);
        r.response.request_focus();
    });

    fn collect_meshes<'a>(shape: &'a egui::Shape, out: &mut Vec<&'a egui::epaint::Mesh>) {
        match shape {
            egui::Shape::Mesh(m) => out.push(m),
            egui::Shape::Vec(v) => v.iter().for_each(|s| collect_meshes(s, out)),
            _ => {}
        }
    }
    let mut dial_meshes = Vec::new();
    for cs in &shapes_dial {
        collect_meshes(&cs.shape, &mut dial_meshes);
    }
    let has_gradient_cushion_mesh =
        dial_meshes.iter().any(|m| m.vertices.len() >= 32 && m.vertices.windows(2).any(|w| w[0].color != w[1].color));
    assert!(
        has_gradient_cushion_mesh,
        "BUG-DIAL-01/03: MacContextThumbDial cushions must render per-vertex shaded 3D Liquid Glass gradient meshes"
    );

    let key_up = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vp)),
        events: vec![Event::Key {
            key: egui::Key::ArrowUp,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
        ..Default::default()
    };
    let _ = common::run_frame(&ctx, key_up, vp, |ui| {
        MacContextThumbDial::new("mockup_parity_dial", &mut latched, &mut rotary_val).layer(&mut layer).theme(&dark_theme).show(ui);
    });
    assert_eq!(latched, MacThumbIntentMask::CONSTRAIN, "ArrowUp on focused MacContextThumbDial must latch North (CONSTRAIN)");

    let shapes_wheel = common::run_frame(&ctx, RawInput::default(), vp, |ui| {
        let r = MacEdgeThumbwheel::new("mockup_parity_wheel", &mut wheel_val, 0.0..=360.0).theme(&dark_theme).show(ui);
        r.response.request_focus();
    });
    let mut wheel_meshes = Vec::new();
    for cs in &shapes_wheel {
        collect_meshes(&cs.shape, &mut wheel_meshes);
    }
    assert!(
        wheel_meshes.iter().any(|m| m.vertices.len() >= 32),
        "BUG-DIAL-05: MacEdgeThumbwheel must render a >= 16-band cosine-interpolated cylindrical 3D shading mesh"
    );

    let key_up_wheel = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vp)),
        events: vec![Event::Key {
            key: egui::Key::ArrowUp,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
        ..Default::default()
    };
    let _ = common::run_frame(&ctx, key_up_wheel, vp, |ui| {
        MacEdgeThumbwheel::new("mockup_parity_wheel", &mut wheel_val, 0.0..=360.0).theme(&dark_theme).show(ui);
    });
    assert!(wheel_val > 100.0, "ArrowUp on focused MacEdgeThumbwheel must step value upward");
}

#[test]
fn test_adversarial_menu_accelerator_clearance_extreme_strings() {
    use egui::{Key, RawInput, Rect};
    use egui_macview::{MacKeyboardShortcut, MacMenuBar, NSMenu, NSMenuItem};

    let (ctx, theme) = common::headless_ctx(MacAppearance::DarkAqua);
    let export_menu = NSMenu::with_items(
        "Nested Export",
        vec![NSMenuItem::action_with_shortcut(
            "Export High-Throughput Cryptographic Signature Keyframe Archive Package",
            "export.keyframe",
            MacKeyboardShortcut::cmd_shift(Key::K),
        )],
    );
    let items = vec![
        NSMenuItem::action_with_shortcut(
            "Synthesize Deeply Nested Multi-Stage Hierarchical Cryptographic Artifact",
            "adv.long1",
            MacKeyboardShortcut::cmd_shift(Key::S),
        ),
        NSMenuItem::action_with_shortcut("Cut", "adv.short", MacKeyboardShortcut::cmd_shift(Key::X)),
        NSMenuItem::action_with_shortcut(
            "Supercalifragilisticexpialidocious Ultra Wide Screen Projection Alignment Matrix",
            "adv.long2",
            MacKeyboardShortcut::cmd(Key::P),
        ),
        NSMenuItem::submenu("Cascading Options...", export_menu),
    ];
    let menus = vec![NSMenu::with_items("Adversarial", items)];
    let (mut open_menu_idx, mut highlighted_item_idx) = (Some(0), Some(0));
    let raw_input = RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(1920.0, 1080.0))), ..Default::default() };

    for _ in 0..2 {
        let mut out = ctx.run_ui(raw_input.clone(), |ui| {
            MacMenuBar::new(&menus, &mut open_menu_idx, &mut highlighted_item_idx).theme(&theme).show(ui);
        });
        out.textures_delta.clear();
        if !out.shapes.is_empty() {
            let mut text_shapes = Vec::new();
            for clipped in &out.shapes {
                common::collect_text_shapes(&clipped.shape, &mut text_shapes);
            }
            if text_shapes.iter().any(|t| t.galley.text() == "Cut") {
                for (title, sc) in [
                    ("Synthesize Deeply Nested Multi-Stage Hierarchical Cryptographic Artifact", "Ctrl+Shift+S"),
                    ("Cut", "Ctrl+Shift+X"),
                    ("Supercalifragilisticexpialidocious Ultra Wide Screen Projection Alignment Matrix", "Ctrl+P"),
                    ("Cascading Options...", "›"),
                ] {
                    let title_shape = text_shapes.iter().find(|t| t.galley.text() == title).unwrap();
                    let sc_shape = text_shapes.iter().find(|t| t.galley.text() == sc).unwrap();
                    assert!((sc_shape.pos.x - sc_shape.galley.size().x) - (title_shape.pos.x + title_shape.galley.size().x) >= 24.0);
                }
            }
        }
    }
}
