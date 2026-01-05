#[cfg(test)]
mod tests {
    use crate::{area_lights::*, god_rays::*, light_linking::*, light_probe::*, volumetric_fog::*};
    use praxis_math::Vec3;

    #[test]
    fn test_light_probe_creation() {
        let probe = LightProbe::new(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(probe.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(probe.intensity, 1.0);
        assert_eq!(probe.radius, 10.0);
    }

    #[test]
    fn test_light_probe_grid_creation() {
        let grid = LightProbeGrid::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 5.0, 10.0),
            [3, 2, 3],
        );
        assert_eq!(grid.dimensions, [3, 2, 3]);
        assert_eq!(grid.probes.len(), 18);
    }

    #[test]
    fn test_light_probe_grid_access() {
        let grid = LightProbeGrid::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 5.0, 10.0),
            [3, 2, 3],
        );

        let probe = grid.probe_at(1, 1, 1);
        assert!(probe.is_some());

        let probe = grid.probe_at(5, 5, 5);
        assert!(probe.is_none());
    }

    #[test]
    fn test_volumetric_fog_config_default() {
        let config = VolumetricFogConfig::default();
        assert_eq!(config.num_steps, 64);
        assert!(config.density > 0.0);
    }

    #[test]
    fn test_fog_density_function() {
        let uniform = FogDensityFunction::Uniform;
        let exponential = FogDensityFunction::Exponential { falloff: 0.5 };
        let height_based = FogDensityFunction::HeightBased {
            base_height: 0.0,
            falloff: 0.1,
        };

        assert!(matches!(uniform, FogDensityFunction::Uniform));
        assert!(matches!(
            exponential,
            FogDensityFunction::Exponential { .. }
        ));
        assert!(matches!(
            height_based,
            FogDensityFunction::HeightBased { .. }
        ));
    }

    #[test]
    fn test_god_rays_config() {
        let config = GodRaysConfig {
            num_samples: 64,
            density: 0.5,
            weight: 0.3,
            decay: 0.95,
            exposure: 0.8,
            threshold: 0.8,
        };

        assert_eq!(config.num_samples, 64);
        assert_eq!(config.density, 0.5);
    }

    #[test]
    fn test_area_light_rectangle() {
        let light = AreaLight::new_rectangle(Vec3::ZERO, 2.0, 1.0);
        assert!(matches!(
            light.light_type,
            AreaLightType::Rectangle {
                width: 2.0,
                height: 1.0
            }
        ));
        assert_eq!(light.position, Vec3::ZERO);
    }

    #[test]
    fn test_area_light_disk() {
        let light = AreaLight::new_disk(Vec3::new(1.0, 2.0, 3.0), 1.5);
        assert!(matches!(
            light.light_type,
            AreaLightType::Disk { radius: 1.5 }
        ));
    }

    #[test]
    fn test_area_light_sphere() {
        let light = AreaLight::new_sphere(Vec3::new(0.0, 5.0, 0.0), 2.0);
        assert!(matches!(
            light.light_type,
            AreaLightType::Sphere { radius: 2.0 }
        ));
        assert!(light.two_sided);
    }

    #[test]
    fn test_area_light_builder() {
        let light = AreaLight::new_rectangle(Vec3::ZERO, 1.0, 1.0)
            .with_color(Vec3::new(1.0, 0.5, 0.0))
            .with_intensity(5.0);

        assert_eq!(light.color, Vec3::new(1.0, 0.5, 0.0));
        assert_eq!(light.intensity, 5.0);
    }

    #[test]
    fn test_light_linking_mask_all() {
        let mask = LightLinkingMask::all();
        assert_eq!(mask.mask, DEFAULT_LIGHT_CHANNEL);
    }

    #[test]
    fn test_light_linking_mask_none() {
        let mask = LightLinkingMask::none();
        assert_eq!(mask.mask, 0);
    }

    #[test]
    fn test_light_linking_mask_channel() {
        let mask = LightLinkingMask::channel(3);
        assert_eq!(mask.mask, 0b1000);
        assert!(mask.has_channel(3));
        assert!(!mask.has_channel(2));
    }

    #[test]
    fn test_light_linking_mask_channels() {
        let mask = LightLinkingMask::channels(&[0, 2, 4]);
        assert_eq!(mask.mask, 0b10101);
        assert!(mask.has_channel(0));
        assert!(mask.has_channel(2));
        assert!(mask.has_channel(4));
        assert!(!mask.has_channel(1));
    }

    #[test]
    fn test_light_linking_mask_operations() {
        let mut mask = LightLinkingMask::new(0b0001);

        mask.add_channel(2);
        assert_eq!(mask.mask, 0b0101);

        mask.remove_channel(0);
        assert_eq!(mask.mask, 0b0100);

        mask.toggle_channel(1);
        assert_eq!(mask.mask, 0b0110);

        mask.toggle_channel(1);
        assert_eq!(mask.mask, 0b0100);
    }

    #[test]
    fn test_light_linking_includes() {
        let mask1 = LightLinkingMask::new(0b1010);
        let mask2 = LightLinkingMask::new(0b0010);
        let mask3 = LightLinkingMask::new(0b0100);

        assert!(mask1.includes(&mask2));
        assert!(!mask1.includes(&mask3));
    }

    #[test]
    fn test_light_linking_manager_basic() {
        let mut manager = LightLinkingManager::new();

        manager.set_object_mask("obj1", 0b0001).unwrap();
        manager.set_light_channel("light1", 0).unwrap();

        assert!(manager.can_light_affect_object("light1", "obj1"));
    }

    #[test]
    fn test_light_linking_manager_multiple() {
        let mut manager = LightLinkingManager::new();

        manager.set_object_mask("obj1", 0b0011).unwrap();
        manager.set_light_channel("light1", 0).unwrap();
        manager.set_light_channel("light2", 1).unwrap();
        manager.set_light_channel("light3", 2).unwrap();

        assert!(manager.can_light_affect_object("light1", "obj1"));
        assert!(manager.can_light_affect_object("light2", "obj1"));
        assert!(!manager.can_light_affect_object("light3", "obj1"));
    }

    #[test]
    fn test_light_linking_channel_names() {
        let mut manager = LightLinkingManager::new();

        manager.register_channel(0, "hero".to_string());
        manager.register_channel(1, "environment".to_string());

        assert_eq!(manager.get_channel_by_name("hero"), Some(0));
        assert_eq!(manager.get_channel_by_name("environment"), Some(1));
        assert_eq!(manager.get_channel_by_name("unknown"), None);
    }

    #[test]
    fn test_light_probe_blend_modes() {
        let mut grid = LightProbeGrid::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 10.0, 10.0),
            [3, 3, 3],
        );

        grid.blend_mode = ProbeBlendMode::Nearest;
        assert_eq!(grid.blend_mode, ProbeBlendMode::Nearest);

        grid.blend_mode = ProbeBlendMode::Trilinear;
        assert_eq!(grid.blend_mode, ProbeBlendMode::Trilinear);
    }

    #[test]
    fn test_volumetric_fog_component() {
        let fog = VolumetricFog::default();
        assert!(fog.enabled);

        let mut fog = VolumetricFog::new(VolumetricFogConfig::default());
        assert!(fog.enabled);

        fog.disable();
        assert!(!fog.enabled);

        fog.enable();
        assert!(fog.enabled);
    }

    #[test]
    fn test_god_rays_component() {
        let god_rays = GodRays::default();
        assert!(god_rays.enabled);
        assert_eq!(god_rays.intensity, 1.0);

        let god_rays = GodRays::new(GodRaysConfig::default()).with_intensity(0.5);
        assert_eq!(god_rays.intensity, 0.5);
    }

    #[test]
    fn test_area_light_transform() {
        let light = AreaLight::new_rectangle(Vec3::new(0.0, 5.0, 0.0), 2.0, 1.0)
            .with_direction(Vec3::new(0.0, -1.0, 0.0));

        let transform = light.compute_transform();
        assert!(
            transform
                .col(3)
                .truncate()
                .distance(Vec3::new(0.0, 5.0, 0.0))
                < 0.001
        );
    }

    #[test]
    fn test_light_probe_data_conversion() {
        let probe = LightProbe::new(Vec3::new(1.0, 2.0, 3.0))
            .with_intensity(1.5)
            .with_radius(15.0);

        let data = LightProbeData::from(&probe);
        assert_eq!(data.position[0], 1.0);
        assert_eq!(data.position[1], 2.0);
        assert_eq!(data.position[2], 3.0);
        assert_eq!(data.intensity, 1.5);
        assert_eq!(data.radius, 15.0);
    }

    #[test]
    fn test_area_light_data_conversion() {
        let light = AreaLight::new_rectangle(Vec3::ZERO, 2.0, 1.0)
            .with_color(Vec3::new(1.0, 0.5, 0.0))
            .with_intensity(10.0);

        let data = AreaLightData::from(&light);
        assert_eq!(data.color[0], 1.0);
        assert_eq!(data.color[1], 0.5);
        assert_eq!(data.color[2], 0.0);
        assert_eq!(data.intensity, 10.0);
        assert_eq!(data.light_type, 0);
    }
}
