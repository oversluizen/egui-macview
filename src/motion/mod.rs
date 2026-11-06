//! macOS CoreAnimation Spring Physics Engine.
//!
//! Models a physical damped harmonic oscillator identical to Apple's
//! `CASpringAnimation` and SwiftUI `.spring(response:dampingRatio:)`.

use serde::{Deserialize, Serialize};

/// Authentic macOS CoreAnimation damped harmonic oscillator spring model.
///
/// Matches Apple's `CASpringAnimation` and SwiftUI `.spring(response:dampingRatio:)`.
/// Provides exact closed-form analytical solutions for underdamped ($\zeta < 1$),
/// critically damped ($\zeta = 1$), and overdamped ($\zeta > 1$) regimes without
/// numerical Euler drift or floating-point overflow.
///
/// # Example
///
/// ```rust
/// use egui_macview::CASpring;
///
/// // Create a responsive spring (0.35s period, 0.85 damping ratio)
/// let spring = CASpring::new(0.35, 0.85);
///
/// // Evaluate position x(t) in [0.0, 1.0]
/// let x_start = spring.evaluate(0.0);
/// let x_mid = spring.evaluate(0.15);
/// let x_settled = spring.evaluate(spring.settling_duration());
///
/// assert_eq!(x_start, 0.0);
/// assert!(x_mid > 0.0 && x_mid < 1.1);
/// assert!((x_settled - 1.0).abs() < 0.005);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CASpring {
    /// Perceptual oscillation period $T_0$ in seconds ($\ge 0.0001$).
    pub response: f32,
    /// Dimensionless damping ratio $\zeta$ ($\ge 0.0$).
    /// - $\zeta < 1.0$: Underdamped (oscillates with decaying overshoot).
    /// - $\zeta = 1.0$: Critically damped (fastest decay with zero overshoot).
    /// - $\zeta > 1.0$: Overdamped (smooth monotonic decay without oscillation).
    pub damping_ratio: f32,
}

impl Default for CASpring {
    fn default() -> Self {
        Self {
            response: 0.35,
            damping_ratio: 0.85,
        }
    }
}

impl CASpring {
    /// Creates a new `CASpring` from cycle period `response` (in seconds)
    /// and dimensionless `damping_ratio` ($\zeta$).
    pub fn new(response: f32, damping_ratio: f32) -> Self {
        Self {
            response: response.max(0.0001),
            damping_ratio: damping_ratio.max(0.0),
        }
    }

    /// Creates a `CASpring` using Apple SwiftUI / AppKit perceptual parameters
    /// `response` (in seconds) and `damping_ratio`.
    #[inline]
    pub fn from_perceptual(response: f32, damping_ratio: f32) -> Self {
        Self::new(response, damping_ratio)
    }

    /// Creates a `CASpring` from undamped angular frequency $\omega_0$ (in rad/s)
    /// and damping ratio $\zeta$.
    pub fn from_frequency(omega_0: f32, damping_ratio: f32) -> Self {
        let safe_omega = omega_0.max(0.0001);
        let response = 2.0 * std::f32::consts::PI / safe_omega;
        Self::new(response, damping_ratio)
    }

    /// Creates a `CASpring` from physical mass $m$, stiffness $k$, and damping $c$.
    pub fn from_physical(mass: f32, stiffness: f32, damping: f32) -> Self {
        let m = mass.max(0.0001);
        let k = stiffness.max(0.0001);
        let omega_0 = (k / m).sqrt();
        let damping_ratio = damping / (2.0 * (k * m).sqrt());
        Self::from_frequency(omega_0, damping_ratio)
    }

    /// Undamped natural angular frequency $\omega_0 = \frac{2\pi}{\text{response}}$ in rad/s.
    #[inline]
    pub fn omega_0(&self) -> f32 {
        2.0 * std::f32::consts::PI / self.response
    }

    /// Damped angular frequency $\omega_d = \omega_0 \sqrt{|1 - \zeta^2|}$ in rad/s.
    #[inline]
    pub fn omega_d(&self) -> f32 {
        let zeta = self.damping_ratio;
        self.omega_0() * (1.0 - zeta * zeta).abs().sqrt()
    }

