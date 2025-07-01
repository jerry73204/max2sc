//! Distance-based effects and early reflections converter

use max2sc_core::{Result, SCObject, SCValue};

/// Effects converter for spatial audio effects
pub struct EffectsConverter;

impl EffectsConverter {
    /// Generate distance-based effects (air absorption, Doppler, etc.)
    pub fn generate_distance_effects(effect_type: DistanceEffectType) -> Result<SCObject> {
        match effect_type {
            DistanceEffectType::AirAbsorption => Self::generate_air_absorption(),
            DistanceEffectType::DopplerShift => Self::generate_doppler_shift(),
            DistanceEffectType::DistanceAttenuation => Self::generate_distance_attenuation(),
            DistanceEffectType::Combined => Self::generate_combined_distance_effects(),
        }
    }

    /// Generate air absorption filter based on distance
    fn generate_air_absorption() -> Result<SCObject> {
        Ok(SCObject::new("AirAbsorption")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("distance".to_string()))
            .arg(SCValue::Symbol("humidity".to_string()))
            .arg(SCValue::Symbol("temperature".to_string()))
            .prop("frequency_bands", 8)
            .prop("reference_distance", 1.0)
            .prop("comment", "Distance-based air absorption filter"))
    }

    /// Generate Doppler shift effect
    fn generate_doppler_shift() -> Result<SCObject> {
        Ok(SCObject::new("DopplerShift")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("source_velocity".to_string()))
            .arg(SCValue::Symbol("listener_velocity".to_string()))
            .arg(343.0) // speed of sound
            .prop("max_shift", 2.0)
            .prop("interpolation", "linear")
            .prop("comment", "Doppler frequency shift effect"))
    }

    /// Generate distance-based amplitude attenuation
    fn generate_distance_attenuation() -> Result<SCObject> {
        Ok(SCObject::new("DistanceAttenuation")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("distance".to_string()))
            .arg(1.0) // reference distance
            .arg(SCValue::Symbol("attenuation_law".to_string())) // inverse, inverse_square, custom
            .prop("min_distance", 0.1)
            .prop("max_distance", 100.0)
            .prop("comment", "Distance-based amplitude attenuation"))
    }

    /// Generate combined distance effects
    fn generate_combined_distance_effects() -> Result<SCObject> {
        Ok(SCObject::new("CombinedDistanceEffects")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("distance".to_string()))
            .arg(SCValue::Symbol("velocity".to_string()))
            .prop("air_absorption", true)
            .prop("doppler_shift", true)
            .prop("distance_attenuation", true)
            .prop("comment", "Combined distance-based effects"))
    }

    /// Generate early reflections network
    pub fn generate_early_reflections(reflection_type: EarlyReflectionType) -> Result<SCObject> {
        match reflection_type {
            EarlyReflectionType::Room => Self::generate_room_reflections(),
            EarlyReflectionType::Hall => Self::generate_hall_reflections(),
            EarlyReflectionType::Cathedral => Self::generate_cathedral_reflections(),
            EarlyReflectionType::Custom => Self::generate_custom_reflections(),
        }
    }

    /// Generate room-style early reflections
    fn generate_room_reflections() -> Result<SCObject> {
        Ok(SCObject::new("EarlyReflectionsRoom")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("room_size".to_string()))
            .arg(SCValue::Symbol("damping".to_string()))
            .arg(SCValue::Symbol("source_position".to_string()))
            .arg(SCValue::Symbol("listener_position".to_string()))
            .prop("num_reflections", 12)
            .prop("max_delay", 0.08) // 80ms
            .prop("reflection_pattern", "rectangular")
            .prop("comment", "Room-style early reflections"))
    }

    /// Generate hall-style early reflections
    fn generate_hall_reflections() -> Result<SCObject> {
        Ok(SCObject::new("EarlyReflectionsHall")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("hall_length".to_string()))
            .arg(SCValue::Symbol("hall_width".to_string()))
            .arg(SCValue::Symbol("hall_height".to_string()))
            .arg(SCValue::Symbol("damping".to_string()))
            .prop("num_reflections", 24)
            .prop("max_delay", 0.15) // 150ms
            .prop("reflection_pattern", "shoebox")
            .prop("comment", "Concert hall early reflections"))
    }

    /// Generate cathedral-style early reflections
    fn generate_cathedral_reflections() -> Result<SCObject> {
        Ok(SCObject::new("EarlyReflectionsCathedral")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("space_size".to_string()))
            .arg(SCValue::Symbol("stone_absorption".to_string()))
            .arg(SCValue::Symbol("height".to_string()))
            .prop("num_reflections", 36)
            .prop("max_delay", 0.25) // 250ms
            .prop("reflection_pattern", "complex")
            .prop("diffusion", true)
            .prop("comment", "Cathedral-style early reflections"))
    }

    /// Generate custom early reflections
    fn generate_custom_reflections() -> Result<SCObject> {
        Ok(SCObject::new("EarlyReflectionsCustom")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("delay_times".to_string()))
            .arg(SCValue::Symbol("gains".to_string()))
            .arg(SCValue::Symbol("filter_freqs".to_string()))
            .arg(SCValue::Symbol("pan_positions".to_string()))
            .prop("interpolation", "cubic")
            .prop("modulation", true)
            .prop("comment", "Custom early reflections network"))
    }

    /// Generate wall reflection simulation
    pub fn generate_wall_reflection(
        wall_material: WallMaterial,
        angle_of_incidence: f32,
    ) -> Result<SCObject> {
        let absorption_coeff = match wall_material {
            WallMaterial::Concrete => 0.02,
            WallMaterial::Wood => 0.08,
            WallMaterial::Carpet => 0.35,
            WallMaterial::Glass => 0.04,
            WallMaterial::Curtain => 0.75,
        };

        Ok(SCObject::new("WallReflection")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("delay_time".to_string()))
            .arg(angle_of_incidence)
            .arg(absorption_coeff)
            .prop("material", format!("{wall_material:?}"))
            .prop("frequency_dependent", true)
            .prop("comment", format!("Wall reflection: {wall_material:?}")))
    }

    /// Generate diffuse reflection field
    pub fn generate_diffuse_reflections(diffusion_amount: f32) -> Result<SCObject> {
        Ok(SCObject::new("DiffuseReflections")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(diffusion_amount)
            .arg(SCValue::Symbol("room_size".to_string()))
            .arg(SCValue::Symbol("scattering_coefficient".to_string()))
            .prop("num_taps", 64)
            .prop("density", "high")
            .prop("modulation_depth", 0.02)
            .prop("comment", "Diffuse reflection field"))
    }

    /// Generate frequency-dependent absorption
    pub fn generate_frequency_absorption(material: AbsorptionMaterial) -> Result<SCObject> {
        let (low_abs, mid_abs, high_abs) = match material {
            AbsorptionMaterial::Air => (0.001, 0.002, 0.008),
            AbsorptionMaterial::Foam => (0.1, 0.8, 0.9),
            AbsorptionMaterial::Fabric => (0.05, 0.3, 0.7),
            AbsorptionMaterial::Wood => (0.1, 0.15, 0.2),
            AbsorptionMaterial::Stone => (0.01, 0.02, 0.03),
        };

        Ok(SCObject::new("FrequencyAbsorption")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(low_abs) // 125 Hz
            .arg(mid_abs) // 1 kHz
            .arg(high_abs) // 8 kHz
            .prop("material", format!("{material:?}"))
            .prop("octave_bands", 8)
            .prop("comment", format!("Frequency absorption: {material:?}")))
    }
}

