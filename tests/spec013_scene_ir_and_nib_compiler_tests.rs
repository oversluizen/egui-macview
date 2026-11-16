//! Normative COTS Verification Suite for SPEC-013 (`egui-macview` v0.7.0):
//! Three-Layer UI Compiler (`NibArchiveFrontend`, `XibXmlFrontend`, `SwiftUiFrontend`,
//! `StringsCatalog`, `MacSceneIR`, `SpatialSemanticNormalizer`, `SceneSearchIndex`,
//! `VisibilityRule`, `SceneContractVerifier`, `SceneExporter`, and Dual-Mode Layout Solver).

use egui::{pos2, vec2, Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect};
use egui_macview::scene::{
    evaluate_arithmetic_expr, ActionId, BindingId, BindingValueType, CompiledConstraintSchedule, ConstraintAttr, ConstraintRelation,
    CustomClassMapper, HostContractSchema, LayoutConstraintSpec, LayoutPolicy, MacSceneDataSource, MacSceneSpec, NibArchiveFrontend,
    NodeId, SceneExporter, SceneNodeKind, StringsCatalog, SwiftUiFrontend, ValueTransformer, XibXmlFrontend,
};
use egui_macview::{
    MacAppearance, MacBox, MacControlSize, MacImageFrameStyle, MacImageScaling, MacImageView, MacSwitch, MacSymbol, MacTextView, MacTheme,
};
use std::collections::BTreeMap;

#[derive(Default)]
struct TestSceneHost {
    bools: BTreeMap<u32, bool>,
    floats: BTreeMap<u32, f64>,
    ints: BTreeMap<u32, i64>,
    strings: BTreeMap<u32, String>,
    options: BTreeMap<u32, Vec<String>>,
    fired_actions: Vec<(ActionId, NodeId)>,
    custom_slots_rendered: Vec<(String, Rect)>,
}

impl MacSceneDataSource for TestSceneHost {
    fn get_bool(&self, id: BindingId) -> bool {
        self.bools.get(&id.0).copied().unwrap_or(false)
    }
    fn set_bool(&mut self, id: BindingId, val: bool) {
        self.bools.insert(id.0, val);
    }
    fn get_f64(&self, id: BindingId) -> f64 {
        self.floats.get(&id.0).copied().unwrap_or(0.0)
    }
    fn set_f64(&mut self, id: BindingId, val: f64) {
        self.floats.insert(id.0, val);
    }
    fn get_i64(&self, id: BindingId) -> i64 {
        self.ints.get(&id.0).copied().unwrap_or(0)
    }
    fn set_i64(&mut self, id: BindingId, val: i64) {
        self.ints.insert(id.0, val);
    }
    fn get_str(&self, id: BindingId) -> &str {
        self.strings.get(&id.0).map(String::as_str).unwrap_or("")
    }
    fn set_str(&mut self, id: BindingId, val: &str) {
        self.strings.insert(id.0, val.to_string());
    }
    fn get_options(&self, id: BindingId) -> &[String] {
        self.options.get(&id.0).map(Vec::as_slice).unwrap_or(&[])
    }
    fn on_action(&mut self, action: ActionId, sender: NodeId) {
        self.fired_actions.push((action, sender));
    }
    fn render_custom_slot(&mut self, ui: &mut egui::Ui, theme: &MacTheme, slot_class: &str, rect: Rect) {
        ui.painter().rect_filled(rect, 2.0, theme.control_background());
        self.custom_slots_rendered.push((slot_class.to_string(), rect));
    }
}