    /// Analytical settling duration $t_{\text{settle}} = \frac{-\ln(\epsilon)}{\zeta \omega_0} = \frac{6.907755}{\zeta \omega_0}$
    /// for underdamped/critically damped oscillators ($\zeta \le 1$), and the dominant pole
    /// $\frac{6.907755}{\zeta \omega_0 - \omega_d}$ for overdamped systems ($\zeta > 1$),
    /// to within $\epsilon = 0.001$ (0.1%) of target resting state.
    pub fn settling_duration(&self) -> f32 {
        let zeta = self.damping_ratio;
        if zeta <= 0.0 {
            return f32::INFINITY;
        }
        let omega0 = self.omega_0();
        if zeta <= 1.0 {
            6.907755 / (zeta * omega0)
        } else {
            let alpha = self.omega_d();
            let rate = (zeta * omega0 - alpha).max(0.0001);
            6.907755 / rate
        }
    }

    /// Evaluates displacement $x(t) \in [0.0, 1.0]$ at time $t$ in seconds.
    #[inline]
    pub fn position_at(&self, t: f32) -> f32 {
        self.evaluate(t)
    }

    /// Theoretical peak overshoot ratio over resting state (0.0 if critically or overdamped).
    pub fn overshoot_ratio(&self) -> f32 {
        let zeta = self.damping_ratio;
        if (0.0..1.0).contains(&zeta) {
            (-std::f32::consts::PI * zeta / (1.0 - zeta * zeta).sqrt()).exp()
        } else {
            0.0
        }
    }

    /// Evaluates displacement $x(t) \in [0.0, 1.0]$ at time $t$ in seconds.
    pub fn evaluate(&self, t: f32) -> f32 {
        if t <= 0.0 {
            return 0.0;
        }

        let omega0 = self.omega_0();
        let zeta = self.damping_ratio;

        if zeta < 1.0 {
            // Underdamped
            let omega_d = self.omega_d();
            let decay = (-zeta * omega0 * t).exp();
            if decay == 0.0 {
                1.0
            } else {
                let envelope =
                    decay * ((omega_d * t).cos() + (zeta * omega0 / omega_d) * (omega_d * t).sin());
                1.0 - envelope
            }
        } else if (zeta - 1.0).abs() < 1e-5 {
            // Critically damped
            let decay = (-omega0 * t).exp();
            if decay == 0.0 {
                1.0
            } else {
                1.0 - decay * (1.0 + omega0 * t)
            }
        } else {
            // Overdamped: analytically rewritten using decaying exponentials
            // to unconditionally prevent sinh/cosh floating-point overflow to infinity.
            let alpha = self.omega_d();
            let rate_minus = (zeta * omega0 - alpha).max(0.0);
            let rate_plus = zeta * omega0 + alpha;
            let exp_minus = (-rate_minus * t).exp();
            let exp_plus = (-rate_plus * t).exp();
            let c = zeta * omega0 / alpha;
            let term = 0.5 * ((1.0 + c) * exp_minus + (1.0 - c) * exp_plus);
            1.0 - term
        }
    }

    /// Analytical velocity derivative $\dot{x}(t)$ at time $t$ in seconds.
    pub fn velocity(&self, t: f32) -> f32 {
        if t <= 0.0 {
            return 0.0;
        }

        let omega0 = self.omega_0();
        let zeta = self.damping_ratio;

        if zeta < 1.0 {
            // Underdamped
            let omega_d = self.omega_d();
            let decay = (-zeta * omega0 * t).exp();
            if decay == 0.0 {
                0.0
            } else {
                decay * (omega0 * omega0 / omega_d) * (omega_d * t).sin()
            }
        } else if (zeta - 1.0).abs() < 1e-5 {
            // Critically damped
            let decay = (-omega0 * t).exp();
            if decay == 0.0 {
                0.0
            } else {
                omega0 * omega0 * t * decay
            }
        } else {
            // Overdamped
            let alpha = self.omega_d();
            let rate_minus = (zeta * omega0 - alpha).max(0.0);
            let rate_plus = zeta * omega0 + alpha;
            let exp_minus = (-rate_minus * t).exp();
            let exp_plus = (-rate_plus * t).exp();
            (omega0 * omega0 / (2.0 * alpha)) * (exp_minus - exp_plus)
        }
    }
}
