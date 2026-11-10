//! The IO pool (ADR 0016), and the Chunk streamer it serves each frame.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ops::Range;
use std::sync::mpsc;
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use anyhow::Context;
use bevy_ecs::prelude::*;
use glam::DVec3;

use crate::assets::{AssetReader, MaterialPool, MeshPool, MeshUse, PoolLoader};
use crate::camera::Camera;
use crate::cooked::{AssetId, CookedCache, CookedChunk, CookedMaterial, CookedMesh, CookedTexture};
use crate::render::{Renderer, mips_written_bytes, texture_written_bytes};
use crate::scenes::ReferenceScene;
use crate::streaming::chunks::load_cooked_chunk;
use crate::streaming::mips::{MipPriority, MipRequest};
use crate::streaming::{
    ChunkFrame, ChunkIo, ChunkStreamer, FinishedLoad, LoadedChunk, LoadedChunks, distance_metres,
};

const IO_THREADS: usize = 4;

/// The Asset IDs the IO pool need not decode: those loaded, and those
/// decoded but not yet loaded. One the pools free leaves it as it is freed.
type Claimed = Arc<Mutex<HashSet<AssetId>>>;

/// Decoded Assets by Asset ID.
#[derive(Default)]
pub(crate) struct DecodedAssets {
    meshes: HashMap<AssetId, CookedMesh>,
    materials: HashMap<AssetId, CookedMaterial>,
    textures: HashMap<AssetId, CookedTexture>,
    /// Assets a load in flight claimed that the main thread read from the
    /// Cooked cache first, whose decoded copies are dropped when they come.
    read_from_cache: HashSet<AssetId>,
}

impl DecodedAssets {
    /// Takes `other`'s Assets, but those already read from the Cooked cache.
    fn extend(&mut self, other: DecodedAssets) {
        let read_from_cache = &mut self.read_from_cache;
        let mut wanted = |id: &AssetId| !read_from_cache.remove(id);
        let meshes = other.meshes.into_iter().filter(|(id, _)| wanted(id));
        self.meshes.extend(meshes);
        let materials = other.materials.into_iter().filter(|(id, _)| wanted(id));
        self.materials.extend(materials);
        let textures = other.textures.into_iter().filter(|(id, _)| wanted(id));
        self.textures.extend(textures);
    }
}

/// A Chunk the IO pool read and decoded, with the Assets it names, through
/// its Meshes' Materials to their Textures, that no earlier load claimed.
struct DecodedChunk {
    id: AssetId,
    coordinates: [i32; 2],
    chunk: CookedChunk,
    assets: DecodedAssets,
    bytes_read: u64,
}

/// Mips of a Texture the IO pool read and decoded.
struct DecodedMips {
    levels: Vec<Vec<u8>>,
    bytes_read: u64,
}

/// What the IO pool reads: the Chunk with an Asset ID at its coordinates,
/// with the Assets it names, or a Texture's mips.
#[derive(Debug)]
enum IoRequest {
    Chunk([i32; 2], AssetId),
    Mips(MipRequest),
}

enum Finished {
    Chunk([i32; 2], Box<anyhow::Result<DecodedChunk>>),
    Mips(MipRequest, anyhow::Result<DecodedMips>),
}

/// A few std threads that read Chunks and the Assets they name, and
/// Textures' mips, from the Cooked cache and decode them, off the main
/// thread.
struct IoPool {
    requests: Option<mpsc::Sender<IoRequest>>,
    finished: mpsc::Receiver<Finished>,
    threads: Vec<JoinHandle<()>>,
    in_flight: usize,
    /// Loads the threads finished and the main thread has not taken, by
    /// coordinates.
    ready: BTreeMap<[i32; 2], anyhow::Result<DecodedChunk>>,
    /// Mips the threads read and the main thread has not applied, by the
    /// finest requested, then Texture, with the mips requested.
    ready_mips: BTreeMap<(u32, AssetId), (Range<u32>, anyhow::Result<DecodedMips>)>,
}

