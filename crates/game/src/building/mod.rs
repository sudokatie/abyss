//! Block placement and breaking systems.

mod base;
mod placement;

pub use base::{BaseCompartment, PressureWindow};
pub use placement::BlockInteraction;
