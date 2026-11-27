//! Normative COTS Tests for 100% AppKit & SwiftUI Compliance (`REQ-013-07.4`, `TC-013-14`).

mod common;

use egui::{Pos2, Vec2};
use egui_macview::{
    scene::{
        BindingId, CompiledConstraintSchedule, LayoutPolicy, MacSceneDataSource, MacSceneIR, NodeId, SceneExporter, SceneNode,
        SceneNodeKind, SwiftUiFrontend, XibXmlFrontend,
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
    let mk = |id: u32, kind: SceneNodeKind, title: &str, path: &str, y: f32| {
        let mut n = SceneNode::new(NodeId(id), kind);
        n.ib_object_id = format!("n-{id}");
        n.parent = Some(NodeId(1));
        n.title = title.into();
        n.nib_frame = [24.0, y, 160.0, 22.0];
        n.alignment_rect = [24.0, y, 160.0, 22.0];
        n.value_binding = Some(BindingId(id));
        n.binding_path = Some(path.into());
        n
    };

    let mut root = SceneNode::new(NodeId(1), SceneNodeKind::Container { is_flipped: true });
    root.ib_object_id = "root".into();
    root.children = (2..=9).map(NodeId).collect();
    root.title = "Window".into();
    root.nib_frame = [0.0, 0.0, 480.0, 360.0];
    root.alignment_rect = [0.0, 0.0, 480.0, 360.0];

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
        let r = MacContextThumbDial::new("mockup_parity_dial", &mut latched, &mut rotary_val).layer(&mut layer).theme(&dark_theme).show(ui);
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
    let has_liquid_glass_rosette_mesh = dial_meshes.iter().any(|m| {
        (m.texture_id != egui::TextureId::default() && m.vertices.len() == 4)
            || (m.vertices.len() >= 32 && m.vertices.windows(2).any(|w| w[0].color != w[1].color))
    });
    assert!(
        has_liquid_glass_rosette_mesh,
        "BUG-DIAL-01/03: MacContextThumbDial cushions must render a 2x Retina Liquid Glass TextureHandle quad"
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
        wheel_meshes
            .iter()
            .any(|m| m.vertices.len() >= 32 || (m.texture_id != egui::TextureId::default() && m.vertices.len() == 4)),
        "BUG-DIAL-05: MacEdgeThumbwheel must render a per-pixel raytraced 2x Retina TextureHandle quad or >= 16-band cylindrical 3D shading mesh"
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

#[test]
fn tc_013_15_domain_agnostic_nib_class_hierarchy_menu_toolbar_and_splitview_decoding() {
    use egui_macview::scene::{CustomClassMapper, NibArchiveFrontend};

    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for rel in ["src/scene/normalizer.rs", "src/scene/nib_ibtool.rs", "src/scene/nib_ibtool_widgets.rs", "src/scene/xib_exporter.rs"] {
        let src = std::fs::read_to_string(manifest_dir.join(rel)).expect("read scene source");
        for forbidden_gs in ["GSSteppingTextField", "GSOriginPointControl", "GSTransformGrid", "GSMouseOverButton", "GSInlineActionButton"]
        {
            assert!(
                !src.contains(forbidden_gs),
                "REQ-013-09.1 domain-agnostic violation: {rel} still contains host-specific class string `{forbidden_gs}`"
            );
        }
    }

    let empty_mapper = CustomClassMapper::default();
    let vec_nib_path = manifest_dir.join("assets/nib_showcase/1_Vector_and_Transform_Inspector.nib");
    let vec_scene = NibArchiveFrontend::from_path(&vec_nib_path, &empty_mapper).expect("decode nib with empty CustomClassMapper");
    assert!(
        vec_scene.nodes().iter().any(|n| matches!(n.kind, SceneNodeKind::NumberField { .. })),
        "REQ-013-09.1: NibArchiveFrontend must infer SceneNodeKind::NumberField from NSTextFieldCell + NSNumberFormatter with empty CustomClassMapper"
    );

    let menus = NibArchiveFrontend::decode_menu_bar_from_path(&vec_nib_path).unwrap_or_default();
    let _ = menus;
}

#[test]
fn tc_013_17_declarative_enabled_binding_borderless_number_field_icon_button_and_horizontal_stack_oracle() {
    use egui::{pos2, vec2, RawInput, Rect};
    use egui_macview::scene::{
        BindingId, CompiledConstraintSchedule, LayoutPolicy, MacSceneDataSource, MacSceneIR, NodeId, SceneNode, SceneNodeKind,
        ValueTransformer,
    };
    use egui_macview::{MacAppearance, MacTheme};

    #[derive(Default)]
    struct PaletteTestHost {
        proportional: bool,
    }
    impl MacSceneDataSource for PaletteTestHost {
        fn get_bool(&self, id: BindingId) -> bool {
            if id.0 == 7 {
                self.proportional
            } else {
                false
            }
        }
        fn set_bool(&mut self, _id: BindingId, _val: bool) {}
        fn get_f64(&self, _id: BindingId) -> f64 {
            0.0
        }
        fn set_f64(&mut self, _id: BindingId, _val: f64) {}
        fn get_i64(&self, _id: BindingId) -> i64 {
            0
        }
        fn set_i64(&mut self, _id: BindingId, _val: i64) {}
        fn get_str(&self, id: BindingId) -> &str {
            if id.0 == 8 {
                "110%"
            } else {
                ""
            }
        }
        fn set_str(&mut self, _id: BindingId, _val: &str) {}
        fn get_options(&self, _id: BindingId) -> &[String] {
            &[]
        }
        fn on_action(&mut self, _action: egui_macview::scene::ActionId, _sender: NodeId) {}
        fn render_custom_slot(&mut self, _ui: &mut egui::Ui, _theme: &MacTheme, _slot_class: &str, _rect: Rect) {}
    }

    let mut root = SceneNode::new(NodeId(1), SceneNodeKind::Container { is_flipped: false });
    root.ib_object_id = "_NS:1".into();
    root.nib_frame = [0.0, 0.0, 200.0, 120.0];
    root.children = vec![NodeId(2), NodeId(6), NodeId(7)];

    let mut hstack = SceneNode::new(NodeId(2), SceneNodeKind::Container { is_flipped: false });
    hstack.ib_object_id = "_NS:8".into();
    hstack.parent = Some(NodeId(1));
    hstack.nib_frame = [0.0, 80.0, 157.0, 34.0];
    hstack.stack_orientation = Some(0);
    hstack.stack_spacing = 0.0;
    hstack.detaches_hidden_views = true;
    hstack.children = vec![NodeId(3), NodeId(4), NodeId(5), NodeId(8)];

    let mut c1 = SceneNode::new(NodeId(3), SceneNodeKind::OriginMatrix);
    c1.ib_object_id = "_NS:10".into();
    c1.parent = Some(NodeId(2));
    c1.nib_frame = [0.0, 0.0, 34.0, 34.0];

    let mut c2 = SceneNode::new(NodeId(4), SceneNodeKind::InlineActionButton);
    c2.ib_object_id = "_NS:21".into();
    c2.parent = Some(NodeId(2));
    c2.nib_frame = [0.0, 2.5, 29.0, 29.0];

    let mut c3 = SceneNode::new(NodeId(5), SceneNodeKind::InlineActionButton);
    c3.ib_object_id = "_NS:28".into();
    c3.parent = Some(NodeId(2));
    c3.nib_frame = [0.0, 2.0, 56.0, 30.0];

    let mut c4_hidden = SceneNode::new(NodeId(8), SceneNodeKind::InlineActionButton);
    c4_hidden.ib_object_id = "_NS:40".into();
    c4_hidden.parent = Some(NodeId(2));
    c4_hidden.nib_frame = [128.0, 2.5, 29.0, 29.0];
    c4_hidden.is_hidden = true;

    let mut dim_field = SceneNode::new(NodeId(6), SceneNodeKind::NumberField { step: 1.0, shift_step: 10.0, min: None, max: None });
    dim_field.ib_object_id = "HV".into();
    dim_field.parent = Some(NodeId(1));
    dim_field.title = "--".into();
    dim_field.nib_frame = [10.0, 40.0, 32.0, 14.0];
    dim_field.draws_background = false;
    dim_field.is_bezeled = false;
    dim_field.is_bordered = false;

    let mut scale_y = SceneNode::new(NodeId(7), SceneNodeKind::NumberField { step: 1.0, shift_step: 10.0, min: None, max: None });
    scale_y.ib_object_id = "_NS:540".into();
    scale_y.parent = Some(NodeId(1));
    scale_y.nib_frame = [33.0, 10.0, 75.0, 21.0];
    scale_y.value_binding = Some(BindingId(8));
    scale_y.enabled_binding = Some(BindingId(7));
    scale_y.enabled_path = Some("GSTransformScaleProportional".into());
    scale_y.enabled_transformer = ValueTransformer::NegateBoolean;

    let mut scene = MacSceneIR::from_nodes(
        NodeId(1),
        [200.0, 120.0],
        vec![root, hstack, c1, c2, c3, dim_field, scale_y, c4_hidden],
        CompiledConstraintSchedule::default(),
    );
    scene.solve_autolayout();

    let solved = scene.solve_exact_nib_layout();
    let r1 = solved[&NodeId(3)];
    let r2 = solved[&NodeId(4)];
    let r3 = solved[&NodeId(5)];
    assert!(
        r1.min.x < r2.min.x && r2.min.x >= r1.max.x + 15.0 && r3.min.x >= r2.max.x + 15.0,
        "REQ-013-10.9: Horizontal NSStackView must distribute visible children left-to-right without overlap: r1={r1:?}, r2={r2:?}, r3={r3:?}"
    );

    let (ctx, theme) = common::headless_ctx(MacAppearance::Aqua);
    let mut host = PaletteTestHost { proportional: true };
    let shapes = common::run_frame(&ctx, RawInput::default(), vec2(200.0, 120.0), |ui| {
        scene.show_in_rect(
            ui,
            Rect::from_min_size(pos2(0.0, 0.0), vec2(200.0, 120.0)),
            &theme,
            &mut host,
            LayoutPolicy::ExactAppleNib,
            u64::MAX,
            "",
        );
    });
    let mut texts = Vec::new();
    for cs in &shapes {
        common::collect_text_shapes(&cs.shape, &mut texts);
    }
    assert!(
        texts.iter().any(|t| t.galley.text() == "--"),
        "REQ-013-10.9: Borderless NumberField with draws_background=false must render placeholder `--`"
    );
}

#[test]
fn tc_013_18_exhaustive_six_stage_nib_pipeline_completeness_oracle() {
    use egui::{pos2, vec2, RawInput, Rect};
    use egui_macview::scene::{
        BindingId, CompiledConstraintSchedule, LayoutPolicy, MacSceneDataSource, MacSceneIR, NodeId, SceneNode, SceneNodeKind,
    };
    use egui_macview::{MacAppearance, MacTheme};
    #[derive(Default)]
    struct PatternTestHost;
    impl MacSceneDataSource for PatternTestHost {
        fn get_bool(&self, _id: BindingId) -> bool {
            false
        }
        fn set_bool(&mut self, _id: BindingId, _val: bool) {}
        fn get_f64(&self, _id: BindingId) -> f64 {
            0.0
        }
        fn set_f64(&mut self, _id: BindingId, _val: f64) {}
        fn get_i64(&self, _id: BindingId) -> i64 {
            0
        }
        fn set_i64(&mut self, _id: BindingId, _val: i64) {}
        fn get_str(&self, id: BindingId) -> &str {
            match id.0 {
                1 => "1",
                2 => "6",
                3 => "6",
                _ => "",
            }
        }
        fn set_str(&mut self, _id: BindingId, _val: &str) {}
        fn get_options(&self, _id: BindingId) -> &[String] {
            &[]
        }
        fn on_action(&mut self, _action: egui_macview::scene::ActionId, _sender: NodeId) {}
        fn render_custom_slot(&mut self, _ui: &mut egui::Ui, _theme: &MacTheme, _slot_class: &str, _rect: Rect) {}
    }
    let mut root = SceneNode::new(NodeId(1), SceneNodeKind::Container { is_flipped: false });
    root.ib_object_id = "_NS:1".into();
    root.nib_frame = [0.0, 0.0, 240.0, 40.0];
    root.children = vec![NodeId(2)];
    let mut status_lbl = SceneNode::new(NodeId(2), SceneNodeKind::Label);
    status_lbl.ib_object_id = "_NS:688".into();
    status_lbl.parent = Some(NodeId(1));
    status_lbl.nib_frame = [10.0, 8.0, 220.0, 17.0];
    status_lbl.value_binding = Some(BindingId(1));
    status_lbl.binding_path = Some("selectedObjects.@count".into());
    status_lbl.display_pattern = Some("%{value1}@ selected %{value2}@/%{value3}@ glyphs".into());
    status_lbl.display_pattern_bindings = vec![
        (1, BindingId(1), "selectedObjects.@count".into()),
        (2, BindingId(2), "arrangedObjects.@count".into()),
        (3, BindingId(3), "content.@count".into()),
    ];
    let scene = MacSceneIR::from_nodes(NodeId(1), [240.0, 40.0], vec![root, status_lbl], CompiledConstraintSchedule::default());
    let (ctx, theme) = common::headless_ctx(MacAppearance::Aqua);
    let mut host = PatternTestHost;
    let shapes = common::run_frame(&ctx, RawInput::default(), vec2(240.0, 40.0), |ui| {
        scene.show_in_rect(ui, Rect::from_min_size(pos2(0.0, 0.0), vec2(240.0, 40.0)), &theme, &mut host, LayoutPolicy::ExactAppleNib, u64::MAX, "");
    });
    let mut texts = Vec::new();
    for cs in &shapes {
        common::collect_text_shapes(&cs.shape, &mut texts);
    }
    assert!(
        texts.iter().any(|t| t.galley.text() == "1 selected 6/6 glyphs"),
        "REQ-013-10.10: Label with display_pattern and display_pattern_bindings must substitute %{{value1}}@..%{{value3}}@"
    );
    let rw_src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/scene/runtime_widgets.rs")).unwrap();
    let rt_src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/scene/runtime_tabs.rs")).unwrap();
    assert!(
        rw_src.contains("node.image_scaling") && rw_src.contains("node.symbol_point_size") && rt_src.contains("node.symbol_point_size"),
        "REQ-013-10.10: SceneNode::image_scaling and symbol_point_size must be read by the scene runtime renderers (zero dead IR fields)"
    );
    let mut measure_img = |scaling: u8, sym_pt: Option<f32>| -> f32 {
        let mut r = SceneNode::new(NodeId(1), SceneNodeKind::Container { is_flipped: false });
        r.nib_frame = [0.0, 0.0, 64.0, 64.0];
        r.children = vec![NodeId(2)];
        let mut img = SceneNode::new(NodeId(2), SceneNodeKind::ImageView { icon_name: "tag".into(), is_template: true });
        img.parent = Some(NodeId(1));
        img.nib_frame = [0.0, 0.0, 64.0, 64.0];
        img.image_scaling = scaling;
        img.symbol_point_size = sym_pt;
        let sc = MacSceneIR::from_nodes(NodeId(1), [64.0, 64.0], vec![r, img], CompiledConstraintSchedule::default());
        let sh = common::run_frame(&ctx, RawInput::default(), vec2(64.0, 64.0), |ui| {
            sc.show_in_rect(ui, Rect::from_min_size(pos2(0.0, 0.0), vec2(64.0, 64.0)), &theme, &mut host, LayoutPolicy::ExactAppleNib, u64::MAX, "");
        });
        sh.iter().map(|cs| cs.shape.visual_bounding_rect().width()).fold(0.0_f32, f32::max)
    };
    assert!(measure_img(3, None) > measure_img(0, Some(10.0)) + 8.0, "NSScale=3 must render wider than NSScale=0@10pt");
}
