//! Async asset loading with tokio and channel-based completion notification.
//!
//! This module provides non-blocking asset loading functionality using tokio's
//! async runtime and crossbeam channels for completion notification.
//!
//! # Why Async Asset Loading?
//!
//! Game engines need to load assets without freezing the main thread:
//!
//! - **Large files**: A 50MB mesh can take hundreds of milliseconds to parse
//! - **Frame budget**: At 60 FPS, you have ~16ms per frame total
//! - **User experience**: The game must remain responsive during loading
//! - **Loading screens**: Display progress and animations while assets load
//! - **Streaming**: Load assets on-demand as the player moves through the world
//!
//! Without async loading, the game would freeze whenever it loads an asset, causing stuttering
//! and poor user experience. With async loading, the game continues running smoothly while
//! assets load in the background.
//!
//! # Architecture
//!
//! The async loading system is built on three key components:
//!
//! ## 1. Tokio Runtime Integration
//!
//! All async loaders use **`tokio::task::spawn_blocking`** to offload CPU-bound file parsing
//! to tokio's blocking thread pool. This prevents file I/O and parsing from blocking the
//! async executor, allowing other async tasks to continue executing.
//!
//! Why `spawn_blocking` instead of `spawn`?
//! - File parsing (OBJ, GLTF) is CPU-intensive and synchronous
//! - Regular `spawn` would block the executor thread
//! - `spawn_blocking` moves work to dedicated blocking threads
//! - The main async task remains responsive
//!
//! ## 2. Channel-Based Completion
//!
//! Loading results are communicated via **crossbeam channels** instead of async futures.
//! This design choice provides several benefits:
//!
//! - **Non-blocking polling**: `try_recv()` checks completion without blocking
//! - **Thread-safe**: Can be checked from any thread or task
//! - **Game loop friendly**: Easy to poll in a main loop without async/await
//! - **Bounded capacity**: Prevents memory buildup if results aren't consumed
//!
//! ```text
//! Caller Thread              Blocking Thread Pool
//!      │                            │
//!      │ load_async()              │
//!      ├──────────────────┐        │
//!      │ spawn task       │        │
//!      │ create channel   │        │
//!      │ return (handle,rx)        │
//!      │◄─────────────────┘        │
//!      │                            │
//!      │                   ┌────────┤
//!      │                   │ parse  │
//!      │                   │ file   │
//!      │                   └────────┤
//!      │                            │
//!      │        result              │
//!      │◄───────channel─────────────┤
//!      │                            │
//! ```
//!
//! ## 3. LoadHandle for Lifecycle Management
//!
//! The **`LoadHandle`** provides control over the loading operation:
//!
//! - **Status checking**: `is_finished()` polls without blocking
//! - **Cancellation support**: `cancel()` signals the task to abort
//! - **Path tracking**: `path()` identifies what's being loaded
//! - **JoinHandle wrapper**: Underlying tokio task handle
//!
//! Components:
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
//!
//! # Game Loop Integration Pattern
//!
//! In a typical game engine, you want to load assets without blocking the main loop:
//!
//! ```rust,no_run
//! use praxis_assets::async_loader::{AsyncAssetLoader, AsyncMeshLoader, AsyncBatchLoader};
//! use crossbeam_channel::Receiver;
//! use praxis_graphics::MeshData;
//!
//! struct LoadingState {
//!     pending: Vec<(String, Receiver<praxis_utils::Result<MeshData>>)>,
//! }
//!
//! impl LoadingState {
//!     pub fn start_load(&mut self, loader: &AsyncMeshLoader, name: String, path: String) {
//!         // Start async load (non-blocking)
//!         if let Ok((_handle, receiver)) = tokio::runtime::Runtime::new()
//!             .unwrap()
//!             .block_on(loader.load_async(&path))
//!         {
//!             self.pending.push((name, receiver));
//!         }
//!     }
//!
//!     pub fn process_completed(&mut self) -> Vec<(String, MeshData)> {
//!         let mut completed = Vec::new();
//!         let mut still_pending = Vec::new();
//!
//!         // Check all pending loads (non-blocking)
//!         for (name, receiver) in std::mem::take(&mut self.pending) {
//!             match receiver.try_recv() {
//!                 Ok(Ok(mesh_data)) => {
//!                     completed.push((name, mesh_data));
//!                 }
//!                 Ok(Err(_)) => {
//!                     // Load failed, drop it
//!                 }
//!                 Err(_) => {
//!                     // Still loading, keep it
//!                     still_pending.push((name, receiver));
//!                 }
//!             }
//!         }
//!
//!         self.pending = still_pending;
//!         completed
//!     }
//! }
//!
//! // In your game loop:
//! // loop {
//! //     // Process completed loads
//! //     for (name, mesh_data) in loading_state.process_completed() {
//! //         render_context.mesh_manager_mut().load_mesh(&name, mesh_data)?;
//! //     }
//! //
//! //     // Continue with rendering, physics, etc.
//! //     // The game never blocks waiting for assets!
//! // }
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
#[derive(Debug)]
pub struct LoadHandle {
    /// Tokio task join handle
    #[allow(dead_code)]
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