#[test]
fn tc_013_01_binary_nib_archive_and_bplist_lowering_with_control_cell_fusion() {
    let mapper = CustomClassMapper::default_studio_mappings();
    let raw_nib = NibArchiveFrontend::build_real_varint_nib_archive();
    assert!(raw_nib.starts_with(b"NIBArchive\x01\x00\x00\x00"));
    let scene = NibArchiveFrontend::from_bytes(&raw_nib, &mapper).expect("Binary NIBArchive v1 must lower into MacSceneIR");

    let has_number_field = scene.nodes().iter().any(|n| {
        matches!(n.kind, SceneNodeKind::NumberField { .. })
            && n.control_size == "small"
            && n.binding_path.as_deref() == Some("scalePercent")
            && n.value_transformer == ValueTransformer::NegateBoolean
            && n.autoresizing_mask == 2
    });
    let has_default_push = scene.nodes().iter().any(|n| {
        matches!(n.kind, SceneNodeKind::PushButton { is_default: true, .. }) && n.action_selector.as_deref() == Some("applyTransform:")
    });
    let has_checkbox = scene.nodes().iter().any(|n| matches!(n.kind, SceneNodeKind::Checkbox));
    let has_origin_matrix = scene.nodes().iter().any(|n| matches!(n.kind, SceneNodeKind::OriginMatrix));
    assert!(has_number_field, "NIBArchive NSCell cFlags(Small) + NSClassSwapper + NSNibBindingConnector(arrangedObjects.scalePercent, NSNegateBoolean) must fuse into NumberField");
    assert!(has_default_push, "NIBArchive NSButtonCell NSKeyEquivalent '\\r' + NSNibControlConnector must fuse into default PushButton");
    assert!(has_checkbox, "NIBArchive NSButtonFlags bit 23 (0x0080_0000) must decode into Checkbox");
    assert!(has_origin_matrix, "NIBArchive NSClassSwapper OriginPointControl must lower to OriginMatrix");

    let bplist_bytes = NibArchiveFrontend::build_real_nskeyedarchiver_bplist();
    assert!(bplist_bytes.starts_with(b"bplist00"));
    let bplist_scene = NibArchiveFrontend::from_bytes(&bplist_bytes, &mapper).expect("bplist00 keyed archive must lower into MacSceneIR");
    let has_table = bplist_scene
        .nodes()
        .iter()
        .any(|n| matches!(&n.kind, SceneNodeKind::TableView { columns } if columns.len() == 2 && columns[0].0 == "Glyph"));
    assert!(has_table, "bplist00 NSScrollView -> NSTableView must unwrap into SceneNodeKind::TableView with columns");
}

#[test]
fn tc_013_02_xib_xml_and_strings_catalog_localization_and_search_rebuild() {
    let xib_xml = r#"
    <document type="com.apple.InterfaceBuilder3.Cocoa.XIB" version="3.0">
      <objects>
        <window id="win-1" title="Preferences">
          <rect key="frame" x="0" y="0" width="480" height="320"/>
          <view key="contentView" id="root-view">
            <rect key="frame" x="0" y="0" width="480" height="320"/>
            <subviews>
              <box id="box-1" title="Typography">
                <rect key="frame" x="16" y="180" width="448" height="120"/>
                <view key="contentView" id="box-cv">
                  <subviews>
                    <textField id="141">
                      <rect key="frame" x="20" y="260" width="110" height="16"/>
                      <textFieldCell key="cell" title="Grid Spacing:"/>
                    </textField>
                    <textField id="142">
                      <rect key="frame" x="138" y="258" width="90" height="22"/>
                      <textFieldCell key="cell" editable="YES" bordered="YES" title="10"/>
                      <connections>
                        <binding destination="defaults" name="value" keyPath="arrangedObjects.gridSpacing" id="b-1"/>
                      </connections>
                    </textField>
                  </subviews>
                </view>
              </box>
              <scrollView id="sv-1">
                <rect key="frame" x="16" y="60" width="448" height="100"/>
                <clipView key="contentView" id="cv-1">
                  <subviews>
                    <tableView id="tbl-1">
                      <tableColumns>
                        <tableColumn identifier="glyph" width="140">
                          <tableHeaderCell key="headerCell" title="Glyph Name"/>
                        </tableColumn>
                      </tableColumns>
                    </tableView>
                  </subviews>
                </clipView>
              </scrollView>
              <button id="143">
                <rect key="frame" x="360" y="20" width="100" height="32"/>
                <buttonCell key="cell" type="push" title="Export" bezelStyle="rounded" keyEquivalent="&#13;"/>
                <connections>
                  <action selector="exportNow:" target="-2" id="a-1"/>
                </connections>
              </button>
            </subviews>
            <constraints>
              <constraint firstItem="142" firstAttribute="leading" secondItem="141" secondAttribute="trailing" constant="12" id="c-1"/>
            </constraints>
          </view>
        </window>
      </objects>
    </document>
    "#;

    let mut scene = XibXmlFrontend::from_xml_str(xib_xml).expect("Parse XIB XML");
    let box_node = scene.nodes().iter().find(|n| n.ib_object_id == "box-1").expect("box-1 must exist");
    let lbl_node = scene.nodes().iter().find(|n| n.ib_object_id == "141").expect("141 must exist");
    let fld_node = scene.nodes().iter().find(|n| n.ib_object_id == "142").expect("142 must exist");
    assert_eq!(lbl_node.parent, Some(box_node.id), "Nested <subviews> inside <box> must preserve box parent NodeId");
    assert_eq!(fld_node.binding_path.as_deref(), Some("gridSpacing"), "Controller prefix arrangedObjects. must be flattened");
    assert!(scene
        .nodes()
        .iter()
        .any(|n| matches!(&n.kind, SceneNodeKind::TableView { columns } if columns.len() == 1 && columns[0].0 == "Glyph Name")));

    let solved = scene.solve_layout(vec2(480.0, 320.0), LayoutPolicy::ExactAppleNib, u64::MAX, "");
    assert!(
        (solved[&fld_node.id].min.x - (solved[&lbl_node.id].max.x + 12.0)).abs() <= 0.5,
        "XIB <constraint> must position 142.leading at 141.trailing + 12pt"
    );

    let strings_src = r#"
        /* German localization */
        "141.title" = "Rasterabstand:";
        "143.title" = "Exportieren";
        "143.ibShadowedToolTip" = "Schrift jetzt exportieren";
    "#;
    let catalog = StringsCatalog::from_str(strings_src).expect("Parse .strings");
    scene.apply_strings_catalog(&catalog);
    assert!(scene.filter_by_search("rasterabstand").matched_node_count() >= 1);
    assert!(scene.filter_by_search("schrift jetzt").matched_node_count() >= 1);
}

