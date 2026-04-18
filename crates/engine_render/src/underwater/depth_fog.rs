//! Depth-based underwater fog using Beer-Lambert absorption.
//!
//! Visibility decreases exponentially with depth. Fog density
//! and color shift as water absorbs different wavelengths.

/// Default absorption coefficient for clear ocean water.
pub const DEFAULT_ABSORPTION_COEFF: f32 = 0.04;

/// Depth at which fog is considered maximum (visibility near zero).
pub const MAX_FOG_DEPTH: f32 = 500.0;

/// Minimum visibility factor (never fully opaque for gameplay).
pub const MIN_VISIBILITY: f32 = 0.01;

/// Surface visibility (100%).
pub const SURFACE_VISIBILITY: f32 = 1.0;

/// Depth fog configuration.
#[derive(Debug, Clone)]
pub struct DepthFogConfig {
    /// Absorption coefficient (higher = fog appears at shallower depth).
    pub absorption_coeff: f32,
    /// Maximum depth for fog calculation.
    pub max_depth: f32,
    /// Minimum visibility (gameplay floor).
    pub min_visibility: f32,
    /// Fog color at surface (light blue).
    pub surface_color: [f32; 3],
    /// Fog color at max depth (near black).
    pub deep_color: [f32; 3],
}

impl Default for DepthFogConfig {
    fn default() -> Self {
        Self::ocean()
    }
}

impl DepthFogConfig {
    /// Standard ocean water fog.
    #[must_use]
    pub fn ocean() -> Self {
        Self {
            absorption_coeff: DEFAULT_ABSORPTION_COEFF,
            max_depth: MAX_FOG_DEPTH,
            min_visibility: MIN_VISIBILITY,
            surface_color: [0.1, 0.4, 0.7],  // Light blue
            deep_color: [0.01, 0.02, 0.05], // Near black
        }
    }

    /// Murky/coastal water (denser fog).
    #[must_use]
    pub fn murky() -> Self {
        Self {
            absorption_coeff: 0.08,
            max_depth: 300.0,
            min_visibility: 0.02,
            surface_color: [0.2, 0.35, 0.4],  // Greenish
            deep_color: [0.02, 0.03, 0.03],  // Dark green-black
        }
    }

    /// Clear tropical water (lighter fog).
    #[must_use]
    pub fn tropical() -> Self {
        Self {
            absorption_coeff: 0.02,
            max_depth: 600.0,
            min_visibility: 0.005,
            surface_color: [0.0, 0.5, 0.8],  // Bright cyan
            deep_color: [0.0, 0.1, 0.3],     // Deep blue
        }
    }
}

/// Calculates depth-based fog parameters for underwater rendering.
#[derive(Debug, Clone)]
pub struct DepthFogCalculator {
    config: DepthFogConfig,
}

impl Default for DepthFogCalculator {
    fn default() -> Self {
        Self::new(DepthFogConfig::default())
    }
}

impl DepthFogCalculator {
    /// Create a new depth fog calculator with the given config.
    #[must_use]
    pub fn new(config: DepthFogConfig) -> Self {
        Self { config }
    }

    /// Get the current configuration.
    #[must_use]
    pub fn config(&self) -> &DepthFogConfig {
        &self.config
    }

    /// Calculate visibility factor at a given depth using Beer-Lambert law.
    ///
    /// Returns 0.0 (invisible) to 1.0 (fully visible).
    #[must_use]
    pub fn visibility_at_depth(&self, depth: f32) -> f32 {
        if depth <= 0.0 {
            return SURFACE_VISIBILITY;
        }
        // Beer-Lambert: I = I0 * exp(-a * d)
        let visibility = SURFACE_VISIBILITY * (-self.config.absorption_coeff * depth).exp();
        visibility.max(self.config.min_visibility)
    }

    /// Calculate fog density at a given depth.
    ///
    /// Returns 0.0 (no fog) to 1.0 (full fog).
    #[must_use]
    pub fn fog_density_at_depth(&self, depth: f32) -> f32 {
        1.0 - self.visibility_at_depth(depth)
    }

    /// Calculate fog color at a given depth.
    ///
    /// Interpolates between surface and deep fog colors based on depth.
    #[must_use]
    pub fn fog_color_at_depth(&self, depth: f32) -> [f32; 3] {
        let t = (depth / self.config.max_depth).min(1.0);
        [
            self.config.surface_color[0] + (self.config.deep_color[0] - self.config.surface_color[0]) * t,
            self.config.surface_color[1] + (self.config.deep_color[1] - self.config.surface_color[1]) * t,
            self.config.surface_color[2] + (self.config.deep_color[2] - self.config.surface_color[2]) * t,
        ]
    }

    /// Apply fog to a fragment color at a given depth.
    ///
    /// Mixes the fragment color with the fog color based on visibility.
    #[must_use]
    pub fn apply_fog(&self, fragment_color: [f32; 3], depth: f32) -> [f32; 3] {
        let visibility = self.visibility_at_depth(depth);
        let fog_color = self.fog_color_at_depth(depth);
        [
            fragment_color[0] * visibility + fog_color[0] * (1.0 - visibility),
            fragment_color[1] * visibility + fog_color[1] * (1.0 - visibility),
            fragment_color[2] * visibility + fog_color[2] * (1.0 - visibility),
        ]
    }

