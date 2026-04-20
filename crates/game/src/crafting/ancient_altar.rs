//! Ancient Altar crafting station for endgame items.
//!
//! Found in deep ocean ruins. Crafts artifact weapons and pressure vessels
//! from ancient artifacts, pressure gems, and abyssalite.

use serde::{Deserialize, Serialize};

use crate::world::resources::ResourceType;

/// Ancient Altar crafting station.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AncientAltar {
    /// Whether the altar is active (powered by artifacts).
    active: bool,
    /// Current charge level (0-100, consumed per craft).
    charge: f32,
    /// Maximum charge.
    max_charge: f32,
    /// Number of items crafted at this altar.
    crafts_completed: u32,
}

impl AncientAltar {
    /// Charge cost per craft.
    pub const CRAFT_COST: f32 = 50.0;
    /// Maximum charge.
    pub const MAX_CHARGE: f32 = 100.0;

    /// Create a new inactive ancient altar.
    #[must_use]
    pub fn new() -> Self {
        Self {
            active: false,
            charge: 0.0,
            max_charge: Self::MAX_CHARGE,
            crafts_completed: 0,
        }
    }

    /// Check if the altar is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get the current charge level.
    #[must_use]
    pub fn charge(&self) -> f32 {
        self.charge
    }

    /// Get the charge as a percentage (0-100).
    #[must_use]
    pub fn charge_percent(&self) -> f32 {
        if self.max_charge > 0.0 {
            (self.charge / self.max_charge) * 100.0
        } else {
            0.0
        }
    }

    /// Get the number of completed crafts.
    #[must_use]
    pub fn crafts_completed(&self) -> u32 {
        self.crafts_completed
    }

    /// Insert an artifact to charge the altar.
    /// Returns the charge gained.
    pub fn insert_artifact(&mut self, artifact: ResourceType) -> f32 {
        let charge_gain = match artifact {
            ResourceType::AncientArtifact => 50.0,
            ResourceType::PressureGem => 25.0,
            ResourceType::Abyssalite => 10.0,
            _ => 0.0,
        };

        if charge_gain > 0.0 {
            self.charge = (self.charge + charge_gain).min(self.max_charge);
            if self.charge >= Self::CRAFT_COST {
                self.active = true;
            }
        }

        charge_gain
    }

    /// Check if the altar has enough charge to craft.
    #[must_use]
    pub fn can_craft(&self) -> bool {
        self.active && self.charge >= Self::CRAFT_COST
    }

    /// Consume charge for crafting.
    /// Returns true if charge was consumed successfully.
    pub fn consume_charge(&mut self) -> bool {
        if !self.can_craft() {
            return false;
        }

        self.charge -= Self::CRAFT_COST;
        self.crafts_completed += 1;

        if self.charge < Self::CRAFT_COST {
            self.active = false;
        }

        true
    }

    /// Get the list of endgame items this altar can craft.
    #[must_use]
    pub fn available_recipes() -> Vec<AncientRecipe> {
        vec![
            AncientRecipe::PressureVessel,
            AncientRecipe::ThermalSuit,
            AncientRecipe::SonarBurst,
            AncientRecipe::PressureWave,
            AncientRecipe::LightNova,
        ]
    }
}

impl Default for AncientAltar {
    fn default() -> Self {
        Self::new()
    }
}

/// Endgame crafting recipes available at the Ancient Altar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AncientRecipe {
    /// Pressure vessel - maximum depth protection.
    PressureVessel,
    /// Thermal suit - immune to heat damage.
    ThermalSuit,
    /// Sonar burst weapon - reveals all creatures in range.
    SonarBurst,
    /// Pressure wave weapon - AoE damage.
    PressureWave,
    /// Light nova weapon - blinds and damages all nearby creatures.
    LightNova,
}