/// Distance effect types
#[derive(Debug, Clone, Copy)]
pub enum DistanceEffectType {
    AirAbsorption,
    DopplerShift,
    DistanceAttenuation,
    Combined,
}

/// Early reflection types
#[derive(Debug, Clone, Copy)]
pub enum EarlyReflectionType {
    Room,
    Hall,
    Cathedral,
    Custom,
}

/// Wall material types for reflection simulation
#[derive(Debug, Clone, Copy)]
pub enum WallMaterial {
    Concrete,
    Wood,
    Carpet,
    Glass,
    Curtain,
}

/// Absorption material types
#[derive(Debug, Clone, Copy)]
pub enum AbsorptionMaterial {
    Air,
    Foam,
    Fabric,
    Wood,
    Stone,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_air_absorption_generation() {
        let result = EffectsConverter::generate_distance_effects(DistanceEffectType::AirAbsorption);

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "AirAbsorption");
        assert_eq!(obj.method, Some("ar".to_string()));
    }

    #[test]
    fn test_doppler_shift_generation() {
        let result = EffectsConverter::generate_distance_effects(DistanceEffectType::DopplerShift);

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "DopplerShift");
        assert_eq!(obj.method, Some("ar".to_string()));
    }

    #[test]
    fn test_room_reflections_generation() {
        let result = EffectsConverter::generate_early_reflections(EarlyReflectionType::Room);

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "EarlyReflectionsRoom");
        assert_eq!(obj.method, Some("ar".to_string()));
    }

    #[test]
    fn test_hall_reflections_generation() {
        let result = EffectsConverter::generate_early_reflections(EarlyReflectionType::Hall);

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "EarlyReflectionsHall");
        assert_eq!(obj.method, Some("ar".to_string()));
    }

    #[test]
    fn test_wall_reflection_generation() {
        let result = EffectsConverter::generate_wall_reflection(WallMaterial::Concrete, 45.0);

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "WallReflection");
        assert_eq!(obj.method, Some("ar".to_string()));
    }

    #[test]
    fn test_diffuse_reflections_generation() {
        let result = EffectsConverter::generate_diffuse_reflections(0.5);

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "DiffuseReflections");
        assert_eq!(obj.method, Some("ar".to_string()));
    }

    #[test]
    fn test_frequency_absorption_generation() {
        let result = EffectsConverter::generate_frequency_absorption(AbsorptionMaterial::Foam);

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "FrequencyAbsorption");
        assert_eq!(obj.method, Some("ar".to_string()));
    }
}
