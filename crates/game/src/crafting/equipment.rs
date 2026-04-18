//! Diving equipment tiers.
//!
//! Four tiers: Basic, Standard, Advanced, Endgame.
//! Each provides oxygen, pressure, speed, and light bonuses.

use crate::world::ResourceType;

/// Equipment tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EquipmentTier {
    Basic,
    Standard,
    Advanced,
    Endgame,
}

impl EquipmentTier {
    /// Tier number (1-4).
    #[must_use]
    pub fn level(&self) -> u32 {
        match self {
            EquipmentTier::Basic => 1,
            EquipmentTier::Standard => 2,
            EquipmentTier::Advanced => 3,
            EquipmentTier::Endgame => 4,
        }
    }
}

/// Equipment slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipmentSlot {
    Suit,
    Tank,
    Mask,
    Flippers,
    Light,
}

impl EquipmentSlot {
    /// Human-readable name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            EquipmentSlot::Suit => "Suit",
            EquipmentSlot::Tank => "Tank",
            EquipmentSlot::Mask => "Mask",
            EquipmentSlot::Flippers => "Flippers",
            EquipmentSlot::Light => "Light",
        }
    }
}

/// Oxygen drain modifier (lower = better).
fn oxygen_modifier(tier: EquipmentTier) -> f32 {
    match tier {
        EquipmentTier::Basic => 0.8,
        EquipmentTier::Standard => 0.6,
        EquipmentTier::Advanced => 0.4,
        EquipmentTier::Endgame => 0.2,
    }
}

/// Maximum depth rating (meters).
fn depth_rating(tier: EquipmentTier) -> f32 {
    match tier {
        EquipmentTier::Basic => 200.0,
        EquipmentTier::Standard => 500.0,
        EquipmentTier::Advanced => 1000.0,
        EquipmentTier::Endgame => 6000.0,
    }
}

/// Speed modifier (higher = faster).
fn speed_modifier(tier: EquipmentTier) -> f32 {
    match tier {
        EquipmentTier::Basic => 1.0,
        EquipmentTier::Standard => 1.1,
        EquipmentTier::Advanced => 1.2,
        EquipmentTier::Endgame => 1.3,
    }
}

/// Light radius (blocks, 0 = no light).
fn light_radius(tier: EquipmentTier) -> f32 {
    match tier {
        EquipmentTier::Basic => 5.0,
        EquipmentTier::Standard => 10.0,
        EquipmentTier::Advanced => 20.0,
        EquipmentTier::Endgame => 40.0,
    }
}

/// A piece of diving equipment.
#[derive(Debug, Clone)]
pub struct DivingEquipment {
    pub slot: EquipmentSlot,
    pub tier: EquipmentTier,
    pub durability: f32,
    pub max_durability: f32,
}

impl DivingEquipment {
    /// Create new equipment.
    #[must_use]
    pub fn new(slot: EquipmentSlot, tier: EquipmentTier) -> Self {
        let max_durability = match tier {
            EquipmentTier::Basic => 100.0,
            EquipmentTier::Standard => 200.0,
            EquipmentTier::Advanced => 400.0,
            EquipmentTier::Endgame => 800.0,
        };
        Self {
            slot,
            tier,
            durability: max_durability,
            max_durability,
        }
    }

    /// Get oxygen drain modifier.
    #[must_use]
    pub fn oxygen_modifier(&self) -> f32 {
        if self.slot == EquipmentSlot::Suit || self.slot == EquipmentSlot::Tank {
            oxygen_modifier(self.tier)
        } else {
            1.0
        }
    }

    /// Get depth rating.
    #[must_use]
    pub fn depth_rating(&self) -> f32 {
        if self.slot == EquipmentSlot::Suit {
            depth_rating(self.tier)
        } else {
            0.0
        }
    }

    /// Get speed modifier.
    #[must_use]
    pub fn speed_modifier(&self) -> f32 {
        if self.slot == EquipmentSlot::Flippers {
            speed_modifier(self.tier)
        } else {
            1.0
        }
    }

    /// Get light radius.
    #[must_use]
    pub fn light_radius(&self) -> f32 {
        if self.slot == EquipmentSlot::Light {
            light_radius(self.tier)
        } else {
            0.0
        }
    }

    /// Check if equipment is broken.
    #[must_use]
    pub fn is_broken(&self) -> bool {
        self.durability <= 0.0
    }

    /// Apply wear (degradation from depth/pressure).
    pub fn apply_wear(&mut self, depth: f32, delta: f32) {
        let wear_rate = match self.slot {
            EquipmentSlot::Suit => 0.1 * (depth / depth_rating(self.tier)).min(2.0),
            _ => 0.01,
        };
        self.durability = (self.durability - wear_rate * delta).max(0.0);
    }