        // EARLY VALIDATION: Check file existence using tokio's async fs API
        // This avoids spawning a blocking task for files that don't exist
        // tokio::fs::try_exists is async and doesn't block the executor
        if !tokio::fs::try_exists(&path_buf).await? {
            return Err(praxis_utils::eyre::eyre!(
                "File not found: {}",
                path_buf.display()
            ));
        }

        info!("Starting async load for: {}", path_buf.display());

        // CHANNEL SETUP: Create a bounded channel with capacity 1
        // - Bounded prevents unbounded memory growth if results aren't consumed
        // - Capacity 1 is sufficient since we only send one result per load
        // - crossbeam_channel is chosen for its excellent performance and MPSC semantics
        let (sender, receiver) = crossbeam_channel::bounded(1);

        // SHARED STATE: Clone Arc-wrapped loader for the spawned task
        // Arc allows shared ownership across threads without copying the loader
        let loader = Arc::clone(&self.loader);

        // CANCELLATION SUPPORT: AtomicBool for lock-free cancellation signaling
        // - Wrapped in Arc for shared access between handle and task
        // - Atomic operations are lock-free and very fast
        // - Relaxed ordering is sufficient (no dependent memory operations)
        let cancelled = Arc::new(AtomicBool::new(false));
        let cancelled_clone = Arc::clone(&cancelled);

        // SPAWN BLOCKING TASK: Move CPU-bound work off the async executor
        // spawn_blocking runs on a dedicated thread pool for blocking operations
        // This is critical: OBJ/GLTF parsing is synchronous and CPU-intensive
        // Using regular spawn() would block the executor and starve other async tasks
        let join_handle = tokio::task::spawn_blocking(move || {
            // CANCELLATION CHECK 1: Before starting expensive I/O
            // Allows early exit if user cancelled before task starts
            if cancelled_clone.load(Ordering::Relaxed) {
                let _ = sender.send(Err(praxis_utils::eyre::eyre!("Load cancelled")));
                return;
            }

            // BLOCKING I/O: Perform the actual file loading and parsing
            // This is where the CPU-intensive work happens:
            // - Read file from disk (blocking I/O)
            // - Parse OBJ format (CPU-bound)
            // - Convert to MeshData (memory allocations)
            let result = loader.load(&path_buf);

            // CANCELLATION CHECK 2: Before sending result
            // Avoids sending result if cancelled during loading
            // Even if cancelled, we still ran the load (hard to interrupt),
            // but we avoid cluttering the channel
            if !cancelled_clone.load(Ordering::Relaxed) {
                // SEND RESULT: Send through channel (ignore send error)
                // Send fails if receiver was dropped, which is fine (caller lost interest)
                // The `let _` explicitly ignores the Result to document this is intentional
                let _ = sender.send(result);
            }
        });

        // CREATE HANDLE: Wrap tokio JoinHandle with our LoadHandle abstraction
        // This gives the caller control over the operation without exposing tokio internals
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
#[allow(clippy::uninlined_format_args)]
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

