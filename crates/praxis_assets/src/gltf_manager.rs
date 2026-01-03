//! GLTF asset manager for caching loaded GLTF assets.
//!
//! This module provides a manager for GLTF assets that caches loaded files
//! to avoid redundant loading and parsing operations.

use crate::loader::{GltfAsset, GltfLoader};
use praxis_utils::{debug, info, Result};
use std::collections::HashMap;
use std::path::Path;

/// Manager for GLTF assets with caching support.
///
/// This manager loads GLTF files and caches them by file path to avoid
/// redundant loading operations. It provides methods to load, access, and
/// manage GLTF assets throughout the application lifetime.
///
/// # Caching Strategy
///
/// Assets are cached by their file path. Once an asset is loaded, subsequent
/// requests for the same path will return a reference to the cached asset
/// instead of re-loading from disk.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_assets::GltfAssetManager;
///
/// let mut manager = GltfAssetManager::new();
///
/// // Load a GLTF file (caches the result)
/// let asset = manager.load("assets/models/scene.gltf")?;
/// println!("Loaded {} meshes", asset.meshes.len());
///
/// // Loading the same file again returns the cached asset
/// let asset2 = manager.load("assets/models/scene.gltf")?;
/// # Ok::<(), praxis_utils::eyre::Report>(())
/// ```
pub struct GltfAssetManager {
    /// GLTF loader instance.
    loader: GltfLoader,
    /// Cache of loaded assets by file path.
    assets: HashMap<String, GltfAsset>,
}

impl GltfAssetManager {
    /// Creates a new GLTF asset manager.
    ///
    /// # Example
    ///
    /// ```rust
    /// use praxis_assets::GltfAssetManager;
    ///
    /// let manager = GltfAssetManager::new();
    /// ```
    pub fn new() -> Self {
        Self {
            loader: GltfLoader::new(),
            assets: HashMap::new(),
        }
    }

    /// Loads a GLTF file, using the cached version if already loaded.
    ///
    /// If the asset at the given path has already been loaded, returns a reference
    /// to the cached asset. Otherwise, loads the file, caches it, and returns a reference.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the GLTF or GLB file to load
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file doesn't exist or cannot be read
    /// - The GLTF format is invalid
    /// - Required data is missing or malformed
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_assets::GltfAssetManager;
    ///
    /// let mut manager = GltfAssetManager::new();
    /// let asset = manager.load("assets/models/scene.gltf")?;
    /// # Ok::<(), praxis_utils::eyre::Report>(())
    /// ```
    pub fn load(&mut self, path: impl AsRef<Path>) -> Result<&GltfAsset> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        if !self.assets.contains_key(&path_str) {
            debug!("Loading GLTF asset: {}", path_str);
            let asset = self.loader.load_gltf(path)?;
            self.assets.insert(path_str.clone(), asset);
            info!("GLTF asset cached: {}", path_str);
        } else {
            debug!("Using cached GLTF asset: {}", path_str);
        }

