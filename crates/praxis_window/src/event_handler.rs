//! Event handler trait for window events.
//!
//! This module defines the `WindowEventHandler` trait, which provides callbacks
//! for various window lifecycle and input events. Implement this trait to create
//! custom application logic that responds to window events.

use winit::window::Window;

/// Trait for handling window events.
///
/// Implement this trait to define custom behavior for window lifecycle events.
/// All methods have default implementations that do nothing, so you only need
/// to implement the ones you care about.
///
/// # Lifecycle
///
/// The methods are called in this order:
/// 1. `on_init()` - Called once after window creation
/// 2. `on_update()` + `on_render()` - Called repeatedly in the main loop
/// 3. `on_close()` - Called when window close is requested
///
/// # Examples
///
/// ```rust,ignore
/// use praxis_window::{WindowEventHandler, Window};
///
/// struct MyApp {
///     frame_count: u32,
/// }
///
/// impl WindowEventHandler for MyApp {
///     fn on_init(&mut self, window: &Window) {
///         println!("Window initialized: {:?}", window.inner_size());
///     }
///
///     fn on_update(&mut self, delta_time: f32) {
///         // Update game logic
///     }
///
///     fn on_render(&mut self, _window: &Window) {
///         self.frame_count += 1;
///     }
///
///     fn on_resize(&mut self, width: u32, height: u32) {
///         println!("Window resized to {}x{}", width, height);
///     }
///
///     fn on_close(&mut self) -> bool {
///         println!("Closing after {} frames", self.frame_count);
///         true // Allow close
///     }
/// }
/// ```
pub trait WindowEventHandler {
    /// Called once after the window is created and before the main loop starts.
    ///
    /// Use this to initialize graphics contexts, load assets, or perform other
    /// one-time setup that requires the window to exist.
    ///
    /// # Arguments
    ///
    /// * `window` - Reference to the newly created window
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn on_init(&mut self, window: &Window) {
    ///     // Initialize Vulkan surface
    ///     self.graphics = GraphicsContext::new(window)?;
    ///     
    ///     // Load initial assets
    ///     self.assets.load_scene("main_menu")?;
    /// }
    /// ```
    fn on_init(&mut self, _window: &Window) {}

    /// Called each frame before rendering to update application logic.
    ///
    /// This is where you should update game state, physics, animations, etc.
    /// It's called before `on_render()` to ensure logic updates complete before
    /// drawing the frame.
    ///
    /// # Arguments
    ///
    /// * `delta_time` - Time elapsed since last frame in seconds
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn on_update(&mut self, delta_time: f32) {
    ///     // Update physics
    ///     self.physics.step(delta_time);
    ///     
    ///     // Update animations
    ///     self.animation_player.update(delta_time);
    ///     
    ///     // Update game logic
    ///     self.game_state.update(delta_time);
    /// }
    /// ```
    fn on_update(&mut self, _delta_time: f32) {}

    /// Called each frame to render the current state.
    ///
    /// This is where you should submit draw calls to your graphics API.
    /// The window is provided so you can access its size or other properties.
    ///
    /// # Arguments
    ///
    /// * `window` - Reference to the window being rendered to
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn on_render(&mut self, window: &Window) {
    ///     // Acquire next swapchain image
    ///     let image = self.graphics.acquire_image()?;
    ///     
    ///     // Record and submit draw commands
    ///     self.graphics.draw_frame(&self.scene)?;
    ///     
    ///     // Present to screen
    ///     self.graphics.present()?;
    /// }
    /// ```
    fn on_render(&mut self, _window: &Window) {}

    /// Called when the window is resized.
    ///
    /// This is debounced to avoid excessive calls during window drag operations.
    /// Use this to recreate graphics resources that depend on window size
    /// (swapchains, framebuffers, camera aspect ratio, etc.).
    ///
    /// # Arguments
    ///
    /// * `width` - New window width in pixels
    /// * `height` - New window height in pixels
    ///
    /// # Note
    ///
    /// Zero-size resizes (when window is minimized) are filtered out and
    /// will not trigger this callback.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn on_resize(&mut self, width: u32, height: u32) {
    ///     // Recreate swapchain
    ///     self.graphics.resize(width, height)?;
    ///     
    ///     // Update camera aspect ratio
    ///     self.camera.set_aspect_ratio(width as f32 / height as f32);
    /// }
    /// ```
    fn on_resize(&mut self, _width: u32, _height: u32) {}

    /// Called when the window close button is clicked or close is requested.
    ///
    /// Return `true` to allow the window to close, or `false` to prevent it.
    /// This allows you to show "unsaved changes" dialogs or perform cleanup
    /// before allowing the application to exit.
    ///
    /// # Returns
    ///
    /// * `true` - Allow the window to close and application to exit
    /// * `false` - Prevent closing (e.g., waiting for user confirmation)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn on_close(&mut self) -> bool {
    ///     if self.has_unsaved_changes() {
    ///         // Show dialog asking user to confirm
    ///         self.show_unsaved_changes_dialog();
    ///         false // Don't close yet
    ///     } else {
    ///         // Clean up resources
    ///         self.cleanup();
    ///         true // Allow close
    ///     }
    /// }
    /// ```
    fn on_close(&mut self) -> bool {
        true
    }

    /// Called when the window gains focus.
    ///
    /// Use this to resume game logic, restart background music, or re-enable
    /// input handling when the user returns to your application.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn on_focused(&mut self) {
    ///     // Resume game
    ///     self.paused = false;
    ///     
    ///     // Resume audio
    ///     self.audio.resume();
    /// }
    /// ```
    fn on_focused(&mut self) {}

    /// Called when the window loses focus.
    ///
    /// Use this to pause game logic, mute audio, or disable input handling
    /// when the user switches to another application.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn on_unfocused(&mut self) {
    ///     // Pause game
    ///     self.paused = true;
    ///     
    ///     // Mute audio
    ///     self.audio.pause();
    /// }
    /// ```
    fn on_unfocused(&mut self) {}

    /// Called when a key is pressed.
    ///
    /// # Arguments
    ///
    /// * `key` - The key that was pressed
    /// * `is_repeat` - Whether this is a key repeat event (key held down)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use winit::keyboard::{Key, NamedKey};
    ///
    /// fn on_key_pressed(&mut self, key: Key, _is_repeat: bool) {
    ///     match key {
    ///         Key::Named(NamedKey::Escape) => self.paused = !self.paused,
    ///         Key::Named(NamedKey::Space) => self.player.jump(),
    ///         Key::Character(c) if c == "w" => self.player.move_forward(),
    ///         _ => {}
    ///     }
    /// }
    /// ```
    fn on_key_pressed(&mut self, _key: winit::keyboard::Key, _is_repeat: bool) {}

    /// Called when a key is released.
    ///
    /// # Arguments
    ///
    /// * `key` - The key that was released
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn on_key_released(&mut self, key: Key) {
    ///     if let Key::Character(c) = key {
    ///         if c == "w" {
    ///             self.player.stop_moving();
    ///         }
    ///     }
    /// }
    /// ```
    fn on_key_released(&mut self, _key: winit::keyboard::Key) {}

    /// Called when the mouse is moved.
    ///
    /// # Arguments
    ///
    /// * `x` - Mouse X position in pixels
    /// * `y` - Mouse Y position in pixels
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn on_mouse_moved(&mut self, x: f64, y: f64) {
    ///     self.cursor_position = (x, y);
    ///     self.hovered_entity = self.pick_entity_at(x, y);
    /// }
    /// ```
    fn on_mouse_moved(&mut self, _x: f64, _y: f64) {}

    /// Called when a mouse button is pressed.
    ///
    /// # Arguments
    ///
    /// * `button` - The mouse button that was pressed
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use winit::event::MouseButton;
    ///
    /// fn on_mouse_button_pressed(&mut self, button: MouseButton) {
    ///     match button {
    ///         MouseButton::Left => self.fire_weapon(),
    ///         MouseButton::Right => self.aim_weapon(),
    ///         _ => {}
    ///     }
    /// }
    /// ```
    fn on_mouse_button_pressed(&mut self, _button: winit::event::MouseButton) {}

    /// Called when a mouse button is released.
    ///
    /// # Arguments
    ///
    /// * `button` - The mouse button that was released
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn on_mouse_button_released(&mut self, button: MouseButton) {
    ///     if button == MouseButton::Right {
    ///         self.stop_aiming();
    ///     }
    /// }
    /// ```
    fn on_mouse_button_released(&mut self, _button: winit::event::MouseButton) {}

    /// Called when the mouse wheel is scrolled.
    ///
    /// # Arguments
    ///
    /// * `delta_x` - Horizontal scroll delta (rarely used)
    /// * `delta_y` - Vertical scroll delta (positive = scroll up, negative = scroll down)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn on_mouse_wheel(&mut self, _delta_x: f32, delta_y: f32) {
    ///     // Zoom camera
    ///     self.camera.zoom(delta_y * 0.1);
    /// }
    /// ```
    fn on_mouse_wheel(&mut self, _delta_x: f32, _delta_y: f32) {}
}

/// Default implementation for unit type (no-op handler).
///
/// This allows creating a window without a custom handler:
/// ```rust,ignore
/// let mut manager = WindowManager::with_handler(config, ())?;
/// ```
impl WindowEventHandler for () {}
