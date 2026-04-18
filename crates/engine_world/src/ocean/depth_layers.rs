//! Ocean depth layer system.
//!
//! Five depth zones with distinct properties: fog, color, sounds, temperature.
//! Smooth transitions between zones.

/// Sunlight zone: 0-200m. Warm, bright, most life.
pub const SUNLIGHT_MAX: f32 = 200.0;

/// Twilight zone: 200-500m. Dimming light, transition.
pub const TWILIGHT_MAX: f32 = 500.0;

/// Midnight zone: 500-1000m. No sunlight, bioluminescence.
pub const MIDNIGHT_MAX: f32 = 1000.0;

/// Abyssal zone: 1000-6000m. Near-freezing, extreme pressure.
pub const ABYSSAL_MAX: f32 = 6000.0;

/// Hadal zone: 6000m+. Deepest trenches.
/// (In practice, most gameplay won't reach here.)

/// Transition distance between zones (meters).
/// Properties blend over this range at zone boundaries.
pub const ZONE_TRANSITION_WIDTH: f32 = 20.0;

/// Depth zone identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DepthZone {
    /// 0-200m: Sunlight, warm, bright
    Sunlight,
    /// 200-500m: Twilight, dimming
    Twilight,
    /// 500-1000m: Midnight, dark, bioluminescent
    Midnight,
    /// 1000-6000m: Abyssal, near-freezing
    Abyssal,
    /// 6000m+: Hadal, extreme
    Hadal,
}

impl DepthZone {
    /// Determine zone from depth.
    #[must_use]
    pub fn from_depth(depth: f32) -> Self {
        if depth < SUNLIGHT_MAX {
            DepthZone::Sunlight
        } else if depth < TWILIGHT_MAX {
            DepthZone::Twilight
        } else if depth < MIDNIGHT_MAX {
            DepthZone::Midnight
        } else if depth < ABYSSAL_MAX {
            DepthZone::Abyssal
        } else {
            DepthZone::Hadal
        }
    }

    /// Human-readable zone name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            DepthZone::Sunlight => "Sunlight Zone",
            DepthZone::Twilight => "Twilight Zone",
            DepthZone::Midnight => "Midnight Zone",
            DepthZone::Abyssal => "Abyssal Zone",
            DepthZone::Hadal => "Hadal Zone",
        }
    }

    /// Minimum depth for this zone.
    #[must_use]
    pub fn min_depth(&self) -> f32 {
        match self {
            DepthZone::Sunlight => 0.0,
            DepthZone::Twilight => SUNLIGHT_MAX,
            DepthZone::Midnight => TWILIGHT_MAX,
            DepthZone::Abyssal => MIDNIGHT_MAX,
            DepthZone::Hadal => ABYSSAL_MAX,
        }
    }

    /// Maximum depth for this zone.
    #[must_use]
    pub fn max_depth(&self) -> f32 {
        match self {
            DepthZone::Sunlight => SUNLIGHT_MAX,
            DepthZone::Twilight => TWILIGHT_MAX,
            DepthZone::Midnight => MIDNIGHT_MAX,
            DepthZone::Abyssal => ABYSSAL_MAX,
            DepthZone::Hadal => f32::MAX,
        }
    }

    /// Ambient light level (0.0 to 1.0).
    #[must_use]
    pub fn ambient_light(&self) -> f32 {
        match self {
            DepthZone::Sunlight => 1.0,
            DepthZone::Twilight => 0.3,
            DepthZone::Midnight => 0.02,
            DepthZone::Abyssal => 0.005,
            DepthZone::Hadal => 0.0,
        }
    }

    /// Water temperature in Celsius.
    #[must_use]
    pub fn temperature(&self) -> f32 {
        match self {
            DepthZone::Sunlight => 20.0,
            DepthZone::Twilight => 10.0,
            DepthZone::Midnight => 4.0,
            DepthZone::Abyssal => 2.0,
            DepthZone::Hadal => 1.0,
        }
    }

    /// Fog density multiplier.
    #[must_use]
    pub fn fog_density(&self) -> f32 {
        match self {
            DepthZone::Sunlight => 0.2,
            DepthZone::Twilight => 0.5,
            DepthZone::Midnight => 0.8,
            DepthZone::Abyssal => 0.95,
            DepthZone::Hadal => 1.0,
        }
    }

    /// Primary color palette (RGB, 0-1).
    #[must_use]
    pub fn color_palette(&self) -> [f32; 3] {
        match self {
            DepthZone::Sunlight => [0.1, 0.6, 0.8],   // Bright cyan-blue
            DepthZone::Twilight => [0.05, 0.3, 0.5],  // Dark blue
            DepthZone::Midnight => [0.02, 0.1, 0.2],   // Near-black blue
            DepthZone::Abyssal => [0.01, 0.05, 0.1],   // Very dark
            DepthZone::Hadal => [0.0, 0.02, 0.05],     // Almost black
        }
    }
}

