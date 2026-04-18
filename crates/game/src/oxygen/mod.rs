//! Oxygen management system for underwater survival.
//!
//! Implements spec 2.1: oxygen bar with depth-based drain,
//! oxygen tanks, equipment modifiers, and drowning damage.

mod supply;
mod tank;

pub use supply::{OxygenRefillSource, OxygenSupply, OXYGEN_BAR_BASE, OXYGEN_DROWNING_DAMAGE};
pub use tank::{OxygenTank, TankSlot};
