//! ISO/IEC/IEEE 29119-4:2021 High-Assurance Property-Based Invariant Test Suite.
//!
//! Normative references:
//! - SPEC-001 Section 1, 2.1, 4.1 & High-Assurance Property-Based Verification.
//! - ISO/IEC/IEEE 29119-4: Combinatorial, Invariant, and Generative Domain Verification.
//! - DO-178C Level A/B: Mathematical Determinism and Absence of Numerical Anomalies.
//!
//! Verifies 4 core systemic invariants across continuous parameter spaces:
//! 1. `prop_caspring_numerical_stability`: Total absence of NaN/Inf across all times and damping ratios.
//! 2. `prop_subpixel_snap_hardware_exactness`: Machine-exact hardware pixel boundary alignment.
//! 3. `prop_tab_chain_fsm_cyclic_preservation`: Strict permutation closure under Tab and Shift+Tab.
//! 4. `prop_transactional_buffer_rollback_integrity`: Transactional isolation and 100% rollback guarantee.

use egui_macview::layout::subpixel_snap;
use egui_macview::motion::CASpring;
use egui_macview::{
    MacAppearance, MacLevelIndicator, MacLevelIndicatorStyle, MacLevelOrientation,
    MacProgressIndicator, MacTheme, MacTimelineZone, MacTrimScrubber,
};
use proptest::prelude::*;