impl IoPool {
    fn new(cache: Arc<CookedCache>, claimed: Claimed) -> anyhow::Result<Self> {
        let (request_sender, request_receiver) = mpsc::channel::<IoRequest>();
        let (finished_sender, finished) = mpsc::channel();
        let request_receiver = Arc::new(Mutex::new(request_receiver));
        let threads = (0..IO_THREADS)
            .map(|_| {
                let (requests, finished, cache, claimed) = (
                    request_receiver.clone(),
                    finished_sender.clone(),
                    cache.clone(),
                    claimed.clone(),
                );
                std::thread::Builder::new()
                    .name("IO Pool".to_string())
                    .spawn(move || {
                        loop {
                            let request = requests
                                .lock()
                                .unwrap_or_else(PoisonError::into_inner)
                                .recv();
                            let finished_load = match request {
                                Ok(IoRequest::Chunk(coordinates, id)) => Finished::Chunk(
                                    coordinates,
                                    Box::new(decode_chunk(&cache, &claimed, id, coordinates)),
                                ),
                                Ok(IoRequest::Mips(request)) => {
                                    let decoded = decode_mips(&cache, &request);
                                    Finished::Mips(request, decoded)
                                }
                                Err(_) => return,
                            };
                            if finished.send(finished_load).is_err() {
                                return;
                            }
                        }
                    })
                    .context("starting the IO pool")
            })
            .collect::<anyhow::Result<_>>()?;
        Ok(Self {
            requests: Some(request_sender),
            finished,
            threads,
            in_flight: 0,
            ready: BTreeMap::new(),
            ready_mips: BTreeMap::new(),
        })
    }

    fn request(&mut self, request: IoRequest) {
        let refused = match &self.requests {
            Some(requests) => requests.send(request).err().map(|error| error.0),
            None => Some(request),
        };
        match refused {
            None => self.in_flight += 1,
            Some(request) => tracing::error!(?request, "not requested: the IO pool stopped"),
        }
    }

    /// Takes the loads finished so far into `ready` and `ready_mips`,
    /// without waiting.
    fn receive(&mut self) {
        while let Ok(finished) = self.finished.try_recv() {
            self.take(finished);
        }
    }

    /// Waits for every load in flight, into `ready` and `ready_mips`.
    fn receive_all(&mut self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.in_flight > 0,
            "the Chunk streamer waits on loads the IO pool does not hold"
        );
        while self.in_flight > 0 {
            let finished = self
                .finished
                .recv()
                .context("the IO pool's threads stopped")?;
            self.take(finished);
        }
        Ok(())
    }

    fn take(&mut self, finished: Finished) {
        match finished {
            Finished::Chunk(coordinates, decoded) => {
                self.ready.insert(coordinates, *decoded);
            }
            Finished::Mips(request, decoded) => {
                let key = (request.levels.start, request.texture);
                self.ready_mips.insert(key, (request.levels, decoded));
            }
        }
        self.in_flight -= 1;
    }
}

impl Drop for IoPool {
    fn drop(&mut self) {
        self.requests = None;
        for thread in self.threads.drain(..) {
            thread.join().ok();
        }
    }
}

/// Reads and decodes the Chunk `id` at `coordinates` from `cache`, and each
/// Mesh, Material and Texture it names that `claimed` lacks, which it
/// claims.
fn decode_chunk(
    cache: &CookedCache,
    claimed: &Mutex<HashSet<AssetId>>,
    id: AssetId,
    coordinates: [i32; 2],
) -> anyhow::Result<DecodedChunk> {
    let mut bytes_read = 0;
    let mut read = |asset: AssetId| {
        let blob = cache.read(asset)?;
        bytes_read += blob.len() as u64;
        anyhow::Ok(blob)
    };
    let mut seen = HashSet::new();
    let mut claim = |asset: AssetId| {
        seen.insert(asset)
            && claimed
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .insert(asset)
    };
    let chunk =
        CookedChunk::from_blob(&read(id)?).with_context(|| format!("reading Chunk {id}"))?;
    let mut assets = DecodedAssets::default();
    let mut materials: Vec<AssetId> = chunk
        .placements
        .iter()
        .filter_map(|placement| placement.material)
        .collect();
    for placement in &chunk.placements {
        let mesh_id = placement.mesh;
        if claim(mesh_id) {
            let mesh = CookedMesh::from_blob(&read(mesh_id)?)
                .with_context(|| format!("reading Mesh {mesh_id}"))?;
            materials.extend(
                mesh.primitives
                    .iter()
                    .filter_map(|primitive| primitive.material),
            );
            assets.meshes.insert(mesh_id, mesh);
        }
    }
    let mut textures = Vec::new();
    for material_id in materials {
        if claim(material_id) {
            let material = CookedMaterial::from_blob(&read(material_id)?)
                .with_context(|| format!("reading Material {material_id}"))?;
            textures.extend(material.textures.iter().flatten());
            assets.materials.insert(material_id, material);
        }
    }
    for texture_id in textures {
        if claim(texture_id) {
            let texture = CookedTexture::resident_tail_from_blob(&read(texture_id)?)
                .with_context(|| format!("reading Texture {texture_id}"))?;
            assets.textures.insert(texture_id, texture);
        }
    }
    Ok(DecodedChunk {
        id,
        coordinates,
        chunk,
        assets,
        bytes_read,
    })
}

