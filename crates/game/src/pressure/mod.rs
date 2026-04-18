//! Pressure and depth zone system for underwater survival.
//!
//! Implements spec 2.2: five depth zones with increasing pressure damage,
//! pressure suits, suit integrity, and decompression sickness.

mod suit;
mod zones;

pub use suit::{PressureSuit, SuitTier};
pub use zones::{DepthZone, PressureZones};