#[test]
fn tc_013_03_four_new_contemporary_primitives_render_and_bind() {
    let ctx = Context::default();
    let aqua = MacTheme::new(MacAppearance::Aqua);
    let dark = MacTheme::new(MacAppearance::DarkAqua);

    let mut sw_on = false;
    let mut notes_text = "Line 1\nLine 2\nLine 3".to_string();
    let mut switch_rect = Rect::NOTHING;

    for theme in [&aqua, &dark] {
        let mut out = ctx.run_ui(RawInput::default(), |ui| {
            MacImageView::from_symbol(MacSymbol::Gear)
                .theme(theme)
                .template(true)
                .scaling(MacImageScaling::ProportionallyDown)
                .frame_style(MacImageFrameStyle::GrayBezel)
                .fixed_rect(Rect::from_min_size(pos2(16.0, 16.0), vec2(32.0, 32.0)))
                .show(ui);

            switch_rect = MacSwitch::new(&mut sw_on)
                .theme(theme)
                .control_size(MacControlSize::Regular)
                .fixed_rect(Rect::from_min_size(pos2(60.0, 20.0), vec2(38.0, 22.0)))
                .show(ui)
                .rect;

            MacTextView::new("notes_editor", &mut notes_text)
                .theme(theme)
                .monospace(true)
                .line_numbers(true)
                .fixed_rect(Rect::from_min_size(pos2(16.0, 60.0), vec2(260.0, 90.0)))
                .show(ui);

            MacBox::separator_at(ui, Rect::from_min_size(pos2(16.0, 160.0), vec2(260.0, 5.0)), theme);
        });
        out.textures_delta.clear();
        assert!(!out.shapes.is_empty());
    }

    let click_in = RawInput {
        events: vec![
            Event::PointerMoved(switch_rect.center()),
            Event::PointerButton { pos: switch_rect.center(), button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE },
            Event::PointerButton { pos: switch_rect.center(), button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE },
        ],
        ..Default::default()
    };
    let mut out = ctx.run_ui(click_in, |ui| {
        MacSwitch::new(&mut sw_on).theme(&aqua).fixed_rect(switch_rect).show(ui);
    });
    out.textures_delta.clear();
    assert!(sw_on, "Clicking MacSwitch must toggle boolean state");
}

