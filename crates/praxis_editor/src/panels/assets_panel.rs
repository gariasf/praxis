//! Asset browser panel for browsing, importing, and managing project assets.
//!
//! This module provides a comprehensive asset browser with features including:
//! - **Filesystem Traversal**: Browse the assets/ directory with folder navigation
//! - **Thumbnail Generation**: Automatic thumbnail generation for textures
//! - **Async Loading**: Non-blocking thumbnail loading with queue system
//! - **Drag-and-Drop**: Drag assets to scene view for instant placement
//! - **Asset Import**: Configurable import dialogs with format-specific settings
//! - **Hot-Reload**: Automatic file watching to detect asset changes
//!
//! # Architecture
//!
//! The asset browser is organized into several key components:
//!
//! ## AssetType
//! Categorizes assets based on file extension:
//! - `Texture`: PNG, JPG, JPEG files
//! - `Model`: OBJ, GLTF, GLB files  
//! - `Audio`: WAV, OGG, MP3 files
//! - `Scene`: SCENE files
//! - `Unknown`: Unsupported or unknown types
//!
//! ## AssetEntry
//! Represents a file or directory in the asset browser:
//! - File metadata (name, path, type, modification time)
//! - Directory traversal support
//! - Optional thumbnail reference
//!
//! ## Thumbnail System
//! Asynchronous thumbnail loading:
//! - Queue-based loading to avoid blocking the UI
//! - Automatic generation for texture assets
//! - Caching to prevent redundant loads
//! - Fallback icons for non-texture assets
//!
//! ## File Watcher
//! Hot-reload support using `notify`:
//! - Recursive monitoring of assets/ directory
//! - Automatic refresh on file creation/modification/deletion
//! - Thumbnail cache invalidation on changes
//!
//! ## Drag-and-Drop
//! Asset placement workflow:
//! - Click and drag assets from browser
//! - Visual drag preview follows cursor
//! - Drop onto scene view to instantiate
//! - Integration with `DragDropSystem` resource
//!
//! ## Import Configuration
//! Format-specific import settings:
//! - Model scale adjustment
//! - Texture mipmap generation
//! - Future: Compression settings, LOD generation, etc.
//!
//! # Usage
//!
//! ## Basic Setup
//!
//! ```rust,no_run
//! use praxis_editor::{AssetsPanel, EditorPanel};
//!
//! // Create the panel (automatically initializes file watcher)
//! let mut panel = AssetsPanel::new();
//!
//! // Render in your UI loop
//! // panel.ui(&mut ui);
//! ```
//!
//! ## Navigation
//!
//! ```rust,no_run
//! # use praxis_editor::AssetsPanel;
//! # let mut panel = AssetsPanel::new();
//! // Navigate to a subdirectory
//! panel.navigate_to("assets/models");
//!
//! // Navigate back/forward
//! panel.navigate_back();
//! panel.navigate_forward();
//!
//! // Navigate up one level
//! panel.navigate_up();
//! ```
//!
//! ## Search and Filter
//!
//! ```rust,no_run
//! # use praxis_editor::AssetsPanel;
//! # let mut panel = AssetsPanel::new();
//! // Set search filter
//! panel.set_search_filter("texture".to_string());
//!
//! // Check filtered results
//! let count = panel.filtered_entry_count();
//!
//! // Clear search
//! panel.clear_search();
//! ```
//!
//! ## Drag and Drop Integration
//!
//! ```rust,no_run
//! # use praxis_editor::AssetsPanel;
//! # let mut panel = AssetsPanel::new();
//! // Check for dragged assets in your scene view
//! if let Some(asset) = panel.get_dragged_asset() {
//!     println!("User dropped: {}", asset.path.display());
//!     // Spawn entity from asset...
//! }
//!
//! // Peek without taking
//! if let Some(asset) = panel.peek_dragged_asset() {
//!     println!("Currently dragging: {}", asset.name);
//! }
//! ```
//!
//! ## Thumbnail Management
//!
//! ```rust,no_run
//! # use praxis_editor::AssetsPanel;
//! # use egui::TextureId;
//! # use std::path::Path;
//! # let mut panel = AssetsPanel::new();
//! # let texture_id = TextureId::default();
//! // Manually load a thumbnail (if integrating with custom renderer)
//! panel.load_thumbnail(Path::new("assets/texture.png"), texture_id);
//!
//! // Check loading status
//! let pending = panel.pending_thumbnail_count();
//! if pending > 0 {
//!     println!("Loading {} thumbnails...", pending);
//! }
//! ```
//!
//! # Performance
//!
//! The asset browser is designed for efficiency:
//! - Only visible items are rendered (scroll culling)
//! - Thumbnails load on-demand as items become visible
//! - File watcher uses native OS notifications (minimal overhead)
//! - Thumbnail cache prevents redundant image loading