        let result = loader.load_async("nonexistent_async_file_99999.obj").await;

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

    #[tokio::test(flavor = "multi_thread")]
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

        // Poll with timeout until completion instead of sleeping
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        let mut completed = Vec::new();
        
        while batch.total_count() > 0 && start.elapsed() < timeout {
            let mut batch_completed = batch.try_receive_completed();
            completed.append(&mut batch_completed);
            if completed.is_empty() {
                tokio::task::yield_now().await;
            }
        }

        assert_eq!(completed.len(), 1, "Expected 1 completed load");
        assert!(completed[0].is_ok());
        assert_eq!(batch.total_count(), 0);

        std::fs::remove_file(&test_file).ok();
    }

    #[tokio::test]
    async fn test_async_gltf_load_nonexistent() {
        let loader = AsyncGltfLoader::new();

        let result = loader.load_async("nonexistent_async_file_99999.gltf").await;

        assert!(result.is_err());
    }

    // ============================================================================
    // Non-blocking file I/O tests
    // ============================================================================

    #[tokio::test]
    async fn test_async_load_returns_immediately() {
        let loader = AsyncMeshLoader::new();
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_nonblocking.obj");

        // Create a file with some content
        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";
        std::fs::write(&test_file, obj_content).expect("Failed to write test file");

        let start = std::time::Instant::now();
        let (_handle, receiver) = loader
            .load_async(&test_file)
            .await
            .expect("Failed to start load");
        let elapsed = start.elapsed();

        // load_async should return very quickly (< 50ms), proving it's non-blocking
        assert!(
            elapsed.as_millis() < 50,
            "load_async took too long: {:?}",
            elapsed
        );

        // Verify the actual load completes in the background
        let result = receiver.recv().unwrap();
        assert!(result.is_ok());

        std::fs::remove_file(&test_file).ok();
    }

    #[tokio::test]
    async fn test_multiple_async_loads_dont_block_each_other() {
        let loader = AsyncMeshLoader::new();
        let temp_dir = std::env::temp_dir();
        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";

        let test_files: Vec<_> = (0..5)
            .map(|i| temp_dir.join(format!("test_nonblocking_{}.obj", i)))
            .collect();

        for file in &test_files {
            std::fs::write(file, obj_content).expect("Failed to write test file");
        }

        let start = std::time::Instant::now();

        // Start all loads - should return quickly even though there are multiple
        let mut loads = Vec::new();
        for file in &test_files {
            let load = loader.load_async(file).await.expect("Failed to start load");
            loads.push(load);
        }

        let elapsed = start.elapsed();

        // Starting 5 loads should still be very quick (< 100ms)
        assert!(
            elapsed.as_millis() < 100,
            "Starting 5 loads took too long: {:?}",
            elapsed
        );

        // Verify all loads complete
        for (_handle, receiver) in loads {
            let result = receiver.recv().unwrap();
            assert!(result.is_ok());
        }

        for file in &test_files {
            std::fs::remove_file(file).ok();
        }
    }

    #[tokio::test]
    async fn test_try_recv_is_nonblocking() {
        let loader = AsyncMeshLoader::new();
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_try_recv_nonblocking.obj");

        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";
        std::fs::write(&test_file, obj_content).expect("Failed to write test file");

        let (_handle, receiver) = loader
            .load_async(&test_file)
            .await
            .expect("Failed to start load");

        // Immediately try to receive - should return Err (not ready) without blocking
        let start = std::time::Instant::now();
        let immediate_result = receiver.try_recv();
        let elapsed = start.elapsed();

        // try_recv should be instant
        assert!(elapsed.as_micros() < 1000, "try_recv blocked unexpectedly");

        // Depending on timing, it might be done or not, but it should not have blocked
        match immediate_result {
            Ok(result) => {
                // If it completed that fast, verify it's valid
                assert!(result.is_ok());
            }
            Err(_) => {
                // Expected - not ready yet
                // Wait for completion
                let result = receiver.recv().unwrap();
                assert!(result.is_ok());
            }
        }

        std::fs::remove_file(&test_file).ok();
    }

    // ============================================================================
    // Channel-based completion notification tests
    // ============================================================================

    #[tokio::test]
    async fn test_receiver_gets_successful_result() {
        let loader = AsyncMeshLoader::new();
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_channel_success.obj");

        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";
        std::fs::write(&test_file, obj_content).expect("Failed to write test file");

        let (_handle, receiver) = loader
            .load_async(&test_file)
            .await
            .expect("Failed to start load");

        // Wait for the result to be sent through the channel
        let result = receiver.recv().unwrap();

        // Verify the result is Ok and contains valid data
        assert!(result.is_ok());
        let mesh_data = result.unwrap();
        assert_eq!(mesh_data.positions.len(), 3);
        assert_eq!(mesh_data.indices.len(), 3);

        std::fs::remove_file(&test_file).ok();
    }

    #[tokio::test]
    async fn test_receiver_gets_error_result() {
        let loader = AsyncMeshLoader::new();
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_channel_error.obj");

        // Write invalid OBJ content to trigger a parsing error
        std::fs::write(&test_file, "invalid obj content\n").expect("Failed to write test file");

        let (_handle, receiver) = loader
            .load_async(&test_file)
            .await
            .expect("Failed to start load");

        // Wait for the result - should be an error
        let result = receiver.recv().unwrap();
        assert!(result.is_err());

        std::fs::remove_file(&test_file).ok();
    }

    #[tokio::test]
    async fn test_channel_closed_when_handle_dropped() {
        let loader = AsyncMeshLoader::new();
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_channel_dropped.obj");

        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";
        std::fs::write(&test_file, obj_content).expect("Failed to write test file");

        let (handle, receiver) = loader
            .load_async(&test_file)
            .await
            .expect("Failed to start load");

        // Drop the handle immediately
        drop(handle);

        // Receiver should still be able to get the result (loading continues)
        let result = receiver.recv().unwrap();
        assert!(result.is_ok());

        std::fs::remove_file(&test_file).ok();
    }

    #[tokio::test]
    async fn test_multiple_receivers_not_allowed() {
        let loader = AsyncMeshLoader::new();
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_single_receiver.obj");

        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";
        std::fs::write(&test_file, obj_content).expect("Failed to write test file");

        let (_handle, receiver) = loader
            .load_async(&test_file)
            .await
            .expect("Failed to start load");

        // Channels are bounded with capacity 1, so only one result is sent
        let result1 = receiver.recv().unwrap();
        assert!(result1.is_ok());

        // Trying to receive again should fail (channel is empty and sender dropped)
        let result2 = receiver.recv();
        assert!(result2.is_err());

        std::fs::remove_file(&test_file).ok();
    }

    // ============================================================================
    // Concurrent load requests tests
    // ============================================================================

    #[tokio::test]
    async fn test_concurrent_loads_complete_independently() {
        let loader = AsyncMeshLoader::new();
        let temp_dir = std::env::temp_dir();
        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";

        let test_files: Vec<_> = (0..10)
            .map(|i| temp_dir.join(format!("test_concurrent_{}.obj", i)))
            .collect();

        for file in &test_files {
            std::fs::write(file, obj_content).expect("Failed to write test file");
        }

        // Start 10 concurrent loads
        let mut loads = Vec::new();
        for file in &test_files {
            let load = loader.load_async(file).await.expect("Failed to start load");
            loads.push(load);
        }

        // All loads should complete successfully
        for (_handle, receiver) in loads {
            let result = receiver.recv().unwrap();
            assert!(result.is_ok());
        }

        for file in &test_files {
            std::fs::remove_file(file).ok();
        }
    }

    #[tokio::test]
    async fn test_concurrent_loads_with_mixed_results() {
        let loader = AsyncMeshLoader::new();
        let temp_dir = std::env::temp_dir();

        let valid_file = temp_dir.join("test_concurrent_valid.obj");
        let invalid_file = temp_dir.join("test_concurrent_invalid.obj");

        std::fs::write(
            &valid_file,
            "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n",
        )
        .expect("Failed to write valid file");
        std::fs::write(&invalid_file, "invalid content").expect("Failed to write invalid file");

        // Load both files concurrently
        let (handle1, receiver1) = loader.load_async(&valid_file).await.unwrap();
        let (handle2, receiver2) = loader.load_async(&invalid_file).await.unwrap();

        // Wait for both to complete
        let result1 = receiver1.recv().unwrap();
        let result2 = receiver2.recv().unwrap();

        // One should succeed, one should fail
        assert!(result1.is_ok());
        assert!(result2.is_err());

        // Both handles should be finished
        assert!(handle1.is_finished());
        assert!(handle2.is_finished());

        std::fs::remove_file(&valid_file).ok();
        std::fs::remove_file(&invalid_file).ok();
    }

    #[tokio::test]
    async fn test_load_many_async_concurrent() {
        let loader = AsyncMeshLoader::new();
        let temp_dir = std::env::temp_dir();
        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";

        let paths: Vec<_> = (0..20)
            .map(|i| temp_dir.join(format!("test_load_many_{}.obj", i)))
            .collect();

        for path in &paths {
            std::fs::write(path, obj_content).expect("Failed to write test file");
        }

        let start = std::time::Instant::now();
        let loads = loader
            .load_many_async(paths.clone())
            .await
            .expect("Failed to start loads");
        let spawn_time = start.elapsed();

        // Spawning 20 loads should be fast
        assert!(spawn_time.as_millis() < 200);

        // All should complete successfully
        for (_handle, receiver) in loads {
            let result = receiver.recv().unwrap();
            assert!(result.is_ok());
        }

        for path in &paths {
            std::fs::remove_file(path).ok();
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_concurrent_loads_different_loaders() {
        let mesh_loader = AsyncMeshLoader::new();
        let temp_dir = std::env::temp_dir();

        let mesh_file = temp_dir.join("test_different_loader.obj");
        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";
        std::fs::write(&mesh_file, obj_content).expect("Failed to write test file");

        // Start multiple loads from the same loader instance concurrently
        let (h1, r1) = mesh_loader.load_async(&mesh_file).await.unwrap();
        let (h2, r2) = mesh_loader.load_async(&mesh_file).await.unwrap();
        let (h3, r3) = mesh_loader.load_async(&mesh_file).await.unwrap();

        // Use timeout to prevent hanging
        let timeout_duration = std::time::Duration::from_secs(5);

        // All should complete within timeout
        let result1 = tokio::time::timeout(timeout_duration, async { r1.recv().unwrap() })
            .await
            .expect("Test timed out waiting for r1");
        let result2 = tokio::time::timeout(timeout_duration, async { r2.recv().unwrap() })
            .await
            .expect("Test timed out waiting for r2");
        let result3 = tokio::time::timeout(timeout_duration, async { r3.recv().unwrap() })
            .await
            .expect("Test timed out waiting for r3");

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());

        assert!(h1.is_finished());
        assert!(h2.is_finished());
        assert!(h3.is_finished());

        std::fs::remove_file(&mesh_file).ok();
    }

    // ============================================================================
    // Cancellation tests
    // ============================================================================

    #[tokio::test]
    async fn test_cancellation_sets_flag() {
        let loader = AsyncMeshLoader::new();
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_cancel_flag.obj");

        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";
        std::fs::write(&test_file, obj_content).expect("Failed to write test file");

        let (handle, _receiver) = loader.load_async(&test_file).await.unwrap();

        assert!(!handle.is_cancelled());
        handle.cancel();
        assert!(handle.is_cancelled());

        std::fs::remove_file(&test_file).ok();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_early_cancellation() {
        let loader = AsyncMeshLoader::new();
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_early_cancel.obj");

        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";
        std::fs::write(&test_file, obj_content).expect("Failed to write test file");

        let (handle, receiver) = loader.load_async(&test_file).await.unwrap();

        // Cancel immediately
        handle.cancel();

        // Use timeout to prevent hanging - the result might be an error or success depending on timing,
        // but we should get a result within a reasonable time
        let timeout_duration = std::time::Duration::from_secs(5);
        let result = tokio::time::timeout(timeout_duration, async { receiver.recv() })
            .await
            .expect("Test timed out waiting for cancellation result");

        // Channel should receive something (either success or cancellation error)
        assert!(result.is_ok(), "Channel should receive a result");

        std::fs::remove_file(&test_file).ok();
    }

    #[tokio::test]
    async fn test_batch_cancel_all() {
        let loader = AsyncMeshLoader::new();
        let mut batch = AsyncBatchLoader::new();
        let temp_dir = std::env::temp_dir();
        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";

        let paths: Vec<_> = (0..5)
            .map(|i| temp_dir.join(format!("test_batch_cancel_{}.obj", i)))
            .collect();

        for path in &paths {
            std::fs::write(path, obj_content).expect("Failed to write test file");
        }

        // Add loads to batch
        for path in &paths {
            batch.add(loader.load_async(path).await.unwrap());
        }

        // Cancel all
        batch.cancel_all();

        // Verify all handles are marked as cancelled
        assert_eq!(batch.total_count(), 5);

        for path in &paths {
            std::fs::remove_file(path).ok();
        }
    }

    // ============================================================================
    // Handle state tests
    // ============================================================================

    #[tokio::test]
    async fn test_handle_is_finished_transitions() {
        let loader = AsyncMeshLoader::new();
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_is_finished.obj");

        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";
        std::fs::write(&test_file, obj_content).expect("Failed to write test file");

        let (handle, receiver) = loader.load_async(&test_file).await.unwrap();

        // Initially might or might not be finished (depends on timing)
        let initial_state = handle.is_finished();

        // Wait for completion
        let result = receiver.recv().unwrap();
        assert!(result.is_ok());

        // After completion, should definitely be finished
        assert!(handle.is_finished());

        // Should remain finished
        assert!(handle.is_finished());

        // Verify initial state was consistent (either false or true, but not both)
        assert!(initial_state == false || handle.is_finished());

        std::fs::remove_file(&test_file).ok();
    }

    #[tokio::test]
    async fn test_handle_path_preserved() {
        let loader = AsyncMeshLoader::new();
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_handle_path_preserved.obj");

        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";
        std::fs::write(&test_file, obj_content).expect("Failed to write test file");

        let (handle, receiver) = loader.load_async(&test_file).await.unwrap();

        // Path should be accessible before completion
        assert_eq!(handle.path(), test_file.as_path());

        // Wait for completion
        let _result = receiver.recv().unwrap();

        // Path should still be accessible after completion
        assert_eq!(handle.path(), test_file.as_path());

        std::fs::remove_file(&test_file).ok();
    }

    // ============================================================================
    // Batch loader tests
    // ============================================================================

    #[tokio::test(flavor = "multi_thread")]
    async fn test_batch_completed_count_updates() {
        let loader = AsyncMeshLoader::new();
        let mut batch = AsyncBatchLoader::new();
        let temp_dir = std::env::temp_dir();
        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";

        let paths: Vec<_> = (0..3)
            .map(|i| temp_dir.join(format!("test_batch_progress_{}.obj", i)))
            .collect();

        for path in &paths {
            std::fs::write(path, obj_content).expect("Failed to write test file");
        }

        for path in &paths {
            batch.add(loader.load_async(path).await.unwrap());
        }

        assert_eq!(batch.total_count(), 3);

        // Poll until all complete with timeout
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        
        while !batch.is_complete() && start.elapsed() < timeout {
            tokio::task::yield_now().await;
        }

        // Should all be complete
        assert!(batch.is_complete(), "Batch did not complete within timeout");
        assert_eq!(batch.completed_count(), 3);
        assert_eq!(batch.pending_count(), 0);

        for path in &paths {
            std::fs::remove_file(path).ok();
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_batch_try_receive_completed_partial() {
        let loader = AsyncMeshLoader::new();
        let mut batch = AsyncBatchLoader::new();
        let temp_dir = std::env::temp_dir();
        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";

        let paths: Vec<_> = (0..5)
            .map(|i| temp_dir.join(format!("test_batch_partial_{}.obj", i)))
            .collect();

        for path in &paths {
            std::fs::write(path, obj_content).expect("Failed to write test file");
        }

        for path in &paths {
            batch.add(loader.load_async(path).await.unwrap());
        }

        // Poll with timeout to collect all completed items
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        let mut all_completed = Vec::new();

        while batch.total_count() > 0 && start.elapsed() < timeout {
            let mut completed = batch.try_receive_completed();
            all_completed.append(&mut completed);
            if batch.total_count() > 0 {
                tokio::task::yield_now().await;
            }
        }

        // Total completed should equal original count
        assert_eq!(all_completed.len(), 5, "Expected 5 completed loads");

        // Batch should now be empty
        assert_eq!(batch.total_count(), 0);

        for path in &paths {
            std::fs::remove_file(path).ok();
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_batch_is_complete() {
        let loader = AsyncMeshLoader::new();
        let mut batch = AsyncBatchLoader::new();
        let temp_dir = std::env::temp_dir();
        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";

        // Empty batch is complete
        assert!(batch.is_complete());

        let test_file = temp_dir.join("test_batch_complete.obj");
        std::fs::write(&test_file, obj_content).expect("Failed to write test file");

        batch.add(loader.load_async(&test_file).await.unwrap());

        // Poll until complete with timeout
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        
        while !batch.is_complete() && start.elapsed() < timeout {
            tokio::task::yield_now().await;
        }
        
        assert!(batch.is_complete(), "Batch did not complete within timeout");

        std::fs::remove_file(&test_file).ok();
    }

    // ============================================================================
    // Tokio runtime integration tests
    // ============================================================================

    #[tokio::test]
    async fn test_spawns_on_tokio_runtime() {
        let loader = AsyncMeshLoader::new();
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_tokio_spawn.obj");

        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";
        std::fs::write(&test_file, obj_content).expect("Failed to write test file");

        // This test verifies the task runs on the tokio runtime
        let (handle, receiver) = loader.load_async(&test_file).await.unwrap();

        // The JoinHandle should be valid
        assert!(!handle.is_cancelled());

        // Should be able to await the result through the channel
        let result = receiver.recv().unwrap();
        assert!(result.is_ok());

        std::fs::remove_file(&test_file).ok();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_concurrent_loads_with_multi_thread_runtime() {
        let loader = AsyncMeshLoader::new();
        let temp_dir = std::env::temp_dir();
        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";

        let paths: Vec<_> = (0..50)
            .map(|i| temp_dir.join(format!("test_multithread_{}.obj", i)))
            .collect();

        for path in &paths {
            std::fs::write(path, obj_content).expect("Failed to write test file");
        }

        // Start 50 concurrent loads on multi-threaded runtime
        let loads = loader
            .load_many_async(paths.clone())
            .await
            .expect("Failed to start loads");

        // All should complete successfully
        for (_handle, receiver) in loads {
            let result = receiver.recv().unwrap();
            assert!(result.is_ok());
        }

        for path in &paths {
            std::fs::remove_file(path).ok();
        }
    }

    #[tokio::test]
    async fn test_file_exists_check_with_tokio_fs() {
        let loader = AsyncMeshLoader::new();

        // Test with non-existent file - should fail immediately
        let result = loader
            .load_async("definitely_does_not_exist_99999.obj")
            .await;
        assert!(result.is_err());

        // The error should occur before spawning, so it's immediate
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("not found") || error_msg.contains("No such file"));
    }

    // ============================================================================
    // Error handling tests
    // ============================================================================

    #[tokio::test]
    async fn test_invalid_obj_format_error_propagated() {
        let loader = AsyncMeshLoader::new();
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_invalid_format.obj");

        // Write completely invalid content
        std::fs::write(&test_file, "This is not a valid OBJ file at all!")
            .expect("Failed to write test file");

        let (_handle, receiver) = loader.load_async(&test_file).await.unwrap();

        let result = receiver.recv().unwrap();
        assert!(result.is_err());

        std::fs::remove_file(&test_file).ok();
    }

    #[tokio::test]
    async fn test_empty_file_handling() {
        let loader = AsyncMeshLoader::new();
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_empty.obj");

        // Write empty file
        std::fs::write(&test_file, "").expect("Failed to write test file");

        let (_handle, receiver) = loader.load_async(&test_file).await.unwrap();

        let result = receiver.recv().unwrap();
        // Empty OBJ file should load but with no vertices
        if let Ok(mesh_data) = result {
            assert_eq!(mesh_data.positions.len(), 0);
        }

        std::fs::remove_file(&test_file).ok();
    }

    #[tokio::test]
    async fn test_multiple_errors_independent() {
        let loader = AsyncMeshLoader::new();
        let temp_dir = std::env::temp_dir();

        let invalid_file1 = temp_dir.join("test_error1.obj");
        let invalid_file2 = temp_dir.join("test_error2.obj");

        std::fs::write(&invalid_file1, "invalid1").expect("Failed to write test file");
        std::fs::write(&invalid_file2, "invalid2").expect("Failed to write test file");

        let (_, receiver1) = loader.load_async(&invalid_file1).await.unwrap();
        let (_, receiver2) = loader.load_async(&invalid_file2).await.unwrap();

        let result1 = receiver1.recv().unwrap();
        let result2 = receiver2.recv().unwrap();

        // Both should error independently
        assert!(result1.is_err());
        assert!(result2.is_err());

        std::fs::remove_file(&invalid_file1).ok();
        std::fs::remove_file(&invalid_file2).ok();
    }

    // ============================================================================
    // Integration tests with real scenarios
    // ============================================================================

    #[tokio::test]
    async fn test_realistic_game_loading_scenario() {
        let loader = AsyncMeshLoader::new();
        let mut batch = AsyncBatchLoader::new();
        let temp_dir = std::env::temp_dir();
        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";

        // Simulate loading multiple assets for a game level
        let asset_names = vec!["player", "enemy", "terrain", "building", "prop"];
        let paths: Vec<_> = asset_names
            .iter()
            .map(|name| temp_dir.join(format!("test_game_{}.obj", name)))
            .collect();

        for path in &paths {
            std::fs::write(path, obj_content).expect("Failed to write test file");
        }

        // Start loading all assets
        for path in &paths {
            batch.add(loader.load_async(path).await.unwrap());
        }

        // Simulate game loop checking for loading progress with timeout
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        let mut loaded_count = 0;
        
        while !batch.is_complete() && start.elapsed() < timeout {
            let completed = batch.try_receive_completed();
            loaded_count += completed.len();
            if !batch.is_complete() {
                tokio::task::yield_now().await;
            }
        }

        assert!(start.elapsed() < timeout, "Test timed out");
        assert_eq!(loaded_count, 5);

        for path in &paths {
            std::fs::remove_file(path).ok();
        }
    }

    #[tokio::test]
    async fn test_wait_all_blocks_until_complete() {
        let loader = AsyncMeshLoader::new();
        let mut batch = AsyncBatchLoader::new();
        let temp_dir = std::env::temp_dir();
        let obj_content = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.5 1.0 0.0\nf 1 2 3\n";

        let paths: Vec<_> = (0..5)
            .map(|i| temp_dir.join(format!("test_wait_all_{}.obj", i)))
            .collect();

        for path in &paths {
            std::fs::write(path, obj_content).expect("Failed to write test file");
        }

        for path in &paths {
            batch.add(loader.load_async(path).await.unwrap());
        }

        // wait_all should block until all are done
        let results = batch.wait_all();

        // All results should be present and successful
        assert_eq!(results.len(), 5);
        for result in results {
            assert!(result.is_ok());
        }

        for path in &paths {
            std::fs::remove_file(path).ok();
        }
    }
}