proptest! {
    /// Invariant 1 (ISO 29119-4 / DO-178C Level A):
    /// Physical CoreAnimation Spring Physics MUST remain unconditionally numerically stable.
    /// It MUST NEVER evaluate to NaN, +Infinity, or -Infinity for any positive response, damping, or time.
    #[test]
    fn prop_caspring_numerical_stability(
        response in 0.02f32..5.0f32,
        damping_ratio in 0.05f32..5.0f32,
        t in 0.0f32..100.0f32,
    ) {
        let spring = CASpring::new(response, damping_ratio);
        let x = spring.evaluate(t);
        let v = spring.velocity(t);

        prop_assert!(!x.is_nan(), "Position x(t) produced NaN at t={}, response={}, zeta={}", t, response, damping_ratio);
        prop_assert!(!x.is_infinite(), "Position x(t) produced Infinite at t={}, response={}, zeta={}", t, response, damping_ratio);
        prop_assert!(!v.is_nan(), "Velocity v(t) produced NaN at t={}, response={}, zeta={}", t, response, damping_ratio);
        prop_assert!(!v.is_infinite(), "Velocity v(t) produced Infinite at t={}, response={}, zeta={}", t, response, damping_ratio);

        // Physical boundary: at t=0, initial displacement is exactly 0.0
        if t == 0.0 {
            prop_assert!((x - 0.0).abs() < 1e-6);
        }

        // Asymptotic convergence: after 3 settling durations, must be within 5% of target 1.0
        let t_settle = spring.settling_duration();
        if damping_ratio >= 0.2 && t > 3.0 * t_settle {
            prop_assert!(
                (x - 1.0).abs() < 0.05,
                "Position did not converge after 3*t_settle: x({}) = {}, target = 1.0",
                t, x
            );
        }
    }

    /// Invariant 2 (macOS HIG / Windows Display Scaling Parity):
    /// Subpixel-snapped logical points MUST align with physical hardware pixel grids
    /// on Windows display scale factors (100% to 300%) within f32 machine precision.
    #[test]
    fn prop_subpixel_snap_hardware_exactness(
        x in -2500.0f32..2500.0f32,
        scale in prop_oneof![
            Just(1.0f32),
            Just(1.25f32),
            Just(1.50f32),
            Just(1.75f32),
            Just(2.00f32),
            Just(2.25f32),
            Just(2.50f32),
            Just(3.00f32),
            1.0f32..3.0f32,
        ],
    ) {
        let snapped_logical = subpixel_snap(x, scale);
        let physical_pixels = snapped_logical * scale;
        let nearest_integer = physical_pixels.round();
        let residual_error = (physical_pixels - nearest_integer).abs();

        // Machine precision bound for f32 at magnitude 2500 * 3 = 7500:
        // 7500 * 2^-24 ≈ 0.00045, hence tolerance is 5e-4.
        prop_assert!(
            residual_error < 5e-4,
            "Snapped coordinate {} at scale {} resulted in fractional physical pixel {} (error = {})",
            snapped_logical, scale, physical_pixels, residual_error
        );
    }

    /// Invariant 3 (FSM Verification / ISO 29119-4 State Transition Testing):
    /// In an N-element `MacInputCell` container, the Tab-focus chain forms a closed cyclic group (Z_N).
    /// Pressing `Key::Tab` advances focus by exactly 1 field per frame (zero domino cascade),
    /// and `Shift+Tab` produces the exact inverse permutation.
    #[test]
    fn prop_tab_chain_fsm_cyclic_preservation(
        field_count in 2usize..7usize,
        start_index in 0usize..6usize,
        steps in 1usize..10usize,
    ) {
        let start = start_index % field_count;
        let ctx = egui::Context::default();
        let theme = egui_macview::MacTheme::default();
        let ids: Vec<egui::Id> = (0..field_count)
            .map(|i| egui::Id::NULL.with(("tab_cycle_cell", i)))
            .collect();
        let mut values: Vec<String> = (0..field_count).map(|i| format!("v{i}")).collect();

        let render_chain = |ctx: &egui::Context, input: egui::RawInput, values: &mut [String]| {
            let mut out = ctx.run_ui(input, |ui| {
                for i in 0..field_count {
                    let next_id = ids[(i + 1) % field_count];
                    let prev_id = ids[(i + field_count - 1) % field_count];
                    egui_macview::MacInputCell::with_id(ids[i], &mut values[i])
                        .theme(&theme)
                        .next_focus(next_id)
                        .prev_focus(prev_id)
                        .show(ui);
                }
            });
            out.textures_delta.clear();
        };

        // Frame 0: initialize widgets and focus `ids[start]`
        render_chain(&ctx, egui::RawInput::default(), &mut values);
        ctx.memory_mut(|m| m.request_focus(ids[start]));
        render_chain(&ctx, egui::RawInput::default(), &mut values);
        prop_assert_eq!(ctx.memory(|m| m.focused()), Some(ids[start]));

        // Advance `field_count` steps forward via Key::Tab; each frame must advance by EXACTLY 1 field
        for step in 1..=field_count {
            let mut raw = egui::RawInput::default();
            raw.events.push(egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            });
            render_chain(&ctx, raw, &mut values);
            let expected = ids[(start + step) % field_count];
            prop_assert_eq!(
                ctx.memory(|m| m.focused()),
                Some(expected),
                "Single Tab press in frame {} must advance focus by exactly 1 field without domino cascade",
                step
            );
        }

        // Move `steps` forward then `steps` backward via Shift+Tab
        for _ in 0..steps {
            let mut raw = egui::RawInput::default();
            raw.events.push(egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            });
            render_chain(&ctx, raw, &mut values);
        }
        for _ in 0..steps {
            let mut raw = egui::RawInput::default();
            raw.events.push(egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::SHIFT,
            });
            render_chain(&ctx, raw, &mut values);
        }
        prop_assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(ids[start]),
            "S forward Tab presses followed by S backward Shift+Tab presses must restore initial focus"
        );
    }

    /// Invariant 4 (Transactional Memory / Undo Buffer Isolation on Real `MacInputCell`):
    /// - Focusing `MacInputCell` snapshots the initial string, types edits, and pressing `Key::Escape`
    ///   rolls back the caller's `&mut String` to the exact pre-edit value and surrenders focus.
    /// - Typing edits and pressing `Key::Enter` commits the edited text to the caller's `&mut String`.
    #[test]
    fn prop_transactional_buffer_rollback_integrity(
        canonical_initial in "[a-zA-Z0-9_]{1,12}",
        appended_edit in "[a-zA-Z0-9_]{1,8}",
    ) {
        let ctx = egui::Context::default();
        let theme = egui_macview::MacTheme::default();
        let cell_id = egui::Id::NULL.with("transactional_prop_cell");
        let mut value = canonical_initial.clone();

        let run_frame = |ctx: &egui::Context, input: egui::RawInput, val: &mut String| {
            let mut out = ctx.run_ui(input, |ui| {
                egui_macview::MacInputCell::with_id(cell_id, val)
                    .theme(&theme)
                    .show(ui);
            });
            out.textures_delta.clear();
        };

        // 1. Focus cell and type `appended_edit`, then press Escape -> must roll back to `canonical_initial`
        run_frame(&ctx, egui::RawInput::default(), &mut value);
        ctx.memory_mut(|m| m.request_focus(cell_id));
        run_frame(&ctx, egui::RawInput::default(), &mut value);

        let mut type_raw = egui::RawInput::default();
        type_raw.events.push(egui::Event::Text(appended_edit.clone()));
        run_frame(&ctx, type_raw, &mut value);

        let mut esc_raw = egui::RawInput::default();
        esc_raw.events.push(egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        });
        run_frame(&ctx, esc_raw, &mut value);

        prop_assert_eq!(
            &value,
            &canonical_initial,
            "Escape must roll back MacInputCell value to the initial pre-edit snapshot"
        );
        prop_assert!(
            !ctx.memory(|m| m.has_focus(cell_id)),
            "Escape must surrender focus from MacInputCell"
        );

        // 2. Focus cell again, type `appended_edit`, then press Enter -> must commit edit
        ctx.memory_mut(|m| m.request_focus(cell_id));
        run_frame(&ctx, egui::RawInput::default(), &mut value);

        let mut type_raw2 = egui::RawInput::default();
        type_raw2.events.push(egui::Event::Text(appended_edit.clone()));
        run_frame(&ctx, type_raw2, &mut value);

        let mut enter_raw = egui::RawInput::default();
        enter_raw.events.push(egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        });
        run_frame(&ctx, enter_raw, &mut value);

        prop_assert!(
            value.contains(&appended_edit) && value != canonical_initial,
            "Enter must commit the edited text into caller's String (got {:?})",
            value
        );
    }

    /// Invariant 5 (SPEC-011 Studio Controls Clamping & Range Monotonicity):
    /// `MacProgressIndicator` clamps progress to `[0.0, 1.0]` and `MacTrimScrubber` preserves
    /// `0.0 <= in_point <= playhead <= out_point <= duration`.
    #[test]
    fn prop_studio_scrubber_and_progress_invariants(
        raw_progress in -5.0f32..5.0f32,
        duration in 10.0f64..600.0f64,
        in_frac in 0.0f64..1.0f64,
        out_frac in 0.0f64..1.0f64,
        play_frac in 0.0f64..1.0f64,
    ) {
        let (ctx, theme) = (egui::Context::default(), MacTheme::default());
        let (mut in_pt, mut out_pt, mut playhead) = (in_frac * duration, out_frac * duration, play_frac * duration);
        let (samples, zones) = ([0.2f32, 0.7, 0.95, 0.4, 0.6], [MacTimelineZone::new(0.0, duration * 0.5, "Intro")]);
        let mut clamped_p = None;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            let pi = MacProgressIndicator::bar(raw_progress).theme(&theme);
            clamped_p = pi.clamped_progress();
            pi.show(ui);
            MacTrimScrubber::new("prop_scrubber", duration, &mut in_pt, &mut out_pt, &mut playhead)
                .theme(&theme)
                .waveform(&samples)
                .zones(&zones)
                .show(ui);
        });
        out.textures_delta.clear();
        prop_assert!((0.0..=1.0).contains(&clamped_p.unwrap()) && in_pt >= 0.0 && in_pt <= out_pt && out_pt <= duration && playhead >= in_pt && playhead <= out_pt);
    }
}

