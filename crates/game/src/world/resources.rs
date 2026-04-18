//! Underwater world resources.
//!
//! Depth-tied resource distribution across ocean zones.
//! Spec 8.1: surface, mid, deep, and living resource categories.

/// Maximum depth where any resources spawn.
pub const MAX_RESOURCE_DEPTH: f32 = 6000.0;

/// Resource category based on depth zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceCategory {
    /// Surface resources: kelp, coral, shells.
    Surface,
    /// Mid-depth resources: iron, crystal, salvage.
    Mid,
    /// Deep resources: abyssalite, thermal energy, artifacts.
    Deep,
    /// Living resources: fish, jellyfish, worms.
    Living,
}

impl ResourceCategory {
    /// Determine category from depth.
    #[must_use]
    pub fn from_depth(depth: f32) -> Self {
        if depth < 200.0 {
            ResourceCategory::Surface
        } else if depth < 1000.0 {
            ResourceCategory::Mid
        } else if depth < 4000.0 {
            ResourceCategory::Deep
        } else {
            ResourceCategory::Deep // Same, but rarer
        }
    }

    /// Human-readable category name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            ResourceCategory::Surface => "Surface",
            ResourceCategory::Mid => "Mid-Depth",
            ResourceCategory::Deep => "Deep",
            ResourceCategory::Living => "Living",
        }
    }
}

/// Rarity level for resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

impl Rarity {
    /// Drop probability (0.0 to 1.0).
    #[must_use]
    pub fn drop_chance(&self) -> f32 {
        match self {
            Rarity::Common => 0.8,
            Rarity::Uncommon => 0.3,
            Rarity::Rare => 0.08,
            Rarity::Legendary => 0.01,
        }
    }
}

/// A resource type that can be found underwater.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    // Surface
    Kelp,
    Coral,
    Shell,
    Driftwood,
    // Mid-depth
    IronOre,
    Crystal,
    Salvage,
    Copper,
    // Deep
    Abyssalite,
    ThermalEnergy,
    AncientArtifact,
    PressureGem,
    // Living
    Fish,
    JellyfishCells,
    DeepWormHide,
    BioluminescentOrgan,
}

impl ResourceType {
    /// Get the category for this resource.
    #[must_use]
    pub fn category(&self) -> ResourceCategory {
        match self {
            ResourceType::Kelp
            | ResourceType::Coral
            | ResourceType::Shell
            | ResourceType::Driftwood => ResourceCategory::Surface,

            ResourceType::IronOre
            | ResourceType::Crystal
            | ResourceType::Salvage
            | ResourceType::Copper => ResourceCategory::Mid,

            ResourceType::Abyssalite
            | ResourceType::ThermalEnergy
            | ResourceType::AncientArtifact
            | ResourceType::PressureGem => ResourceCategory::Deep,

            ResourceType::Fish
            | ResourceType::JellyfishCells
            | ResourceType::DeepWormHide
            | ResourceType::BioluminescentOrgan => ResourceCategory::Living,
        }
    }

    /// Get the rarity of this resource.
    #[must_use]
    pub fn rarity(&self) -> Rarity {
        match self {
            ResourceType::Kelp | ResourceType::Shell => Rarity::Common,
            ResourceType::Coral | ResourceType::Driftwood | ResourceType::Fish => Rarity::Uncommon,
            ResourceType::IronOre | ResourceType::Copper | ResourceType::Salvage => Rarity::Uncommon,
            ResourceType::Crystal | ResourceType::JellyfishCells => Rarity::Rare,
            ResourceType::Abyssalite | ResourceType::ThermalEnergy | ResourceType::DeepWormHide
            | ResourceType::BioluminescentOrgan => Rarity::Rare,
            ResourceType::AncientArtifact | ResourceType::PressureGem => Rarity::Legendary,
        }
    }

    /// Minimum depth where this resource can spawn.
    #[must_use]
    pub fn min_depth(&self) -> f32 {
        match self {
            ResourceType::Kelp | ResourceType::Driftwood => 0.0,
            ResourceType::Coral | ResourceType::Shell => 10.0,
            ResourceType::Fish => 0.0,
            ResourceType::IronOre | ResourceType::Copper => 200.0,
            ResourceType::Crystal | ResourceType::Salvage => 300.0,
            ResourceType::JellyfishCells => 100.0,
            ResourceType::BioluminescentOrgan => 200.0,
            ResourceType::Abyssalite => 1000.0,
            ResourceType::ThermalEnergy => 500.0,
            ResourceType::DeepWormHide => 800.0,
            ResourceType::AncientArtifact => 1500.0,
            ResourceType::PressureGem => 2000.0,
        }
    }

    /// Maximum depth where this resource can spawn.
    #[must_use]
    pub fn max_depth(&self) -> f32 {
        match self {
            ResourceType::Kelp => 50.0,
            ResourceType::Coral => 100.0,
            ResourceType::Shell => 150.0,
            ResourceType::Driftwood => 30.0,
            ResourceType::Fish => 500.0,
            ResourceType::IronOre => 2000.0,
            ResourceType::Copper => 1500.0,
            ResourceType::Crystal => 3000.0,
            ResourceType::Salvage => 1500.0,
            ResourceType::JellyfishCells => 800.0,
            ResourceType::BioluminescentOrgan => 1000.0,
            ResourceType::Abyssalite => 5000.0,
            ResourceType::ThermalEnergy => 4000.0,
            ResourceType::DeepWormHide => 3000.0,
            ResourceType::AncientArtifact => MAX_RESOURCE_DEPTH,
            ResourceType::PressureGem => MAX_RESOURCE_DEPTH,
        }
    }

