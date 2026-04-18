//! Depth zones and pressure damage calculations.

/// Number of defined depth zones.
pub const ZONE_COUNT: usize = 5;

/// A depth zone with associated pressure properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthZone {
    /// 0-50m: Safe zone, no pressure damage.
    Sunlight,
    /// 50-200m: Moderate pressure, damage without suit.
    Twilight,
    /// 200-500m: High pressure, significant damage.
    Midnight,
    /// 500-1000m: Extreme pressure, deadly without reinforced suit.
    Abyssal,
    /// 1000m+: Crushing, instant death without pressure vessel.
    Hadal,
}

impl DepthZone {
    /// Get the zone for a given depth (meters).
    #[must_use]
    pub fn from_depth(depth: f32) -> Self {
        if depth < 50.0 {
            Self::Sunlight
        } else if depth < 200.0 {
            Self::Twilight
        } else if depth < 500.0 {
            Self::Midnight
        } else if depth < 1000.0 {
            Self::Abyssal
        } else {
            Self::Hadal
        }
    }

    /// Minimum depth for this zone.
    #[must_use]
    pub fn min_depth(self) -> f32 {
        match self {
            Self::Sunlight => 0.0,
            Self::Twilight => 50.0,
            Self::Midnight => 200.0,
            Self::Abyssal => 500.0,
            Self::Hadal => 1000.0,
        }
    }

    /// Maximum depth for this zone (exclusive).
    #[must_use]
    pub fn max_depth(self) -> f32 {
        match self {
            Self::Sunlight => 50.0,
            Self::Twilight => 200.0,
            Self::Midnight => 500.0,
            Self::Abyssal => 1000.0,
            Self::Hadal => f32::MAX,
        }
    }

    /// Pressure damage per second without any suit.
    #[must_use]
    pub fn base_damage(self) -> f32 {
        match self {
            Self::Sunlight => 0.0,
            Self::Twilight => 1.0,
            Self::Midnight => 3.0,
            Self::Abyssal => 5.0,
            Self::Hadal => 100.0, // Effectively instant death
        }
    }

    /// Human-readable zone name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Sunlight => "Sunlight Zone",
            Self::Twilight => "Twilight Zone",
            Self::Midnight => "Midnight Zone",
            Self::Abyssal => "Abyssal Zone",
            Self::Hadal => "Hadal Zone",
        }
    }

    /// Whether this zone requires a suit to survive.
    #[must_use]
    pub fn requires_suit(self) -> bool {
        self.base_damage() > 0.0
    }

    /// All zones in depth order.
    pub const ALL: [DepthZone; ZONE_COUNT] = [
        Self::Sunlight,
        Self::Twilight,
        Self::Midnight,
        Self::Abyssal,
        Self::Hadal,
    ];
}

/// Manages pressure calculations for the game world.
#[derive(Debug, Clone)]
pub struct PressureZones {
    /// Whether pressure damage is enabled.
    enabled: bool,
}

impl Default for PressureZones {
    fn default() -> Self {
        Self::new()
    }
}

impl PressureZones {
    /// Create a new pressure zone manager.
    #[must_use]
    pub fn new() -> Self {
        Self { enabled: true }
    }

    /// Enable or disable pressure damage.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if pressure damage is enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the depth zone for a position.
    #[must_use]
    pub fn zone_at_depth(&self, depth: f32) -> DepthZone {
        DepthZone::from_depth(depth)
    }

