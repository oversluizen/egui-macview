//! SF Pro dynamic typography and optical tracking tokens.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Single entry in the SF Pro tracking table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackingTableEntry {
    pub size_pt: u32,
    pub subfamily: String,
    pub tracking_em_1000: i32,
    pub tracking_pt: f32,
    #[serde(rename = "$description", default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CubicCoefficients {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CubicFit {
    pub formula: String,
    pub coefficients: CubicCoefficients,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuadraticCoefficients {
    pub a2: f64,
    pub a1: f64,
    pub a0: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuadraticFit {
    pub formula: String,
    pub coefficients: QuadraticCoefficients,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackingPoint {
    pub pt: f64,
    pub tracking_pt: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolynomialSpec001 {
    #[serde(default)]
    pub description: Option<String>,
    pub quadratic_inspector_fit: QuadraticFit,
    pub cubic_apple_hig_fit: CubicFit,
    pub points: HashMap<String, TrackingPoint>,
}

/// Typography and tracking tokens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacTypographyTokens {
    pub tracking_table: HashMap<String, TrackingTableEntry>,
    pub polynomial_spec001: PolynomialSpec001,
}

impl MacTypographyTokens {
    /// Evaluates continuous optical tracking in pt using Apple HIG cubic regression.
    pub fn tracking_for_size(&self, pt: f64) -> f64 {
        let c = &self.polynomial_spec001.cubic_apple_hig_fit.coefficients;
        c.a * pt.powi(3) + c.b * pt.powi(2) + c.c * pt + c.d
    }

    /// Evaluates quadratic inspector fit.
    pub fn tracking_quadratic(&self, pt: f64) -> f64 {
        let c = &self.polynomial_spec001.quadratic_inspector_fit.coefficients;
        c.a2 * pt.powi(2) + c.a1 * pt + c.a0
    }
}

/// Continuous SF Pro tracking polynomial using canonical Apple HIG fit:
/// $\text{Tracking}(p) = a \cdot p^3 + b \cdot p^2 + c \cdot p + d$
#[inline]
pub fn sf_pro_tracking(size_pt: f32) -> f32 {
    let p = size_pt as f64;
    let a = 0.000864840354;
    let b = -0.0339060217;
    let c = 0.355814827;
    let d = -0.891241741;
    (a * p.powi(3) + b * p.powi(2) + c * p + d) as f32
}