/// TC-011-04: Verifies `MacLevelIndicator` uniform `.discreteCapacity` color, tick marks, and orientations.
#[test]
fn test_level_indicator_thresholds_and_orientations() {
    let (ctx, theme) = (egui::Context::default(), MacTheme::new(MacAppearance::Aqua));
    let meter = MacLevelIndicator::new(-14.0, -60.0..=0.0)
        .theme(&theme)
        .warning_value(-18.0)
        .critical_value(-6.0)
        .peak_hold(-3.0);
    assert_eq!(meter.color_for_value(&theme, -30.0), theme.system_green());
    assert_eq!(meter.color_for_value(&theme, -12.0), theme.system_yellow());
    assert_eq!(meter.color_for_value(&theme, -2.0), theme.system_red());

    let (mut horiz_rect, mut vert_rect) = (egui::Rect::NOTHING, egui::Rect::NOTHING);
    let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
        horiz_rect = MacLevelIndicator::new(-2.0, -60.0..=0.0)
            .theme(&theme)
            .warning_value(-18.0)
            .critical_value(-6.0)
            .style(MacLevelIndicatorStyle::Discrete)
            .tick_marks(5)
            .orientation(MacLevelOrientation::Horizontal)
            .length(160.0)
            .thickness(16.0)
            .show(ui)
            .rect;
        vert_rect = MacLevelIndicator::new(-14.0, -60.0..=0.0)
            .theme(&theme)
            .style(MacLevelIndicatorStyle::Continuous)
            .orientation(MacLevelOrientation::Vertical)
            .length(120.0)
            .thickness(16.0)
            .show(ui)
            .rect;
    });
    out.textures_delta.clear();
    let non_red = out
        .shapes
        .iter()
        .filter(|cs| match &cs.shape {
            egui::Shape::Rect(r) if horiz_rect.contains(r.rect.center()) => {
                r.fill == theme.system_green() || r.fill == theme.system_yellow()
            }
            _ => false,
        })
        .count();
    let ticks = out
        .shapes
        .iter()
        .filter(|cs| matches!(&cs.shape, egui::Shape::LineSegment { .. }))
        .count();
    assert!(
        non_red == 0
            && ticks >= 5
            && horiz_rect.size() == egui::Vec2::new(160.0, 16.0)
            && vert_rect.size() == egui::Vec2::new(16.0, 120.0)
    );
}
