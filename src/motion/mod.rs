//! macOS CoreAnimation Spring Physics Engine.
//!
//! Models a physical damped harmonic oscillator identical to Apple's
//! `CASpringAnimation` and SwiftUI `.spring(response:dampingRatio:)`.

#[derive(Debug, Clone, Copy)]
pub struct CASpring {
    pub response: f32,
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
    pub fn new(response: f32, damping_ratio: f32) -> Self {
        Self {
            response: response.max(0.001),
            damping_ratio: damping_ratio.max(0.0),
        }
    }

    /// Evaluates value $x(t) \in [0.0, 1.0]$ at time $t$ in seconds.
    pub fn evaluate(&self, t: f32) -> f32 {
        if t <= 0.0 {
            return 0.0;
        }

        let omega0 = 2.0 * std::f32::consts::PI / self.response;
        let zeta = self.damping_ratio;

        if zeta < 1.0 {
            // Underdamped
            let omega_d = omega0 * (1.0 - zeta * zeta).sqrt();
            let decay = (-zeta * omega0 * t).exp();
            let envelope = decay * ((omega_d * t).cos() + (zeta * omega0 / omega_d) * (omega_d * t).sin());
            1.0 - envelope
        } else if (zeta - 1.0).abs() < 1e-5 {
            // Critically damped
            let decay = (-omega0 * t).exp();
            1.0 - decay * (1.0 + omega0 * t)
        } else {
            // Overdamped
            let alpha = omega0 * (zeta * zeta - 1.0).sqrt();
            let decay = (-zeta * omega0 * t).exp();
            let term1 = (zeta * omega0 / alpha) * (alpha * t).sinh();
            let term2 = (alpha * t).cosh();
            1.0 - decay * (term1 + term2)
        }
    }
}