#[test]
fn tc_013_04_spatial_semantic_normalizer_pairs_labels_and_infers_sections() {
    let xib_xml = r#"
    <document type="com.apple.InterfaceBuilder3.Cocoa.XIB" version="3.0">
      <objects>
        <window id="win-1" title="Inspector">
          <view key="contentView" id="root-view" flipped="YES">
            <rect key="frame" x="0" y="0" width="480" height="320"/>
            <subviews>
              <box id="box-dim" title="Dimensions">
                <rect key="frame" x="12" y="10" width="456" height="90"/>
              </box>
              <textField id="lbl-l"><rect key="frame" x="24" y="28" width="20" height="16"/><textFieldCell key="cell" title="L:"/></textField>
              <textField id="fld-l"><rect key="frame" x="48" y="25" width="60" height="22"/><textFieldCell key="cell" editable="YES" title="10"/></textField>
              <textField id="lbl-r"><rect key="frame" x="130" y="28" width="20" height="16"/><textFieldCell key="cell" title="R:"/></textField>
              <textField id="fld-r"><rect key="frame" x="154" y="25" width="60" height="22"/><textFieldCell key="cell" editable="YES" title="20"/></textField>
              <box id="sep-1" boxType="separator"><rect key="frame" x="12" y="110" width="456" height="2"/></box>
              <textField id="lbl-w"><rect key="frame" x="24" y="130" width="40" height="16"/><textFieldCell key="cell" title="Width:"/></textField>
              <textField id="fld-w"><rect key="frame" x="72" y="127" width="80" height="22"/><textFieldCell key="cell" editable="YES" title="600"/></textField>
            </subviews>
          </view>
        </window>
      </objects>
    </document>
    "#;
    let scene = XibXmlFrontend::from_xml_str(xib_xml).expect("Compile XIB");
    assert_eq!(scene.semantic_sections().len(), 2, "NSBoxSeparator must split scene into 2 sections");
    assert_eq!(scene.semantic_sections()[0].title, "Dimensions", "First section must inherit title from enclosing NSBox 'Dimensions'");
    assert_eq!(scene.semantic_sections()[0].pairs.len(), 2, "Multi-field row L: and R: must form 2 LabeledFieldPairs in Section 1");
    let exported = SceneExporter::to_swiftui_source(&scene);
    assert!(exported.contains("HStack {"), "Multi-field row on same Y baseline must export as HStack in SwiftUI:\n{exported}");
}

#[test]
fn tc_013_05_scene_search_index_filters_and_collapses_in_adaptive_reflow_mode() {
    let nib_bytes = NibArchiveFrontend::build_synthetic_preferences_nib();
    let scene = NibArchiveFrontend::from_bytes(&nib_bytes, &CustomClassMapper::default_studio_mappings()).expect("Compile preferences NIB");

    let all_rects = scene.solve_layout(vec2(520.0, 400.0), LayoutPolicy::AdaptiveSemanticReflow, u64::MAX, "");
    let filtered_rects = scene.solve_layout(vec2(520.0, 400.0), LayoutPolicy::AdaptiveSemanticReflow, u64::MAX, "snap");

    assert!(filtered_rects.len() < all_rects.len());
    assert!(!filtered_rects.is_empty());
}

#[test]
fn tc_013_06_visibility_rule_hides_context_tools_and_closes_toolbar_gaps() {
    let mut scene = NibArchiveFrontend::build_synthetic_toolbar_scene();
    const MODE_DRAWING: u64 = 1 << 0;
    const MODE_INSTRUCTOR: u64 = 1 << 1;

    let rects_all = scene.solve_layout(vec2(600.0, 38.0), LayoutPolicy::AdaptiveSemanticReflow, MODE_DRAWING | MODE_INSTRUCTOR, "");
    let rects_drawing = scene.solve_layout(vec2(600.0, 38.0), LayoutPolicy::AdaptiveSemanticReflow, MODE_DRAWING, "");

    assert!(rects_drawing.len() < rects_all.len());
    let mut xs: Vec<f32> = rects_drawing.values().map(|r| r.min.x).collect();
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    for pair in xs.windows(2) {
        assert!(
            pair[1] - pair[0] <= 48.0,
            "Hiding specialized tools in drawing mode must close horizontal gaps (found gap {})",
            pair[1] - pair[0]
        );
    }
    let _ = &mut scene;
}

#[test]
fn tc_013_07_scene_contract_verifier_detects_missing_bindings_actions_and_icons() {
    let nib_bytes = NibArchiveFrontend::build_synthetic_preferences_nib();
    let scene = NibArchiveFrontend::from_bytes(&nib_bytes, &CustomClassMapper::default_studio_mappings()).expect("Compile preferences NIB");

    let empty_schema = HostContractSchema::new();
    let bad_report = scene.verify_contract(&empty_schema);
    assert!(!bad_report.is_ok(), "Empty schema must fail contract verification");
    assert!(!bad_report.missing_bindings.is_empty() || !bad_report.missing_actions.is_empty());

    let valid_schema = scene.derive_matching_schema();
    let ok_report = scene.verify_contract(&valid_schema);
    assert!(ok_report.is_ok(), "Matching schema must pass 100%: {ok_report:#?}");
}