/// Reads and decodes `request`'s mips from `cache`.
fn decode_mips(cache: &CookedCache, request: &MipRequest) -> anyhow::Result<DecodedMips> {
    let blob = cache.read(request.texture)?;
    let levels = CookedTexture::mips_from_blob(&blob, request.levels.clone())
        .with_context(|| format!("reading Texture {}'s mips", request.texture))?;
    Ok(DecodedMips {
        levels,
        bytes_read: blob.len() as u64,
    })
}

/// Reads each Asset from what the IO pool decoded, taking it, or else from
/// the Cooked cache, and claims each it read, so the IO pool skips it.
pub(crate) struct DecodedReader<'a> {
    decoded: RefCell<&'a mut DecodedAssets>,
    cache: &'a CookedCache,
    claimed: &'a Mutex<HashSet<AssetId>>,
    /// Whether a Texture read from the Cooked cache is its Resident tail
    /// alone, as the IO pool decodes it, rather than its whole chain.
    reads_resident_tails: bool,
}

impl DecodedReader<'_> {
    fn claimed<T>(
        &self,
        id: AssetId,
        decoded: Option<T>,
        read: impl FnOnce() -> anyhow::Result<T>,
    ) -> anyhow::Result<T> {
        let decoded_on_the_io_pool = decoded.is_some();
        let asset = match decoded {
            Some(asset) => asset,
            None => read()?,
        };
        let newly_claimed = self
            .claimed
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(id);
        if !newly_claimed && !decoded_on_the_io_pool {
            self.decoded.borrow_mut().read_from_cache.insert(id);
        }
        Ok(asset)
    }
}

impl AssetReader for DecodedReader<'_> {
    fn material(&self, id: AssetId) -> anyhow::Result<CookedMaterial> {
        let decoded = self.decoded.borrow_mut().materials.remove(&id);
        self.claimed(id, decoded, || self.cache.material(id))
    }

    fn texture(&self, id: AssetId) -> anyhow::Result<CookedTexture> {
        let decoded = self.decoded.borrow_mut().textures.remove(&id);
        self.claimed(id, decoded, || {
            if self.reads_resident_tails {
                CookedTexture::resident_tail_from_blob(&self.cache.read(id)?)
                    .with_context(|| format!("reading Texture {id}"))
            } else {
                self.cache.texture(id)
            }
        })
    }

    fn mesh(&self, id: AssetId) -> anyhow::Result<CookedMesh> {
        let decoded = self.decoded.borrow_mut().meshes.remove(&id);
        self.claimed(id, decoded, || self.cache.mesh(id))
    }

    fn chunk(&self, id: AssetId) -> anyhow::Result<CookedChunk> {
        self.cache.chunk(id)
    }

    fn forget(&self, assets: &[AssetId]) {
        let mut claimed = self.claimed.lock().unwrap_or_else(PoisonError::into_inner);
        for asset in assets {
            claimed.remove(asset);
        }
    }
}

/// What streaming did in one frame, for its Frame stats.
#[derive(Default)]
pub(crate) struct StreamedFrame {
    pub late_chunks: u32,
    pub chunks_loaded: u32,
    pub chunks_unloaded: u32,
    pub bytes_read: u64,
    /// Bytes the frame's loads wrote into the load staging.
    pub written_bytes: u64,
    pub pool_entries_freed: u32,
    pub meshes_refused: u32,
    pub materials_refused: u32,
    pub chunks_not_loaded: u32,
    pub late_mips: u32,
    /// Whether the camera still waits for a Chunk within the Load radius
    /// since it last arrived.
    pub holding: bool,
    /// On the frame a Fast travel's hold ends, its time from the jump.
    pub fast_travel_hold: Option<Duration>,
}

/// A Reference scene's Chunks streamed around the camera: the Chunk
/// streamer, served by the IO pool, keeping the World's `LoadedChunks`.
pub(crate) struct ChunkStreaming {
    streamer: ChunkStreamer,
    io: IoPool,
    scene: ReferenceScene,
    shift: DVec3,
    cache: Arc<CookedCache>,
    claimed: Claimed,
    /// Assets the IO pool decoded that no load has taken yet.
    decoded: DecodedAssets,
    /// Chunks loaded into the pools this frame, by coordinates, until the
    /// Chunk streamer takes or drops them.
    finished: BTreeMap<[i32; 2], LoadedChunk>,
    /// A load's failure while settling, which ends the wait.
    error: Option<anyhow::Error>,
    cook_chunk: Option<CookChunk>,
    /// The bytes a frame's loads may write into the load staging; a Chunk
    /// that alone writes more loads alone.
    load_bytes_per_frame: u64,
    /// The Instances a frame's Chunk loads may place; a Chunk that alone
    /// places more loads alone.
    instances_placed_per_frame: usize,
    mips: MipPriority,
    /// When the Fast travel whose hold has not ended jumped.
    fast_travel_started: Option<Instant>,
}

