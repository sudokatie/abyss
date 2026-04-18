//! Underwater crafting stations.
//!
//! Workbench (surface), Deep Forge (thermal vent), Ancient Altar (ruins).
//! Each has proximity requirements and unique recipes.

/// Crafting station type with depth requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeepStation {
    /// Surface workbench - basic recipes.
    Workbench,
    /// Deep forge - requires thermal vent proximity. Advanced recipes.
    DeepForge,
    /// Ancient altar - requires ruins proximity. Endgame recipes.
    AncientAltar,
}

impl DeepStation {
    /// Human-readable name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            DeepStation::Workbench => "Workbench",
            DeepStation::DeepForge => "Deep Forge",
            DeepStation::AncientAltar => "Ancient Altar",
        }
    }

    /// Maximum depth for this station.
    #[must_use]
    pub fn max_depth(&self) -> f32 {
        match self {
            DeepStation::Workbench => 50.0,
            DeepStation::DeepForge => 2000.0,
            DeepStation::AncientAltar => 6000.0,
        }
    }

    /// Minimum depth for this station.
    #[must_use]
    pub fn min_depth(&self) -> f32 {
        match self {
            DeepStation::Workbench => 0.0,
            DeepStation::DeepForge => 100.0,
            DeepStation::AncientAltar => 500.0,
        }
    }

    /// Proximity requirement to special feature (blocks).
    /// 0.0 means no proximity requirement.
    #[must_use]
    pub fn proximity_requirement(&self) -> f32 {
        match self {
            DeepStation::Workbench => 0.0,
            DeepStation::DeepForge => 16.0, // Must be near thermal vent
            DeepStation::AncientAltar => 32.0, // Must be near ruins
        }
    }

    /// Whether this station requires a nearby feature.
    #[must_use]
    pub fn requires_proximity(&self) -> bool {
        self.proximity_requirement() > 0.0
    }

    /// Feature name for proximity requirement.
    #[must_use]
    pub fn required_feature(&self) -> Option<&'static str> {
        match self {
            DeepStation::Workbench => None,
            DeepStation::DeepForge => Some("Thermal Vent"),
            DeepStation::AncientAltar => Some("Ancient Ruins"),
        }
    }

    /// Maximum equipment tier this station can craft.
    #[must_use]
    pub fn max_tier(&self) -> u32 {
        match self {
            DeepStation::Workbench => 2,    // Basic + Standard
            DeepStation::DeepForge => 3,    // + Advanced
            DeepStation::AncientAltar => 4, // + Endgame
        }
    }

    /// Check if a depth is valid for this station.
    #[must_use]
    pub fn is_valid_depth(&self, depth: f32) -> bool {
        depth >= self.min_depth() && depth <= self.max_depth()
    }

    /// Check proximity to a required feature position.
    #[must_use]
    pub fn is_proximity_met(&self, station_pos: [f32; 3], feature_pos: [f32; 3]) -> bool {
        if !self.requires_proximity() {
            return true;
        }
        let dx = station_pos[0] - feature_pos[0];
        let dy = station_pos[1] - feature_pos[1];
        let dz = station_pos[2] - feature_pos[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        dist <= self.proximity_requirement()
    }
}

/// A placed crafting station in the world.
#[derive(Debug, Clone)]
pub struct DeepCraftingStation {
    pub station_type: DeepStation,
    pub position: [f32; 3],
    /// Position of the required nearby feature (vent/ruins).
    pub feature_position: Option<[f32; 3]>,
}

impl DeepCraftingStation {
    /// Create a new crafting station.
    #[must_use]
    pub fn new(station_type: DeepStation, position: [f32; 3]) -> Self {
        Self {
            station_type,
            position,
            feature_position: None,
        }
    }

    /// Create with a required feature position.
    #[must_use]
    pub fn with_feature(station_type: DeepStation, position: [f32; 3], feature_pos: [f32; 3]) -> Self {
        Self {
            station_type,
            position,
            feature_position: Some(feature_pos),
        }
    }

    /// Check if this station is usable.
    #[must_use]
    pub fn is_usable(&self) -> bool {
        let depth = -self.position[1];
        if !self.station_type.is_valid_depth(depth) {
            return false;
        }
        if let Some(feature_pos) = self.feature_position {
            self.station_type.is_proximity_met(self.position, feature_pos)
        } else {
            !self.station_type.requires_proximity()
        }
    }

    /// Get the depth of this station.
    #[must_use]
    pub fn depth(&self) -> f32 {
        -self.position[1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workbench_surface() {
        assert!(DeepStation::Workbench.is_valid_depth(10.0));
        assert!(!DeepStation::Workbench.is_valid_depth(100.0));
    }

    #[test]
    fn test_deep_forge_depth() {
        assert!(!DeepStation::DeepForge.is_valid_depth(50.0));
        assert!(DeepStation::DeepForge.is_valid_depth(300.0));
    }

    #[test]
    fn test_ancient_altar_deep() {
        assert!(DeepStation::AncientAltar.is_valid_depth(1000.0));
        assert!(!DeepStation::AncientAltar.is_valid_depth(100.0));
    }

    #[test]
    fn test_proximity_workbench() {
        assert!(!DeepStation::Workbench.requires_proximity());
    }

    #[test]
    fn test_proximity_deep_forge() {
        assert!(DeepStation::DeepForge.requires_proximity());
        assert_eq!(DeepStation::DeepForge.required_feature(), Some("Thermal Vent"));
    }

    #[test]
    fn test_station_usable_workbench() {
        let station = DeepCraftingStation::new(DeepStation::Workbench, [0.0, -10.0, 0.0]);
        assert!(station.is_usable());
    }

    #[test]
    fn test_station_forge_proximity_met() {
        let station = DeepCraftingStation::with_feature(
            DeepStation::DeepForge,
            [10.0, -200.0, 10.0],
            [12.0, -200.0, 12.0], // Within 16 blocks
        );
        assert!(station.is_usable());
    }

    #[test]
    fn test_station_forge_proximity_not_met() {
        let station = DeepCraftingStation::with_feature(
            DeepStation::DeepForge,
            [0.0, -200.0, 0.0],
            [100.0, -200.0, 100.0], // Too far
        );
        assert!(!station.is_usable());
    }

    use approx::assert_relative_eq;
}
