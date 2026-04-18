//! Ocean world generation.
//!
//! Procedural trench, depth layers, and underwater world features.

mod depth_layers;
mod trench;

pub use depth_layers::{DepthProperties, DepthZone};
pub use trench::{TrenchConfig, TrenchGenerator};
