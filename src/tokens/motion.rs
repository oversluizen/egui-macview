//! CoreAnimation and SwiftUI spring physics motion preset tokens.

use serde::{Deserialize, Serialize};

/// CoreAnimation and SwiftUI spring motion preset parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MotionPresetToken {
    pub mass: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub omega_0: f32,
    #[serde(default)]
    pub omega_d: Option<f32>,
    pub damping_ratio: f32,
    pub response: f32,
    #[serde(default)]
    pub settling_duration: Option<f32>,
    #[serde(rename = "$description", default)]
    pub description: Option<String>,
}

impl MotionPresetToken {
    pub fn to_spring(&self) -> crate::motion::CASpring {
        crate::motion::CASpring::new(self.response, self.damping_ratio)
    }
}

/// Motion token collection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacMotionTokens {
    pub caspring_default: MotionPresetToken,
    pub swiftui_spring_default: MotionPresetToken,
    pub swiftui_snappy: MotionPresetToken,
    pub swiftui_bouncy: MotionPresetToken,
    pub swiftui_smooth: MotionPresetToken,
    pub hud_panel_transition: MotionPresetToken,
}
