//! GLTF skeletal animation loader demonstration.
//!
//! This example demonstrates loading skeletal animations from GLTF files:
//! - Loading GLTF files with skins and animations
//! - Accessing skeleton and animation data
//! - Creating animation players from GLTF animations
//! - Playing loaded animations on entities

use praxis_assets::GltfLoader;
use praxis_ecs::{Query, Schedule, World};
use praxis_scene::{AnimatedPose, AnimationPlayer};

fn main() {
    println!("GLTF Skeletal Animation Loader Demo");
    println!("====================================\n");

    // Create a GLTF loader
    let loader = GltfLoader::new();

    // Example: Load a GLTF file with skeletal animations
    // Note: This is a demonstration of the API. You'll need an actual GLTF file
    // with skeletal animations to run this example.
    match loader.load_gltf("assets/models/animated_character.gltf") {
        Ok(asset) => {
            println!("Successfully loaded GLTF asset!");
            println!("  - {} meshes", asset.meshes.len());
            println!("  - {} materials", asset.materials.len());
            println!("  - {} textures", asset.textures.len());
            println!("  - {} nodes", asset.nodes.len());
            println!("  - {} skins", asset.skins.len());
            println!("  - {} animations\n", asset.animations.len());

            // Display information about each skin
            for (i, skin) in asset.skins.iter().enumerate() {
                println!("Skin {}: {:?}", i, skin.name);
                println!("  - {} bones", skin.skeleton.bone_count());

                // Print bone names
                for (bone_idx, bone) in skin.skeleton.bones().iter().enumerate() {
                    let parent_str = match bone.parent_index {
                        Some(p) => format!("parent: {p}"),
                        None => "root bone".to_string(),
                    };
                    println!("    Bone {}: {} ({})", bone_idx, bone.name, parent_str);
                }
                println!();
            }

            // Display information about each animation
            for (i, animation) in asset.animations.iter().enumerate() {
                println!("Animation {}: {:?}", i, animation.name);
                println!("  - Duration: {:.2}s", animation.duration);
                println!("  - {} bone tracks", animation.clip.track_count());
                println!();
            }

            // Example: Create an animation player with loaded animations
            if !asset.animations.is_empty() && !asset.skins.is_empty() {
                println!("Creating animation player...");

                let mut player = AnimationPlayer::new();

                // Add all animations to the player
                for animation in &asset.animations {
                    let name = animation
                        .name
                        .clone()
                        .unwrap_or_else(|| format!("Animation{}", player.clips().len()));
                    player.add_clip(name.clone(), animation.clip.clone());
                    println!("  Added animation: {name}");
                }

                // Play the first animation
                if let Some(first_anim) = asset.animations.first() {
                    let name = first_anim.name.as_deref().unwrap_or("Animation0");
                    player.play(name);
                    println!("\nPlaying animation: {name}");
                }

                // Example: Spawn an entity with skeleton and animation player
                let mut world = World::new();

                if let Some(skin) = asset.skins.first() {
                    let skeleton = skin.skeleton.clone();
                    let pose = AnimatedPose::new(skeleton.bone_count());

                    world.spawn((skeleton, player, pose));
                    println!("\nSpawned animated entity in ECS world");
                }

                // Example: Update animations in a system
                let mut schedule = Schedule::default();
                schedule.add_systems(animation_update_system);

                println!("Animation system ready to update");
            }

            // Example: Find specific animation by name
            if let Some(walk_anim) = asset.find_animation("Walk") {
                println!("\nFound 'Walk' animation:");
                println!("  Duration: {:.2}s", walk_anim.duration);
            }

            // Example: Find specific skin by name
            if let Some(skin) = asset.find_skin("CharacterSkin") {
                println!("\nFound 'CharacterSkin':");
                println!("  Bones: {}", skin.skeleton.bone_count());
            }
        }
        Err(e) => {
            println!("Failed to load GLTF file: {e}");
            println!("\nNote: This example requires a GLTF file with skeletal animations.");
            println!("Place a compatible file at: assets/models/animated_character.gltf");
        }
    }

    println!("\n=== Demo Complete ===");
}

/// Animation update system that advances animation playback.
fn animation_update_system(
    mut query: Query<(
        &praxis_scene::Skeleton,
        &mut AnimationPlayer,
        &mut AnimatedPose,
    )>,
) {
    let delta_time = 0.016; // 60 FPS
    praxis_scene::update_animations(delta_time, &mut query);
}