    /// Crafting materials required.
    #[must_use]
    pub fn crafting_materials(&self) -> Vec<(ResourceType, u32)> {
        match (self.slot, self.tier) {
            (EquipmentSlot::Suit, EquipmentTier::Basic) =>
                vec![(ResourceType::Kelp, 5), (ResourceType::Shell, 3)],
            (EquipmentSlot::Suit, EquipmentTier::Standard) =>
                vec![(ResourceType::IronOre, 5), (ResourceType::Copper, 3), (ResourceType::Shell, 2)],
            (EquipmentSlot::Suit, EquipmentTier::Advanced) =>
                vec![(ResourceType::Crystal, 3), (ResourceType::Abyssalite, 2), (ResourceType::IronOre, 5)],
            (EquipmentSlot::Suit, EquipmentTier::Endgame) =>
                vec![(ResourceType::PressureGem, 2), (ResourceType::AncientArtifact, 1), (ResourceType::Abyssalite, 5)],
            (EquipmentSlot::Tank, EquipmentTier::Basic) =>
                vec![(ResourceType::Shell, 4), (ResourceType::Driftwood, 2)],
            (EquipmentSlot::Tank, EquipmentTier::Standard) =>
                vec![(ResourceType::IronOre, 3), (ResourceType::Copper, 2)],
            (EquipmentSlot::Tank, EquipmentTier::Advanced) =>
                vec![(ResourceType::Crystal, 2), (ResourceType::IronOre, 3)],
            (EquipmentSlot::Tank, EquipmentTier::Endgame) =>
                vec![(ResourceType::PressureGem, 1), (ResourceType::Abyssalite, 3)],
            (EquipmentSlot::Flippers, _) =>
                vec![(ResourceType::Kelp, 3), (ResourceType::Driftwood, 1)],
            (EquipmentSlot::Mask, _) =>
                vec![(ResourceType::Shell, 2), (ResourceType::Coral, 1)],
            (EquipmentSlot::Light, EquipmentTier::Basic) =>
                vec![(ResourceType::Driftwood, 2), (ResourceType::BioluminescentOrgan, 1)],
            (EquipmentSlot::Light, EquipmentTier::Standard) =>
                vec![(ResourceType::Copper, 2), (ResourceType::BioluminescentOrgan, 2)],
            (EquipmentSlot::Light, EquipmentTier::Advanced) =>
                vec![(ResourceType::Crystal, 1), (ResourceType::BioluminescentOrgan, 3)],
            (EquipmentSlot::Light, EquipmentTier::Endgame) =>
                vec![(ResourceType::PressureGem, 1), (ResourceType::BioluminescentOrgan, 2)],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_suit() {
        let suit = DivingEquipment::new(EquipmentSlot::Suit, EquipmentTier::Basic);
        assert_relative_eq!(suit.depth_rating(), 200.0);
        assert!(!suit.is_broken());
    }

    #[test]
    fn test_endgame_depth() {
        let suit = DivingEquipment::new(EquipmentSlot::Suit, EquipmentTier::Endgame);
        assert_relative_eq!(suit.depth_rating(), 6000.0);
    }

    #[test]
    fn test_oxygen_improves_with_tier() {
        let basic = DivingEquipment::new(EquipmentSlot::Suit, EquipmentTier::Basic);
        let endgame = DivingEquipment::new(EquipmentSlot::Suit, EquipmentTier::Endgame);
        assert!(endgame.oxygen_modifier() < basic.oxygen_modifier());
    }

    #[test]
    fn test_wear_degrades() {
        let mut suit = DivingEquipment::new(EquipmentSlot::Suit, EquipmentTier::Basic);
        let start_dur = suit.durability;
        suit.apply_wear(100.0, 10.0);
        assert!(suit.durability < start_dur);
    }

    #[test]
    fn test_wear_faster_at_depth() {
        let mut shallow = DivingEquipment::new(EquipmentSlot::Suit, EquipmentTier::Basic);
        let mut deep = DivingEquipment::new(EquipmentSlot::Suit, EquipmentTier::Basic);
        shallow.apply_wear(50.0, 10.0);
        deep.apply_wear(150.0, 10.0);
        assert!(deep.durability < shallow.durability);
    }

    #[test]
    fn test_crafting_materials_exist() {
        for slot in [EquipmentSlot::Suit, EquipmentSlot::Tank, EquipmentSlot::Flippers, EquipmentSlot::Mask, EquipmentSlot::Light] {
            for tier in [EquipmentTier::Basic, EquipmentTier::Standard, EquipmentTier::Advanced, EquipmentTier::Endgame] {
                let equip = DivingEquipment::new(slot, tier);
                assert!(!equip.crafting_materials().is_empty(), "No materials for {:?} {:?}", slot, tier);
            }
        }
    }

    #[test]
    fn test_tank_no_depth_rating() {
        let tank = DivingEquipment::new(EquipmentSlot::Tank, EquipmentTier::Advanced);
        assert_relative_eq!(tank.depth_rating(), 0.0);
    }

    #[test]
    fn test_flippers_speed() {
        let flippers = DivingEquipment::new(EquipmentSlot::Flippers, EquipmentTier::Advanced);
        assert!(flippers.speed_modifier() > 1.0);
    }

    #[test]
    fn test_light_radius() {
        let light = DivingEquipment::new(EquipmentSlot::Light, EquipmentTier::Standard);
        assert!(light.light_radius() > 0.0);
    }

    use approx::assert_relative_eq;
}