#[test]
fn tc_013_08_exact_apple_nib_alignment_insets_autoresizing_and_constraint_dag() {
    let nib_bytes = NibArchiveFrontend::build_synthetic_nib_fixture();
    let scene = NibArchiveFrontend::from_bytes(&nib_bytes, &CustomClassMapper::default_studio_mappings()).expect("Compile NIB");

    let base_rects = scene.solve_layout(vec2(400.0, 300.0), LayoutPolicy::ExactAppleNib, u64::MAX, "");
    let resized_rects = scene.solve_layout(vec2(600.0, 450.0), LayoutPolicy::ExactAppleNib, u64::MAX, "");
    assert_eq!(base_rects.len(), resized_rects.len());

    let push_id = scene.nodes().iter().find(|n| matches!(n.kind, SceneNodeKind::PushButton { .. })).map(|n| n.id).unwrap();
    let push_rect = base_rects[&push_id];
    assert!(
        (push_rect.height() - 28.0).abs() <= 0.5 || (push_rect.height() - 20.0).abs() <= 0.5,
        "Outset NIB push button frame [-6, -6, 131, 40] must normalize to 28pt or 20pt visual height, got {}",
        push_rect.height()
    );

    let (n1, n2) = (NodeId(1), NodeId(2));
    let mk_c = |fi, fa, si, sa, c| LayoutConstraintSpec {
        first_item: fi,
        first_attr: fa,
        relation: ConstraintRelation::Equal,
        second_item: si,
        second_attr: sa,
        multiplier: 1.0,
        constant: c,
        priority: 1000,
    };
    let schedule = CompiledConstraintSchedule::compile(vec![
        mk_c(Some(n1), ConstraintAttr::Left, None, ConstraintAttr::Left, 20.0),
        mk_c(Some(n1), ConstraintAttr::Width, None, ConstraintAttr::NotAnAttribute, 120.0),
        mk_c(Some(n2), ConstraintAttr::Left, Some(n1), ConstraintAttr::Right, 12.0),
    ]);
    let mut solved = BTreeMap::new();
    solved.insert(n1, Rect::from_min_size(pos2(0.0, 10.0), vec2(50.0, 22.0)));
    solved.insert(n2, Rect::from_min_size(pos2(0.0, 10.0), vec2(80.0, 22.0)));
    schedule.evaluate(Rect::from_min_size(Pos2::ZERO, vec2(500.0, 300.0)), &mut solved);
    assert!(
        (solved[&n1].min.x - 20.0).abs() < 1e-3 && (solved[&n1].width() - 120.0).abs() < 1e-3 && (solved[&n2].min.x - 152.0).abs() < 1e-3
    );
}

#[test]
fn tc_013_09_subscene_mounting_controllers_transformers_arithmetic_and_json_roundtrip() {
    assert!((evaluate_arithmetic_expr("100 + 25 * 2").unwrap() - 150.0).abs() < 1e-6);
    assert!((evaluate_arithmetic_expr("(1000 - 200) / 4").unwrap() - 200.0).abs() < 1e-6);
    assert!(ValueTransformer::NegateBoolean.apply_bool(false) && !ValueTransformer::NegateBoolean.apply_bool(true));
    assert!(
        ValueTransformer::IsNotNil.apply_str("Glyph") && !ValueTransformer::IsNotNil.apply_str("") && ValueTransformer::IsNil.apply_str("")
    );

    let mapper = CustomClassMapper::default_studio_mappings();
    let mut parent_scene = NibArchiveFrontend::from_bytes(&NibArchiveFrontend::build_synthetic_nib_fixture(), &mapper).unwrap();
    let child_scene = NibArchiveFrontend::from_bytes(&NibArchiveFrontend::build_synthetic_preferences_nib(), &mapper).unwrap();
    let before_count = parent_scene.nodes().len();
    parent_scene.mount_subscene(parent_scene.root_id(), &child_scene);
    assert!(parent_scene.nodes().len() > before_count);

    let spec_json = SceneExporter::to_json_string(&parent_scene).unwrap();
    let roundtrip_scene = MacSceneSpec::from_json_str(&spec_json).unwrap().into_scene_ir();
    assert_eq!(parent_scene.nodes().len(), roundtrip_scene.nodes().len());

    let (ctx, theme, mut host) = (Context::default(), MacTheme::new(MacAppearance::Aqua), TestSceneHost::default());
    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        parent_scene.show(ui, &theme, &mut host, LayoutPolicy::ExactAppleNib, u64::MAX, "");
    });
    out.textures_delta.clear();
    assert!(!out.shapes.is_empty());
}

