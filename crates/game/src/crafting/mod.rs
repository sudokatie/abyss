//! Crafting system with recipes and execution.

mod deep_forge;
mod equipment;
mod executor;
mod furnace;
mod registry;

pub use deep_forge::{DeepCraftingStation, DeepStation};
pub use equipment::{DivingEquipment, EquipmentSlot, EquipmentTier};
pub use executor::{check_craft, execute_craft, execute_craft_by_id, CraftError, CraftRequirements};
pub use furnace::{
    Furnace, FurnaceState, FuelEntry, DEFAULT_SMELT_TIME, FUEL_CHARCOAL, FUEL_COAL,
    FUEL_LAVA_BUCKET, FUEL_STICK, FUEL_WOOD,
};
pub use registry::{CraftingStation, Ingredient, Recipe, RecipeRegistry};