/// The Instances a frame's streamed Chunk loads place, as prepare and the
/// static TLAS take each one on the main thread in the frame it loads; sized
/// in `docs/performance.md`, "Measurements behind the code".
const INSTANCES_PLACED_PER_FRAME: usize = 131_072;

/// Cooks a Chunk, by Asset ID, and the Assets it names, where missing or
/// stale.
type CookChunk = Box<dyn FnMut(AssetId) -> anyhow::Result<()>>;

impl ChunkStreaming {
    /// Streams `scene`'s Chunks, moved by `shift` metres, from `cache`, those
    /// within `load_radius_metres` of the camera drawn, writing at most
    /// `load_bytes_per_frame` into the load staging a frame.
    pub(crate) fn new(
        scene: ReferenceScene,
        shift: DVec3,
        load_radius_metres: f64,
        load_bytes_per_frame: u64,
        cache: Arc<CookedCache>,
    ) -> anyhow::Result<Self> {
        let claimed = Claimed::default();
        Ok(Self {
            streamer: ChunkStreamer::new(scene.chunk_coordinates(), load_radius_metres),
            io: IoPool::new(cache.clone(), claimed.clone())?,
            scene,
            shift,
            cache,
            claimed,
            decoded: DecodedAssets::default(),
            finished: BTreeMap::new(),
            error: None,
            cook_chunk: None,
            load_bytes_per_frame,
            instances_placed_per_frame: INSTANCES_PLACED_PER_FRAME,
            mips: MipPriority::default(),
            fast_travel_started: None,
        })
    }

    /// Runs `cook_chunk` on each Chunk it requests, on the main thread,
    /// before the IO pool reads it: a dev launch's cook on a miss.
    #[cfg(feature = "cook")]
    pub(crate) fn cook_each_chunk(&mut self, cook_chunk: CookChunk) {
        self.cook_chunk = Some(cook_chunk);
    }

    /// Runs the dev launch's cook, where there is one, on each Chunk within
    /// the hold radius of any of `cameras`, World positions, now rather than
    /// on the main thread as each is requested, so frames that stream time no
    /// cook.
    pub(crate) fn cook_ahead(
        &mut self,
        cameras: impl IntoIterator<Item = DVec3>,
    ) -> anyhow::Result<()> {
        let Some(cook_chunk) = &mut self.cook_chunk else {
            return Ok(());
        };
        let mut chunks = BTreeSet::new();
        for camera in cameras {
            chunks.extend(self.streamer.within_hold_radius(camera - self.shift));
        }
        for coordinates in chunks {
            cook_chunk(self.scene.chunk_id(coordinates))?;
        }
        Ok(())
    }

    /// Streams with a Load radius of `metres` instead, before any load.
    #[cfg(test)]
    pub(crate) fn set_load_radius(&mut self, metres: f64) {
        self.streamer = ChunkStreamer::new(self.scene.chunk_coordinates(), metres);
    }

    /// Writes at most `bytes` a frame into the load staging instead.
    #[cfg(test)]
    pub(crate) fn set_load_bytes_per_frame(&mut self, bytes: u64) {
        self.load_bytes_per_frame = bytes;
    }

    /// Places at most `most_instances` a frame through Chunk loads instead.
    #[cfg(test)]
    pub(crate) fn set_instances_placed_per_frame(&mut self, most_instances: usize) {
        self.instances_placed_per_frame = most_instances;
    }

    /// Waits until the IO pool has read every Chunk requested so far.
    #[cfg(test)]
    pub(crate) fn wait_for_reads(&mut self) -> anyhow::Result<()> {
        self.io.receive_all()
    }

