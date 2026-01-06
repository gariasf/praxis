//! Async asset loading with tokio and channel-based completion notification.
//!
//! This module provides non-blocking asset loading functionality using tokio's
//! async runtime and crossbeam channels for completion notification.
//!
//! # Architecture
//!
//! - **`AsyncAssetLoader<T>`**: Core trait for async loading of any asset type
//! - **`AsyncMeshLoader`**: Async loader for OBJ meshes
//! - **`AsyncGltfLoader`**: Async loader for GLTF/GLB files
//! - **`LoadHandle`**: Handle to track and potentially cancel loading operations
//! - **Channel-based completion**: Uses crossbeam channels for thread-safe notification
//!
//! # Example: Basic Async Loading
//!
//! ```rust,no_run
//! use praxis_assets::async_loader::{AsyncAssetLoader, AsyncMeshLoader};
//!
//! # async fn example() -> praxis_utils::Result<()> {
//! let loader = AsyncMeshLoader::new();
//! let (handle, receiver) = loader.load_async("assets/models/cube.obj").await?;
//!
//! // Wait for completion
//! let mesh_data = receiver.recv().unwrap()?;
//! println!("Loaded {} vertices", mesh_data.positions.len());
//! # Ok(())
//! # }
//! ```
//!
//! # Example: Non-Blocking Check
//!
//! ```rust,no_run
//! use praxis_assets::async_loader::{AsyncAssetLoader, AsyncMeshLoader};
//!
//! # async fn example() -> praxis_utils::Result<()> {
//! let loader = AsyncMeshLoader::new();
//! let (handle, receiver) = loader.load_async("assets/models/cube.obj").await?;
//!
//! // Do other work...
//!
//! // Check if ready (non-blocking)
//! match receiver.try_recv() {
//!     Ok(result) => {
//!         let mesh_data = result?;
//!         println!("Ready! Loaded {} vertices", mesh_data.positions.len());
//!     }
//!     Err(_) => {
//!         println!("Still loading...");
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Example: Multiple Concurrent Loads
//!
//! ```rust,no_run
//! use praxis_assets::async_loader::{AsyncAssetLoader, AsyncMeshLoader};
//!
//! # async fn example() -> praxis_utils::Result<()> {
//! let loader = AsyncMeshLoader::new();
//!
//! // Start multiple loads concurrently
//! let (handle1, receiver1) = loader.load_async("assets/models/cube.obj").await?;
//! let (handle2, receiver2) = loader.load_async("assets/models/sphere.obj").await?;
//! let (handle3, receiver3) = loader.load_async("assets/models/cylinder.obj").await?;
//!
//! // Wait for all to complete
//! let mesh1 = receiver1.recv().unwrap()?;
//! let mesh2 = receiver2.recv().unwrap()?;
//! let mesh3 = receiver3.recv().unwrap()?;
//!
//! println!("All meshes loaded!");
//! # Ok(())
//! # }
//! ```

use crate::loader::{AssetLoader, GltfAsset, GltfLoader, MeshLoader};
use crossbeam_channel::Receiver;
use praxis_graphics::MeshData;
use praxis_utils::{info, Result};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::task::JoinHandle;

/// Handle to an async loading operation.
///
/// Provides methods to check the status and potentially cancel the operation.
/// When dropped, the loading operation continues in the background, but results
/// will be sent to the channel which can be safely ignored.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_assets::async_loader::{AsyncAssetLoader, AsyncMeshLoader};
///
/// # async fn example() -> praxis_utils::Result<()> {
/// let loader = AsyncMeshLoader::new();
/// let (handle, receiver) = loader.load_async("assets/models/cube.obj").await?;
///
/// // Check if the operation is complete
/// if handle.is_finished() {
///     println!("Loading complete!");
/// }
///
/// // The receiver can still be used even if the handle is dropped
/// drop(handle);
/// let result = receiver.recv().unwrap();
/// # Ok(())
/// # }
/// ```
pub struct LoadHandle {
    /// Tokio task join handle
    join_handle: JoinHandle<()>,
    /// Path being loaded (for debugging)
    path: PathBuf,
    /// Flag to signal cancellation
    cancelled: Arc<AtomicBool>,
}

impl LoadHandle {
    /// Creates a new load handle.
    fn new(join_handle: JoinHandle<()>, path: PathBuf, cancelled: Arc<AtomicBool>) -> Self {
        Self {
            join_handle,
            path,
            cancelled,
        }
    }

