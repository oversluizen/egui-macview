//! Canonical COTS Test Harness for `egui-macview` (`REQ-001-06.3`).
//!
//! Provides deterministic headless `egui::Context` initialization, multi-frame pointer click/drag
//! simulation, `MacSceneIR` widget rect lookup by `ib_object_id`, and shape inspection helpers.

#![allow(dead_code)]

use egui::{epaint::TextShape, Color32, Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Shape, Vec2};
use egui_macview::{
    scene::{LayoutPolicy, MacSceneIR},
    setup_symbols_font, MacAppearance, MacTheme,
};

/// Creates a deterministic headless [`egui::Context`] with open-source fonts and [`MacTheme`] installed.
pub fn headless_ctx(appearance: MacAppearance) -> (Context, MacTheme) {
    let ctx = Context::default();
    let mut fonts = egui::FontDefinitions::default();
    setup_symbols_font(&mut fonts);
    ctx.set_fonts(fonts);
    let theme = MacTheme::new(appearance);
    theme.apply_to_egui(&ctx);
    (ctx, theme)
}

/// Runs a single headless UI frame at `viewport` size and returns the clipped shapes.
pub fn run_frame(ctx: &Context, mut input: RawInput, viewport: Vec2, mut f: impl FnMut(&mut egui::Ui)) -> Vec<egui::epaint::ClippedShape> {
    input.screen_rect = Some(Rect::from_min_size(Pos2::ZERO, viewport));
    let mut out = ctx.run_ui(input, |ui| {
        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, &mut f);
    });
    out.textures_delta.clear();
    out.shapes
}

/// Executes a genuine 3-frame `egui` primary click (`Hover -> Press -> Release`) at `pos`.
pub fn click_at_multi_frame(
    ctx: &Context,
    pos: Pos2,
    viewport: Vec2,
    mut f: impl FnMut(&mut egui::Ui),
) -> Vec<egui::epaint::ClippedShape> {
    let hover = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, viewport)),
        events: vec![Event::PointerMoved(pos)],
        ..Default::default()
    };
    let _ = run_frame(ctx, hover, viewport, &mut f);

    let press = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, viewport)),
        events: vec![
            Event::PointerMoved(pos),
            Event::PointerButton { pos, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE },
        ],
        ..Default::default()
    };
    let _ = run_frame(ctx, press, viewport, &mut f);

    let release = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, viewport)),
        events: vec![
            Event::PointerMoved(pos),
            Event::PointerButton { pos, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE },
        ],
        ..Default::default()
    };
    run_frame(ctx, release, viewport, &mut f)
}

/// Resolves the solved `egui::Rect` of a node in `scene` by its `ib_object_id` (`REQ-001-06.3`).
pub fn resolved_rect_by_id(scene: &MacSceneIR, ib_object_id: &str, viewport: Vec2, policy: LayoutPolicy) -> Option<Rect> {
    let node = scene.nodes().iter().find(|n| n.ib_object_id == ib_object_id)?;
    let solved = scene.solve_layout(viewport, policy, 0, "");
    solved.get(&node.id).copied()
}

/// Recursively collects all [`TextShape`] primitives from a [`Shape`] tree.
pub fn collect_text_shapes<'a>(shape: &'a Shape, out: &mut Vec<&'a TextShape>) {
    match shape {
        Shape::Text(t) => out.push(t),
        Shape::Vec(v) => {
            for s in v {
                collect_text_shapes(s, out);
            }
        }
        _ => {}
    }
}

/// Finds the stroke color of the first `Shape::LineSegment` whose midpoint lies inside `target_rect`.
pub fn first_line_stroke_color_in_rect(shapes: &[egui::epaint::ClippedShape], target_rect: Rect) -> Option<Color32> {
    fn search_shape(shape: &Shape, target_rect: Rect) -> Option<Color32> {
        match shape {
            Shape::LineSegment { points, stroke } => {
                let mid = Pos2::new((points[0].x + points[1].x) * 0.5, (points[0].y + points[1].y) * 0.5);
                target_rect.expand(2.0).contains(mid).then_some(stroke.color)
            }
            Shape::Vec(list) => list.iter().find_map(|s| search_shape(s, target_rect)),
            _ => None,
        }
    }
    shapes.iter().find_map(|cs| search_shape(&cs.shape, target_rect))
}
