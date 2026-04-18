//! Depth gauge HUD element.
//!
//! Always visible, color-coded by depth zone.

use engine_world::ocean::DepthZone;

/// Depth gauge display state.
#[derive(Debug, Clone)]
pub struct DepthGauge {
    /// Current depth in meters.
    pub depth: f32,
    /// Current zone.
    pub zone: DepthZone,
}

impl DepthGauge {
    /// Create a new depth gauge.
    #[must_use]
    pub fn new() -> Self {
        Self {
            depth: 0.0,
            zone: DepthZone::Sunlight,
        }
    }

    /// Update gauge from current depth.
    pub fn update(&mut self, depth: f32) {
        self.depth = depth;
        self.zone = DepthZone::from_depth(depth);
    }

    /// Get color for the current zone (RGB 0-1).
    #[must_use]
    pub fn zone_color(&self) -> [f32; 3] {
        self.zone.color_palette()
    }

    /// Formatted depth string.
    #[must_use]
    pub fn depth_text(&self) -> String {
        format!("{:.0}m", self.depth)
    }

    /// Zone name for display.
    #[must_use]
    pub fn zone_text(&self) -> &'static str {
        self.zone.name()
    }

    /// Whether depth is dangerous (no suit).
    #[must_use]
    pub fn is_dangerous(&self) -> bool {
        self.depth > 50.0
    }
}

impl Default for DepthGauge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depth_gauge_update() {
        let mut gauge = DepthGauge::new();
        gauge.update(150.0);
        assert_relative_eq!(gauge.depth, 150.0);
        assert_eq!(gauge.zone, DepthZone::Sunlight);
    }

    #[test]
    fn test_depth_gauge_zone_changes() {
        let mut gauge = DepthGauge::new();
        gauge.update(350.0);
        assert_eq!(gauge.zone, DepthZone::Twilight);
    }

    #[test]
    fn test_depth_gauge_dangerous() {
        let mut gauge = DepthGauge::new();
        gauge.update(10.0);
        assert!(!gauge.is_dangerous());
        gauge.update(100.0);
        assert!(gauge.is_dangerous());
    }

    use approx::assert_relative_eq;
}