    /// Checks if the loading operation has finished.
    ///
    /// This is a non-blocking check.
    ///
    /// # Returns
    ///
    /// `true` if the operation is complete (successfully or not), `false` otherwise.
    pub fn is_finished(&self) -> bool {
        self.join_handle.is_finished()
    }

    /// Gets the path being loaded.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Cancels the loading operation.
    ///
    /// Note: This sets a cancellation flag, but the actual I/O operation
    /// may complete anyway. The result will still be sent to the channel,
    /// but you can choose to ignore it.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    /// Checks if the operation has been cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}

/// Core trait for async asset loading.
///
/// Implementors provide non-blocking asset loading with tokio and channel-based
/// completion notification.
///
/// # Type Parameters
///
/// * `T` - The output type produced by this loader (e.g., `MeshData`, `GltfAsset`)
///
/// # Example
///
/// ```rust,no_run
/// use praxis_assets::async_loader::{AsyncAssetLoader, AsyncMeshLoader};
/// use praxis_graphics::MeshData;
///
/// async fn load_asset<L: AsyncAssetLoader<MeshData>>(
///     loader: &L,
///     path: &str
/// ) -> praxis_utils::Result<MeshData> {
///     let (_handle, receiver) = loader.load_async(path).await?;
///     receiver.recv().unwrap()
/// }
/// ```
#[async_trait::async_trait]
pub trait AsyncAssetLoader<T: Send>: Send + Sync {
    /// Starts loading an asset asynchronously.
    ///
    /// This method returns immediately with a handle and a receiver. The actual
    /// loading happens in a background tokio task. When loading completes, the
    /// result is sent through the channel.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the asset file to load
    ///
    /// # Returns
    ///
    /// A tuple of:
    /// - `LoadHandle`: Handle to track/cancel the operation
    /// - `Receiver<Result<T>>`: Channel receiver for the loading result
    ///
    /// # Errors
    ///
    /// Returns an error immediately if:
    /// - The initial file check fails
    /// - The task cannot be spawned
    ///
    /// Loading errors are sent through the channel.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_assets::async_loader::{AsyncAssetLoader, AsyncMeshLoader};
    ///
    /// # async fn example() -> praxis_utils::Result<()> {
    /// let loader = AsyncMeshLoader::new();
    /// let (handle, receiver) = loader.load_async("assets/models/cube.obj").await?;
    ///
    /// // Non-blocking check
    /// if let Ok(result) = receiver.try_recv() {
    ///     let mesh_data = result?;
    ///     println!("Loaded!");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn load_async(
        &self,
        path: impl AsRef<Path> + Send,
    ) -> Result<(LoadHandle, Receiver<Result<T>>)>;

    /// Loads multiple assets concurrently.
    ///
    /// This is a convenience method that starts loading multiple assets
    /// in parallel and returns all handles and receivers.
    ///
    /// # Arguments
    ///
    /// * `paths` - Iterator of paths to load
    ///
    /// # Returns
    ///
    /// A vector of tuples (LoadHandle, Receiver) for each path
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_assets::async_loader::{AsyncAssetLoader, AsyncMeshLoader};
    ///
    /// # async fn example() -> praxis_utils::Result<()> {
    /// let loader = AsyncMeshLoader::new();
    /// let paths = vec!["cube.obj", "sphere.obj", "cylinder.obj"];
    ///
    /// let loads = loader.load_many_async(paths).await?;
    ///
    /// for (_handle, receiver) in loads {
    ///     let mesh_data = receiver.recv().unwrap()?;
    ///     println!("Loaded {} vertices", mesh_data.positions.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn load_many_async(
        &self,
        paths: impl IntoIterator<Item = impl AsRef<Path> + Send> + Send,
    ) -> Result<Vec<(LoadHandle, Receiver<Result<T>>)>>
    where
        Self: Sized,
    {
        let mut results = Vec::new();
        let path_vec: Vec<_> = paths.into_iter().collect();
        for path in path_vec {
            let result = self.load_async(path).await?;
            results.push(result);
        }
        Ok(results)
    }
}

/// Async loader for OBJ mesh files.
///
/// Provides non-blocking loading of Wavefront OBJ files using tokio's async I/O.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_assets::async_loader::AsyncMeshLoader;
///
/// # async fn example() -> praxis_utils::Result<()> {
/// let loader = AsyncMeshLoader::new();
/// let (handle, receiver) = loader.load_async("assets/models/cube.obj").await?;
///
/// println!("Loading started for: {}", handle.path().display());
///
/// // Wait for completion
/// let mesh_data = receiver.recv().unwrap()?;
/// println!("Loaded {} vertices", mesh_data.positions.len());
/// # Ok(())
/// # }
/// ```
pub struct AsyncMeshLoader {
    /// Underlying synchronous loader (used in spawned tasks)
    loader: Arc<MeshLoader>,
}

