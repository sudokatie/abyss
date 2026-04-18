//! Underwater rendering effects.
//!
//! Implements spec 2.4 and 10.1: depth-based fog, color shift,
//! and caustic lighting for underwater environments.

mod caustics;
mod color_shift;
mod depth_fog;

pub use caustics::CausticRenderer;
pub use color_shift::ColorShiftCalculator;
pub use depth_fog::{DepthFogCalculator, DepthFogConfig};