#[test]
fn tc_013_10_bidirectional_swiftui_roundtrip_from_legacy_nib() {
    let full_swiftui_src = r#"
    struct InspectorView: View {
        var body: some View {
            Form {
                Section("Metrics") {
                    HStack {
                        LabeledContent("L:") { TextField("L:", text: $glyph.leftSidebearing).controlSize(.small).frame(width: 60.0, height: 19.0) }
                        LabeledContent("R:") { TextField("R:", text: $glyph.rightSidebearing).controlSize(.small).frame(width: 60.0, height: 19.0) }
                    }
                    Picker("Alignment", selection: $glyph.alignment) { Text("Left").tag(0) }.pickerStyle(.segmented).controlSize(.small)
                    Toggle("Auto Kern", isOn: $glyph.autoKern).toggleStyle(.switch).disabled(true)
                    Stepper("Weight", value: $glyph.weight, in: 100...900, step: 100)
                    Image(systemName: "NSActionTemplate")
                    TextEditor(text: $glyph.notes)
                }
            }
        }
    }
    "#;
    let parsed_scene = SwiftUiFrontend::parse(full_swiftui_src).expect("Parse full contemporary SwiftUI view");
    let re_exported = SceneExporter::to_swiftui_source(&parsed_scene);
    assert!(re_exported.contains(".pickerStyle(.segmented)"), "Must preserve SegmentedControl:\n{re_exported}");
    assert!(re_exported.contains(".toggleStyle(.switch)"), "Must preserve Switch:\n{re_exported}");
    assert!(re_exported.contains(".disabled(true)"), "Must preserve .disabled(true):\n{re_exported}");
    assert!(re_exported.contains("Stepper("), "Must preserve Stepper:\n{re_exported}");
    assert!(re_exported.contains("Image(systemName: \"NSActionTemplate\")"), "Must preserve ImageView:\n{re_exported}");
    assert!(re_exported.contains("TextEditor("), "Must preserve TextView:\n{re_exported}");

    let nib_bytes = NibArchiveFrontend::build_synthetic_preferences_nib();
    let nib_scene = NibArchiveFrontend::from_bytes(&nib_bytes, &CustomClassMapper::default_studio_mappings()).unwrap();
    let swiftui_code = SceneExporter::to_swiftui_source(&nib_scene);
    assert!(swiftui_code.contains("Section(") && swiftui_code.contains(".controlSize(") && swiftui_code.contains('$'));

    let swiftui_scene = SwiftUiFrontend::parse(&swiftui_code).unwrap();
    assert_eq!(nib_scene.semantic_sections().len(), swiftui_scene.semantic_sections().len());
    assert_eq!(nib_scene.interactive_binding_ids(), swiftui_scene.interactive_binding_ids());

    let (ctx, theme, mut host_a, mut host_b) =
        (Context::default(), MacTheme::new(MacAppearance::Aqua), TestSceneHost::default(), TestSceneHost::default());
    for b_id in nib_scene.interactive_binding_ids() {
        host_a.set_bool(b_id, true);
        host_b.set_bool(b_id, true);
        host_a.set_str(b_id, "42");
        host_b.set_str(b_id, "42");
    }
    let mut out_a = ctx.run_ui(RawInput::default(), |ui| {
        nib_scene.show(ui, &theme, &mut host_a, LayoutPolicy::AdaptiveSemanticReflow, u64::MAX, "");
    });
    out_a.textures_delta.clear();
    let mut out_b = ctx.run_ui(RawInput::default(), |ui| {
        swiftui_scene.show(ui, &theme, &mut host_b, LayoutPolicy::AdaptiveSemanticReflow, u64::MAX, "");
    });
    out_b.textures_delta.clear();
    assert!(!out_a.shapes.is_empty() && !out_b.shapes.is_empty());
    let _ = BindingValueType::Bool;
}