    /// A reader of the Cooked cache whose reads the IO pool then skips, for
    /// Assets loaded outside Chunks.
    pub(crate) fn reader(&mut self) -> DecodedReader<'_> {
        DecodedReader {
            decoded: RefCell::new(&mut self.decoded),
            cache: &self.cache,
            claimed: &self.claimed,
            reads_resident_tails: false,
        }
    }

    /// The Cooked cache's `content_hash`: the Blobs loaded so far.
    pub(crate) fn content_hash(&self) -> String {
        self.cache.content_hash()
    }

    /// Runs the Chunk streamer for the World's camera: loads the Chunks whose
    /// reads finished into `renderer`'s pools, nearest the camera first, up
    /// to the frame's load bytes, and keeps the World's `LoadedChunks` to
    /// what the streamer holds. With `settle`, waits until the frame is
    /// Settled, loading the Chunks of each wait together once all have
    /// arrived, in coordinate order and past the frame's load bytes, so
    /// their order never depends on the IO pool's timing; a load that fails
    /// then fails the frame. Ends the frame's frees to make room, which it
    /// reports with the pools' refusals since the frame before, and clears
    /// the camera's `teleported`, timing the hold of the Fast travel it
    /// marks.
    pub(crate) fn update(
        &mut self,
        world: &mut World,
        renderer: &mut Renderer,
        settle: bool,
    ) -> anyhow::Result<StreamedFrame> {
        let teleported = std::mem::take(&mut world.resource_mut::<Camera>().teleported);
        if teleported {
            self.fast_travel_started = Some(Instant::now());
        }
        let streamed = self.stream(world, renderer, settle, teleported);
        let frees =
            renderer.mesh_pool.take_frame_frees() + renderer.material_pool.take_frame_frees();
        let mut streamed = streamed?;
        streamed.pool_entries_freed = frees as u32;
        streamed.meshes_refused = renderer.mesh_pool.take_refusals();
        streamed.materials_refused = renderer.material_pool.take_refusals();
        if !streamed.holding {
            streamed.fast_travel_hold = self.fast_travel_started.take().map(|jump| jump.elapsed());
        }
        Ok(streamed)
    }

    fn stream(
        &mut self,
        world: &mut World,
        renderer: &mut Renderer,
        settle: bool,
        teleported: bool,
    ) -> anyhow::Result<StreamedFrame> {
        let camera = world.resource::<Camera>().position - self.shift;
        let mut streamed = StreamedFrame::default();
        let mut placed_instances = 0;
        loop {
            let Self {
                streamer,
                io,
                scene,
                shift,
                cache,
                claimed,
                decoded,
                finished,
                error,
                cook_chunk,
                load_bytes_per_frame,
                instances_placed_per_frame,
                mips,
                fast_travel_started: _,
            } = self;
            let mut frame_io = FrameIo {
                cook_chunk,
                io,
                scene: *scene,
                shift: *shift,
                camera,
                cache,
                claimed,
                decoded,
                finished,
                renderer,
                settle,
                load_bytes_per_frame: *load_bytes_per_frame,
                instances_placed_per_frame: *instances_placed_per_frame,
                written_bytes: &mut streamed.written_bytes,
                placed_instances: &mut placed_instances,
                bytes_read: &mut streamed.bytes_read,
                error,
            };
            let frame = streamer.update(camera, teleported, &mut frame_io);
            streamed.holding = frame.holding;
            streamed.late_chunks += frame.late_chunks;
            streamed.chunks_loaded += frame.loaded.len() as u32;
            streamed.chunks_not_loaded += frame.not_loaded.len() as u32;
            streamed.chunks_unloaded += frame.unloaded.len() as u32;
            keep_loaded_chunks(world, frame, finished, renderer);
            let mip_streaming = MipStreaming {
                io,
                priority: mips,
                renderer,
                settle,
                load_bytes_per_frame: *load_bytes_per_frame,
            };
            let mips_settled = mip_streaming.stream(world, &mut streamed);
            if !settle || (frame.settled && mips_settled) {
                return Ok(streamed);
            }
            if let Some(error) = error.take() {
                return Err(error);
            }
            io.receive_all()?;
        }
    }
}

/// One frame's mip streaming (ADR 0017), after its Chunk loads.
struct MipStreaming<'a> {
    io: &'a mut IoPool,
    priority: &'a mut MipPriority,
    renderer: &'a mut Renderer,
    settle: bool,
    load_bytes_per_frame: u64,
}

