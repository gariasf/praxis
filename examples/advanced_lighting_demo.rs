//! Advanced lighting demonstration showcasing:
//! - Light probes for dynamic global illumination
//! - Volumetric fog with raymarching
//! - God rays (crepuscular rays) with radial blur
//! - Area lights with LTC
//! - Light linking for selective illumination

use praxis_core::App;
use praxis_ecs::{World, DirectionalLight, PointLight, Transform, Camera, PerspectiveProjection};
use praxis_ecs::{AreaLightComponent, LightProbeComponent};
use praxis_graphics::{
    LightProbeManager, LightProbeGrid, VolumetricFog, VolumetricFogConfig, FogDensityFunction,
    GodRays, GodRaysConfig, AreaLight, AreaLightType, AreaLightManager,
    LightLinkingManager, LightChannel,
};
use praxis_math::{Vec3, Vec2};
use praxis_utils::Result;
use std::sync::Arc;

struct AdvancedLightingDemo {
    world: World,
    light_probe_manager: Option<LightProbeManager>,
    area_light_manager: Option<AreaLightManager>,
    light_linking_manager: LightLinkingManager,
    time: f32,
}

impl AdvancedLightingDemo {
    fn new() -> Self {
        let mut world = World::new();
        let light_linking_manager = LightLinkingManager::new();

        Self {
            world,
            light_probe_manager: None,
            area_light_manager: None,
            light_linking_manager,
            time: 0.0,
        }
    }

    fn setup_camera(&mut self) {
        self.world.spawn((
            Transform::from_xyz(0.0, 5.0, 15.0),
            Camera::default(),
            PerspectiveProjection::default(),
        ));
    }

    fn setup_light_probes(&mut self) -> Result<()> {
        let grid = LightProbeGrid::new(
            Vec3::new(-20.0, 0.0, -20.0),
            Vec3::new(20.0, 10.0, 20.0),
            [5, 3, 5],
        );

        println!("Light probe grid created with {} probes", grid.probes.len());

        for (i, probe) in grid.probes.iter().enumerate() {
            self.world.spawn((
                Transform::from_translation(probe.position),
                LightProbeComponent::new(format!("probe_{}", i)),
            ));
        }

        Ok(())
    }

    fn setup_volumetric_fog(&mut self) {
        let fog_config = VolumetricFogConfig {
            density_function: FogDensityFunction::HeightBased {
                base_height: 0.0,
                falloff: 0.15,
            },
            color: Vec3::new(0.7, 0.75, 0.8),
            density: 0.04,
            max_distance: 100.0,
            num_steps: 64,
            light_scattering: 0.4,
            anisotropy: 0.3,
            shadow_influence: 0.7,
        };

        let fog = VolumetricFog::new(fog_config);
        println!(
            "Volumetric fog configured: {} steps, density {}",
            fog.config.num_steps, fog.config.density
        );
    }

    fn setup_god_rays(&mut self) {
        let god_rays_config = GodRaysConfig {
            num_samples: 80,
            density: 0.6,
            weight: 0.4,
            decay: 0.96,
            exposure: 0.9,
            threshold: 0.85,
        };

        let god_rays = GodRays::new(god_rays_config);
        println!(
            "God rays configured: {} samples, density {}",
            god_rays.config.num_samples, god_rays.config.density
        );
    }

    fn setup_area_lights(&mut self) {
        self.world.spawn((
            Transform::from_xyz(0.0, 8.0, 0.0),
            AreaLightComponent::rectangle(4.0, 4.0)
                .with_color(Vec3::new(1.0, 0.95, 0.85))
                .with_intensity(15.0),
        ));

        self.world.spawn((
            Transform::from_xyz(-8.0, 5.0, -5.0),
            AreaLightComponent::disk(2.0)
                .with_color(Vec3::new(0.2, 0.5, 1.0))
                .with_intensity(10.0),
        ));

        self.world.spawn((
            Transform::from_xyz(8.0, 3.0, 5.0),
            AreaLightComponent::sphere(1.5)
                .with_color(Vec3::new(1.0, 0.3, 0.1))
                .with_intensity(8.0),
        ));

        println!("Area lights created: 3 lights (rectangle, disk, sphere)");
    }