#[test]
fn tc_013_11_real_nibarchive_varints_nskeyedarchiver_plist_roxmltree_and_studio_live_drag() {
    let mapper = CustomClassMapper::default_studio_mappings();
    let varint_scene = NibArchiveFrontend::from_bytes(&NibArchiveFrontend::build_real_varint_nib_archive(), &mapper).unwrap();
    assert!(varint_scene.nodes().len() >= 3);
    let bplist_scene = NibArchiveFrontend::from_bytes(&NibArchiveFrontend::build_real_nskeyedarchiver_bplist(), &mapper).unwrap();
    assert!(bplist_scene.nodes().len() >= 3);

    let mut studio_scene = NibArchiveFrontend::from_bytes(&NibArchiveFrontend::build_synthetic_preferences_nib(), &mapper).unwrap();
    let solved = studio_scene.solve_layout(vec2(520.0, 360.0), LayoutPolicy::ExactAppleNib, u64::MAX, "");
    assert_eq!(studio_scene.hit_test(pos2(150.0, 100.0), &solved), Some(NodeId(6)));
    assert_eq!(studio_scene.semantic_sections().len(), 2);
    studio_scene.validate_invariants().unwrap();
    let snap_0 = studio_scene.clone();
    assert!(studio_scene.set_node_nib_frame(NodeId(6), [132.0, 48.0, 140.0, 18.0]));
    assert!(studio_scene.set_node_nib_frame(NodeId(5), [24.0, 49.0, 100.0, 16.0]));
    studio_scene.validate_invariants().unwrap();
    assert_ne!(snap_0, studio_scene);
    assert_eq!(studio_scene.semantic_sections().len(), 1);
    assert_eq!(SceneExporter::to_swiftui_source(&studio_scene).matches("Section(").count(), 1);

    let (ctx, theme, mut host) = (Context::default(), MacTheme::new(MacAppearance::Aqua), TestSceneHost::default());
    let t0 = std::time::Instant::now();
    let mut last_delay = std::time::Duration::ZERO;
    for _ in 0..100 {
        let mut out = ctx.run_ui(RawInput::default(), |ui| {
            studio_scene.show(ui, &theme, &mut host, LayoutPolicy::ExactAppleNib, u64::MAX, "");
        });
        out.textures_delta.clear();
        last_delay = out.viewport_output.get(&egui::ViewportId::ROOT).map(|v| v.repaint_delay).unwrap_or_default();
    }
    assert!(t0.elapsed().as_millis() < 250 && last_delay > std::time::Duration::ZERO);
}

#[test]
fn tc_013_12_sub_ui_offset_translation_and_clean_swiftui_syntax() {
    let mapper = CustomClassMapper::default_studio_mappings();
    let scene = NibArchiveFrontend::from_bytes(&NibArchiveFrontend::build_real_varint_nib_archive(), &mapper).unwrap();
    let swiftui_src = SceneExporter::to_swiftui_source(&scene);
    assert!(!swiftui_src.contains('#') && !swiftui_src.contains("Text(\"\")") && swiftui_src.contains("OriginPointPicker("));
    assert!(swiftui_src.contains("LabeledContent(\"Scale:\")"), "Scale: label must pair with 100 field:\n{swiftui_src}");

    let (ctx, theme, mut host) = (Context::default(), MacTheme::new(MacAppearance::Aqua), TestSceneHost::default());
    let sub_rect = Rect::from_min_size(pos2(60.0, 140.0), vec2(480.0, 260.0));
    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(sub_rect));
        scene.show(&mut child_ui, &theme, &mut host, LayoutPolicy::AdaptiveSemanticReflow, u64::MAX, "");
    });
    out.textures_delta.clear();
    for clipped in &out.shapes {
        let bounds = clipped.shape.visual_bounding_rect();
        if bounds.width() > 0.0 && bounds.height() > 0.0 {
            assert!(bounds.min.x >= 55.0 && bounds.min.y >= 135.0, "Widget painted outside child_ui offset at {bounds:?}");
        }
    }
}

