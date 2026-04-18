//! Abyss deep sea survival game client.
//!
//! Core game logic including ECS components, systems, and entity management.
//! Built on the Lattice engine with underwater survival extensions.

pub mod ai;
pub mod bioluminescence;
pub mod building;
pub mod crafting;
pub mod creatures;
pub mod diving;
pub mod ecs;
pub mod entities;
pub mod inventory;
pub mod networking;
pub mod oxygen;
pub mod pressure;
pub mod survival;
pub mod world;

#[cfg(test)]
mod integration_tests;