    /// Calculate pressure damage per second at a depth without a suit.
    #[must_use]
    pub fn damage_at_depth(&self, depth: f32) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        DepthZone::from_depth(depth).base_damage()
    }

    /// Calculate pressure damage per second with a suit.
    ///
    /// Returns the damage after suit protection is applied.
    /// If the suit's rated depth is exceeded, full damage applies.
    #[must_use]
    pub fn damage_with_suit(&self, depth: f32, suit_rated_depth: f32, suit_protection: f32) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        if depth <= suit_rated_depth {
            // Within suit rating - protected
            let base = DepthZone::from_depth(depth).base_damage();
            (base * (1.0 - suit_protection)).max(0.0)
        } else {
            // Beyond suit rating - full damage
            DepthZone::from_depth(depth).base_damage()
        }
    }

    /// Check if depth is in the safe zone (no pressure damage).
    #[must_use]
    pub fn is_safe_depth(&self, depth: f32) -> bool {
        depth < 50.0
    }

    /// Check if depth requires a pressure vessel (Hadal zone).
    #[must_use]
    pub fn requires_vessel(&self, depth: f32) -> bool {
        depth >= 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_from_depth_sunlight() {
        assert_eq!(DepthZone::from_depth(0.0), DepthZone::Sunlight);
        assert_eq!(DepthZone::from_depth(25.0), DepthZone::Sunlight);
        assert_eq!(DepthZone::from_depth(49.9), DepthZone::Sunlight);
    }

    #[test]
    fn test_zone_from_depth_twilight() {
        assert_eq!(DepthZone::from_depth(50.0), DepthZone::Twilight);
        assert_eq!(DepthZone::from_depth(150.0), DepthZone::Twilight);
        assert_eq!(DepthZone::from_depth(199.9), DepthZone::Twilight);
    }

    #[test]
    fn test_zone_from_depth_midnight() {
        assert_eq!(DepthZone::from_depth(200.0), DepthZone::Midnight);
        assert_eq!(DepthZone::from_depth(350.0), DepthZone::Midnight);
    }

    #[test]
    fn test_zone_from_depth_abyssal() {
        assert_eq!(DepthZone::from_depth(500.0), DepthZone::Abyssal);
        assert_eq!(DepthZone::from_depth(750.0), DepthZone::Abyssal);
    }

    #[test]
    fn test_zone_from_depth_hadal() {
        assert_eq!(DepthZone::from_depth(1000.0), DepthZone::Hadal);
        assert_eq!(DepthZone::from_depth(5000.0), DepthZone::Hadal);
    }

    #[test]
    fn test_zone_damage_values() {
        assert_eq!(DepthZone::Sunlight.base_damage(), 0.0);
        assert_eq!(DepthZone::Twilight.base_damage(), 1.0);
        assert_eq!(DepthZone::Midnight.base_damage(), 3.0);
        assert_eq!(DepthZone::Abyssal.base_damage(), 5.0);
        assert_eq!(DepthZone::Hadal.base_damage(), 100.0);
    }

    #[test]
    fn test_zone_min_max_depths() {
        assert_eq!(DepthZone::Sunlight.min_depth(), 0.0);
        assert_eq!(DepthZone::Sunlight.max_depth(), 50.0);
        assert_eq!(DepthZone::Hadal.min_depth(), 1000.0);
    }

    #[test]
    fn test_damage_at_depth() {
        let zones = PressureZones::new();
        assert_eq!(zones.damage_at_depth(25.0), 0.0); // Sunlight
        assert_eq!(zones.damage_at_depth(100.0), 1.0); // Twilight
        assert_eq!(zones.damage_at_depth(350.0), 3.0); // Midnight
        assert_eq!(zones.damage_at_depth(750.0), 5.0); // Abyssal
        assert_eq!(zones.damage_at_depth(1500.0), 100.0); // Hadal
    }

    #[test]
    fn test_damage_with_suit_within_rating() {
        let zones = PressureZones::new();
        // Basic suit: 200m rated, 80% protection
        let damage = zones.damage_with_suit(100.0, 200.0, 0.8);
        // Twilight zone: 1.0 * (1 - 0.8) = 0.2
        assert!((damage - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_damage_with_suit_beyond_rating() {
        let zones = PressureZones::new();
        // Basic suit: 200m rated, but we're at 500m
        let damage = zones.damage_with_suit(500.0, 200.0, 0.8);
        // Beyond rating: full Abyssal damage
        assert_eq!(damage, 5.0);
    }

    #[test]
    fn test_disabled_no_damage() {
        let mut zones = PressureZones::new();
        zones.set_enabled(false);
        assert_eq!(zones.damage_at_depth(500.0), 0.0);
    }

    #[test]
    fn test_safe_depth() {
        let zones = PressureZones::new();
        assert!(zones.is_safe_depth(25.0));
        assert!(!zones.is_safe_depth(50.0));
    }

    #[test]
    fn test_requires_vessel() {
        let zones = PressureZones::new();
        assert!(!zones.requires_vessel(500.0));
        assert!(zones.requires_vessel(1000.0));
    }

    #[test]
    fn test_zone_names() {
        assert_eq!(DepthZone::Sunlight.name(), "Sunlight Zone");
        assert_eq!(DepthZone::Hadal.name(), "Hadal Zone");
    }

    #[test]
    fn test_zone_requires_suit() {
        assert!(!DepthZone::Sunlight.requires_suit());
        assert!(DepthZone::Twilight.requires_suit());
        assert!(DepthZone::Hadal.requires_suit());
    }
}
