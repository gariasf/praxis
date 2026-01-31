//! Window configuration and builder patterns.
//!
//! This module provides the `WindowConfig` struct for configuring window attributes
//! before creation. It uses the builder pattern for ergonomic, chainable configuration.

/// Configuration for window creation.
///
/// Uses the builder pattern to provide a fluent API for configuring window attributes.
/// All fields have sensible defaults, so you can create a window with just `WindowConfig::default()`.
///
/// # Examples
///
/// ```rust,ignore
/// use praxis_window::WindowConfig;
///
/// // Minimal configuration
/// let config = WindowConfig::default();
///
/// // Full configuration
/// let config = WindowConfig::default()
///     .with_title("My Game")
///     .with_size(1920, 1080)
///     .with_resizable(true)
///     .with_maximized(false)
///     .with_vsync(true);
/// ```
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Window title displayed in title bar
    pub title: String,

    /// Initial window width in pixels
    pub width: u32,

    /// Initial window height in pixels
    pub height: u32,

    /// Whether the window can be resized by the user
    pub resizable: bool,

    /// Whether the window starts maximized
    pub maximized: bool,

    /// Whether to enable vertical synchronization (limits FPS to display refresh rate)
    pub vsync: bool,

    /// Target frames per second (None = unlimited)
    pub target_fps: Option<f32>,

    /// Whether to show the window immediately after creation
    pub visible: bool,

    /// Whether the window has decorations (title bar, borders)
    pub decorations: bool,

    /// Whether the window is transparent (requires platform support)
    pub transparent: bool,

    /// Window position (None = OS decides)
    pub position: Option<(i32, i32)>,

    /// Minimum window size
    pub min_size: Option<(u32, u32)>,

    /// Maximum window size
    pub max_size: Option<(u32, u32)>,
}

impl Default for WindowConfig {
    /// Creates a default window configuration.
    ///
    /// Defaults:
    /// - Title: "Praxis Engine"
    /// - Size: 1920×1080
    /// - Resizable: true
    /// - Maximized: false
    /// - VSync: true
    /// - Target FPS: None (unlimited, controlled by VSync)
    /// - Visible: true
    /// - Decorations: true
    /// - Transparent: false
    /// - Position: None (OS decides)
    /// - Min Size: None
    /// - Max Size: None
    fn default() -> Self {
        Self {
            title: "Praxis Engine".to_string(),
            width: 1920,
            height: 1080,
            resizable: true,
            maximized: false,
            vsync: true,
            target_fps: None,
            visible: true,
            decorations: true,
            transparent: false,
            position: None,
            min_size: None,
            max_size: None,
        }
    }
}

impl WindowConfig {
    /// Creates a new window configuration with default values.
    ///
    /// Equivalent to `WindowConfig::default()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the window title.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let config = WindowConfig::default().with_title("My Game");
    /// ```
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the window size in pixels.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let config = WindowConfig::default().with_size(800, 600);
    /// ```
    #[must_use]
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets whether the window is resizable.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let config = WindowConfig::default().with_resizable(false);
    /// ```
    #[must_use]
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Sets whether the window starts maximized.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let config = WindowConfig::default().with_maximized(true);
    /// ```
    #[must_use]
    pub fn with_maximized(mut self, maximized: bool) -> Self {
        self.maximized = maximized;
        self
    }

    /// Sets whether vertical synchronization is enabled.
    ///
    /// When VSync is enabled, the frame rate is limited to the display's refresh rate
    /// (typically 60 Hz), which prevents screen tearing but may introduce input lag.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let config = WindowConfig::default().with_vsync(false);
    /// ```
    #[must_use]
    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    /// Sets a target frame rate limit.
    ///
    /// When set, the event loop will sleep between frames to maintain the target FPS.
    /// This is independent of VSync - you can have both, neither, or one of them.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Limit to 30 FPS
    /// let config = WindowConfig::default().with_target_fps(Some(30.0));
    ///
    /// // Unlimited FPS
    /// let config = WindowConfig::default().with_target_fps(None);
    /// ```
    #[must_use]
    pub fn with_target_fps(mut self, fps: Option<f32>) -> Self {
        self.target_fps = fps;
        self
    }