impl AsyncMeshLoader {
    /// Creates a new async mesh loader.
    ///
    /// # Example
    ///
    /// ```rust
    /// use praxis_assets::async_loader::AsyncMeshLoader;
    ///
    /// let loader = AsyncMeshLoader::new();
    /// ```
    pub fn new() -> Self {
        Self {
            loader: Arc::new(MeshLoader::new()),
        }
    }
}

impl Default for AsyncMeshLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl AsyncAssetLoader<MeshData> for AsyncMeshLoader {
    async fn load_async(
        &self,
        path: impl AsRef<Path> + Send,
    ) -> Result<(LoadHandle, Receiver<Result<MeshData>>)> {
        let path_buf = path.as_ref().to_path_buf();
        let path_for_handle = path_buf.clone();

        // Verify file exists before spawning task
        if !tokio::fs::try_exists(&path_buf).await? {
            return Err(praxis_utils::eyre::eyre!(
                "File not found: {}",
                path_buf.display()
            ));
        }

        info!("Starting async load for: {}", path_buf.display());

        let (sender, receiver) = crossbeam_channel::bounded(1);
        let loader = Arc::clone(&self.loader);
        let cancelled = Arc::new(AtomicBool::new(false));
        let cancelled_clone = Arc::clone(&cancelled);

        // Spawn loading task
        let join_handle = tokio::task::spawn_blocking(move || {
            // Check if cancelled before loading
            if cancelled_clone.load(Ordering::Relaxed) {
                let _ = sender.send(Err(praxis_utils::eyre::eyre!("Load cancelled")));
                return;
            }

            // Perform the actual loading (blocking I/O)
            let result = loader.load(&path_buf);

            // Check if cancelled before sending result
            if !cancelled_clone.load(Ordering::Relaxed) {
                // Send result through channel (ignore error if receiver dropped)
                let _ = sender.send(result);
            }
        });

        let handle = LoadHandle::new(join_handle, path_for_handle, cancelled);

        Ok((handle, receiver))
    }
}

/// Async loader for GLTF/GLB files.
///
/// Provides non-blocking loading of GLTF and GLB files using tokio's async I/O.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_assets::async_loader::AsyncGltfLoader;
///
/// # async fn example() -> praxis_utils::Result<()> {
/// let loader = AsyncGltfLoader::new();
/// let (handle, receiver) = loader.load_async("assets/models/scene.gltf").await?;
///
/// println!("Loading started for: {}", handle.path().display());
///
/// // Wait for completion
/// let asset = receiver.recv().unwrap()?;
/// println!("Loaded {} meshes", asset.meshes.len());
/// # Ok(())
/// # }
/// ```
pub struct AsyncGltfLoader {
    /// Underlying synchronous loader (used in spawned tasks)
    loader: Arc<GltfLoader>,
}

impl AsyncGltfLoader {
    /// Creates a new async GLTF loader.
    ///
    /// # Example
    ///
    /// ```rust
    /// use praxis_assets::async_loader::AsyncGltfLoader;
    ///
    /// let loader = AsyncGltfLoader::new();
    /// ```
    pub fn new() -> Self {
        Self {
            loader: Arc::new(GltfLoader::new()),
        }
    }
}

impl Default for AsyncGltfLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl AsyncAssetLoader<GltfAsset> for AsyncGltfLoader {
    async fn load_async(
        &self,
        path: impl AsRef<Path> + Send,
    ) -> Result<(LoadHandle, Receiver<Result<GltfAsset>>)> {
        let path_buf = path.as_ref().to_path_buf();
        let path_for_handle = path_buf.clone();

        // Verify file exists before spawning task
        if !tokio::fs::try_exists(&path_buf).await? {
            return Err(praxis_utils::eyre::eyre!(
                "File not found: {}",
                path_buf.display()
            ));
        }

        info!("Starting async load for: {}", path_buf.display());

        let (sender, receiver) = crossbeam_channel::bounded(1);
        let loader = Arc::clone(&self.loader);
        let cancelled = Arc::new(AtomicBool::new(false));
        let cancelled_clone = Arc::clone(&cancelled);

        // Spawn loading task
        let join_handle = tokio::task::spawn_blocking(move || {
            // Check if cancelled before loading
            if cancelled_clone.load(Ordering::Relaxed) {
                let _ = sender.send(Err(praxis_utils::eyre::eyre!("Load cancelled")));
                return;
            }

            // Perform the actual loading (blocking I/O)
            let result = loader.load_gltf(&path_buf);

            // Check if cancelled before sending result
            if !cancelled_clone.load(Ordering::Relaxed) {
                // Send result through channel (ignore error if receiver dropped)
                let _ = sender.send(result);
            }
        });

        let handle = LoadHandle::new(join_handle, path_for_handle, cancelled);

        Ok((handle, receiver))
    }
}

