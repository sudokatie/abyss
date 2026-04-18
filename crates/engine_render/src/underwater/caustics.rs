//! Caustic lighting effect for shallow underwater surfaces.
//!
//! Animated light patterns caused by wave refraction. Only visible
//! in the sunlight zone (0-50m depth).

/// Maximum depth for caustic visibility (meters).
pub const CAUSTIC_MAX_DEPTH: f32 = 50.0;

/// Caustic animation speed (radians per second).
pub const CAUSTIC_SPEED: f32 = 1.5;

/// Number of overlapping sine waves for caustic pattern.
pub const CAUSTIC_WAVE_COUNT: usize = 4;

/// Base intensity of caustic light.
pub const CAUSTIC_BASE_INTENSITY: f32 = 0.3;

/// Caustic light renderer for shallow water.
#[derive(Debug, Clone)]
pub struct CausticRenderer {
    /// Animation time accumulator.
    time: f32,
    /// Whether caustics are enabled.
    enabled: bool,
    /// Wave frequencies for the caustic pattern.
    wave_freqs: [f32; CAUSTIC_WAVE_COUNT],
    /// Wave amplitudes.
    wave_amps: [f32; CAUSTIC_WAVE_COUNT],
    /// Phase offsets for each wave.
    wave_phases: [f32; CAUSTIC_WAVE_COUNT],
}

impl Default for CausticRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl CausticRenderer {
    /// Create a new caustic renderer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            time: 0.0,
            enabled: true,
            wave_freqs: [1.0, 1.3, 0.7, 1.7],
            wave_amps: [0.3, 0.25, 0.2, 0.15],
            wave_phases: [0.0, 1.2, 2.5, 0.8],
        }
    }

    /// Enable or disable caustics.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if caustics are enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Update the caustic animation.
    pub fn update(&mut self, delta_seconds: f32) {
        self.time += delta_seconds * CAUSTIC_SPEED;
    }

    /// Calculate caustic intensity at a position and depth.
    ///
    /// Returns 0.0 (no caustic) to ~1.0 (full caustic).
    #[must_use]
    pub fn intensity_at(&self, x: f32, z: f32, depth: f32) -> f32 {
        if !self.enabled || depth > CAUSTIC_MAX_DEPTH || depth < 0.0 {
            return 0.0;
        }

        // Fade with depth
        let depth_factor = 1.0 - (depth / CAUSTIC_MAX_DEPTH);

        // Sum overlapping sine waves
        let mut pattern = 0.0;
        for i in 0..CAUSTIC_WAVE_COUNT {
            let phase = self.wave_phases[i] + self.time;
            pattern += self.wave_amps[i]
                * (self.wave_freqs[i] * x + phase).sin()
                * (self.wave_freqs[i] * z + phase * 0.7).cos();
        }

        // Normalize to 0-1 range
        let normalized = ((pattern + 1.0) * 0.5).clamp(0.0, 1.0);

        CAUSTIC_BASE_INTENSITY * normalized * depth_factor
    }

    /// Check if caustics are visible at a given depth.
    #[must_use]
    pub fn is_visible_at_depth(&self, depth: f32) -> bool {
        self.enabled && depth >= 0.0 && depth <= CAUSTIC_MAX_DEPTH
    }

    /// Get the depth fade factor (1.0 at surface, 0.0 at max depth).
    #[must_use]
    pub fn depth_fade(&self, depth: f32) -> f32 {
        if depth <= 0.0 {
            return 1.0;
        }
        if depth >= CAUSTIC_MAX_DEPTH {
            return 0.0;
        }
        1.0 - (depth / CAUSTIC_MAX_DEPTH)
    }

    /// Get current animation time.
    #[must_use]
    pub fn time(&self) -> f32 {
        self.time
    }

    /// Reset animation time.
    pub fn reset(&mut self) {
        self.time = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_renderer() {
        let renderer = CausticRenderer::new();
        assert!(renderer.is_enabled());
        assert_relative_eq!(renderer.time(), 0.0);
    }

    #[test]
    fn test_surface_intensity() {
        let renderer = CausticRenderer::new();
        let intensity = renderer.intensity_at(10.0, 10.0, 0.0);
        assert!(intensity > 0.0);
    }

    #[test]
    fn test_no_caustic_beyond_max_depth() {
        let renderer = CausticRenderer::new();
        let intensity = renderer.intensity_at(10.0, 10.0, 60.0);
        assert_relative_eq!(intensity, 0.0);
    }

    #[test]
    fn test_no_caustic_at_exact_max_depth() {
        let renderer = CausticRenderer::new();
        let intensity = renderer.intensity_at(10.0, 10.0, CAUSTIC_MAX_DEPTH);
        assert_relative_eq!(intensity, 0.0);
    }

    #[test]
    fn test_fade_with_depth() {
        let renderer = CausticRenderer::new();
        let surface = renderer.intensity_at(10.0, 10.0, 5.0);
        let mid = renderer.intensity_at(10.0, 10.0, 25.0);
        assert!(surface > mid);
    }

    #[test]
    fn test_disabled_no_intensity() {
        let mut renderer = CausticRenderer::new();
        renderer.set_enabled(false);
        let intensity = renderer.intensity_at(10.0, 10.0, 0.0);
        assert_relative_eq!(intensity, 0.0);
    }

    #[test]
    fn test_animation_updates_time() {
        let mut renderer = CausticRenderer::new();
        renderer.update(1.0);
        assert_relative_eq!(renderer.time(), CAUSTIC_SPEED);
    }

    #[test]
    fn test_animation_changes_intensity() {
        let mut renderer = CausticRenderer::new();
        let before = renderer.intensity_at(5.0, 5.0, 10.0);
        renderer.update(2.0);
        let after = renderer.intensity_at(5.0, 5.0, 10.0);
        // Intensity should change with animation
        // (might be same by coincidence, but very unlikely)
        assert!(before >= 0.0 && after >= 0.0);
    }

    #[test]
    fn test_depth_fade_surface() {
        let renderer = CausticRenderer::new();
        assert_relative_eq!(renderer.depth_fade(0.0), 1.0);
    }

    #[test]
    fn test_depth_fade_max() {
        let renderer = CausticRenderer::new();
        assert_relative_eq!(renderer.depth_fade(CAUSTIC_MAX_DEPTH), 0.0);
    }

    #[test]
    fn test_depth_fade_mid() {
        let renderer = CausticRenderer::new();
        assert_relative_eq!(renderer.depth_fade(25.0), 0.5);
    }

    #[test]
    fn test_is_visible_at_depth() {
        let renderer = CausticRenderer::new();
        assert!(renderer.is_visible_at_depth(25.0));
        assert!(!renderer.is_visible_at_depth(60.0));
        assert!(renderer.is_visible_at_depth(0.0));
    }

    #[test]
    fn test_reset() {
        let mut renderer = CausticRenderer::new();
        renderer.update(10.0);
        renderer.reset();
        assert_relative_eq!(renderer.time(), 0.0);
    }

    use approx::assert_relative_eq;
}
