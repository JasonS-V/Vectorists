use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Maximum harmonic order used for the continuous polar Fourier series.
pub const FOURIER_ORDER: usize = 4;
pub const TWO_PI: f32 = 2.0 * PI;

/// A 2D motion vector decomposed into polar representation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolarVector {
    /// Radial magnitude (distance in counts).
    pub r: f32,
    /// Angle in radians in the range [0, 2π).
    pub theta: f32,
    /// Original x displacement.
    pub dx: f32,
    /// Original y displacement.
    pub dy: f32,
}

impl PolarVector {
    /// Constructs a polar vector from Cartesian counts (dx, dy).
    #[inline]
    pub fn from_cartesian(dx: f32, dy: f32) -> Self {
        let r = (dx * dx + dy * dy).sqrt();
        let mut theta = dy.atan2(dx);
        if theta < 0.0 {
            theta += TWO_PI;
        }
        Self { r, theta, dx, dy }
    }

    /// Angle in degrees in the range [0.0, 360.0).
    #[inline]
    pub fn angle_degrees(&self) -> f32 {
        self.theta.to_degrees()
    }

    /// Reconstructs Cartesian (dx, dy) applying a scaling factor S.
    #[inline]
    pub fn to_cartesian_scaled(&self, scale: f32) -> (f32, f32) {
        if self.r <= f32::EPSILON {
            return (0.0, 0.0);
        }
        let scaled_r = self.r * scale;
        (scaled_r * self.theta.cos(), scaled_r * self.theta.sin())
    }
}

/// Helper wrapper for angle manipulations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Angle(pub f32);

impl Angle {
    #[inline]
    pub fn from_radians(rad: f32) -> Self {
        let mut normalized = rad % TWO_PI;
        if normalized < 0.0 {
            normalized += TWO_PI;
        }
        Self(normalized)
    }

    #[inline]
    pub fn from_degrees(deg: f32) -> Self {
        Self::from_radians(deg.to_radians())
    }

    #[inline]
    pub fn radians(&self) -> f32 {
        self.0
    }

    #[inline]
    pub fn degrees(&self) -> f32 {
        self.0.to_degrees()
    }
}

/// Periodic Fourier Series truncated to N harmonics:
/// S(θ) = a0 + sum_{n=1}^N (an * cos(nθ) + bn * sin(nθ))
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FourierModel {
    /// DC offset / base scale factor (nominally ~1.0).
    pub a0: f32,
    /// Cosine harmonic coefficients [a1, a2, a3, a4].
    pub a: [f32; FOURIER_ORDER],
    /// Sine harmonic coefficients [b1, b2, b3, b4].
    pub b: [f32; FOURIER_ORDER],
}

impl Default for FourierModel {
    fn default() -> Self {
        Self {
            a0: 1.0,
            a: [0.0; FOURIER_ORDER],
            b: [0.0; FOURIER_ORDER],
        }
    }
}

impl FourierModel {
    /// Creates a neutral model that applies a 1:1 scale (no directional bias).
    pub fn identity() -> Self {
        Self::default()
    }

    /// Evaluates the sensitivity scaling factor S(θ) at the given angle in radians.
    #[inline]
    pub fn evaluate(&self, theta: f32) -> f32 {
        let mut result = self.a0;
        for n in 1..=FOURIER_ORDER {
            let n_theta = (n as f32) * theta;
            result += self.a[n - 1] * n_theta.cos() + self.b[n - 1] * n_theta.sin();
        }
        // Protect against negative or absurd scale multipliers
        result.clamp(0.1, 4.0)
    }

    /// Evaluates the final scaling factor attenuated by user strength α ∈ [0.0, 2.0]:
    /// S_final(θ) = 1.0 + α * (S(θ) - 1.0)
    #[inline]
    pub fn evaluate_with_strength(&self, theta: f32, alpha: f32) -> f32 {
        let raw_scale = self.evaluate(theta);
        (1.0 + alpha * (raw_scale - 1.0)).clamp(0.1, 4.0)
    }

    /// Transforms an incoming (dx, dy) mouse report by the polar correction function.
    #[inline]
    pub fn correct_delta(&self, dx: f32, dy: f32, alpha: f32) -> (f32, f32) {
        if dx.abs() <= f32::EPSILON && dy.abs() <= f32::EPSILON {
            return (0.0, 0.0);
        }
        let polar = PolarVector::from_cartesian(dx, dy);
        let scale = self.evaluate_with_strength(polar.theta, alpha);
        polar.to_cartesian_scaled(scale)
    }