    /// Check if depth is in the "twilight" visibility range.
    #[must_use]
    pub fn is_twilight_depth(&self, depth: f32) -> bool {
        let vis = self.visibility_at_depth(depth);
        vis > 0.01 && vis < 0.3
    }

    /// Get depth at which visibility drops below a threshold.
    #[must_use]
    pub fn depth_for_visibility(&self, threshold: f32) -> f32 {
        if threshold >= SURFACE_VISIBILITY {
            return 0.0;
        }
        // Solve: threshold = exp(-a * d) => d = -ln(threshold) / a
        (-threshold.ln() / self.config.absorption_coeff).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_surface_visibility() {
        let calc = DepthFogCalculator::default();
        assert_relative_eq!(calc.visibility_at_depth(0.0), 1.0);
    }

    #[test]
    fn test_visibility_decreases_with_depth() {
        let calc = DepthFogCalculator::default();
        let v50 = calc.visibility_at_depth(50.0);
        let v100 = calc.visibility_at_depth(100.0);
        let v200 = calc.visibility_at_depth(200.0);
        assert!(v50 > v100);
        assert!(v100 > v200);
    }

    #[test]
    fn test_visibility_at_50m() {
        let calc = DepthFogCalculator::default();
        let vis = calc.visibility_at_depth(50.0);
        // exp(-0.04 * 50) = exp(-2.0) ~= 0.135
        assert!(vis > 0.1 && vis < 0.2);
    }

    #[test]
    fn test_visibility_at_200m() {
        let calc = DepthFogCalculator::default();
        let vis = calc.visibility_at_depth(200.0);
        // exp(-0.04 * 200) = exp(-8) ~= 0.000335, clamped to min_visibility
        assert!(vis <= MIN_VISIBILITY + 0.01);
    }

    #[test]
    fn test_visibility_minimum_floor() {
        let calc = DepthFogCalculator::default();
        let vis = calc.visibility_at_depth(10000.0);
        assert_relative_eq!(vis, MIN_VISIBILITY);
    }

    #[test]
    fn test_fog_density_inverse_of_visibility() {
        let calc = DepthFogCalculator::default();
        let vis = calc.visibility_at_depth(100.0);
        let density = calc.fog_density_at_depth(100.0);
        assert_relative_eq!(vis + density, 1.0, max_relative = 0.001);
    }

    #[test]
    fn test_fog_color_interpolation() {
        let calc = DepthFogCalculator::default();
        let surface = calc.fog_color_at_depth(0.0);
        assert_relative_eq!(surface[0], 0.1);
        assert_relative_eq!(surface[1], 0.4);
        assert_relative_eq!(surface[2], 0.7);
    }

    #[test]
    fn test_fog_color_deep() {
        let calc = DepthFogCalculator::default();
        let deep = calc.fog_color_at_depth(10000.0); // Way past max
        assert_relative_eq!(deep[0], 0.01);
        assert_relative_eq!(deep[1], 0.02);
    }

    #[test]
    fn test_apply_fog_surface() {
        let calc = DepthFogCalculator::default();
        let result = calc.apply_fog([1.0, 0.0, 0.0], 0.0);
        // At surface, full visibility, no fog
        assert_relative_eq!(result[0], 1.0);
    }

    #[test]
    fn test_apply_fog_deep() {
        let calc = DepthFogCalculator::default();
        let result = calc.apply_fog([1.0, 0.0, 0.0], 500.0);
        // Deep: mostly fog color
        assert!(result[0] < 0.1);
    }

    #[test]
    fn test_murky_water_less_visibility() {
        let ocean = DepthFogCalculator::new(DepthFogConfig::ocean());
        let murky = DepthFogCalculator::new(DepthFogConfig::murky());
        let depth = 50.0; // Use shallower depth where both are above min
        assert!(murky.visibility_at_depth(depth) < ocean.visibility_at_depth(depth));
    }

    #[test]
    fn test_tropical_water_more_visibility() {
        let ocean = DepthFogCalculator::new(DepthFogConfig::ocean());
        let tropical = DepthFogCalculator::new(DepthFogConfig::tropical());
        let depth = 100.0;
        assert!(tropical.visibility_at_depth(depth) > ocean.visibility_at_depth(depth));
    }

    #[test]
    fn test_depth_for_visibility_threshold() {
        let calc = DepthFogCalculator::default();
        let depth = calc.depth_for_visibility(0.5);
        // Verify roundtrip
        let vis = calc.visibility_at_depth(depth);
        assert_relative_eq!(vis, 0.5, max_relative = 0.01);
    }

    #[test]
    fn test_is_twilight_depth() {
        let calc = DepthFogCalculator::default();
        assert!(!calc.is_twilight_depth(0.0)); // Surface
        assert!(calc.is_twilight_depth(100.0)); // Twilight zone
    }
}