/// Batch async asset loader for managing multiple concurrent loads.
///
/// Provides convenient methods for loading multiple assets with progress tracking.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_assets::async_loader::{AsyncBatchLoader, AsyncMeshLoader};
///
/// # async fn example() -> praxis_utils::Result<()> {
/// let mesh_loader = AsyncMeshLoader::new();
/// let mut batch = AsyncBatchLoader::new();
///
/// // Queue multiple loads
/// batch.add(mesh_loader.load_async("cube.obj").await?);
/// batch.add(mesh_loader.load_async("sphere.obj").await?);
/// batch.add(mesh_loader.load_async("cylinder.obj").await?);
///
/// // Check progress
/// println!("Loaded: {}/{}", batch.completed_count(), batch.total_count());
///
/// // Wait for all
/// let results = batch.wait_all();
/// println!("All {} assets loaded", results.len());
/// # Ok(())
/// # }
/// ```
pub struct AsyncBatchLoader<T> {
    loads: Vec<(LoadHandle, Receiver<Result<T>>)>,
}

impl<T> AsyncBatchLoader<T> {
    /// Creates a new batch loader.
    pub fn new() -> Self {
        Self { loads: Vec::new() }
    }

    /// Adds a load operation to the batch.
    pub fn add(&mut self, load: (LoadHandle, Receiver<Result<T>>)) {
        self.loads.push(load);
    }

    /// Returns the total number of assets being loaded.
    pub fn total_count(&self) -> usize {
        self.loads.len()
    }

    /// Returns the number of completed loads (both successful and failed).
    pub fn completed_count(&self) -> usize {
        self.loads
            .iter()
            .filter(|(handle, _)| handle.is_finished())
            .count()
    }

    /// Returns the number of loads still in progress.
    pub fn pending_count(&self) -> usize {
        self.total_count() - self.completed_count()
    }

    /// Checks if all loads are complete.
    pub fn is_complete(&self) -> bool {
        self.loads.iter().all(|(handle, _)| handle.is_finished())
    }

    /// Waits for all loads to complete and returns the results.
    ///
    /// This is a blocking operation.
    pub fn wait_all(self) -> Vec<Result<T>> {
        self.loads
            .into_iter()
            .map(|(_, receiver)| receiver.recv().unwrap())
            .collect()
    }

    /// Tries to receive all completed results without blocking.
    ///
    /// Returns results for completed loads and keeps pending ones in the batch.
    pub fn try_receive_completed(&mut self) -> Vec<Result<T>> {
        let mut completed = Vec::new();
        let mut remaining = Vec::new();

        for (handle, receiver) in std::mem::take(&mut self.loads) {
            match receiver.try_recv() {
                Ok(result) => completed.push(result),
                Err(_) => remaining.push((handle, receiver)),
            }
        }

        self.loads = remaining;
        completed
    }

    /// Cancels all pending load operations.
    pub fn cancel_all(&self) {
        for (handle, _) in &self.loads {
            handle.cancel();
        }
    }
}