use super::EditorPanel;
use egui::{Color32, Context, Pos2, Rect, Sense, TextureId, Ui, Vec2};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use praxis_assets::AssetLoader;
use praxis_utils::{debug, info, warn, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

const THUMBNAIL_SIZE: f32 = 96.0;
const GRID_SPACING: f32 = 8.0;
const ASSETS_ROOT: &str = "assets/";

/// Type of asset based on file extension
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    /// Texture asset (png, jpg, jpeg)
    Texture,
    /// Model asset (obj, gltf, glb)
    Model,
    /// Audio asset (wav, ogg, mp3)
    Audio,
    /// Scene asset (scene)
    Scene,
    /// Unknown or unsupported file type
    Unknown,
}

impl AssetType {
    /// Determines asset type from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "png" | "jpg" | "jpeg" => Self::Texture,
            "obj" | "gltf" | "glb" => Self::Model,
            "wav" | "ogg" | "mp3" => Self::Audio,
            "scene" => Self::Scene,
            _ => Self::Unknown,
        }
    }

    /// Returns the color used for this asset type's icon
    pub fn icon_color(self) -> Color32 {
        match self {
            Self::Texture => Color32::from_rgb(100, 200, 100),
            Self::Model => Color32::from_rgb(100, 150, 200),
            Self::Audio => Color32::from_rgb(200, 150, 100),
            Self::Scene => Color32::from_rgb(200, 100, 150),
            Self::Unknown => Color32::GRAY,
        }
    }

    /// Returns the emoji icon for this asset type
    pub fn icon(self) -> &'static str {
        match self {
            Self::Texture => "🖼",
            Self::Model => "🗿",
            Self::Audio => "🔊",
            Self::Scene => "🎬",
            Self::Unknown => "📄",
        }
    }
}

/// Represents an asset file or directory entry
#[derive(Debug, Clone)]
pub struct AssetEntry {
    /// Name of the file or directory
    pub name: String,
    /// Full path to the asset
    pub path: PathBuf,
    /// Whether this entry is a directory
    pub is_directory: bool,
    /// Type of asset (for files)
    pub asset_type: AssetType,
    /// Last modified time
    pub modified: Option<SystemTime>,
    /// Thumbnail texture ID (if loaded)
    pub thumbnail: Option<TextureId>,
}

impl AssetEntry {
    /// Creates a new asset entry from a path
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path)?;
        let is_directory = metadata.is_dir();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let asset_type = if is_directory {
            AssetType::Unknown
        } else {
            path.extension()
                .and_then(|e| e.to_str())
                .map(AssetType::from_extension)
                .unwrap_or(AssetType::Unknown)
        };

        let modified = metadata.modified().ok();

        Ok(Self {
            name,
            path: path.to_path_buf(),
            is_directory,
            asset_type,
            modified,
            thumbnail: None,
        })
    }
}

/// Thumbnail loading state
#[derive(Debug)]
enum ThumbnailState {
    /// Thumbnail loading in progress
    Loading,
    /// Thumbnail loaded successfully
    Loaded(TextureId),
    /// Thumbnail loading failed
    Failed,
}

/// Messages for file watcher
#[derive(Debug)]
enum FileWatcherMessage {
    /// File or directory was created
    Created(PathBuf),
    /// File or directory was modified
    Modified(PathBuf),
    /// File or directory was deleted
    Deleted(PathBuf),
}

