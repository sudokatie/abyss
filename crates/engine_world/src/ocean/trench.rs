//! Procedural ocean trench generation.
//!
//! Generates a main trench running N-S with increasing depth,
//! side branches, caves, air pockets, and thermal vent placement.

use noise::{Perlin, NoiseFn};

/// Default seed for trench generation.
pub const TRENCH_DEFAULT_SEED: u32 = 42;

/// Main trench width at the surface (blocks).
pub const TRENCH_SURFACE_WIDTH: f64 = 40.0;

/// Maximum trench depth (meters).
pub const TRENCH_MAX_DEPTH: f64 = 1200.0;

/// How quickly the trench narrows with depth.
pub const TRENCH_NARROW_RATE: f64 = 0.02;

/// Probability of a side branch per depth segment.
pub const BRANCH_PROBABILITY: f64 = 0.3;

/// Minimum branch length (blocks).
pub const BRANCH_MIN_LENGTH: f64 = 10.0;

/// Maximum branch length (blocks).
pub const BRANCH_MAX_LENGTH: f64 = 40.0;

/// Thermal vent probability per 100 blocks of depth.
pub const VENT_PROBABILITY: f64 = 0.15;

/// Air pocket probability per branch.
pub const AIR_POCKET_PROBABILITY: f64 = 0.25;

/// A point of interest within the trench.
#[derive(Debug, Clone)]
pub enum TrenchFeature {
    /// A thermal vent at a position.
    ThermalVent { x: i32, y: i32, z: i32 },
    /// An air pocket in a cave.
    AirPocket { x: i32, y: i32, z: i32, radius: f64 },
    /// A side branch cave entrance.
    CaveEntrance { x: i32, y: i32, z: i32, direction: BranchDirection },
}

/// Direction a branch cave extends from the main trench.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BranchDirection {
    East,
    West,
}

/// A generated side branch.
#[derive(Debug, Clone)]
pub struct Branch {
    /// Entrance position.
    pub entrance: [i32; 3],
    /// Direction from main trench.
    pub direction: BranchDirection,
    /// Length of the branch.
    pub length: f64,
    /// Whether this branch has an air pocket.
    pub has_air_pocket: bool,
    /// Air pocket position (if present).
    pub air_pocket_pos: Option<[i32; 3]>,
    /// Whether this branch is a dead end.
    pub is_dead_end: bool,
}

/// Configuration for trench generation.
#[derive(Debug, Clone)]
pub struct TrenchConfig {
    /// Noise seed.
    pub seed: u32,
    /// Surface width of the trench.
    pub surface_width: f64,
    /// Maximum depth.
    pub max_depth: f64,
    /// Narrowing rate.
    pub narrow_rate: f64,
    /// Branch probability per segment.
    pub branch_probability: f64,
    /// Thermal vent probability.
    pub vent_probability: f64,
    /// Air pocket probability per branch.
    pub air_pocket_probability: f64,
}

impl Default for TrenchConfig {
    fn default() -> Self {
        Self {
            seed: TRENCH_DEFAULT_SEED,
            surface_width: TRENCH_SURFACE_WIDTH,
            max_depth: TRENCH_MAX_DEPTH,
            narrow_rate: TRENCH_NARROW_RATE,
            branch_probability: BRANCH_PROBABILITY,
            vent_probability: VENT_PROBABILITY,
            air_pocket_probability: AIR_POCKET_PROBABILITY,
        }
    }
}

/// Procedural trench generator.
#[derive(Debug)]
pub struct TrenchGenerator {
    config: TrenchConfig,
    noise: Perlin,
    width_noise: Perlin,
    branch_noise: Perlin,
}

