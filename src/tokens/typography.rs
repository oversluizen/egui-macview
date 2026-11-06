//! SF Pro dynamic typography and optical tracking tokens.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Single entry in the SF Pro tracking table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackingTableEntry {
    /// Font size in points.
    pub size_pt: u32,
    /// Subfamily / weight name (e.g. Regular, Medium, Bold).
    pub subfamily: String,
    /// Tracking value in 1/1000 em.
    pub tracking_em_1000: i32,
    /// Tracking value in points.
    pub tracking_pt: f32,
    /// Optional description.
    #[serde(rename = "$description", default)]
    pub description: Option<String>,
}

/// Coefficients for a cubic polynomial regression $a \cdot x^3 + b \cdot x^2 + c \cdot x + d$.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CubicCoefficients {
    /// Cubic coefficient ($x^3$).
    pub a: f64,
    /// Quadratic coefficient ($x^2$).
    pub b: f64,
    /// Linear coefficient ($x$).
    pub c: f64,
    /// Constant offset ($1$).
    pub d: f64,
}

/// Cubic polynomial regression fit specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CubicFit {
    /// Formula string representation.
    pub formula: String,
    /// Polynomial coefficients.
    pub coefficients: CubicCoefficients,
}

/// Coefficients for a quadratic polynomial regression $a_2 \cdot x^2 + a_1 \cdot x + a_0$.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuadraticCoefficients {
    /// Quadratic coefficient ($x^2$).
    pub a2: f64,
    /// Linear coefficient ($x$).
    pub a1: f64,
    /// Constant offset ($1$).
    pub a0: f64,
}

/// Quadratic polynomial regression fit specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuadraticFit {
    /// Formula string representation.
    pub formula: String,
    /// Polynomial coefficients.
    pub coefficients: QuadraticCoefficients,
}

/// Discrete tracking measurement point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackingPoint {
    /// Font size in points.
    pub pt: f64,
    /// Measured optical tracking in points.
    pub tracking_pt: f64,
}

/// Continuous polynomial tracking model defined in SPEC-001.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolynomialSpec001 {
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
    /// Quadratic curve fit for inspector display.
    pub quadratic_inspector_fit: QuadraticFit,
    /// Canonical Apple HIG cubic regression curve fit.
    pub cubic_apple_hig_fit: CubicFit,
    /// Discrete verification data points.
    pub points: HashMap<String, TrackingPoint>,
}

/// Typography and tracking tokens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacTypographyTokens {
    /// Discrete SF Pro tracking lookup table.
    pub tracking_table: HashMap<String, TrackingTableEntry>,
    /// Continuous polynomial regression models.
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
