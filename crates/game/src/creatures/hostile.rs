//! Hostile underwater creatures.
//!
//! 7 creature types with depth ranges, damage, and AI behavior.
//! Trench Warden boss at hadal depths.

use crate::world::ResourceType;

/// Hostile creature type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostileType {
    /// Shallow pack predator.
    SeaSnake,
    /// Shallow/mid ambush predator.
    Barracuda,
    /// Mid-depth with bioluminescent lure.
    AnglerFish,
    /// Mid-depth territorial.
    GiantSquid,
    /// Deep slow massive.
    AbyssalLeviathan,
    /// Deep wall-climbing pack.
    PressureCrawler,
    /// Hadal boss.
    TrenchWarden,
}

impl HostileType {
    /// Human-readable name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            HostileType::SeaSnake => "Sea Snake",
            HostileType::Barracuda => "Barracuda",
            HostileType::AnglerFish => "Angler Fish",
            HostileType::GiantSquid => "Giant Squid",
            HostileType::AbyssalLeviathan => "Abyssal Leviathan",
            HostileType::PressureCrawler => "Pressure Crawler",
            HostileType::TrenchWarden => "Trench Warden",
        }
    }

    /// Minimum depth this creature spawns.
    #[must_use]
    pub fn min_depth(&self) -> f32 {
        match self {
            HostileType::SeaSnake => 0.0,
            HostileType::Barracuda => 50.0,
            HostileType::AnglerFish => 200.0,
            HostileType::GiantSquid => 300.0,
            HostileType::AbyssalLeviathan => 800.0,
            HostileType::PressureCrawler => 500.0,
            HostileType::TrenchWarden => 4000.0,
        }
    }

    /// Maximum depth this creature spawns.
    #[must_use]
    pub fn max_depth(&self) -> f32 {
        match self {
            HostileType::SeaSnake => 200.0,
            HostileType::Barracuda => 400.0,
            HostileType::AnglerFish => 600.0,
            HostileType::GiantSquid => 800.0,
            HostileType::AbyssalLeviathan => 2000.0,
            HostileType::PressureCrawler => 1500.0,
            HostileType::TrenchWarden => 11000.0,
        }
    }

    /// Base damage per hit.
    #[must_use]
    pub fn damage(&self) -> f32 {
        match self {
            HostileType::SeaSnake => 5.0,
            HostileType::Barracuda => 12.0,
            HostileType::AnglerFish => 8.0,
            HostileType::GiantSquid => 20.0,
            HostileType::AbyssalLeviathan => 35.0,
            HostileType::PressureCrawler => 10.0,
            HostileType::TrenchWarden => 50.0,
        }
    }

    /// Maximum health.
    #[must_use]
    pub fn max_health(&self) -> f32 {
        match self {
            HostileType::SeaSnake => 20.0,
            HostileType::Barracuda => 40.0,
            HostileType::AnglerFish => 60.0,
            HostileType::GiantSquid => 150.0,
            HostileType::AbyssalLeviathan => 500.0,
            HostileType::PressureCrawler => 80.0,
            HostileType::TrenchWarden => 2000.0,
        }
    }

    /// Movement speed (blocks/sec).
    #[must_use]
    pub fn speed(&self) -> f32 {
        match self {
            HostileType::SeaSnake => 8.0,
            HostileType::Barracuda => 10.0,
            HostileType::AnglerFish => 4.0,
            HostileType::GiantSquid => 6.0,
            HostileType::AbyssalLeviathan => 2.5,
            HostileType::PressureCrawler => 5.0,
            HostileType::TrenchWarden => 4.0,
        }
    }

    /// Whether this is a pack creature.
    #[must_use]
    pub fn is_pack(&self) -> bool {
        match self {
            HostileType::SeaSnake | HostileType::PressureCrawler => true,
            _ => false,
        }
    }

    /// Whether this is a boss.
    #[must_use]
    pub fn is_boss(&self) -> bool {
        self == &HostileType::TrenchWarden
    }

    /// Detection range (blocks).
    #[must_use]
    pub fn detection_range(&self) -> f32 {
        match self {
            HostileType::SeaSnake => 20.0,
            HostileType::Barracuda => 25.0,
            HostileType::AnglerFish => 15.0,
            HostileType::GiantSquid => 40.0,
            HostileType::AbyssalLeviathan => 30.0,
            HostileType::PressureCrawler => 15.0,
            HostileType::TrenchWarden => 50.0,
        }
    }

    /// Resources dropped on defeat.
    #[must_use]
    pub fn drops(&self) -> Vec<ResourceType> {
        match self {
            HostileType::SeaSnake => vec![ResourceType::Fish],
            HostileType::Barracuda => vec![ResourceType::Fish, ResourceType::Shell],
            HostileType::AnglerFish => vec![ResourceType::BioluminescentOrgan, ResourceType::Fish],
            HostileType::GiantSquid => vec![ResourceType::JellyfishCells, ResourceType::Copper],
            HostileType::AbyssalLeviathan => vec![ResourceType::Abyssalite, ResourceType::Crystal],
            HostileType::PressureCrawler => vec![ResourceType::IronOre, ResourceType::DeepWormHide],
            HostileType::TrenchWarden => vec![
                ResourceType::AncientArtifact,
                ResourceType::PressureGem,
                ResourceType::Abyssalite,
            ],
        }
    }

    /// Depth-scaled damage (harder fights in deeper water).
    #[must_use]
    pub fn scaled_damage(&self, depth: f32) -> f32 {
        let base = self.damage();
        let depth_factor = 1.0 + (depth / 1000.0).min(2.0);
        base * depth_factor
    }
}

