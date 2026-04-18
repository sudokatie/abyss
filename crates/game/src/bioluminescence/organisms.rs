//! Bioluminescent organisms.
//!
//! Living light sources that glow based on depth and activity.
//! Harvesting for bio light organs.

use crate::world::ResourceType;

/// Glow intensity for bio organisms.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GlowIntensity {
    /// Dim, barely visible.
    Dim,
    /// Moderate, clearly visible.
    Moderate,
    /// Bright, illuminates surroundings.
    Bright,
}

impl GlowIntensity {
    /// Intensity value 0.0 to 1.0.
    #[must_use]
    pub fn value(&self) -> f32 {
        match self {
            GlowIntensity::Dim => 0.2,
            GlowIntensity::Moderate => 0.5,
            GlowIntensity::Bright => 0.9,
        }
    }

    /// Determine intensity from depth.
    #[must_use]
    pub fn from_depth(depth: f32) -> Self {
        if depth < 200.0 {
            GlowIntensity::Dim
        } else if depth < 500.0 {
            GlowIntensity::Moderate
        } else {
            GlowIntensity::Bright
        }
    }
}

/// Type of bioluminescent organism.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrganismType {
    /// Glowing algae on surfaces.
    GlowAlgae,
    /// Deep sea fungus.
    DeepShroom,
    /// Floating light orb.
    LightOrb,
    /// Pulse worm in sediment.
    PulseWorm,
}

impl OrganismType {
    /// Human-readable name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            OrganismType::GlowAlgae => "Glow Algae",
            OrganismType::DeepShroom => "Deep Shroom",
            OrganismType::LightOrb => "Light Orb",
            OrganismType::PulseWorm => "Pulse Worm",
        }
    }

    /// Minimum depth for this organism.
    #[must_use]
    pub fn min_depth(&self) -> f32 {
        match self {
            OrganismType::GlowAlgae => 100.0,
            OrganismType::DeepShroom => 300.0,
            OrganismType::LightOrb => 200.0,
            OrganismType::PulseWorm => 400.0,
        }
    }

    /// Maximum depth.
    #[must_use]
    pub fn max_depth(&self) -> f32 {
        match self {
            OrganismType::GlowAlgae => 600.0,
            OrganismType::DeepShroom => 1500.0,
            OrganismType::LightOrb => 800.0,
            OrganismType::PulseWorm => 2000.0,
        }
    }

    /// Light radius in blocks.
    #[must_use]
    pub fn light_radius(&self) -> f32 {
        match self {
            OrganismType::GlowAlgae => 3.0,
            OrganismType::DeepShroom => 5.0,
            OrganismType::LightOrb => 8.0,
            OrganismType::PulseWorm => 2.0,
        }
    }

    /// Whether this organism flickers.
    #[must_use]
    pub fn flickers(&self) -> bool {
        match self {
            OrganismType::LightOrb | OrganismType::PulseWorm => true,
            _ => false,
        }
    }

    /// Glow color (RGB).
    #[must_use]
    pub fn glow_color(&self) -> [f32; 3] {
        match self {
            OrganismType::GlowAlgae => [0.1, 0.9, 0.6],    // Green-cyan
            OrganismType::DeepShroom => [0.3, 0.2, 0.9],    // Purple-blue
            OrganismType::LightOrb => [0.2, 1.0, 0.8],      // Cyan
            OrganismType::PulseWorm => [0.9, 0.3, 0.5],     // Pink-red
        }
    }

    /// Resource obtained from harvesting.
    #[must_use]
    pub fn harvest_drop(&self) -> ResourceType {
        ResourceType::BioluminescentOrgan
    }
}

/// A placed bioluminescent organism in the world.
#[derive(Debug, Clone)]
pub struct BioOrganism {
    pub organism_type: OrganismType,
    pub position: [f32; 3],
    pub glow_intensity: GlowIntensity,
    /// Current flicker phase.
    pub flicker_phase: f32,
    /// Whether already harvested.
    pub harvested: bool,
}

impl BioOrganism {
    /// Create a new bio organism at a position.
    #[must_use]
    pub fn new(organism_type: OrganismType, position: [f32; 3]) -> Self {
        let depth = -position[1];
        Self {
            organism_type,
            position,
            glow_intensity: GlowIntensity::from_depth(depth),
            flicker_phase: 0.0,
            harvested: false,
        }
    }

    /// Current effective glow (accounting for flicker).
    #[must_use]
    pub fn effective_glow(&self) -> f32 {
        let base = self.glow_intensity.value();
        if self.harvested {
            return 0.0;
        }
        if self.organism_type.flickers() {
            let flicker = 0.7 + 0.3 * (self.flicker_phase * 2.5).sin();
            base * flicker
        } else {
            base
        }
    }

    /// Update flicker animation.
    pub fn update(&mut self, delta: f32) {
        self.flicker_phase += delta;
    }

    /// Harvest this organism, returning the resource.
    pub fn harvest(&mut self) -> Option<ResourceType> {
        if self.harvested {
            return None;
        }
        self.harvested = true;
        Some(self.organism_type.harvest_drop())
    }

    /// Check if this organism can spawn at a given depth.
    #[must_use]
    pub fn can_spawn_at_depth(depth: f32, organism_type: OrganismType) -> bool {
        depth >= organism_type.min_depth() && depth <= organism_type.max_depth()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glow_intensity_from_depth() {
        assert_eq!(GlowIntensity::from_depth(100.0), GlowIntensity::Dim);
        assert_eq!(GlowIntensity::from_depth(300.0), GlowIntensity::Moderate);
        assert_eq!(GlowIntensity::from_depth(600.0), GlowIntensity::Bright);
    }

    #[test]
    fn test_organism_creation() {
        let org = BioOrganism::new(OrganismType::LightOrb, [0.0, -300.0, 0.0]);
        assert!(!org.harvested);
        assert!(org.effective_glow() > 0.0);
    }

    #[test]
    fn test_harvest() {
        let mut org = BioOrganism::new(OrganismType::DeepShroom, [0.0, -500.0, 0.0]);
        let drop = org.harvest();
        assert!(drop.is_some());
        assert!(org.harvested);
        assert_relative_eq!(org.effective_glow(), 0.0);
    }

    #[test]
    fn test_double_harvest() {
        let mut org = BioOrganism::new(OrganismType::GlowAlgae, [0.0, -200.0, 0.0]);
        org.harvest();
        let second = org.harvest();
        assert!(second.is_none());
    }

    #[test]
    fn test_flicker_varies() {
        let mut org = BioOrganism::new(OrganismType::LightOrb, [0.0, -300.0, 0.0]);
        let g1 = org.effective_glow();
        org.flicker_phase += 1.0;
        let g2 = org.effective_glow();
        // Flicker should change intensity
        assert!(g1 > 0.0 && g2 > 0.0);
    }

    #[test]
    fn test_spawn_depth_check() {
        assert!(BioOrganism::can_spawn_at_depth(300.0, OrganismType::GlowAlgae));
        assert!(!BioOrganism::can_spawn_at_depth(50.0, OrganismType::GlowAlgae));
    }

    use approx::assert_relative_eq;
}