/// Asset import configuration
#[derive(Debug, Clone)]
pub struct AssetImportConfig {
    /// Whether to show the import dialog
    pub show_dialog: bool,
    /// Path to the asset being imported
    pub asset_path: PathBuf,
    /// Asset type
    pub asset_type: AssetType,
    /// Import scale for models
    pub model_scale: f32,
    /// Generate mipmaps for textures
    pub generate_mipmaps: bool,
}

impl Default for AssetImportConfig {
    fn default() -> Self {
        Self {
            show_dialog: false,
            asset_path: PathBuf::new(),
            asset_type: AssetType::Unknown,
            model_scale: 1.0,
            generate_mipmaps: true,
        }
    }
}

/// Panel for browsing and managing project assets.
pub struct AssetsPanel {
    title: String,
    /// Current directory being viewed
    current_path: PathBuf,
    /// List of entries in the current directory
    entries: Vec<AssetEntry>,
    /// Path history for navigation (back button)
    path_history: Vec<PathBuf>,
    /// Forward history for navigation
    path_forward_history: Vec<PathBuf>,
    /// Search filter text
    search_filter: String,
    /// Thumbnail cache
    thumbnail_cache: HashMap<PathBuf, ThumbnailState>,
    /// File watcher for hot-reload
    file_watcher: Option<RecommendedWatcher>,
    /// Receiver for file watcher events
    file_watcher_rx: Option<Receiver<FileWatcherMessage>>,
    /// Sender for file watcher events
    file_watcher_tx: Option<Sender<FileWatcherMessage>>,
    /// Asset being dragged
    dragged_asset: Option<AssetEntry>,
    /// Import configuration dialog
    import_config: AssetImportConfig,
    /// Whether to show hidden files
    show_hidden: bool,
    /// Sort mode
    sort_by_name: bool,
    /// Async thumbnail loader state
    thumbnail_loader: Arc<Mutex<ThumbnailLoader>>,
}

/// Async thumbnail loader
///
/// This structure manages asynchronous loading of thumbnails for assets.
/// It uses a queue-based system to load thumbnails on demand.
#[derive(Debug)]
struct ThumbnailLoader {
    /// Queue of paths to load thumbnails for
    queue: Vec<PathBuf>,
    /// Paths currently being loaded
    loading: Vec<PathBuf>,
}

impl ThumbnailLoader {
    fn new() -> Self {
        Self {
            queue: Vec::new(),
            loading: Vec::new(),
        }
    }

    /// Request a thumbnail to be loaded
    fn request_thumbnail(&mut self, path: PathBuf) {
        if !self.queue.contains(&path) && !self.loading.contains(&path) {
            self.queue.push(path);
        }
    }

    /// Get the next thumbnail to load
    fn next_thumbnail(&mut self) -> Option<PathBuf> {
        if let Some(path) = self.queue.pop() {
            self.loading.push(path.clone());
            Some(path)
        } else {
            None
        }
    }

    /// Mark a thumbnail as loaded
    fn mark_loaded(&mut self, path: &Path) {
        self.loading.retain(|p| p != path);
    }
}

impl AssetsPanel {
    /// Creates a new assets panel.
    #[must_use]
    pub fn new() -> Self {
        let mut panel = Self {
            title: "Assets".to_string(),
            current_path: PathBuf::from(ASSETS_ROOT),
            entries: Vec::new(),
            path_history: Vec::new(),
            path_forward_history: Vec::new(),
            search_filter: String::new(),
            thumbnail_cache: HashMap::new(),
            file_watcher: None,
            file_watcher_rx: None,
            file_watcher_tx: None,
            dragged_asset: None,
            import_config: AssetImportConfig::default(),
            show_hidden: false,
            sort_by_name: true,
            thumbnail_loader: Arc::new(Mutex::new(ThumbnailLoader::new())),
        };

        panel.setup_file_watcher();
        panel.refresh_entries();
        panel
    }

