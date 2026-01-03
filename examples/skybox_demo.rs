//! Skybox demonstration example.
//!
//! This example demonstrates skybox rendering with cubemap textures.
//! It shows how to:
//! - Load a cubemap texture from 6 face images or an equirectangular image
//! - Create a skybox renderer with reversed depth
//! - Render a skybox that always appears at infinite distance
//!
//! Controls:
//! - WASD: Move camera
//! - Mouse: Look around
//! - ESC: Exit

use praxis::prelude::*;
use praxis_graphics::{Cubemap, SkyboxRenderer};
use std::sync::Arc;

fn main() -> Result<()> {
    praxis::run(SkyboxDemo::new)
}

struct SkyboxDemo {
    skybox_renderer: Option<SkyboxRenderer>,
    camera_position: Vec3,
    camera_rotation: Vec2,
    move_speed: f32,
    look_sensitivity: f32,
}

impl SkyboxDemo {
    fn new() -> Self {
        Self {
            skybox_renderer: None,
            camera_position: Vec3::new(0.0, 0.0, 5.0),
            camera_rotation: Vec2::ZERO,
            move_speed: 5.0,
            look_sensitivity: 0.002,
        }
    }
}

impl ApplicationCallbacks for SkyboxDemo {
    fn on_init(&mut self, ctx: &mut ApplicationContext) -> Result<()> {
        info!("Skybox Demo - Initializing");

        // Create a simple procedural cubemap (solid colors for each face)
        // In a real application, you would load actual skybox textures
        let face_size = 512;
        let face_data = [
            // +X (Right) - Red
            vec![255u8, 0, 0, 255].repeat(face_size * face_size),
            // -X (Left) - Cyan
            vec![0u8, 255, 255, 255].repeat(face_size * face_size),
            // +Y (Top) - Green
            vec![0u8, 255, 0, 255].repeat(face_size * face_size),
            // -Y (Bottom) - Magenta
            vec![255u8, 0, 255, 255].repeat(face_size * face_size),
            // +Z (Front) - Blue
            vec![0u8, 0, 255, 255].repeat(face_size * face_size),
            // -Z (Back) - Yellow
            vec![255u8, 255, 0, 255].repeat(face_size * face_size),
        ];

        // Load the cubemap
        let render_context = ctx.render_context_mut();

        let cubemap = Cubemap::from_faces(
            render_context.memory_allocator.clone(),
            render_context.command_buffer_allocator.clone(),
            render_context.graphics_queue.clone(),
            face_size as u32,
            face_data,
        )?;

        render_context.texture_manager_mut().add_texture(
            "skybox",
            praxis_graphics::Texture {
                image: cubemap.image.clone(),
                view: cubemap.view.clone(),
                sampler: cubemap.sampler.clone(),
                width: cubemap.face_size,
                height: cubemap.face_size,
            },
        );

        // Create skybox renderer
        let skybox_renderer = SkyboxRenderer::new(
            render_context.device.clone(),
            render_context.render_pass.clone(),
            render_context.viewport.clone(),
            render_context.memory_allocator.clone(),
        )?;

        self.skybox_renderer = Some(skybox_renderer);

        info!("Skybox Demo - Initialized successfully");
        info!("Controls: WASD to move, Mouse to look, ESC to exit");

        Ok(())
    }

    fn on_update(&mut self, ctx: &mut ApplicationContext) -> Result<()> {
        let delta_time = ctx.frame_timer().delta_seconds();
        let input = ctx.input_state();

        // Camera movement
        let mut movement = Vec3::ZERO;

        if input.is_key_held(praxis_input::KeyCode::KeyW) {
            movement.z -= 1.0;
        }
        if input.is_key_held(praxis_input::KeyCode::KeyS) {
            movement.z += 1.0;
        }
        if input.is_key_held(praxis_input::KeyCode::KeyA) {
            movement.x -= 1.0;
        }
        if input.is_key_held(praxis_input::KeyCode::KeyD) {
            movement.x += 1.0;
        }

        if movement.length_squared() > 0.0 {
            movement = movement.normalize();

            // Apply camera rotation to movement
            let yaw = self.camera_rotation.x;
            let rotated_movement = Vec3::new(
                movement.x * yaw.cos() - movement.z * yaw.sin(),
                movement.y,
                movement.x * yaw.sin() + movement.z * yaw.cos(),
            );

            self.camera_position += rotated_movement * self.move_speed * delta_time;
        }

        // Camera rotation from mouse
        let mouse_delta = input.mouse_delta();
        self.camera_rotation.x -= mouse_delta.x * self.look_sensitivity;
        self.camera_rotation.y -= mouse_delta.y * self.look_sensitivity;

        // Clamp pitch to prevent gimbal lock
        self.camera_rotation.y = self.camera_rotation.y.clamp(-1.5, 1.5);

        Ok(())
    }

    fn on_render(&mut self, ctx: &mut ApplicationContext) -> Result<()> {
        // Calculate view and projection matrices
        let yaw = self.camera_rotation.x;
        let pitch = self.camera_rotation.y;

        let forward = Vec3::new(
            yaw.sin() * pitch.cos(),
            pitch.sin(),
            -yaw.cos() * pitch.cos(),
        )
        .normalize();

        let view = Mat4::look_at_rh(
            self.camera_position,
            self.camera_position + forward,
            Vec3::Y,
        );

        let aspect_ratio =
            ctx.window().inner_size().width as f32 / ctx.window().inner_size().height as f32;
        let proj = Mat4::perspective_rh(70.0_f32.to_radians(), aspect_ratio, 0.1, 1000.0);

        // Render skybox
        if let Some(skybox_renderer) = &self.skybox_renderer {
            let render_context = ctx.render_context();

            if let Some(cubemap_texture) = render_context.texture_manager().get_texture("skybox") {
                // Create descriptor set for skybox
                let descriptor_set = skybox_renderer.create_descriptor_set(
                    render_context.descriptor_set_allocator.clone(),
                    render_context.view_proj_buffer.clone(),
                    cubemap_texture.view.clone(),
                    cubemap_texture.sampler.clone(),
                )?;

                // TODO: Actual rendering would happen in the render context
                // This example shows the API structure
                info!(
                    "Skybox ready to render at camera position: {:?}",
                    self.camera_position
                );
            }
        }

        Ok(())
    }

    fn on_shutdown(&mut self) -> Result<()> {
        info!("Skybox Demo - Shutting down");
        Ok(())
    }
}