        Ok(self
            .assets
            .get(&path_str)
            .expect("Asset should exist after loading"))
    }

    /// Gets a reference to a cached asset by path.
    ///
    /// Returns `None` if the asset hasn't been loaded yet.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_assets::GltfAssetManager;
    ///
    /// let mut manager = GltfAssetManager::new();
    /// manager.load("assets/models/scene.gltf")?;
    ///
    /// if let Some(asset) = manager.get("assets/models/scene.gltf") {
    ///     println!("Asset has {} meshes", asset.meshes.len());
    /// }
    /// # Ok::<(), praxis_utils::eyre::Report>(())
    /// ```
    pub fn get(&self, path: impl AsRef<Path>) -> Option<&GltfAsset> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        self.assets.get(&path_str)
    }

    /// Checks if an asset is cached.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_assets::GltfAssetManager;
    ///
    /// let mut manager = GltfAssetManager::new();
    /// assert!(!manager.is_loaded("assets/models/scene.gltf"));
    ///
    /// manager.load("assets/models/scene.gltf")?;
    /// assert!(manager.is_loaded("assets/models/scene.gltf"));
    /// # Ok::<(), praxis_utils::eyre::Report>(())
    /// ```
    pub fn is_loaded(&self, path: impl AsRef<Path>) -> bool {
        let path_str = path.as_ref().to_string_lossy().to_string();
        self.assets.contains_key(&path_str)
    }

    /// Removes an asset from the cache.
    ///
    /// Returns `true` if the asset was cached and has been removed.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_assets::GltfAssetManager;
    ///
    /// let mut manager = GltfAssetManager::new();
    /// manager.load("assets/models/scene.gltf")?;
    ///
    /// assert!(manager.unload("assets/models/scene.gltf"));
    /// assert!(!manager.is_loaded("assets/models/scene.gltf"));
    /// # Ok::<(), praxis_utils::eyre::Report>(())
    /// ```
    pub fn unload(&mut self, path: impl AsRef<Path>) -> bool {
        let path_str = path.as_ref().to_string_lossy().to_string();
        self.assets.remove(&path_str).is_some()
    }

    /// Returns the number of cached assets.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_assets::GltfAssetManager;
    ///
    /// let mut manager = GltfAssetManager::new();
    /// assert_eq!(manager.asset_count(), 0);
    ///
    /// manager.load("assets/models/scene.gltf")?;
    /// assert_eq!(manager.asset_count(), 1);
    /// # Ok::<(), praxis_utils::eyre::Report>(())
    /// ```
    pub fn asset_count(&self) -> usize {
        self.assets.len()
    }

    /// Clears all cached assets.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_assets::GltfAssetManager;
    ///
    /// let mut manager = GltfAssetManager::new();
    /// manager.load("assets/models/scene1.gltf")?;
    /// manager.load("assets/models/scene2.gltf")?;
    ///
    /// assert_eq!(manager.asset_count(), 2);
    /// manager.clear();
    /// assert_eq!(manager.asset_count(), 0);
    /// # Ok::<(), praxis_utils::eyre::Report>(())
    /// ```
    pub fn clear(&mut self) {
        debug!("Clearing {} cached GLTF assets", self.assets.len());
        self.assets.clear();
    }

    /// Returns an iterator over the paths of all cached assets.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_assets::GltfAssetManager;
    ///
    /// let mut manager = GltfAssetManager::new();
    /// manager.load("assets/models/scene1.gltf")?;
    /// manager.load("assets/models/scene2.gltf")?;
    ///
    /// for path in manager.loaded_paths() {
    ///     println!("Cached: {}", path);
    /// }
    /// # Ok::<(), praxis_utils::eyre::Report>(())
    /// ```
    pub fn loaded_paths(&self) -> impl Iterator<Item = &String> {
        self.assets.keys()
    }
}

impl Default for GltfAssetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = GltfAssetManager::new();
        assert_eq!(manager.asset_count(), 0);
    }

    #[test]
    fn test_manager_default() {
        let manager = GltfAssetManager::default();
        assert_eq!(manager.asset_count(), 0);
    }

    #[test]
    fn test_is_loaded_false() {
        let manager = GltfAssetManager::new();
        assert!(!manager.is_loaded("nonexistent.gltf"));
    }

    #[test]
    fn test_get_none() {
        let manager = GltfAssetManager::new();
        assert!(manager.get("nonexistent.gltf").is_none());
    }

    #[test]
    fn test_unload_nonexistent() {
        let mut manager = GltfAssetManager::new();
        assert!(!manager.unload("nonexistent.gltf"));
    }

    #[test]
    fn test_clear_empty() {
        let mut manager = GltfAssetManager::new();
        manager.clear();
        assert_eq!(manager.asset_count(), 0);
    }

    #[test]
    fn test_loaded_paths_empty() {
        let manager = GltfAssetManager::new();
        assert_eq!(manager.loaded_paths().count(), 0);
    }
}
