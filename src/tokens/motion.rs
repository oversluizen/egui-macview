//! CoreAnimation and SwiftUI spring physics motion preset tokens.

use serde::{Deserialize, Serialize};

/// CoreAnimation and SwiftUI spring motion preset parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MotionPresetToken {
    /// Physical mass ($m$).
    pub mass: f32,
    /// Physical stiffness ($k$).
    pub stiffness: f32,
    /// Physical damping coefficient ($c$).
    pub damping: f32,
    /// Undamped angular frequency ($\omega_0$) in rad/s.
    pub omega_0: f32,
    /// Damped angular frequency ($\omega_d$) in rad/s.
    #[serde(default)]
    pub omega_d: Option<f32>,
    /// Damping ratio ($\zeta$).
    pub damping_ratio: f32,
    /// Perceptual response duration ($T_0$) in seconds.
    pub response: f32,
    /// Analytical settling duration to within 0.1% of rest state.
    #[serde(default)]
    pub settling_duration: Option<f32>,
    /// Optional human-readable description.
    #[serde(rename = "$description", default)]
    pub description: Option<String>,
}

impl MotionPresetToken {
    /// Converts this motion preset token into a [`CASpring`](crate::motion::CASpring) instance.
    pub fn to_spring(&self) -> crate::motion::CASpring {
        crate::motion::CASpring::new(self.response, self.damping_ratio)
    }

    /// Constructs a [`CASpring`](crate::motion::CASpring) from physical parameters if mass, stiffness, and damping are positive.
    pub fn to_spring_physical(&self) -> Option<crate::motion::CASpring> {
        if self.mass > 0.0 && self.stiffness > 0.0 && self.damping > 0.0 {
            Some(crate::motion::CASpring::from_physical(
                self.mass,
                self.stiffness,
                self.damping,
            ))
        } else {
            None
        }
    }
}

/// Motion token collection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacMotionTokens {
    /// Default CoreAnimation spring preset (response 0.35s, damping 0.85).
    pub caspring_default: MotionPresetToken,
    /// Default SwiftUI spring preset.
    pub swiftui_spring_default: MotionPresetToken,
    /// SwiftUI snappy preset (faster settling with minimal bounce).
    pub swiftui_snappy: MotionPresetToken,
    /// SwiftUI bouncy preset (audible overshoot and oscillating spring).
    pub swiftui_bouncy: MotionPresetToken,
    /// SwiftUI smooth preset (critically damped with zero overshoot).
    pub swiftui_smooth: MotionPresetToken,
    /// Floating HUD panel open/close transition preset.
    pub hud_panel_transition: MotionPresetToken,
}