/// A hostile creature instance.
#[derive(Debug, Clone)]
pub struct HostileCreature {
    pub creature_type: HostileType,
    pub health: f32,
    pub position: [f32; 3],
    pub is_aggressive: bool,
}

impl HostileCreature {
    /// Create a new hostile creature.
    #[must_use]
    pub fn new(creature_type: HostileType, position: [f32; 3]) -> Self {
        Self {
            creature_type,
            health: creature_type.max_health(),
            position,
            is_aggressive: creature_type.is_pack() || creature_type.is_boss(),
        }
    }

    /// Check if creature is alive.
    #[must_use]
    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    /// Apply damage and return actual damage dealt.
    pub fn take_damage(&mut self, amount: f32) -> f32 {
        let actual = amount.min(self.health);
        self.health -= actual;
        if self.health <= 0.0 {
            self.health = 0.0;
        }
        actual
    }

    /// Get current depth.
    #[must_use]
    pub fn depth(&self) -> f32 {
        -self.position[1]
    }

    /// Check if creature is within its depth range.
    #[must_use]
    pub fn is_in_depth_range(&self) -> bool {
        let depth = self.depth();
        depth >= self.creature_type.min_depth() && depth <= self.creature_type.max_depth()
    }

    /// Get damage at current depth.
    #[must_use]
    pub fn current_damage(&self) -> f32 {
        self.creature_type.scaled_damage(self.depth())
    }

    /// Get loot drops if creature is dead.
    #[must_use]
    pub fn loot(&self) -> Vec<ResourceType> {
        if self.is_alive() {
            vec![]
        } else {
            self.creature_type.drops()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sea_snake_shallow() {
        assert_relative_eq!(HostileType::SeaSnake.min_depth(), 0.0);
        assert_relative_eq!(HostileType::SeaSnake.max_depth(), 200.0);
    }

    #[test]
    fn test_trench_warden_deep() {
        assert_relative_eq!(HostileType::TrenchWarden.min_depth(), 4000.0);
        assert!(HostileType::TrenchWarden.is_boss());
    }

    #[test]
    fn test_pack_creatures() {
        assert!(HostileType::SeaSnake.is_pack());
        assert!(HostileType::PressureCrawler.is_pack());
        assert!(!HostileType::Barracuda.is_pack());
    }

    #[test]
    fn test_creature_creation() {
        let creature = HostileCreature::new(HostileType::AnglerFish, [0.0, -300.0, 0.0]);
        assert!(creature.is_alive());
        assert_relative_eq!(creature.health, 60.0);
    }

    #[test]
    fn test_take_damage() {
        let mut creature = HostileCreature::new(HostileType::SeaSnake, [0.0, -50.0, 0.0]);
        let dealt = creature.take_damage(10.0);
        assert_relative_eq!(dealt, 10.0);
        assert_relative_eq!(creature.health, 10.0);
    }

    #[test]
    fn test_kill_creature() {
        let mut creature = HostileCreature::new(HostileType::SeaSnake, [0.0, -50.0, 0.0]);
        creature.take_damage(100.0);
        assert!(!creature.is_alive());
        assert_relative_eq!(creature.health, 0.0);
    }

    #[test]
    fn test_depth_scaling() {
        let surface_damage = HostileType::SeaSnake.scaled_damage(0.0);
        let deep_damage = HostileType::SeaSnake.scaled_damage(500.0);
        assert!(deep_damage > surface_damage);
    }

    #[test]
    fn test_loot_on_death() {
        let mut creature = HostileCreature::new(HostileType::GiantSquid, [0.0, -400.0, 0.0]);
        assert!(creature.loot().is_empty());
        creature.take_damage(200.0);
        assert!(!creature.loot().is_empty());
    }

    #[test]
    fn test_depth_range_check() {
        let creature = HostileCreature::new(HostileType::AnglerFish, [0.0, -100.0, 0.0]);
        assert!(!creature.is_in_depth_range()); // Too shallow
    }

    #[test]
    fn test_warden_drops_artifact() {
        let drops = HostileType::TrenchWarden.drops();
        assert!(drops.contains(&ResourceType::AncientArtifact));
    }

    #[test]
    fn test_all_creatures_have_stats() {
        for creature in [
            HostileType::SeaSnake,
            HostileType::Barracuda,
            HostileType::AnglerFish,
            HostileType::GiantSquid,
            HostileType::AbyssalLeviathan,
            HostileType::PressureCrawler,
            HostileType::TrenchWarden,
        ] {
            assert!(creature.damage() > 0.0);
            assert!(creature.max_health() > 0.0);
            assert!(creature.speed() > 0.0);
            assert!(creature.detection_range() > 0.0);
            assert!(!creature.drops().is_empty());
        }
    }

    use approx::assert_relative_eq;
}