    /// Sets whether the window is initially visible.
    ///
    /// You might want to start with a hidden window if you need to perform
    /// initialization before showing anything to the user.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let config = WindowConfig::default().with_visible(false);
    /// ```
    #[must_use]
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Sets whether the window has decorations (title bar, borders).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let config = WindowConfig::default().with_decorations(false);
    /// ```
    #[must_use]
    pub fn with_decorations(mut self, decorations: bool) -> Self {
        self.decorations = decorations;
        self
    }

    /// Sets whether the window is transparent.
    ///
    /// Note: Platform support varies. Not all platforms support transparent windows.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let config = WindowConfig::default().with_transparent(true);
    /// ```
    #[must_use]
    pub fn with_transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }

    /// Sets the initial window position.
    ///
    /// If not set, the OS will choose the position automatically.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let config = WindowConfig::default().with_position(100, 100);
    /// ```
    #[must_use]
    pub fn with_position(mut self, x: i32, y: i32) -> Self {
        self.position = Some((x, y));
        self
    }

    /// Sets the minimum window size.
    ///
    /// Prevents the user from resizing the window smaller than this size.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let config = WindowConfig::default().with_min_size(800, 600);
    /// ```
    #[must_use]
    pub fn with_min_size(mut self, width: u32, height: u32) -> Self {
        self.min_size = Some((width, height));
        self
    }

    /// Sets the maximum window size.
    ///
    /// Prevents the user from resizing the window larger than this size.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let config = WindowConfig::default().with_max_size(3840, 2160);
    /// ```
    #[must_use]
    pub fn with_max_size(mut self, width: u32, height: u32) -> Self {
        self.max_size = Some((width, height));
        self
    }

    /// Converts this config into winit's window attributes.
    ///
    /// This is an internal method used by `WindowManager` to create the actual window.
    pub(crate) fn to_window_attributes(&self) -> winit::window::WindowAttributes {
        use winit::dpi::{PhysicalPosition, PhysicalSize};

        let mut attrs = winit::window::Window::default_attributes()
            .with_title(self.title.clone())
            .with_inner_size(PhysicalSize::new(self.width, self.height))
            .with_resizable(self.resizable)
            .with_maximized(self.maximized)
            .with_visible(self.visible)
            .with_decorations(self.decorations)
            .with_transparent(self.transparent);

        if let Some((x, y)) = self.position {
            attrs = attrs.with_position(PhysicalPosition::new(x, y));
        }

        if let Some((width, height)) = self.min_size {
            attrs = attrs.with_min_inner_size(PhysicalSize::new(width, height));
        }

        if let Some((width, height)) = self.max_size {
            attrs = attrs.with_max_inner_size(PhysicalSize::new(width, height));
        }

        attrs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = WindowConfig::default();
        assert_eq!(config.title, "Praxis Engine");
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert!(config.resizable);
        assert!(!config.maximized);
        assert!(config.vsync);
        assert!(config.visible);
        assert!(config.decorations);
        assert!(!config.transparent);
        assert_eq!(config.position, None);
        assert_eq!(config.min_size, None);
        assert_eq!(config.max_size, None);
    }

    #[test]
    fn test_builder_pattern() {
        let config = WindowConfig::default()
            .with_title("Test Window")
            .with_size(800, 600)
            .with_resizable(false)
            .with_maximized(true);

        assert_eq!(config.title, "Test Window");
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert!(!config.resizable);
        assert!(config.maximized);
    }

    #[test]
    fn test_target_fps() {
        let config = WindowConfig::default().with_target_fps(Some(30.0));
        assert_eq!(config.target_fps, Some(30.0));

        let config = WindowConfig::default().with_target_fps(None);
        assert_eq!(config.target_fps, None);
    }

    #[test]
    fn test_position_and_size_constraints() {
        let config = WindowConfig::default()
            .with_position(100, 200)
            .with_min_size(800, 600)
            .with_max_size(1920, 1080);

        assert_eq!(config.position, Some((100, 200)));
        assert_eq!(config.min_size, Some((800, 600)));
        assert_eq!(config.max_size, Some((1920, 1080)));
    }
}
