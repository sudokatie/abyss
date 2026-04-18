//! Passive underwater creatures.
//!
//! Non-hostile wildlife: fish schools, sea turtles, jellyfish, whales.

use crate::world::ResourceType;

/// Passive creature type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassiveType {
    /// Schooling fish, catchable.
    FishSchool,
    /// Sea turtles, drop shells.
    SeaTurtle,
    /// Bioluminescent jellyfish, contact damage.
    Jellyfish,
    /// Rare surface whales.
    Whale,
}

impl PassiveType {
    /// Human-readable name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            PassiveType::FishSchool => "Fish School",
            PassiveType::SeaTurtle => "Sea Turtle",
            PassiveType::Jellyfish => "Jellyfish",
            PassiveType::Whale => "Whale",
        }
    }

    /// Minimum depth.
    #[must_use]
    pub fn min_depth(&self) -> f32 {
        match self {
            PassiveType::FishSchool => 0.0,
            PassiveType::SeaTurtle => 0.0,
            PassiveType::Jellyfish => 100.0,
            PassiveType::Whale => 0.0,
        }
    }

    /// Maximum depth.
    #[must_use]
    pub fn max_depth(&self) -> f32 {
        match self {
            PassiveType::FishSchool => 200.0,
            PassiveType::SeaTurtle => 150.0,
            PassiveType::Jellyfish => 600.0,
            PassiveType::Whale => 50.0,
        }
    }

    /// Movement speed (blocks/sec).
    #[must_use]
    pub fn speed(&self) -> f32 {
        match self {
            PassiveType::FishSchool => 3.0,
            PassiveType::SeaTurtle => 1.5,
            PassiveType::Jellyfish => 0.5,
            PassiveType::Whale => 2.0,
        }
    }

    /// Whether this creature is bioluminescent.
    #[must_use]
    pub fn is_bioluminescent(&self) -> bool {
        self == &PassiveType::Jellyfish
    }

    /// Contact damage (jellyfish sting).
    #[must_use]
    pub fn contact_damage(&self) -> f32 {
        match self {
            PassiveType::Jellyfish => 3.0,
            _ => 0.0,
        }
    }

    /// Whether this creature is catchable.
    #[must_use]
    pub fn is_catchable(&self) -> bool {
        match self {
            PassiveType::FishSchool => true,
            PassiveType::SeaTurtle => true,
            _ => false,
        }
    }

    /// Resources obtained from catching/harvesting.
    #[must_use]
    pub fn harvest_drops(&self) -> Vec<ResourceType> {
        match self {
            PassiveType::FishSchool => vec![ResourceType::Fish],
            PassiveType::SeaTurtle => vec![ResourceType::Shell],
            PassiveType::Jellyfish => vec![ResourceType::JellyfishCells, ResourceType::BioluminescentOrgan],
            PassiveType::Whale => vec![],
        }
    }

    /// School size (number of individuals in a group).
    #[must_use]
    pub fn school_size(&self) -> usize {
        match self {
            PassiveType::FishSchool => 8,
            PassiveType::SeaTurtle => 1,
            PassiveType::Jellyfish => 5,
            PassiveType::Whale => 1,
        }
    }
}

/// A passive creature instance.
#[derive(Debug, Clone)]
pub struct PassiveCreature {
    pub creature_type: PassiveType,
    pub position: [f32; 3],
    pub health: f32,
}

impl PassiveCreature {
    /// Create a new passive creature.
    #[must_use]
    pub fn new(creature_type: PassiveType, position: [f32; 3]) -> Self {
        Self {
            creature_type,
            position,
            health: 50.0, // Passive creatures have low health
        }
    }

    /// Current depth.
    #[must_use]
    pub fn depth(&self) -> f32 {
        -self.position[1]
    }

    /// Check if in depth range.
    #[must_use]
    pub fn is_in_depth_range(&self) -> bool {
        let depth = self.depth();
        depth >= self.creature_type.min_depth() && depth <= self.creature_type.max_depth()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fish_school_shallow() {
        assert_relative_eq!(PassiveType::FishSchool.min_depth(), 0.0);
        assert_relative_eq!(PassiveType::FishSchool.max_depth(), 200.0);
    }

    #[test]
    fn test_jellyfish_bioluminescent() {
        assert!(PassiveType::Jellyfish.is_bioluminescent());
        assert!(!PassiveType::FishSchool.is_bioluminescent());
    }

    #[test]
    fn test_jellyfish_contact_damage() {
        assert!(PassiveType::Jellyfish.contact_damage() > 0.0);
        assert_relative_eq!(PassiveType::FishSchool.contact_damage(), 0.0);
    }

    #[test]
    fn test_catchable() {
        assert!(PassiveType::FishSchool.is_catchable());
        assert!(PassiveType::SeaTurtle.is_catchable());
        assert!(!PassiveType::Jellyfish.is_catchable());
    }

    #[test]
    fn test_harvest_drops() {
        let fish_drops = PassiveType::FishSchool.harvest_drops();
        assert!(fish_drops.contains(&ResourceType::Fish));

        let jelly_drops = PassiveType::Jellyfish.harvest_drops();
        assert!(jelly_drops.contains(&ResourceType::BioluminescentOrgan));
    }

    #[test]
    fn test_depth_range_check() {
        let creature = PassiveCreature::new(PassiveType::Whale, [0.0, -10.0, 0.0]);
        assert!(creature.is_in_depth_range());

        let deep_jelly = PassiveCreature::new(PassiveType::Jellyfish, [0.0, -50.0, 0.0]);
        assert!(!deep_jelly.is_in_depth_range()); // Too shallow
    }

    use approx::assert_relative_eq;
}