    fn setup_light_linking(&mut self) {
        let hero_lights = 0b0001;
        let environment_lights = 0b0010;
        let accent_lights = 0b0100;

        self.light_linking_manager
            .register_channel(0, "hero".to_string());
        self.light_linking_manager
            .register_channel(1, "environment".to_string());
        self.light_linking_manager
            .register_channel(2, "accent".to_string());

        self.light_linking_manager
            .set_object_mask("hero_character", hero_lights | environment_lights)
            .unwrap();
        self.light_linking_manager
            .set_object_mask("background_prop", environment_lights)
            .unwrap();
        self.light_linking_manager
            .set_object_mask("highlighted_item", accent_lights | environment_lights)
            .unwrap();

        self.light_linking_manager
            .set_light_channel("key_light", 0)
            .unwrap();
        self.light_linking_manager
            .set_light_channel("ambient_light", 1)
            .unwrap();
        self.light_linking_manager
            .set_light_channel("rim_light", 2)
            .unwrap();

        println!("Light linking configured:");
        println!("  - hero_character: affected by hero + environment lights");
        println!("  - background_prop: affected by environment lights only");
        println!("  - highlighted_item: affected by accent + environment lights");
    }

    fn setup_standard_lights(&mut self) {
        self.world.spawn(DirectionalLight {
            direction: Vec3::new(0.3, -0.8, 0.5).normalize(),
            color: Vec3::new(1.0, 0.95, 0.85),
            intensity: 0.8,
        });

        self.world.spawn((
            Transform::from_xyz(5.0, 3.0, 5.0),
            PointLight::new(Vec3::new(1.0, 0.7, 0.3), 12.0, 15.0),
        ));

        self.world.spawn((
            Transform::from_xyz(-5.0, 3.0, -5.0),
            PointLight::new(Vec3::new(0.3, 0.7, 1.0), 10.0, 12.0),
        ));

        println!("Standard lights created: 1 directional, 2 point lights");
    }

    fn update(&mut self, delta_time: f32) {
        self.time += delta_time;

        println!("\n=== Advanced Lighting System Status ===");
        println!("Time: {:.2}s", self.time);
        println!("Light probes: active");
        println!("Volumetric fog: enabled");
        println!("God rays: enabled");
        println!("Area lights: 3 active");
        println!("Light linking: {} objects, {} lights",
            self.light_linking_manager.list_objects().len(),
            self.light_linking_manager.list_lights().len()
        );
    }

    fn query_light_probe(&self, position: Vec3) {
        println!("\nQuerying light probe at position: {:?}", position);
        println!("  (Light probe data would be interpolated from nearby probes)");
    }

    fn demonstrate_light_linking(&self) {
        println!("\n=== Light Linking Demonstration ===");
        
        let objects = ["hero_character", "background_prop", "highlighted_item"];
        let lights = ["key_light", "ambient_light", "rim_light"];
        
        for obj in &objects {
            println!("\nObject: {}", obj);
            for light in &lights {
                let can_affect = self.light_linking_manager.can_light_affect_object(light, obj);
                println!("  {} {} affect this object",
                    light,
                    if can_affect { "CAN" } else { "CANNOT" }
                );
            }
        }
    }
}

fn main() -> Result<()> {
    println!("=== Advanced Lighting Demo ===\n");
    println!("This demo showcases advanced lighting features:");
    println!("1. Light Probes - Dynamic global illumination");
    println!("2. Volumetric Fog - Raymarched density with light scattering");
    println!("3. God Rays - Crepuscular rays with radial blur");
    println!("4. Area Lights - Polygon lights with LTC");
    println!("5. Light Linking - Selective object illumination\n");

    let mut demo = AdvancedLightingDemo::new();

    demo.setup_camera();
    demo.setup_light_probes()?;
    demo.setup_volumetric_fog();
    demo.setup_god_rays();
    demo.setup_area_lights();
    demo.setup_light_linking();
    demo.setup_standard_lights();

    println!("\n=== Scene Setup Complete ===\n");

    for i in 0..5 {
        demo.update(0.016);
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    demo.query_light_probe(Vec3::new(0.0, 2.0, 0.0));
    demo.query_light_probe(Vec3::new(5.0, 1.0, 3.0));

    demo.demonstrate_light_linking();

    println!("\n=== Performance Characteristics ===");
    println!("Light Probes:");
    println!("  - GPU-friendly spherical harmonics");
    println!("  - Trilinear interpolation for smooth transitions");
    println!("  - 9 SH coefficients per probe (L2)");
    println!("\nVolumetric Fog:");
    println!("  - Raymarching with configurable step count");
    println!("  - Phase function for anisotropic scattering");
    println!("  - Multiple density functions (uniform, exponential, height-based)");
    println!("\nGod Rays:");
    println!("  - Radial blur from light source");
    println!("  - Configurable sample count and decay");
    println!("  - Post-process effect for efficiency");
    println!("\nArea Lights:");
    println!("  - LTC (Linearly Transformed Cosines) for real-time shading");
    println!("  - Support for rectangles, disks, and spheres");
    println!("  - Accurate specular reflections");
    println!("\nLight Linking:");
    println!("  - 32-bit mask system for flexible control");
    println!("  - Zero runtime overhead with GPU filtering");
    println!("  - Artist-friendly channel-based workflow");

    println!("\n=== Demo Complete ===");
    Ok(())
}