impl AncientRecipe {
    /// Get the recipe name.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            AncientRecipe::PressureVessel => "Pressure Vessel",
            AncientRecipe::ThermalSuit => "Thermal Suit",
            AncientRecipe::SonarBurst => "Sonar Burst",
            AncientRecipe::PressureWave => "Pressure Wave",
            AncientRecipe::LightNova => "Light Nova",
        }
    }

    /// Get the ingredients required for this recipe.
    #[must_use]
    pub fn ingredients(&self) -> Vec<(ResourceType, u32)> {
        match self {
            AncientRecipe::PressureVessel => vec![
                (ResourceType::PressureGem, 3),
                (ResourceType::Abyssalite, 10),
                (ResourceType::AncientArtifact, 1),
            ],
            AncientRecipe::ThermalSuit => vec![
                (ResourceType::Abyssalite, 8),
                (ResourceType::AncientArtifact, 1),
            ],
            AncientRecipe::SonarBurst => vec![
                (ResourceType::AncientArtifact, 2),
                (ResourceType::PressureGem, 1),
            ],
            AncientRecipe::PressureWave => vec![
                (ResourceType::AncientArtifact, 2),
                (ResourceType::Abyssalite, 5),
            ],
            AncientRecipe::LightNova => vec![
                (ResourceType::AncientArtifact, 3),
                (ResourceType::PressureGem, 2),
            ],
        }
    }

    /// Get the equipment tier this recipe produces.
    #[must_use]
    pub fn tier(&self) -> crate::crafting::equipment::EquipmentTier {
        crate::crafting::equipment::EquipmentTier::Endgame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ancient_altar_new() {
        let altar = AncientAltar::new();
        assert!(!altar.is_active());
        assert!((altar.charge() - 0.0).abs() < f32::EPSILON);
        assert_eq!(altar.crafts_completed(), 0);
    }

    #[test]
    fn test_ancient_altar_insert_artifact() {
        let mut altar = AncientAltar::new();
        let gain = altar.insert_artifact(ResourceType::AncientArtifact);
        assert!((gain - 50.0).abs() < f32::EPSILON);
        assert!(altar.is_active());
    }

    #[test]
    fn test_ancient_altar_insert_pressure_gem() {
        let mut altar = AncientAltar::new();
        let gain = altar.insert_artifact(ResourceType::PressureGem);
        assert!((gain - 25.0).abs() < f32::EPSILON);
        assert!(!altar.is_active()); // 25 < 50 needed
    }

    #[test]
    fn test_ancient_altar_insert_abyssalite() {
        let mut altar = AncientAltar::new();
        let gain = altar.insert_artifact(ResourceType::Abyssalite);
        assert!((gain - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ancient_altar_insert_non_artifact() {
        let mut altar = AncientAltar::new();
        let gain = altar.insert_artifact(ResourceType::Kelp);
        assert!((gain - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ancient_altar_charge_cap() {
        let mut altar = AncientAltar::new();
        altar.insert_artifact(ResourceType::AncientArtifact);
        altar.insert_artifact(ResourceType::AncientArtifact);
        altar.insert_artifact(ResourceType::AncientArtifact);
        assert!((altar.charge() - AncientAltar::MAX_CHARGE).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ancient_altar_charge_percent() {
        let mut altar = AncientAltar::new();
        altar.insert_artifact(ResourceType::AncientArtifact);
        assert!((altar.charge_percent() - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ancient_altar_can_craft() {
        let mut altar = AncientAltar::new();
        assert!(!altar.can_craft());
        altar.insert_artifact(ResourceType::AncientArtifact);
        assert!(altar.can_craft());
    }

    #[test]
    fn test_ancient_altar_consume_charge() {
        let mut altar = AncientAltar::new();
        altar.insert_artifact(ResourceType::AncientArtifact);
        assert!(altar.consume_charge());
        assert_eq!(altar.crafts_completed(), 1);
    }

    #[test]
    fn test_ancient_altar_consume_deactivates() {
        let mut altar = AncientAltar::new();
        altar.insert_artifact(ResourceType::AncientArtifact); // 50 charge
        altar.consume_charge(); // costs 50
        assert!(!altar.is_active());
    }

    #[test]
    fn test_ancient_altar_consume_insufficient() {
        let mut altar = AncientAltar::new();
        assert!(!altar.consume_charge());
    }

    #[test]
    fn test_ancient_recipes_names() {
        assert_eq!(AncientRecipe::PressureVessel.name(), "Pressure Vessel");
        assert_eq!(AncientRecipe::ThermalSuit.name(), "Thermal Suit");
        assert_eq!(AncientRecipe::SonarBurst.name(), "Sonar Burst");
        assert_eq!(AncientRecipe::PressureWave.name(), "Pressure Wave");
        assert_eq!(AncientRecipe::LightNova.name(), "Light Nova");
    }

    #[test]
    fn test_ancient_recipes_ingredients() {
        let ingredients = AncientRecipe::PressureVessel.ingredients();
        assert_eq!(ingredients.len(), 3);
    }

    #[test]
    fn test_ancient_recipes_tier() {
        assert_eq!(AncientRecipe::PressureVessel.tier(), crate::crafting::equipment::EquipmentTier::Endgame);
    }

    #[test]
    fn test_available_recipes() {
        let recipes = AncientAltar::available_recipes();
        assert_eq!(recipes.len(), 5);
    }
}