impl MipStreaming<'_> {
    /// Applies the mips the IO pool read, finest-needed first, then in Asset
    /// ID order, up to the frame's load bytes, or, while settling, all of them once nothing is
    /// in flight, as the Chunk streamer's last take found it; then requests
    /// the mips the loaded Chunks' drawn Placements need from the camera,
    /// finest-needed first, and counts the late mips. Returns whether no
    /// request is in flight. Without mip streaming, it does nothing.
    fn stream(mut self, world: &World, streamed: &mut StreamedFrame) -> bool {
        if !self.renderer.material_pool.streams_mips() {
            return true;
        }
        if !self.settle {
            self.io.receive();
            self.apply(streamed);
        } else if self.io.in_flight == 0 {
            self.apply(streamed);
        }
        let camera = world.resource::<Camera>().position;
        let chunks = world
            .get_resource::<LoadedChunks>()
            .map_or(&[][..], |chunks| &chunks.0);
        let Renderer {
            mesh_pool,
            material_pool,
            ..
        } = &*self.renderer;
        let (late_mips, requests) =
            self.priority
                .plan(camera, chunks, mesh_pool.meshes(), material_pool);
        streamed.late_mips = late_mips;
        for request in requests {
            self.io.request(IoRequest::Mips(request.clone()));
        }
        !self.priority.waiting()
    }

    fn apply(&mut self, streamed: &mut StreamedFrame) {
        let Renderer {
            device,
            material_pool,
            load_staging,
            ..
        } = &mut *self.renderer;
        while let Some(entry) = self.io.ready_mips.first_entry() {
            let (_, texture) = *entry.key();
            if let (levels, Ok(_)) = entry.get()
                && let Some((shape, _)) = material_pool.texture_residency(texture)
            {
                let bytes =
                    mips_written_bytes(shape.format, shape.width, shape.height, levels.clone());
                let past_cap = streamed.written_bytes + bytes > self.load_bytes_per_frame;
                if !self.settle && streamed.written_bytes > 0 && past_cap {
                    return;
                }
            }
            let (levels, decoded) = entry.remove();
            let read_failed = match decoded {
                Ok(decoded) => {
                    streamed.bytes_read += decoded.bytes_read;
                    let written_before = load_staging.written_bytes();
                    let priority = &*self.priority;
                    let eviction_rank = |victim, resident_level| {
                        priority.eviction_rank(victim, resident_level, levels.start)
                    };
                    material_pool.stream_mips(
                        device,
                        load_staging,
                        texture,
                        levels.start,
                        &decoded.levels,
                        &eviction_rank,
                    );
                    streamed.written_bytes += load_staging.written_bytes() - written_before;
                    false
                }
                Err(error) => {
                    let error = format!("{error:#}");
                    tracing::error!(%texture, error, "mips not read");
                    true
                }
            };
            self.priority.finished(texture, read_failed);
        }
    }
}

/// Moves the Chunks the streamer took this frame from `finished` into the
/// World's `LoadedChunks`, removes those it unloaded, marks those it holds,
/// and gives back the pool users of the Chunks it unloaded or dropped.
fn keep_loaded_chunks(
    world: &mut World,
    frame: &ChunkFrame,
    finished: &mut BTreeMap<[i32; 2], LoadedChunk>,
    renderer: &mut Renderer,
) {
    let mut loaded_chunks = world.resource_mut::<LoadedChunks>();
    let chunks = &mut loaded_chunks.bypass_change_detection().0;
    let mut changed = false;
    for coordinates in &frame.loaded {
        if let Some(chunk) = finished.remove(coordinates) {
            chunks.push(chunk);
            changed = true;
        }
    }
    for chunk in std::mem::take(finished).into_values() {
        remove_users(renderer, &chunk);
    }
    if !frame.unloaded.is_empty() {
        chunks.retain(|chunk| {
            let unloaded = frame.unloaded.contains(&chunk.coordinates);
            if unloaded {
                remove_users(renderer, chunk);
            }
            !unloaded
        });
        changed = true;
    }
    for chunk in chunks.iter_mut() {
        let held = frame.held.binary_search(&chunk.coordinates).is_ok();
        changed |= chunk.held != held;
        chunk.held = held;
    }
    if changed {
        loaded_chunks.set_changed();
    }
}

fn remove_users(renderer: &mut Renderer, chunk: &LoadedChunk) {
    for &mesh in &chunk.meshes {
        renderer
            .mesh_pool
            .remove_user(mesh, MeshUse::DrawnAndTraced);
    }
    for &material in &chunk.materials {
        renderer.material_pool.remove_user(material);
    }
}

/// The IO the Chunk streamer reaches in one frame: requests go to the IO
/// pool, and its finished reads are loaded into the pools.
struct FrameIo<'a> {
    cook_chunk: &'a mut Option<CookChunk>,
    io: &'a mut IoPool,
    scene: ReferenceScene,
    shift: DVec3,
    /// The camera's position in the scene's own placement.
    camera: DVec3,
    cache: &'a CookedCache,
    claimed: &'a Mutex<HashSet<AssetId>>,
    decoded: &'a mut DecodedAssets,
    finished: &'a mut BTreeMap<[i32; 2], LoadedChunk>,
    renderer: &'a mut Renderer,
    settle: bool,
    load_bytes_per_frame: u64,
    instances_placed_per_frame: usize,
    /// The bytes the frame's loads wrote into the load staging so far.
    written_bytes: &'a mut u64,
    /// The Instances the frame's loads placed so far.
    placed_instances: &'a mut usize,
    bytes_read: &'a mut u64,
    error: &'a mut Option<anyhow::Error>,
}