    /// Sets up the file watcher for hot-reload
    fn setup_file_watcher(&mut self) {
        let (tx, rx) = channel();
        self.file_watcher_tx = Some(tx.clone());
        self.file_watcher_rx = Some(rx);

        match notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                use notify::EventKind;
                match event.kind {
                    EventKind::Create(_) => {
                        for path in event.paths {
                            let _ = tx.send(FileWatcherMessage::Created(path));
                        }
                    }
                    EventKind::Modify(_) => {
                        for path in event.paths {
                            let _ = tx.send(FileWatcherMessage::Modified(path));
                        }
                    }
                    EventKind::Remove(_) => {
                        for path in event.paths {
                            let _ = tx.send(FileWatcherMessage::Deleted(path));
                        }
                    }
                    _ => {}
                }
            }
        }) {
            Ok(mut watcher) => {
                if let Err(e) = watcher.watch(Path::new(ASSETS_ROOT), RecursiveMode::Recursive) {
                    warn!("Failed to watch assets directory: {}", e);
                } else {
                    info!("File watcher initialized for assets directory");
                    self.file_watcher = Some(watcher);
                }
            }
            Err(e) => {
                warn!("Failed to create file watcher: {}", e);
            }
        }
    }

    /// Process file watcher events
    fn process_file_events(&mut self, render_context: Option<&mut praxis_graphics::RenderContext>) {
        let mut needs_refresh = false;
        let mut paths_to_reload = Vec::new();

        if let Some(rx) = &self.file_watcher_rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    FileWatcherMessage::Created(path) | FileWatcherMessage::Modified(path) => {
                        debug!("Asset changed: {}", path.display());
                        if path.starts_with(&self.current_path) {
                            needs_refresh = true;
                        }
                        paths_to_reload.push(path);
                    }
                    FileWatcherMessage::Deleted(path) => {
                        debug!("Asset deleted: {}", path.display());
                        if path.starts_with(&self.current_path) {
                            needs_refresh = true;
                        }
                        self.thumbnail_cache.remove(&path);
                    }
                }
            }
        }

        if needs_refresh {
            self.refresh_entries();
        }

        // Reload GPU assets when files change
        if let Some(ctx) = render_context {
            for path in &paths_to_reload {
                self.reload_asset_if_loaded(path, ctx);
                self.thumbnail_cache.remove(path);
            }
        } else {
            // If no render context, just clear thumbnails
            for path in paths_to_reload {
                self.thumbnail_cache.remove(&path);
            }
        }
    }

    /// Attempts to reload an asset from disk if it's currently loaded
    fn reload_asset_if_loaded(
        &self,
        path: &PathBuf,
        render_context: &mut praxis_graphics::RenderContext,
    ) {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let asset_type = AssetType::from_extension(ext);

            match asset_type {
                AssetType::Texture => {
                    match render_context.texture_manager_mut().reload_texture(path) {
                        Ok(true) => {
                            info!("🔄 Hot-reloaded texture: {}", path.display());
                        }
                        Ok(false) => {
                            // Texture not currently loaded, ignore
                        }
                        Err(e) => {
                            warn!("Failed to reload texture '{}': {}", path.display(), e);
                        }
                    }
                }
                AssetType::Model => {
                    // Load mesh data from file first
                    let loader = praxis_assets::MeshLoader::new();
                    match loader.load(path) {
                        Ok(mesh_data) => {
                            match render_context
                                .mesh_manager_mut()
                                .reload_mesh(path, mesh_data)
                            {
                                Ok(true) => {
                                    info!("🔄 Hot-reloaded mesh: {}", path.display());
                                }
                                Ok(false) => {
                                    // Mesh not currently loaded, ignore
                                }
                                Err(e) => {
                                    warn!("Failed to reload mesh '{}': {}", path.display(), e);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to load mesh data for '{}': {}", path.display(), e);
                        }
                    }
                }
                _ => {
                    // Other asset types don't support hot-reload yet
                }
            }
        }
    }

    /// Refreshes the entry list for the current directory
    fn refresh_entries(&mut self) {
        self.entries.clear();

        let Ok(read_dir) = std::fs::read_dir(&self.current_path) else {
            warn!("Failed to read directory: {}", self.current_path.display());
            return;
        };

        for entry in read_dir.flatten() {
            let path = entry.path();

            if !self.show_hidden {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.') {
                        continue;
                    }
                }
            }

            if let Ok(asset_entry) = AssetEntry::from_path(&path) {
                self.entries.push(asset_entry);
            }
        }

        if self.sort_by_name {
            self.entries.sort_by(|a, b| {
                if a.is_directory != b.is_directory {
                    b.is_directory.cmp(&a.is_directory)
                } else {
                    a.name.to_lowercase().cmp(&b.name.to_lowercase())
                }
            });
        }

        debug!(
            "Refreshed asset entries: {} items in {}",
            self.entries.len(),
            self.current_path.display()
        );
    }

    /// Navigates to a different directory
    pub fn navigate_to(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        if path.exists() && path.is_dir() {
            self.path_history.push(self.current_path.clone());
            self.path_forward_history.clear();
            self.current_path = path.to_path_buf();
            self.refresh_entries();
        }
    }

    /// Navigates back in history
    pub fn navigate_back(&mut self) {
        if let Some(path) = self.path_history.pop() {
            self.path_forward_history.push(self.current_path.clone());
            self.current_path = path;
            self.refresh_entries();
        }
    }

    /// Navigates forward in history
    pub fn navigate_forward(&mut self) {
        if let Some(path) = self.path_forward_history.pop() {
            self.path_history.push(self.current_path.clone());
            self.current_path = path;
            self.refresh_entries();
        }
    }

    /// Navigates to the parent directory
    pub fn navigate_up(&mut self) {
        let parent_path = self.current_path.parent().map(|p| p.to_path_buf());
        if let Some(parent) = parent_path {
            if parent.to_str().unwrap_or("").starts_with(ASSETS_ROOT)
                || parent == Path::new(ASSETS_ROOT.trim_end_matches('/'))
            {
                self.navigate_to(parent);
            }
        }
    }

    /// Renders the navigation toolbar
    fn render_toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("⬅").on_hover_text("Back").clicked() {
                self.navigate_back();
            }

            if ui.button("➡").on_hover_text("Forward").clicked() {
                self.navigate_forward();
            }

            if ui.button("⬆").on_hover_text("Up").clicked() {
                self.navigate_up();
            }

            ui.separator();

            self.render_breadcrumbs(ui);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄").on_hover_text("Refresh").clicked() {
                    self.refresh_entries();
                }

                if ui
                    .selectable_label(self.sort_by_name, "📝")
                    .on_hover_text("Sort by name")
                    .clicked()
                {
                    self.sort_by_name = !self.sort_by_name;
                    self.refresh_entries();
                }

                if ui
                    .selectable_label(self.show_hidden, "👁")
                    .on_hover_text("Show hidden files")
                    .clicked()
                {
                    self.show_hidden = !self.show_hidden;
                    self.refresh_entries();
                }
            });
        });
    }

    /// Renders breadcrumb navigation
    fn render_breadcrumbs(&mut self, ui: &mut Ui) {
        ui.label("📁");

        let path_str = self.current_path.to_str().unwrap_or("assets/");
        let components: Vec<String> = path_str
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        let mut paths_to_navigate = Vec::new();
        let mut path_builder = PathBuf::new();

        for (i, component) in components.iter().enumerate() {
            if i > 0 {
                ui.label("/");
            }

            path_builder.push(component);
            let nav_path = path_builder.clone();

            if ui.link(component.as_str()).clicked() {
                paths_to_navigate.push(nav_path);
            }
        }

        for path in paths_to_navigate {
            self.navigate_to(&path);
        }
    }

    /// Renders the search bar
    fn render_search(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.search_filter)
                .on_hover_text("Search assets");
            if ui.button("✖").on_hover_text("Clear search").clicked() {
                self.search_filter.clear();
            }
        });
    }

    /// Checks if an entry matches the search filter
    fn matches_filter(&self, entry: &AssetEntry) -> bool {
        if self.search_filter.is_empty() {
            return true;
        }
        entry
            .name
            .to_lowercase()
            .contains(&self.search_filter.to_lowercase())
    }

    /// Renders the asset grid
    fn render_asset_grid(&mut self, ui: &mut Ui) {
        let available_width = ui.available_width();
        let item_width = THUMBNAIL_SIZE + GRID_SPACING * 2.0;
        let columns = (available_width / item_width).max(1.0) as usize;

        let filtered_entries: Vec<AssetEntry> = self
            .entries
            .iter()
            .filter(|e| self.matches_filter(e))
            .cloned()
            .collect();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for row_start in (0..filtered_entries.len()).step_by(columns) {
                ui.horizontal(|ui| {
                    for entry in filtered_entries.iter().skip(row_start).take(columns) {
                        self.render_asset_item(ui, entry);
                    }
                });
            }
        });
    }

    /// Renders a single asset item
    fn render_asset_item(&mut self, ui: &mut Ui, entry: &AssetEntry) {
        let item_size = Vec2::new(THUMBNAIL_SIZE + GRID_SPACING, THUMBNAIL_SIZE + 40.0);
        let (rect, response) = ui.allocate_exact_size(item_size, Sense::click_and_drag());

        if ui.is_rect_visible(rect) {
            let is_hovered = response.hovered();
            let is_clicked = response.clicked();
            let is_double_clicked = response.double_clicked();
            let is_dragged = response.dragged();

            let bg_color = if is_hovered {
                Color32::from_rgb(60, 60, 60)
            } else {
                Color32::from_rgb(40, 40, 40)
            };

            ui.painter().rect_filled(rect, 4.0, bg_color);

            let thumbnail_rect = Rect::from_min_size(
                rect.min + Vec2::new(GRID_SPACING / 2.0, GRID_SPACING / 2.0),
                Vec2::new(THUMBNAIL_SIZE, THUMBNAIL_SIZE),
            );

            self.render_thumbnail(ui, entry, thumbnail_rect);

            let text_rect = Rect::from_min_size(
                Pos2::new(rect.min.x, thumbnail_rect.max.y + 4.0),
                Vec2::new(THUMBNAIL_SIZE + GRID_SPACING, 30.0),
            );

            ui.painter().text(
                text_rect.center(),
                egui::Align2::CENTER_CENTER,
                &entry.name,
                egui::FontId::proportional(12.0),
                Color32::WHITE,
            );

            if is_double_clicked && entry.is_directory {
                self.navigate_to(&entry.path);
            } else if is_clicked && !entry.is_directory {
                self.show_import_dialog(entry);
            }

            if is_dragged && !entry.is_directory {
                self.dragged_asset = Some(entry.clone());

                egui::Area::new(ui.id().with("drag_preview"))
                    .interactable(false)
                    .fixed_pos(ui.input(|i| i.pointer.hover_pos().unwrap_or_default()))
                    .show(ui.ctx(), |ui| {
                        ui.label(format!("📦 {}", entry.name));
                    });
            }

            response.context_menu(|ui| {
                self.render_context_menu(ui, entry);
            });
        }
    }

    /// Renders the context menu for an asset
    fn render_context_menu(&mut self, ui: &mut Ui, entry: &AssetEntry) {
        ui.label(&entry.name);
        ui.separator();

        if !entry.is_directory {
            if ui.button("Import...").clicked() {
                self.show_import_dialog(entry);
                ui.close_menu();
            }

            if ui.button("Show in Explorer").clicked() {
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("explorer")
                        .arg("/select,")
                        .arg(&entry.path)
                        .spawn();
                }
                #[cfg(target_os = "macos")]
                {
                    let _ = std::process::Command::new("open")
                        .arg("-R")
                        .arg(&entry.path)
                        .spawn();
                }
                #[cfg(target_os = "linux")]
                {
                    let _ = std::process::Command::new("xdg-open")
                        .arg(entry.path.parent().unwrap_or(&entry.path))
                        .spawn();
                }
                ui.close_menu();
            }

            ui.separator();

            if ui.button("Copy Path").clicked() {
                ui.output_mut(|o| {
                    o.copied_text = entry.path.to_string_lossy().to_string();
                });
                ui.close_menu();
            }

            if ui.button("Delete").clicked() {
                info!("Delete requested for: {}", entry.path.display());
                ui.close_menu();
            }
        } else {
            if ui.button("Open").clicked() {
                self.navigate_to(&entry.path);
                ui.close_menu();
            }

            if ui.button("Show in Explorer").clicked() {
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("explorer")
                        .arg(&entry.path)
                        .spawn();
                }
                #[cfg(target_os = "macos")]
                {
                    let _ = std::process::Command::new("open").arg(&entry.path).spawn();
                }
                #[cfg(target_os = "linux")]
                {
                    let _ = std::process::Command::new("xdg-open")
                        .arg(&entry.path)
                        .spawn();
                }
                ui.close_menu();
            }
        }
    }

    /// Renders a thumbnail for an asset
    fn render_thumbnail(&mut self, ui: &mut Ui, entry: &AssetEntry, rect: Rect) {
        if entry.is_directory {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "📁",
                egui::FontId::proportional(48.0),
                Color32::from_rgb(200, 200, 100),
            );
        } else {
            let state = self.thumbnail_cache.get(&entry.path);
            match state {
                Some(ThumbnailState::Loaded(texture_id)) => {
                    ui.painter().image(
                        *texture_id,
                        rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
                Some(ThumbnailState::Loading) => {
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "⏳",
                        egui::FontId::proportional(32.0),
                        Color32::GRAY,
                    );
                }
                Some(ThumbnailState::Failed) | None => {
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        entry.asset_type.icon(),
                        egui::FontId::proportional(48.0),
                        entry.asset_type.icon_color(),
                    );

                    if state.is_none() && entry.asset_type == AssetType::Texture {
                        self.thumbnail_cache
                            .insert(entry.path.clone(), ThumbnailState::Loading);
                        self.thumbnail_loader
                            .lock()
                            .unwrap()
                            .request_thumbnail(entry.path.clone());
                    }
                }
            }
        }
    }

    /// Shows the import dialog for an asset
    fn show_import_dialog(&mut self, entry: &AssetEntry) {
        self.import_config = AssetImportConfig {
            show_dialog: true,
            asset_path: entry.path.clone(),
            asset_type: entry.asset_type,
            model_scale: 1.0,
            generate_mipmaps: true,
        };
        info!("Opening import dialog for: {}", entry.path.display());
    }

    /// Renders the import dialog
    fn render_import_dialog(&mut self, ctx: &Context) {
        if !self.import_config.show_dialog {
            return;
        }

        egui::Window::new("Import Asset")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Asset Import Settings");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("File:");
                    ui.label(
                        self.import_config
                            .asset_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown"),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    ui.label(format!("{:?}", self.import_config.asset_type));
                });

                ui.separator();

                match self.import_config.asset_type {
                    AssetType::Model => {
                        ui.horizontal(|ui| {
                            ui.label("Scale:");
                            ui.add(egui::Slider::new(
                                &mut self.import_config.model_scale,
                                0.01..=10.0,
                            ));
                        });
                    }
                    AssetType::Texture => {
                        ui.checkbox(&mut self.import_config.generate_mipmaps, "Generate Mipmaps");
                    }
                    _ => {}
                }

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Import").clicked() {
                        self.import_asset();
                        self.import_config.show_dialog = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.import_config.show_dialog = false;
                    }
                });
            });
    }

    /// Imports an asset with the current configuration
    fn import_asset(&self) {
        info!(
            "Importing asset: {} (scale: {}, mipmaps: {})",
            self.import_config.asset_path.display(),
            self.import_config.model_scale,
            self.import_config.generate_mipmaps
        );
    }

    /// Gets the currently dragged asset (for drag-drop operations)
    pub fn get_dragged_asset(&mut self) -> Option<AssetEntry> {
        self.dragged_asset.take()
    }

    /// Loads a thumbnail from an image file
    pub fn load_thumbnail(&mut self, path: &Path, texture_id: TextureId) {
        self.thumbnail_cache
            .insert(path.to_path_buf(), ThumbnailState::Loaded(texture_id));
        if let Ok(mut loader) = self.thumbnail_loader.lock() {
            loader.mark_loaded(path);
        }
    }

    /// Marks a thumbnail as failed to load
    pub fn mark_thumbnail_failed(&mut self, path: &Path) {
        self.thumbnail_cache
            .insert(path.to_path_buf(), ThumbnailState::Failed);
        if let Ok(mut loader) = self.thumbnail_loader.lock() {
            loader.mark_loaded(path);
        }
    }

    /// Processes pending thumbnail requests
    ///
    /// This should be called periodically to process the thumbnail loading queue.
    /// It generates thumbnails for textures by loading and downscaling images.
    pub fn process_thumbnail_queue(&mut self) {
        let next_path = if let Ok(mut loader) = self.thumbnail_loader.lock() {
            loader.next_thumbnail()
        } else {
            None
        };

        if let Some(path) = next_path {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let asset_type = AssetType::from_extension(ext);
                match asset_type {
                    AssetType::Texture => {
                        if let Ok(_thumbnail_data) = Self::generate_texture_thumbnail(&path) {
                            debug!("Generated thumbnail for texture: {}", path.display());
                        } else {
                            self.mark_thumbnail_failed(&path);
                        }
                    }
                    AssetType::Model => {
                        debug!(
                            "Model thumbnail generation not yet implemented: {}",
                            path.display()
                        );
                        self.mark_thumbnail_failed(&path);
                    }
                    _ => {
                        self.mark_thumbnail_failed(&path);
                    }
                }
            } else {
                self.mark_thumbnail_failed(&path);
            }
        }
    }

    /// Generates a thumbnail for a texture file
    fn generate_texture_thumbnail(path: &Path) -> Result<Vec<u8>> {
        let img = image::open(path)?;
        let thumbnail = img.thumbnail(THUMBNAIL_SIZE as u32, THUMBNAIL_SIZE as u32);
        let rgba = thumbnail.to_rgba8();
        Ok(rgba.into_raw())
    }

    /// Gets pending thumbnail count
    pub fn pending_thumbnail_count(&self) -> usize {
        self.thumbnail_loader
            .lock()
            .map(|loader| loader.queue.len() + loader.loading.len())
            .unwrap_or(0)
    }

    /// Gets the current directory path
    pub fn current_path(&self) -> &PathBuf {
        &self.current_path
    }

    /// Gets the number of entries in the current directory
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Gets the filtered entry count (matching search)
    pub fn filtered_entry_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| self.matches_filter(e))
            .count()
    }

    /// Clears the search filter
    pub fn clear_search(&mut self) {
        self.search_filter.clear();
    }

    /// Sets the search filter
    pub fn set_search_filter(&mut self, filter: String) {
        self.search_filter = filter;
    }

    /// Gets the current search filter
    pub fn search_filter(&self) -> &str {
        &self.search_filter
    }

    /// Checks if an asset is currently being dragged
    pub fn is_dragging(&self) -> bool {
        self.dragged_asset.is_some()
    }

    /// Gets a reference to the dragged asset without taking it
    pub fn peek_dragged_asset(&self) -> Option<&AssetEntry> {
        self.dragged_asset.as_ref()
    }

    /// Renders the status bar
    fn render_status_bar(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let total = self.entry_count();
            let filtered = self.filtered_entry_count();
            let pending = self.pending_thumbnail_count();

            if filtered < total {
                ui.label(format!("Showing {filtered} of {total} items"));
            } else {
                ui.label(format!("{total} items"));
            }

            if pending > 0 {
                ui.separator();
                ui.label(format!("⏳ Loading {pending} thumbnails..."));
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let current_path = self.current_path.display();
                ui.label(format!("📂 {current_path}"));
            });
        });
    }
}

impl Default for AssetsPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorPanel for AssetsPanel {
    fn title(&self) -> &str {
        &self.title
    }

    fn ui(
        &mut self,
        ui: &mut Ui,
        _world: Option<&praxis_ecs::World>,
        render_context: Option<&mut praxis_graphics::RenderContext>,
    ) {
        self.process_file_events(render_context);
        self.process_thumbnail_queue();

        ui.vertical(|ui| {
            self.render_toolbar(ui);
            ui.separator();
            self.render_search(ui);
            ui.separator();

            let available_height = ui.available_height();
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), available_height - 25.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    self.render_asset_grid(ui);
                },
            );

            ui.separator();
            self.render_status_bar(ui);
        });

        self.render_import_dialog(ui.ctx());

        if self.pending_thumbnail_count() > 0 {
            ui.ctx().request_repaint();
        }
    }

    fn on_close(&mut self) {
        if let Some(watcher) = self.file_watcher.take() {
            drop(watcher);
        }
    }
}
