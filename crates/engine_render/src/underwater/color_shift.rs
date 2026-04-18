//! RGB wavelength absorption for underwater color shift.
//!
//! Water absorbs longer wavelengths first: red, then green,
//! then blue. This creates the characteristic blue-shift with depth.

/// Red absorption coefficient (absorbed quickly, ~50m).
pub const RED_ABSORPTION: f32 = 0.04;

/// Green absorption coefficient (absorbed moderately, ~100m).
pub const GREEN_ABSORPTION: f32 = 0.01;

/// Blue absorption coefficient (persists deepest, ~300m).
pub const BLUE_ABSORPTION: f32 = 0.003;

/// Color shift calculator for underwater rendering.
///
/// Each RGB channel attenuates independently using Beer-Lambert law,
/// creating the characteristic deep-blue appearance at depth.
#[derive(Debug, Clone)]
pub struct ColorShiftCalculator {
    /// Red absorption coefficient.
    red_coeff: f32,
    /// Green absorption coefficient.
    green_coeff: f32,
    /// Blue absorption coefficient.
    blue_coeff: f32,
}

impl Default for ColorShiftCalculator {
    fn default() -> Self {
        Self::ocean()
    }
}

impl ColorShiftCalculator {
    /// Create a calculator for standard ocean water.
    #[must_use]
    pub fn ocean() -> Self {
        Self {
            red_coeff: RED_ABSORPTION,
            green_coeff: GREEN_ABSORPTION,
            blue_coeff: BLUE_ABSORPTION,
        }
    }

    /// Create a calculator for murky coastal water.
    #[must_use]
    pub fn murky() -> Self {
        Self {
            red_coeff: 0.06,
            green_coeff: 0.02,
            blue_coeff: 0.005,
        }
    }

    /// Create a calculator with custom coefficients.
    #[must_use]
    pub fn custom(red: f32, green: f32, blue: f32) -> Self {
        Self {
            red_coeff: red,
            green_coeff: green,
            blue_coeff: blue,
        }
    }

    /// Calculate channel attenuation at depth.
    ///
    /// Returns the remaining intensity (0.0 to 1.0) for each channel.
    #[must_use]
    pub fn attenuation_at_depth(&self, depth: f32) -> [f32; 3] {
        if depth <= 0.0 {
            return [1.0, 1.0, 1.0];
        }
        [
            (-self.red_coeff * depth).exp(),
            (-self.green_coeff * depth).exp(),
            (-self.blue_coeff * depth).exp(),
        ]
    }

    /// Apply color shift to a fragment color at a given depth.
    #[must_use]
    pub fn apply(&self, color: [f32; 3], depth: f32) -> [f32; 3] {
        let attenuation = self.attenuation_at_depth(depth);
        [
            color[0] * attenuation[0],
            color[1] * attenuation[1],
            color[2] * attenuation[2],
        ]
    }

    /// Get the dominant wavelength at a given depth.
    ///
    /// Returns which channel has the highest attenuation (is most absorbed).
    #[must_use]
    pub fn dominant_channel_at_depth(&self, depth: f32) -> usize {
        let att = self.attenuation_at_depth(depth);
        if att[2] >= att[1] && att[2] >= att[0] {
            2 // Blue dominates
        } else if att[1] >= att[0] {
            1 // Green dominates
        } else {
            0 // Red dominates
        }
    }

    /// Get depth at which a channel drops below a threshold.
    #[must_use]
    pub fn depth_for_channel_threshold(&self, channel: usize, threshold: f32) -> f32 {
        let coeff = match channel {
            0 => self.red_coeff,
            1 => self.green_coeff,
            2 => self.blue_coeff,
            _ => return 0.0,
        };
        if coeff <= 0.0 || threshold >= 1.0 {
            return 0.0;
        }
        (-threshold.ln() / coeff).max(0.0)
    }

    /// Check if red light is effectively gone at this depth.
    #[must_use]
    pub fn is_red_absorbed(&self, depth: f32) -> bool {
        let att = self.attenuation_at_depth(depth);
        att[0] < 0.01
    }

    /// Check if green light is effectively gone at this depth.
    #[must_use]
    pub fn is_green_absorbed(&self, depth: f32) -> bool {
        let att = self.attenuation_at_depth(depth);
        att[1] < 0.01
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_surface_no_shift() {
        let calc = ColorShiftCalculator::default();
        let color = calc.apply([1.0, 1.0, 1.0], 0.0);
        assert_relative_eq!(color[0], 1.0);
        assert_relative_eq!(color[1], 1.0);
        assert_relative_eq!(color[2], 1.0);
    }

    #[test]
    fn test_red_absorbed_first() {
        let calc = ColorShiftCalculator::default();
        let att = calc.attenuation_at_depth(50.0);
        // Red most attenuated, blue least
        assert!(att[0] < att[1]);
        assert!(att[1] < att[2]);
    }

    #[test]
    fn test_red_effectively_gone_at_100m() {
        let calc = ColorShiftCalculator::default();
        assert!(calc.is_red_absorbed(150.0)); // Red effectively gone by 150m
    }

    #[test]
    fn test_red_present_at_30m() {
        let calc = ColorShiftCalculator::default();
        assert!(!calc.is_red_absorbed(30.0));
    }

    #[test]
    fn test_blue_persists_at_200m() {
        let calc = ColorShiftCalculator::default();
        let att = calc.attenuation_at_depth(200.0);
        assert!(att[2] > 0.1); // Blue still present
    }

    #[test]
    fn test_dominant_channel_surface() {
        let calc = ColorShiftCalculator::default();
        // At surface all channels equal, blue technically highest
        assert_eq!(calc.dominant_channel_at_depth(0.0), 2);
    }

    #[test]
    fn test_dominant_channel_deep() {
        let calc = ColorShiftCalculator::default();
        assert_eq!(calc.dominant_channel_at_depth(200.0), 2); // Blue
    }

    #[test]
    fn test_depth_for_red_threshold() {
        let calc = ColorShiftCalculator::default();
        let depth = calc.depth_for_channel_threshold(0, 0.5);
        // exp(-0.04 * d) = 0.5 => d = ln(2)/0.04 ~= 17.3m
        assert!(depth > 15.0 && depth < 20.0);
    }

    #[test]
    fn test_green_absorbed_deep() {
        let calc = ColorShiftCalculator::default();
        assert!(calc.is_green_absorbed(500.0));
    }

    #[test]
    fn test_green_present_moderate() {
        let calc = ColorShiftCalculator::default();
        assert!(!calc.is_green_absorbed(100.0));
    }

    #[test]
    fn test_murky_shifts_faster() {
        let ocean = ColorShiftCalculator::ocean();
        let murky = ColorShiftCalculator::murky();
        let depth = 50.0;
        // Murky water absorbs more at same depth
        let ocean_att = ocean.attenuation_at_depth(depth);
        let murky_att = murky.attenuation_at_depth(depth);
        assert!(murky_att[0] < ocean_att[0]); // Less red in murky
    }

    #[test]
    fn test_custom_coefficients() {
        let calc = ColorShiftCalculator::custom(0.1, 0.05, 0.01);
        let att = calc.attenuation_at_depth(10.0);
        // Higher coefficients = more absorption
        assert!(att[0] < att[1]);
        assert!(att[1] < att[2]);
    }

    #[test]
    fn test_apply_white_becomes_blue() {
        let calc = ColorShiftCalculator::default();
        let result = calc.apply([1.0, 1.0, 1.0], 100.0);
        // Blue should be strongest channel
        assert!(result[2] > result[1]);
        assert!(result[1] > result[0]);
    }
}
