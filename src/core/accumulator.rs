//! Sub-pixel residual accumulator (anti-jitter) for mouse reports.
//!
//! Because mouse reports in Windows Raw Input are discrete integers (counts),
//! multiplying by continuous floating-point multipliers results in quantization
//! error. At slow tracking speeds or low DPI, naive truncation or rounding causes
//! lost micro-adjustments or perceptible stepping/jitter.
//!
//! This accumulator preserves fractional remainders between consecutive packets,
//! ensuring strict conservation of total physical displacement.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SubpixelAccumulator {
    pub remainder_x: f32,
    pub remainder_y: f32,
}

impl Default for SubpixelAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

impl SubpixelAccumulator {
    /// Initializes a new accumulator with zero remainder.
    pub const fn new() -> Self {
        Self {
            remainder_x: 0.0,
            remainder_y: 0.0,
        }
    }

    /// Consumes continuous displacements (dx, dy), accumulates them with past remainders,
    /// and returns the discrete (out_x, out_y) counts to report to the OS / game,
    /// storing the new sub-pixel residuals.
    #[inline]
    pub fn apply(&mut self, dx: f32, dy: f32) -> (i32, i32) {
        let target_x = dx + self.remainder_x;
        let target_y = dy + self.remainder_y;

        let out_x = target_x.round() as i32;
        let out_y = target_y.round() as i32;

        self.remainder_x = target_x - (out_x as f32);
        self.remainder_y = target_y - (out_y as f32);

        (out_x, out_y)
    }

    /// Resets stored remainders (e.g. when cursor is reset or calibration starts).
    #[inline]
    pub fn reset(&mut self) {
        self.remainder_x = 0.0;
        self.remainder_y = 0.0;
    }

    /// Returns the current fractional remainder in counts.
    #[inline]
    pub fn remainders(&self) -> (f32, f32) {
        (self.remainder_x, self.remainder_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accumulator_exact_integers() {
        let mut acc = SubpixelAccumulator::new();
        let (x, y) = acc.apply(5.0, -3.0);
        assert_eq!((x, y), (5, -3));
        assert_eq!(acc.remainders(), (0.0, 0.0));
    }

    #[test]
    fn test_accumulator_fractional_conservation() {
        let mut acc = SubpixelAccumulator::new();
        // Send 10 packets of 0.3 counts each. Total true distance = 3.0 counts.
        let mut total_x = 0;
        for _ in 0..10 {
            let (x, _) = acc.apply(0.3, 0.0);
            total_x += x;
        }
        assert_eq!(total_x, 3, "Conservation of motion failed over 10 packets");
        assert!(acc.remainder_x.abs() < 1e-5);
    }

    #[test]
    fn test_accumulator_alternating_and_subpixel() {
        let mut acc = SubpixelAccumulator::new();
        // 0.4 -> rounded to 0, remainder 0.4
        let (x1, _) = acc.apply(0.4, 0.0);
        assert_eq!(x1, 0);
        assert!((acc.remainder_x - 0.4).abs() < 1e-5);

        // Another 0.4 -> target is 0.8 -> rounded to 1, remainder -0.2
        let (x2, _) = acc.apply(0.4, 0.0);
        assert_eq!(x2, 1);
        assert!((acc.remainder_x - (-0.2)).abs() < 1e-5);

        // Another 0.4 -> target is 0.2 -> rounded to 0, remainder 0.2
        let (x3, _) = acc.apply(0.4, 0.0);
        assert_eq!(x3, 0);
        assert!((acc.remainder_x - 0.2).abs() < 1e-5);
    }

    #[test]
    fn test_accumulator_reset() {
        let mut acc = SubpixelAccumulator::new();
        acc.apply(0.7, 0.7);
        assert_ne!(acc.remainders(), (0.0, 0.0));
        acc.reset();
        assert_eq!(acc.remainders(), (0.0, 0.0));
    }
}