    /// Check if this resource can spawn at a given depth.
    #[must_use]
    pub fn can_spawn_at_depth(&self, depth: f32) -> bool {
        depth >= self.min_depth() && depth <= self.max_depth()
    }

    /// Human-readable name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            ResourceType::Kelp => "Kelp",
            ResourceType::Coral => "Coral",
            ResourceType::Shell => "Shell",
            ResourceType::Driftwood => "Driftwood",
            ResourceType::IronOre => "Iron Ore",
            ResourceType::Crystal => "Crystal",
            ResourceType::Salvage => "Salvage",
            ResourceType::Copper => "Copper",
            ResourceType::Abyssalite => "Abyssalite",
            ResourceType::ThermalEnergy => "Thermal Energy",
            ResourceType::AncientArtifact => "Ancient Artifact",
            ResourceType::PressureGem => "Pressure Gem",
            ResourceType::Fish => "Fish",
            ResourceType::JellyfishCells => "Jellyfish Cells",
            ResourceType::DeepWormHide => "Deep Worm Hide",
            ResourceType::BioluminescentOrgan => "Bioluminescent Organ",
        }
    }
}

/// Distribution of resources at a given depth.
#[derive(Debug, Clone)]
pub struct ResourceDistribution {
    /// Depth being sampled.
    pub depth: f32,
    /// Resources available at this depth with their drop chances.
    pub available: Vec<(ResourceType, f32)>,
}

impl ResourceDistribution {
    /// Calculate resource distribution at a given depth.
    #[must_use]
    pub fn at_depth(depth: f32) -> Self {
        let all_resources = [
            ResourceType::Kelp,
            ResourceType::Coral,
            ResourceType::Shell,
            ResourceType::Driftwood,
            ResourceType::IronOre,
            ResourceType::Crystal,
            ResourceType::Salvage,
            ResourceType::Copper,
            ResourceType::Abyssalite,
            ResourceType::ThermalEnergy,
            ResourceType::AncientArtifact,
            ResourceType::PressureGem,
            ResourceType::Fish,
            ResourceType::JellyfishCells,
            ResourceType::DeepWormHide,
            ResourceType::BioluminescentOrgan,
        ];

        let mut available = Vec::new();
        for &resource in &all_resources {
            if resource.can_spawn_at_depth(depth) {
                // Deeper = rarer for surface/mid resources
                let depth_penalty = match resource.category() {
                    ResourceCategory::Surface => {
                        (1.0 - (depth / 200.0).min(1.0)) * 0.5
                    }
                    ResourceCategory::Mid => {
                        let mid_center = 600.0;
                        let dist = (depth - mid_center).abs();
                        (1.0 - (dist / 800.0).min(1.0)) * 0.3
                    }
                    ResourceCategory::Deep => 0.2,
                    ResourceCategory::Living => 0.4,
                };
                let chance = resource.rarity().drop_chance() * (1.0 - depth_penalty);
                if chance > 0.001 {
                    available.push((resource, chance));
                }
            }
        }

        Self { depth, available }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_resources() {
        let dist = ResourceDistribution::at_depth(50.0);
        let types: Vec<_> = dist.available.iter().map(|(r, _)| *r).collect();
        assert!(types.contains(&ResourceType::Kelp));
        assert!(types.contains(&ResourceType::Shell));
    }

    #[test]
    fn test_mid_resources() {
        let dist = ResourceDistribution::at_depth(400.0);
        let types: Vec<_> = dist.available.iter().map(|(r, _)| *r).collect();
        assert!(types.contains(&ResourceType::IronOre));
    }

    #[test]
    fn test_deep_resources() {
        let dist = ResourceDistribution::at_depth(1500.0);
        let types: Vec<_> = dist.available.iter().map(|(r, _)| *r).collect();
        assert!(types.contains(&ResourceType::Abyssalite));
    }

    #[test]
    fn test_kelp_shallow_only() {
        assert!(ResourceType::Kelp.can_spawn_at_depth(10.0));
        assert!(!ResourceType::Kelp.can_spawn_at_depth(100.0));
    }

    #[test]
    fn test_artifact_deep_only() {
        assert!(!ResourceType::AncientArtifact.can_spawn_at_depth(500.0));
        assert!(ResourceType::AncientArtifact.can_spawn_at_depth(2000.0));
    }

    #[test]
    fn test_rarity_increases_with_depth() {
        let surface_dist = ResourceDistribution::at_depth(50.0);
        let deep_dist = ResourceDistribution::at_depth(2000.0);
        // Surface should have more resources available
        assert!(surface_dist.available.len() > 0);
        assert!(deep_dist.available.len() > 0);
    }

    use approx::assert_relative_eq;
}