    /// Fits Fourier series to K equidistant angular samples:
    /// θ_k = k * 2π / K, for k in 0..K.
    ///
    /// Uses exact Discrete Fourier decomposition thanks to orthogonality of basis functions.
    pub fn fit_equidistant(samples: &[f32]) -> Self {
        let k_count = samples.len();
        if k_count == 0 {
            return Self::identity();
        }

        let k_f32 = k_count as f32;
        let a0 = samples.iter().sum::<f32>() / k_f32;

        let mut a = [0.0; FOURIER_ORDER];
        let mut b = [0.0; FOURIER_ORDER];

        for n in 1..=FOURIER_ORDER {
            let mut sum_cos = 0.0f32;
            let mut sum_sin = 0.0f32;

            for (k, &multiplier) in samples.iter().enumerate() {
                let theta_k = (k as f32) * TWO_PI / k_f32;
                let n_theta = (n as f32) * theta_k;
                sum_cos += multiplier * n_theta.cos();
                sum_sin += multiplier * n_theta.sin();
            }

            a[n - 1] = (2.0 / k_f32) * sum_cos;
            b[n - 1] = (2.0 / k_f32) * sum_sin;
        }

        Self { a0, a, b }
    }

    /// Fits Fourier series to arbitrary (theta, multiplier) pairs using regularized
    /// linear least squares (normal equations with Tikhonov regularization).
    pub fn fit_arbitrary(points: &[(f32, f32)]) -> Self {
        if points.is_empty() {
            return Self::identity();
        }

        const M: usize = 1 + 2 * FOURIER_ORDER; // 9 parameters: [a0, a1, b1, a2, b2, a3, b3, a4, b4]
        let mut ata = [[0.0f32; M]; M];
        let mut aty = [0.0f32; M];

        for &(theta, y) in points {
            let mut row = [0.0f32; M];
            row[0] = 1.0;
            for n in 1..=FOURIER_ORDER {
                let n_theta = (n as f32) * theta;
                row[1 + 2 * (n - 1)] = n_theta.cos();
                row[2 + 2 * (n - 1)] = n_theta.sin();
            }

            for i in 0..M {
                aty[i] += row[i] * y;
                for j in 0..M {
                    ata[i][j] += row[i] * row[j];
                }
            }
        }

        // Tikhonov regularization: (A^T A + λ I) prevents singularity
        let lambda = 1e-4f32;
        for i in 0..M {
            ata[i][i] += lambda;
        }

        // Solve via Gaussian elimination with partial pivoting
        if let Some(params) = solve_linear_system::<M>(ata, aty) {
            let mut a = [0.0; FOURIER_ORDER];
            let mut b = [0.0; FOURIER_ORDER];
            for n in 1..=FOURIER_ORDER {
                a[n - 1] = params[1 + 2 * (n - 1)];
                b[n - 1] = params[2 + 2 * (n - 1)];
            }
            Self {
                a0: params[0],
                a,
                b,
            }
        } else {
            Self::identity()
        }
    }

    /// Generates discrete multipliers for K equidistant angular directions around the 360° circle.
    pub fn sample_directions(&self, k_count: usize, alpha: f32) -> Vec<(f32, f32)> {
        let mut samples = Vec::with_capacity(k_count);
        for k in 0..k_count {
            let theta = (k as f32) * TWO_PI / (k_count as f32);
            let multiplier = self.evaluate_with_strength(theta, alpha);
            samples.push((theta, multiplier));
        }
        samples
    }
}