impl TrenchGenerator {
    /// Create a new trench generator with default config.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(TrenchConfig::default())
    }

    /// Create a trench generator with custom config.
    #[must_use]
    pub fn with_config(config: TrenchConfig) -> Self {
        let seed = config.seed;
        Self {
            config,
            noise: Perlin::new(seed),
            width_noise: Perlin::new(seed.wrapping_add(100)),
            branch_noise: Perlin::new(seed.wrapping_add(200)),
        }
    }

    /// Get the width of the trench at a given depth.
    #[must_use]
    pub fn width_at_depth(&self, depth: f64) -> f64 {
        if depth <= 0.0 {
            return self.config.surface_width;
        }
        let depth_frac = (depth / self.config.max_depth).min(1.0);
        let base_width = self.config.surface_width * (1.0 - self.config.narrow_rate * depth_frac * 10.0);
        // Add noise variation
        let noise_val = self.width_noise.get([0.0, depth * 0.01]);
        let width = (base_width + noise_val * 8.0).max(6.0);
        width
    }

    /// Check if a position is inside the main trench.
    #[must_use]
    pub fn is_in_trench(&self, x: i32, y: i32, _z: i32) -> bool {
        let depth = (-y) as f64;
        if depth < 0.0 || depth > self.config.max_depth {
            return false;
        }
        let width = self.width_at_depth(depth);
        let half_w = width / 2.0;
        // Add meandering with depth
        let center_offset = self.noise.get([0.0, depth * 0.005]) * 20.0;
        let dist_from_center = (x as f64 - center_offset).abs();
        dist_from_center < half_w
    }

    /// Generate branches for a depth range.
    #[must_use]
    pub fn generate_branches(&self, start_depth: f64, end_depth: f64, trench_center_x: i32) -> Vec<Branch> {
        let mut branches = Vec::new();
        let depth_step = 20.0; // Check every 20m

        let mut depth = start_depth;
        let mut segment_index: u64 = 0;
        while depth < end_depth {
            segment_index += 1;
            let noise_val = self.branch_noise.get([depth * 0.05, 0.0]);

            // Use a simple hash-based check: place a branch every N segments on average
            // based on branch_probability. Higher probability = more frequent branches.
            let interval = (1.0 / self.config.branch_probability).max(1.0) as u64;
            if segment_index % interval == 0 {
                let direction = if noise_val > 0.0 {
                    BranchDirection::East
                } else {
                    BranchDirection::West
                };

                let length = BRANCH_MIN_LENGTH
                    + (BRANCH_MAX_LENGTH - BRANCH_MIN_LENGTH)
                    * ((noise_val + 1.0) / 2.0).abs();

                let y = -(depth as i32);
                let entrance_x = match direction {
                    BranchDirection::East => trench_center_x + (self.width_at_depth(depth) / 2.0) as i32,
                    BranchDirection::West => trench_center_x - (self.width_at_depth(depth) / 2.0) as i32,
                };

                // Air pocket determined by separate noise value
                let pocket_noise = self.branch_noise.get([depth * 0.07, 50.0]);
                let has_pocket = (pocket_noise + 1.0) / 2.0 < self.config.air_pocket_probability as f64 * 2.0;
                let air_pocket_pos = if has_pocket {
                    let pocket_x = match direction {
                        BranchDirection::East => entrance_x + (length * 0.5) as i32,
                        BranchDirection::West => entrance_x - (length * 0.5) as i32,
                    };
                    Some([pocket_x, y, 0])
                } else {
                    None
                };

                branches.push(Branch {
                    entrance: [entrance_x, y, 0],
                    direction,
                    length,
                    has_air_pocket: has_pocket,
                    air_pocket_pos,
                    is_dead_end: true,
                });
            }
            depth += depth_step;
        }

        branches
    }

    /// Generate thermal vent positions along the trench.
    #[must_use]
    pub fn generate_vents(&self, start_depth: f64, end_depth: f64, trench_center_x: i32) -> Vec<[i32; 3]> {
        let mut vents = Vec::new();
        let depth_step = 50.0; // Check every 50m

        let mut depth = start_depth.max(100.0); // No vents above 100m
        let mut segment_index: u64 = 0;
        while depth < end_depth {
            segment_index += 1;
            let noise_val = self.noise.get([depth * 0.03, 100.0]);

            let interval = (1.0 / self.config.vent_probability).max(1.0) as u64;
            if segment_index % interval == 0 {
                let x_offset = (self.width_at_depth(depth) * 0.3 * noise_val) as i32;
                let x = trench_center_x + x_offset;
                let y = -(depth as i32);
                vents.push([x, y, 0]);
            }
            depth += depth_step;
        }
        vents
    }

    /// Get the depth at a position (based on trench shape).
    /// Returns None if position is outside the trench.
    #[must_use]
    pub fn depth_at(&self, x: i32, y: i32, _z: i32) -> Option<f64> {
        if self.is_in_trench(x, y, _z) {
            Some((-y) as f64)
        } else {
            None
        }
    }

    /// Check if a branch position is traversable (not a dead-end wall).
    #[must_use]
    pub fn is_branch_traversable(&self, branch: &Branch, offset_from_entrance: f64) -> bool {
        offset_from_entrance >= 0.0 && offset_from_entrance <= branch.length
    }

    /// Get the trench center X at a given depth (accounting for meandering).
    #[must_use]
    pub fn center_x_at_depth(&self, depth: f64) -> f64 {
        self.noise.get([0.0, depth * 0.005]) * 20.0
    }
}

impl Default for TrenchGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_width_at_surface() {
        let trench = TrenchGenerator::new();
        let width = trench.width_at_depth(0.0);
        assert!(width >= TRENCH_SURFACE_WIDTH * 0.5);
    }

    #[test]
    fn test_width_narrows_with_depth() {
        let trench = TrenchGenerator::new();
        let surface_width = trench.width_at_depth(0.0);
        let deep_width = trench.width_at_depth(800.0);
        assert!(deep_width <= surface_width);
    }

    #[test]
    fn test_width_minimum() {
        let trench = TrenchGenerator::new();
        let width = trench.width_at_depth(TRENCH_MAX_DEPTH);
        assert!(width >= 6.0);
    }

    #[test]
    fn test_is_in_trench_center() {
        let trench = TrenchGenerator::new();
        assert!(trench.is_in_trench(0, -100, 0));
    }

    #[test]
    fn test_not_in_trench_far_away() {
        let trench = TrenchGenerator::new();
        assert!(!trench.is_in_trench(500, -100, 0));
    }

    #[test]
    fn test_not_in_trench_above_surface() {
        let trench = TrenchGenerator::new();
        assert!(!trench.is_in_trench(0, 10, 0));
    }

    #[test]
    fn test_generate_branches() {
        let trench = TrenchGenerator::new();
        let branches = trench.generate_branches(100.0, 800.0, 0);
        assert!(!branches.is_empty());
    }

    #[test]
    fn test_generate_vents() {
        let trench = TrenchGenerator::new();
        let vents = trench.generate_vents(100.0, 800.0, 0);
        // Should have some vents in deep range
        assert!(!vents.is_empty());
    }

    #[test]
    fn test_vents_only_deep() {
        let trench = TrenchGenerator::new();
        let shallow_vents = trench.generate_vents(0.0, 100.0, 0);
        let deep_vents = trench.generate_vents(100.0, 800.0, 0);
        assert!(deep_vents.len() >= shallow_vents.len());
    }

    #[test]
    fn test_branch_has_air_pocket() {
        let trench = TrenchGenerator::new();
        let branches = trench.generate_branches(200.0, 800.0, 0);
        let with_pocket = branches.iter().any(|b| b.has_air_pocket);
        // At least some branches should have air pockets
        assert!(with_pocket || branches.is_empty());
    }

}