impl<T> Default for AsyncBatchLoader<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_mesh_loader_creation() {
        let _loader = AsyncMeshLoader::new();
        let _default_loader = AsyncMeshLoader::default();
    }

    #[tokio::test]
    async fn test_async_gltf_loader_creation() {
        let _loader = AsyncGltfLoader::new();
        let _default_loader = AsyncGltfLoader::default();
    }

    #[tokio::test]
    async fn test_async_batch_loader_creation() {
        let batch: AsyncBatchLoader<MeshData> = AsyncBatchLoader::new();
        assert_eq!(batch.total_count(), 0);
        assert_eq!(batch.completed_count(), 0);
        assert!(batch.is_complete());
    }

    #[tokio::test]
    async fn test_load_handle_path() {
        let loader = AsyncMeshLoader::new();

        // Create a temporary test file
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_async_handle.obj");
        std::fs::write(
            &test_file,
            "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n",
        )
        .expect("Failed to write test file");

        let (handle, _receiver) = loader
            .load_async(&test_file)
            .await
            .expect("Failed to start load");

        assert_eq!(handle.path(), test_file.as_path());

        std::fs::remove_file(&test_file).ok();
    }

    #[tokio::test]
    async fn test_load_handle_cancel() {
        let loader = AsyncMeshLoader::new();

        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_async_cancel.obj");
        std::fs::write(
            &test_file,
            "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n",
        )
        .expect("Failed to write test file");

        let (handle, _receiver) = loader
            .load_async(&test_file)
            .await
            .expect("Failed to start load");

        assert!(!handle.is_cancelled());
        handle.cancel();
        assert!(handle.is_cancelled());

        std::fs::remove_file(&test_file).ok();
    }

    #[tokio::test]
    async fn test_async_mesh_load_success() {
        let loader = AsyncMeshLoader::new();

        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_async_success.obj");
        std::fs::write(
            &test_file,
            "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n",
        )
        .expect("Failed to write test file");

        let (handle, receiver) = loader
            .load_async(&test_file)
            .await
            .expect("Failed to start load");

        // Wait for completion
        let result = receiver.recv().unwrap();
        assert!(result.is_ok());

        let mesh_data = result.unwrap();
        assert_eq!(mesh_data.positions.len(), 3);
        assert_eq!(mesh_data.indices.len(), 3);
        assert!(handle.is_finished());

        std::fs::remove_file(&test_file).ok();
    }

    #[tokio::test]
    async fn test_async_mesh_load_nonexistent() {
        let loader = AsyncMeshLoader::new();

        let result = loader
            .load_async("nonexistent_async_file_99999.obj")
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_async_batch_loader_add() {
        let loader = AsyncMeshLoader::new();
        let mut batch = AsyncBatchLoader::new();

        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_batch.obj");
        std::fs::write(
            &test_file,
            "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n",
        )
        .expect("Failed to write test file");

        let load = loader
            .load_async(&test_file)
            .await
            .expect("Failed to start load");

        batch.add(load);
        assert_eq!(batch.total_count(), 1);
        assert_eq!(batch.pending_count(), 1);

        std::fs::remove_file(&test_file).ok();
    }

    #[tokio::test]
    async fn test_async_batch_loader_wait_all() {
        let loader = AsyncMeshLoader::new();
        let mut batch = AsyncBatchLoader::new();

        let temp_dir = std::env::temp_dir();
        let test_file1 = temp_dir.join("test_batch1.obj");
        let test_file2 = temp_dir.join("test_batch2.obj");

        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";
        std::fs::write(&test_file1, obj_content).expect("Failed to write test file");
        std::fs::write(&test_file2, obj_content).expect("Failed to write test file");

        batch.add(loader.load_async(&test_file1).await.unwrap());
        batch.add(loader.load_async(&test_file2).await.unwrap());

        assert_eq!(batch.total_count(), 2);

        let results = batch.wait_all();
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());

        std::fs::remove_file(&test_file1).ok();
        std::fs::remove_file(&test_file2).ok();
    }

    #[tokio::test]
    async fn test_load_many_async() {
        let loader = AsyncMeshLoader::new();
        let temp_dir = std::env::temp_dir();

        let paths = vec![
            temp_dir.join("test_many1.obj"),
            temp_dir.join("test_many2.obj"),
            temp_dir.join("test_many3.obj"),
        ];

        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";
        for path in &paths {
            std::fs::write(path, obj_content).expect("Failed to write test file");
        }

        let loads = loader.load_many_async(paths.clone()).await.unwrap();
        assert_eq!(loads.len(), 3);

        for (_handle, receiver) in loads {
            let result = receiver.recv().unwrap();
            assert!(result.is_ok());
        }

        for path in &paths {
            std::fs::remove_file(path).ok();
        }
    }

    #[tokio::test]
    async fn test_async_batch_loader_try_receive() {
        let loader = AsyncMeshLoader::new();
        let mut batch = AsyncBatchLoader::new();

        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_try_receive.obj");
        std::fs::write(
            &test_file,
            "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n",
        )
        .expect("Failed to write test file");

        batch.add(loader.load_async(&test_file).await.unwrap());

        // Give it some time to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let completed = batch.try_receive_completed();
        assert_eq!(completed.len(), 1);
        assert!(completed[0].is_ok());
        assert_eq!(batch.total_count(), 0);

        std::fs::remove_file(&test_file).ok();
    }

    #[tokio::test]
    async fn test_async_gltf_load_nonexistent() {
        let loader = AsyncGltfLoader::new();

        let result = loader
            .load_async("nonexistent_async_file_99999.gltf")
            .await;

        assert!(result.is_err());
    }
}
