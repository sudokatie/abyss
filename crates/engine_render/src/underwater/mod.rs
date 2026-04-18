//! Underwater rendering effects.
//!
//! Implements spec 2.4 and 10.1: depth-based fog, color shift,
//! caustic lighting, and particle effects for underwater environments.

mod caustics;
mod color_shift;
mod depth_fog;
pub mod particles;

pub use caustics::CausticRenderer;
pub use color_shift::ColorShiftCalculator;
pub use depth_fog::{DepthFogCalculator, DepthFogConfig};
pub use particles::{
    BioluminescenceLevel, BubbleEmitter, BubbleParticle, PlanktonEmitter, PlanktonParticle,
    SedimentEmitter, SedimentParticle,
};