/// Solves Ax = b for an M x M linear system using Gauss-Jordan elimination with partial pivoting.
fn solve_linear_system<const M: usize>(mut a: [[f32; M]; M], mut b: [f32; M]) -> Option<[f32; M]> {
    for col in 0..M {
        // Find pivot
        let mut max_row = col;
        let mut max_val = a[col][col].abs();
        for row in (col + 1)..M {
            let val = a[row][col].abs();
            if val > max_val {
                max_val = val;
                max_row = row;
            }
        }

        if max_val < 1e-7 {
            return None; // Singular matrix
        }

        // Swap rows
        if max_row != col {
            a.swap(col, max_row);
            b.swap(col, max_row);
        }

        // Normalize pivot row
        let pivot = a[col][col];
        for j in col..M {
            a[col][j] /= pivot;
        }
        b[col] /= pivot;

        // Eliminate column entries in other rows
        for row in 0..M {
            if row != col {
                let factor = a[row][col];
                if factor.abs() > 1e-8 {
                    for j in col..M {
                        a[row][j] -= factor * a[col][j];
                    }
                    b[row] -= factor * b[col];
                }
            }
        }
    }

    Some(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polar_vector_zero() {
        let pv = PolarVector::from_cartesian(0.0, 0.0);
        assert_eq!(pv.r, 0.0);
        let (x, y) = pv.to_cartesian_scaled(1.5);
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn test_polar_vector_quadrants() {
        // Quadrant 1 (Right): dx=10, dy=0 => theta=0
        let p1 = PolarVector::from_cartesian(10.0, 0.0);
        assert!((p1.angle_degrees() - 0.0).abs() < 1e-4);

        // Quadrant 1 (Up): dx=0, dy=10 => theta=90°
        let p2 = PolarVector::from_cartesian(0.0, 10.0);
        assert!((p2.angle_degrees() - 90.0).abs() < 1e-4);

        // Quadrant 2 (Left): dx=-10, dy=0 => theta=180°
        let p3 = PolarVector::from_cartesian(-10.0, 0.0);
        assert!((p3.angle_degrees() - 180.0).abs() < 1e-4);

        // Quadrant 4 (Down): dx=0, dy=-10 => theta=270°
        let p4 = PolarVector::from_cartesian(0.0, -10.0);
        assert!((p4.angle_degrees() - 270.0).abs() < 1e-4);
    }

    #[test]
    fn test_identity_fourier_model() {
        let model = FourierModel::identity();
        for deg in 0..360 {
            let theta = (deg as f32).to_radians();
            let val = model.evaluate(theta);
            assert!((val - 1.0).abs() < 1e-6, "Failed at {} deg: {}", deg, val);
        }
    }

    #[test]
    fn test_fourier_equidistant_fit_constant() {
        let samples = vec![1.2f32; 16];
        let model = FourierModel::fit_equidistant(&samples);
        assert!((model.a0 - 1.2).abs() < 1e-4);
        for i in 0..FOURIER_ORDER {
            assert!(model.a[i].abs() < 1e-4);
            assert!(model.b[i].abs() < 1e-4);
        }
    }

    #[test]
    fn test_fourier_harmonic_recovery() {
        // True function: S(θ) = 1.0 + 0.15 * cos(θ) - 0.08 * sin(2θ)
        let mut samples = Vec::with_capacity(16);
        for k in 0..16 {
            let theta = (k as f32) * TWO_PI / 16.0;
            let val = 1.0 + 0.15 * theta.cos() - 0.08 * (2.0 * theta).sin();
            samples.push(val);
        }

        let model = FourierModel::fit_equidistant(&samples);
        assert!((model.a0 - 1.0).abs() < 1e-4, "a0: {}", model.a0);
        assert!((model.a[0] - 0.15).abs() < 1e-4, "a1: {}", model.a[0]);
        assert!((model.b[1] - (-0.08)).abs() < 1e-4, "b2: {}", model.b[1]);
        assert!(model.a[1].abs() < 1e-4);
        assert!(model.b[0].abs() < 1e-4);
    }

    #[test]
    fn test_fourier_arbitrary_fit() {
        let mut points = Vec::new();
        for deg in (0..360).step_by(30) {
            let theta = (deg as f32).to_radians();
            let y = 1.0 + 0.10 * (2.0 * theta).cos();
            points.push((theta, y));
        }

        let model = FourierModel::fit_arbitrary(&points);
        assert!((model.a0 - 1.0).abs() < 0.02);
        assert!((model.a[1] - 0.10).abs() < 0.02);
    }

    #[test]
    fn test_correct_delta_with_strength() {
        let mut model = FourierModel::identity();
        model.a[0] = 0.2; // boost along 0° (right)

        // At angle 0, dx=10, dy=0
        let (out_x, out_y) = model.correct_delta(10.0, 0.0, 1.0);
        assert!((out_x - 12.0).abs() < 1e-4);
        assert!(out_y.abs() < 1e-4);

        // With strength = 0.5, boost should be 50% => scale = 1.10 => out_x = 11.0
        let (half_x, _) = model.correct_delta(10.0, 0.0, 0.5);
        assert!((half_x - 11.0).abs() < 1e-4);
    }
}
