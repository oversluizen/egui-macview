# SPEC-004: CoreAnimation Spring Physics Runtime (`CASpring`)

| Attribute | Specification Value |
| :--- | :--- |
| **Document ID** | SPEC-004 |
| **Document Title** | CoreAnimation Spring Physics Runtime (`CASpring`) |
| **Document Version** | `0.2.0` |
| **Status** | Accepted / Normative |
| **Target Host** | Standalone crate `egui-macview` (Rust, egui 0.37) |
| **Author** | Jaco Oversluizen / `egui-macview` contributors |
| **Normative Test Suite** | `tests/cots_red_baseline_tests.rs`, `tests/macbook_parity_differential_tests.rs`, `tests/high_assurance_invariants_tests.rs` |
| **Normative Standards** | ISO/IEC/IEEE 42010:2011, ISO/IEC/IEEE 29119-3:2021, RFC 2119/8174, Apple macOS HIG, W3C DTCG, SemVer 2.0.0 |

---

## Revision History

| Version | Date | Author | Description / Milestone |
| :--- | :--- | :--- | :--- |
| `0.1.0` | 2026-11-05 | Architecture Team | Baseline Initial Release: CASpring Closed-Form Oscillator Integration |
| `0.2.0` | 2026-11-06 | Architecture Team | Current Milestone: Complete Perceptual Parameterization & Physical Parameter Coverage |

---

## 1. Scope & Mathematical Foundations

In standard immediate-mode GUI frameworks, animation is often approximated using frame-iterative Euler integration (`pos += vel * dt`). On displays with divergent refresh rates ($60\,\text{Hz}$, $120\,\text{Hz}$ ProMotion, $144\,\text{Hz}$, $240\,\text{Hz}$), iterative integration suffers from numerical drift and velocity truncation.

`egui-macview` implements Apple's `CASpringAnimation` utilizing the exact closed-form analytical solution of the damped harmonic oscillator.

---

## 2. Damped Harmonic Oscillator Kinematics

### 2.1 Closed-Form Displacement Equation
For initial displacement $x_0 = 0.0$ transitioning to target $x_{\text{target}} = 1.0$:

$$x(t) = 1 - e^{-\zeta \omega_0 t} \left( \cos(\omega_d t) + \frac{\zeta \omega_0}{\omega_d} \sin(\omega_d t) \right)$$

Where:
- Dynamic response parameter (token `motion.spring.default.response`): $T_0 = 0.35\,\text{s} \implies \omega_0 = \frac{2\pi}{T_0} \approx 17.95\,\text{rad/s}$
- Damping ratio (token `motion.spring.default.damping_ratio`): $\zeta = 0.85$ (critically-underdamped, organic settling without sustained oscillation)
- Damped angular frequency: $\omega_d = \omega_0 \sqrt{1 - \zeta^2} \approx 9.46\,\text{rad/s}$

### 2.2 Analytical Velocity Equation
$$v(t) = \frac{dx}{dt} = \frac{\omega_0}{\sqrt{1 - \zeta^2}} e^{-\zeta \omega_0 t} \sin(\omega_d t)$$

Because both $x(t)$ and $v(t)$ are evaluated analytically at exact timestamp $t$, displacement is continuous and 100% invariant to frame delta fluctuations.

---

## 3. Parameterization Models

### 3.1 Perceptual Parameterization
Apple's SwiftUI/AppKit standard:
- `response` ($T_0 \in [0.05, 2.0]\,\text{s}$): Duration of one undamped oscillation cycle.
- `damping_ratio` ($\zeta \in [0.1, 1.5]$):
  - $\zeta < 1.0$: Underdamped (springy, subtle bounce).
  - $\zeta = 1.0$: Critically damped (fastest settling without overshoot).
  - $\zeta > 1.0$: Overdamped (sluggish decay).

### 3.2 Physical Parameterization
Traditional Newtonian mechanics:
- Mass ($m = 1.0\,\text{kg}$)
- Stiffness ($k = \omega_0^2 \cdot m$)
- Damping coefficient ($c = 2 \zeta \sqrt{k \cdot m}$)

### 3.3 Settling & Completion Oracle
A spring simulation is classified as settled (`is_settled(t) == true`) when:

$$|x(t) - x_{\text{target}}| < 0.001 \quad \text{AND} \quad |v(t)| < 0.001$$

Upon satisfying this oracle, the animation engine halts repainting, preventing unnecessary CPU/GPU wakeups.
