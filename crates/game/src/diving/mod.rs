//! Diving movement and decompression system.
//!
//! Implements spec 2.3: 3D swimming with buoyancy and current,
//! and spec 2.2: decompression sickness on fast ascent.

mod decompression;
mod movement;

pub use decompression::{DecompressionSickness, SafeAscentRate};
pub use movement::{BuoyancyState, DivingMovement, SwimSpeedModifier};