impl FrameIo<'_> {
    /// The ready load to take next: the Chunk nearest the camera, lowest
    /// coordinates first, or the lowest coordinates while settling.
    fn next_ready(&self) -> Option<[i32; 2]> {
        let mut ready = self.io.ready.keys().copied();
        if self.settle {
            return ready.next();
        }
        ready.min_by(|&left, &right| {
            distance_metres(self.camera, left).total_cmp(&distance_metres(self.camera, right))
        })
    }

    /// Whether loading `chunk` keeps the frame's placed Instances and load
    /// bytes within their caps: always while settling; past the Instances'
    /// cap only while the frame's loads have placed none, and past the
    /// bytes' only while they have written none.
    fn fits(&mut self, chunk: &CookedChunk) -> anyhow::Result<bool> {
        if self.settle {
            return Ok(true);
        }
        let instances = *self.placed_instances + chunk.placements.len();
        if *self.placed_instances > 0 && instances > self.instances_placed_per_frame {
            return Ok(false);
        }
        if *self.written_bytes == 0 {
            return Ok(true);
        }
        let bytes = self.most_written_bytes(chunk)?;
        Ok(*self.written_bytes + bytes <= self.load_bytes_per_frame)
    }

    /// The most bytes loading `chunk` writes into the load staging: those of
    /// each Mesh, Material and Texture it names that the pools lack, each
    /// read into what the IO pool decoded, if it is not there, for the load.
    fn most_written_bytes(&mut self, chunk: &CookedChunk) -> anyhow::Result<u64> {
        let Renderer {
            mesh_pool,
            material_pool,
            ..
        } = &*self.renderer;
        let reader = DecodedReader {
            decoded: RefCell::new(&mut *self.decoded),
            cache: self.cache,
            claimed: self.claimed,
            reads_resident_tails: true,
        };
        let mut seen = HashSet::new();
        let mut bytes = 0;
        let mut materials: Vec<AssetId> = chunk
            .placements
            .iter()
            .filter_map(|placement| placement.material)
            .collect();
        for placement in &chunk.placements {
            let id = placement.mesh;
            if !seen.insert(id) || mesh_pool.is_loaded(id, MeshUse::DrawnAndTraced) {
                continue;
            }
            let mesh = reader.mesh(id)?;
            bytes += MeshPool::written_bytes(&mesh, MeshUse::DrawnAndTraced);
            let primitive_materials = mesh
                .primitives
                .iter()
                .filter_map(|primitive| primitive.material);
            materials.extend(primitive_materials);
            reader.decoded.borrow_mut().meshes.insert(id, mesh);
        }
        let mut textures = Vec::new();
        for id in materials {
            if !seen.insert(id) || material_pool.is_loaded(id) {
                continue;
            }
            let material = reader.material(id)?;
            bytes += MaterialPool::WRITTEN_BYTES;
            textures.extend(material.textures.iter().flatten());
            reader.decoded.borrow_mut().materials.insert(id, material);
        }
        for id in textures {
            if !seen.insert(id) || material_pool.is_texture_loaded(id) {
                continue;
            }
            let texture = reader.texture(id)?;
            bytes += texture_written_bytes(&texture);
            reader.decoded.borrow_mut().textures.insert(id, texture);
        }
        Ok(bytes)
    }

    /// Loads `decoded` into the pools, counting what it read and wrote.
    fn load(&mut self, decoded: DecodedChunk) -> anyhow::Result<LoadedChunk> {
        *self.bytes_read += decoded.bytes_read;
        *self.placed_instances += decoded.chunk.placements.len();
        #[cfg(debug_assertions)]
        let most_written_bytes = self.most_written_bytes(&decoded.chunk)?;
        let written_before = self.renderer.load_staging.written_bytes();
        let loaded = self.load_into_pools(&decoded);
        let written_bytes = self.renderer.load_staging.written_bytes() - written_before;
        #[cfg(debug_assertions)]
        assert!(
            written_bytes <= most_written_bytes,
            "a Chunk wrote {written_bytes} bytes, past the {most_written_bytes} it was sized at"
        );
        *self.written_bytes += written_bytes;
        loaded
    }

    fn load_into_pools(&mut self, decoded: &DecodedChunk) -> anyhow::Result<LoadedChunk> {
        let reader = DecodedReader {
            decoded: RefCell::new(&mut *self.decoded),
            cache: self.cache,
            claimed: self.claimed,
            reads_resident_tails: true,
        };
        let Renderer {
            device,
            mesh_pool,
            material_pool,
            load_staging,
            ..
        } = &mut *self.renderer;
        let mut loader = PoolLoader {
            device,
            staging: load_staging,
            reader: &reader,
            mesh_pool,
            material_pool,
        };
        let mut chunk =
            load_cooked_chunk(&mut loader, decoded.id, decoded.coordinates, &decoded.chunk)?;
        chunk.shift(self.shift);
        Ok(chunk)
    }
}