/// Properties at a specific depth, with smooth transitions between zones.
#[derive(Debug, Clone)]
pub struct DepthProperties {
    /// Current depth zone.
    pub zone: DepthZone,
    /// Ambient light (0-1, interpolated at boundaries).
    pub ambient_light: f32,
    /// Temperature in Celsius (interpolated).
    pub temperature: f32,
    /// Fog density (0-1, interpolated).
    pub fog_density: f32,
    /// Color palette (interpolated).
    pub color: [f32; 3],
    /// Transition factor (0 = fully in current zone, 1 = at next zone boundary).
    pub transition_factor: f32,
}

impl DepthProperties {
    /// Calculate depth properties with smooth zone transitions.
    #[must_use]
    pub fn at_depth(depth: f32) -> Self {
        let zone = DepthZone::from_depth(depth);
        let next_zone = next_zone(zone);

        let (ambient_light, temperature, fog_density, color, transition_factor) =
            if let Some(next) = next_zone {
                let zone_max = zone.max_depth();
                if depth >= zone_max - ZONE_TRANSITION_WIDTH {
                    // In transition zone
                    let t = (depth - (zone_max - ZONE_TRANSITION_WIDTH)) / ZONE_TRANSITION_WIDTH;
                    let t = t.clamp(0.0, 1.0);

                    let ambient = lerp(zone.ambient_light(), next.ambient_light(), t);
                    let temp = lerp(zone.temperature(), next.temperature(), t);
                    let fog = lerp(zone.fog_density(), next.fog_density(), t);
                    let col = lerp_color(zone.color_palette(), next.color_palette(), t);

                    (ambient, temp, fog, col, t)
                } else {
                    (zone.ambient_light(), zone.temperature(), zone.fog_density(), zone.color_palette(), 0.0)
                }
            } else {
                (zone.ambient_light(), zone.temperature(), zone.fog_density(), zone.color_palette(), 0.0)
            };

        Self {
            zone,
            ambient_light,
            temperature,
            fog_density,
            color,
            transition_factor,
        }
    }
}

/// Get the next deeper zone (None for Hadal).
fn next_zone(zone: DepthZone) -> Option<DepthZone> {
    match zone {
        DepthZone::Sunlight => Some(DepthZone::Twilight),
        DepthZone::Twilight => Some(DepthZone::Midnight),
        DepthZone::Midnight => Some(DepthZone::Abyssal),
        DepthZone::Abyssal => Some(DepthZone::Hadal),
        DepthZone::Hadal => None,
    }
}

/// Linear interpolation.
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Linear interpolation for RGB color.
fn lerp_color(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [lerp(a[0], b[0], t), lerp(a[1], b[1], t), lerp(a[2], b[2], t)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sunlight_zone() {
        assert_eq!(DepthZone::from_depth(0.0), DepthZone::Sunlight);
        assert_eq!(DepthZone::from_depth(100.0), DepthZone::Sunlight);
        assert_eq!(DepthZone::from_depth(199.0), DepthZone::Sunlight);
    }

    #[test]
    fn test_twilight_zone() {
        assert_eq!(DepthZone::from_depth(200.0), DepthZone::Twilight);
        assert_eq!(DepthZone::from_depth(350.0), DepthZone::Twilight);
    }

    #[test]
    fn test_midnight_zone() {
        assert_eq!(DepthZone::from_depth(500.0), DepthZone::Midnight);
        assert_eq!(DepthZone::from_depth(750.0), DepthZone::Midnight);
    }

    #[test]
    fn test_abyssal_zone() {
        assert_eq!(DepthZone::from_depth(1000.0), DepthZone::Abyssal);
        assert_eq!(DepthZone::from_depth(3000.0), DepthZone::Abyssal);
    }

    #[test]
    fn test_hadal_zone() {
        assert_eq!(DepthZone::from_depth(6000.0), DepthZone::Hadal);
        assert_eq!(DepthZone::from_depth(10000.0), DepthZone::Hadal);
    }

    #[test]
    fn test_zone_names() {
        assert_eq!(DepthZone::Sunlight.name(), "Sunlight Zone");
        assert_eq!(DepthZone::Hadal.name(), "Hadal Zone");
    }

    #[test]
    fn test_properties_mid_zone() {
        let props = DepthProperties::at_depth(100.0);
        assert_eq!(props.zone, DepthZone::Sunlight);
        assert_relative_eq!(props.transition_factor, 0.0);
    }

    #[test]
    fn test_properties_transition() {
        let props = DepthProperties::at_depth(190.0);
        assert_eq!(props.zone, DepthZone::Sunlight);
        assert!(props.transition_factor > 0.0);
    }

    use approx::assert_relative_eq;
}