#[test]
fn tc_013_13_xcode16_xib_exporter_real_ibtool_nib_and_storyboard_segues() {
    use egui_macview::scene::{SegueKind, StoryboardFrontend};
    let showcase_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join("nib_showcase");
    let mapper = CustomClassMapper::default_studio_mappings();

    for stem in [
        "1_Vector_and_Transform_Inspector",
        "1_Typography_and_Grid_Inspector",
        "2_Appearance_and_Theme_Settings",
        "2_Theme_and_Appearance_Studio",
        "3_AVKit_Audio_Mastering_Strip",
        "3_Audio_Mastering_Channel_Strip",
        "4_Publishing_and_Asset_Queue",
        "Right_Inspector_Default",
    ] {
        let nib_path = showcase_dir.join(format!("{stem}.nib"));
        let nib_bytes = std::fs::read(&nib_path).expect("read .nib");
        assert!(
            nib_bytes.windows(14).any(|w| w == b"NSIBObjectData")
                && !nib_bytes.windows(6).any(|w| w == b"[{\"id\"")
                && !nib_bytes.windows(11).any(|w| w == b"macviewMeta"),
            "{stem}.nib must be a genuine Xcode 16.2 ibtool-compiled NIBArchive with zero JSON or macviewMeta"
        );
        let nib_scene = NibArchiveFrontend::from_path(&nib_path, &mapper).expect("decode real ibtool .nib");

        let xib_path = showcase_dir.join(format!("{stem}.xib"));
        let xib_xml = std::fs::read_to_string(&xib_path).expect("read .xib");
        assert!(xib_xml.contains("toolsVersion=\"23504\""), "{stem}.xib must use Xcode 16.2 toolsVersion=23504");
        assert!(xib_xml.contains("com.apple.InterfaceBuilder.CocoaPlugin"), "{stem}.xib must declare CocoaPlugin dependency");
        assert!(!xib_xml.contains("<disclosureGroup"), "{stem}.xib must not use non-standard <disclosureGroup> tag");
        assert!(!xib_xml.contains("macviewMeta"), "{stem}.xib must not contain hidden macviewMeta JSON strings (REQ-013-05.5)");
        if xib_xml.contains("<numberFormatter") {
            assert!(xib_xml.contains("formatterBehavior=\"default10_4\""), "{stem}.xib numberFormatter must set default10_4");
        }
        if xib_xml.contains("<colorWell") {
            assert!(xib_xml.contains("<color key=\"color\""), "{stem}.xib colorWell must include child <color key=\"color\"/>");
        }

        let xib_scene = XibXmlFrontend::from_xml_str(&xib_xml).expect("parse Xcode 16.2 .xib");
        assert!(
            matches!(xib_scene.nodes()[0].kind, SceneNodeKind::Container { is_flipped: false }),
            "{stem}.xib root NSView must have is_flipped=false (genuine AppKit bottom-up coordinates)"
        );
        let hdr_node = &xib_scene.nodes()[1];
        assert!(
            hdr_node.nib_frame[1] > 400.0 && hdr_node.alignment_rect[1] <= 12.0,
            "{stem}.xib header {} must have bottom-up AppKit nib_frame.y > 400 (got {}) and top-down alignment_rect.y <= 12 (got {})",
            hdr_node.ib_object_id,
            hdr_node.nib_frame[1],
            hdr_node.alignment_rect[1]
        );
        let exported_xib = SceneExporter::to_xib_xml(&xib_scene);
        assert!(exported_xib.contains("toolsVersion=\"23504\"") && !exported_xib.contains("macviewMeta"));
        let roundtrip_xib_scene = XibXmlFrontend::from_xml_str(&exported_xib).expect("re-parse exported .xib");
        assert_eq!(xib_scene.nodes().len(), roundtrip_xib_scene.nodes().len(), "{stem} XIB export round-trip node count");
        assert_eq!(xib_scene.nodes().len(), nib_scene.nodes().len(), "{stem} real ibtool .nib node count must match .xib");
        for (xn, nn) in xib_scene.nodes().iter().zip(nib_scene.nodes().iter()) {
            assert_eq!(xn.icon_name, nn.icon_name, "{stem} node {} icon_name mismatch (.xib vs ibtool .nib)", xn.ib_object_id);
        }
    }

    let sb_xml = std::fs::read_to_string(showcase_dir.join("Showcase_Flow.storyboard")).expect("read Showcase_Flow.storyboard");
    let sb = StoryboardFrontend::from_xml_str(&sb_xml).expect("parse Showcase_Flow.storyboard");
    assert!(sb.scenes.len() >= 4, "Storyboard must contain multiple scenes");
    assert!(sb.scenes.iter().any(|s| s.canvas_location[0] != 0.0 || s.canvas_location[1] != 0.0), "Must parse canvasLocation");
    let all_segues: Vec<_> = sb.scenes.iter().flat_map(|s| s.segues.iter()).collect();
    assert!(all_segues.iter().any(|sg| matches!(sg.kind, SegueKind::Sheet | SegueKind::Popover | SegueKind::Show | SegueKind::Modal)));
    let re_sb_xml = StoryboardFrontend::to_xml_str(&sb);
    let sb_roundtrip = StoryboardFrontend::from_xml_str(&re_sb_xml).expect("re-parse exported storyboard");
    assert_eq!(sb.scenes.len(), sb_roundtrip.scenes.len());

    let bundle_dir = std::env::temp_dir().join("egui_macview_test_bundle.nib");
    let _ = std::fs::create_dir_all(&bundle_dir);
    let vec_nib_bytes = std::fs::read(showcase_dir.join("1_Vector_and_Transform_Inspector.nib")).expect("read vec .nib");
    std::fs::write(bundle_dir.join("keyedobjects-101300.nib"), &vec_nib_bytes).expect("write keyedobjects-101300.nib");
    let bundle_scene = NibArchiveFrontend::from_path(&bundle_dir, &mapper).expect("decode .nib bundle directory");
    assert!(bundle_scene.nodes().len() > 20, "Bundle directory keyedobjects-101300.nib must decode losslessly");
    let _ = std::fs::remove_dir_all(&bundle_dir);
}