impl ChunkIo for FrameIo<'_> {
    fn request(&mut self, coordinates: [i32; 2]) {
        let id = self.scene.chunk_id(coordinates);
        if let Some(cook_chunk) = self.cook_chunk
            && let Err(error) = cook_chunk(id)
        {
            let error = format!("{error:#}");
            tracing::error!(?coordinates, error, "Chunk not cooked");
        }
        self.io.request(IoRequest::Chunk(coordinates, id));
    }

    fn take_finished(&mut self) -> Option<FinishedLoad> {
        self.io.receive();
        for decoded in self.io.ready.values_mut().flatten() {
            self.decoded.extend(std::mem::take(&mut decoded.assets));
        }
        if self.settle && self.io.in_flight > 0 {
            return None;
        }
        let coordinates = self.next_ready()?;
        let decoded = self.io.ready.remove(&coordinates)?;
        let loaded = match decoded {
            Ok(decoded) => match self.fits(&decoded.chunk) {
                Ok(true) => self.load(decoded),
                Ok(false) => {
                    self.io.ready.insert(coordinates, Ok(decoded));
                    return None;
                }
                Err(error) => Err(error),
            },
            Err(error) => Err(error),
        };
        let assets = match loaded {
            Ok(chunk) => {
                let assets = chunk
                    .meshes
                    .iter()
                    .chain(&chunk.materials)
                    .copied()
                    .collect();
                self.finished.insert(coordinates, chunk);
                Some(assets)
            }
            Err(error) => {
                let error = error.context(format!("loading the Chunk at {coordinates:?}"));
                tracing::error!(error = format!("{error:#}"), "Chunk not loaded");
                if self.settle {
                    self.error.get_or_insert(error);
                }
                None
            }
        };
        Some(FinishedLoad {
            coordinates,
            assets,
        })
    }
}

#[cfg(all(test, feature = "cook"))]
mod tests {
    use super::*;
    use crate::cooked::CookedMaterial;

    /// An Asset the pools freed leaves the claimed set, so a later read of it
    /// on the main thread claims it afresh instead of waiting for a decoded
    /// copy no IO thread is making.
    #[test]
    fn an_asset_the_pools_freed_is_claimed_afresh_by_its_next_read() {
        let directory =
            std::env::temp_dir().join(format!("praxis-io-forget-{}", std::process::id()));
        let cache = CookedCache::new(&directory);
        let material = CookedMaterial {
            base_color: [1.0; 4],
            metallic: 0.0,
            roughness: 1.0,
            emissive: [0.0; 3],
            normal_scale: 1.0,
            textures: [None; 5],
        };
        cache.write(AssetId(1), "key", &material.to_blob()).unwrap();
        let claimed = Mutex::new(HashSet::from([AssetId(1)]));
        let mut decoded = DecodedAssets::default();
        let reader = DecodedReader {
            decoded: RefCell::new(&mut decoded),
            cache: &cache,
            claimed: &claimed,
            reads_resident_tails: false,
        };

        reader.forget(&[AssetId(1)]);
        assert!(!claimed.lock().unwrap().contains(&AssetId(1)));
        reader.material(AssetId(1)).unwrap();
        assert!(claimed.lock().unwrap().contains(&AssetId(1)));
        assert!(decoded.read_from_cache.is_empty());
        std::fs::remove_dir_all(&directory).ok();
    }

    /// A Chunk load that reads on the main thread a Texture that a load in
    /// flight claimed reads its Resident tail alone, as the IO pool would
    /// have: a 1,024² BC7 Texture's tail starts at mip 2, 256², 64 KiB.
    #[test]
    fn a_chunk_loads_read_of_a_texture_claimed_in_flight_takes_its_resident_tail() {
        use crate::cooked::level_bytes;

        let directory = std::env::temp_dir().join(format!("praxis-io-tail-{}", std::process::id()));
        let cache = CookedCache::new(&directory);
        let format = wgpu::TextureFormat::Bc7RgbaUnormSrgb;
        let texture = CookedTexture {
            format,
            width: 1024,
            height: 1024,
            first_level: 0,
            levels: (0..11)
                .map(|level| vec![level as u8; level_bytes(format, 1024, 1024, level)])
                .collect(),
        };
        cache
            .write(AssetId(1), "key", &texture.to_blob(1).unwrap())
            .unwrap();
        let claimed = Mutex::new(HashSet::from([AssetId(1)]));
        let mut decoded = DecodedAssets::default();
        let reader = DecodedReader {
            decoded: RefCell::new(&mut decoded),
            cache: &cache,
            claimed: &claimed,
            reads_resident_tails: true,
        };

        let read = reader.texture(AssetId(1)).unwrap();
        assert_eq!(read.first_level, 2);
        assert_eq!(read.levels, texture.levels[2..]);
        assert!(decoded.read_from_cache.contains(&AssetId(1)));
        std::fs::remove_dir_all(&directory).ok();
    }
}
