//! The IO pool (ADR 0016), and the Chunk streamer it serves each frame.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ops::Range;
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex, PoisonError};
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
use crate::streaming::chunks::{far_proxy_transform, load_cooked_chunk, load_far_proxy};
use crate::streaming::mips::{MipPriority, MipRequest, Sight, placement_finest_mip_needed};
use crate::streaming::{
    ChunkFrame, ChunkIo, ChunkStreamer, FinishedLoad, LoadedChunk, LoadedChunks,
    ResidentFarProxies, distance_metres,
};

const IO_THREADS: usize = 4;

/// The IO threads that may read Far proxies at once: all but one, so one
/// thread is always free for a Chunk's read or a Texture's mips.
const FAR_PROXY_READERS: usize = IO_THREADS - 1;

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

    /// Whether loading Placements that name `meshes` and `materials` needs an
    /// Asset, through the Meshes' Materials to their Textures, that is
    /// neither `available` nor decoded here.
    fn lacks(
        &self,
        meshes: &[AssetId],
        materials: &[AssetId],
        available: impl Fn(AssetId) -> bool,
    ) -> bool {
        let mut materials = materials.to_vec();
        for &id in meshes.iter().filter(|&&id| !available(id)) {
            let Some(mesh) = self.meshes.get(&id) else {
                return true;
            };
            materials.extend(
                mesh.primitives
                    .iter()
                    .filter_map(|primitive| primitive.material),
            );
        }
        let mut textures = Vec::new();
        for id in materials.into_iter().filter(|&id| !available(id)) {
            let Some(material) = self.materials.get(&id) else {
                return true;
            };
            textures.extend(material.textures.iter().flatten());
        }
        textures
            .into_iter()
            .any(|id| !available(id) && !self.textures.contains_key(&id))
    }
}

/// What the IO pool reads at a Chunk's coordinates: the Chunk, or its Far
/// proxy.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Load {
    Chunk,
    FarProxy,
}

impl Load {
    /// What its Meshes are loaded for.
    fn mesh_use(self) -> MeshUse {
        match self {
            Load::Chunk => MeshUse::DrawnAndTraced,
            Load::FarProxy => MeshUse::FarProxy,
        }
    }
}

/// A Chunk the IO pool read and decoded, with the Assets it names, through
/// its Meshes' Materials to their Textures, that no earlier load claimed; or
/// a Far proxy, read as a Chunk that names its Mesh alone.
struct DecodedChunk {
    /// The Chunk's Asset ID, or the Far proxy's Mesh's.
    id: AssetId,
    coordinates: [i32; 2],
    /// None of a Far proxy.
    chunk: Option<CookedChunk>,
    /// The Meshes its Placements name, each once.
    meshes: Vec<AssetId>,
    /// The Materials its Placements name, each once.
    materials: Vec<AssetId>,
    assets: DecodedAssets,
    bytes_read: u64,
    /// The Chunk reads requested before its own came back: those that could
    /// have claimed an Asset it names before it.
    reads_requested_before_it_came_back: u64,
}

/// Mips of a Texture the IO pool read and decoded.
struct DecodedMips {
    levels: Vec<Vec<u8>>,
    bytes_read: u64,
}

/// What the IO pool reads: the Chunk or the Far proxy with an Asset ID at
/// its coordinates, with the Assets it names, a Far proxy's Textures down to
/// the mip its distance from a camera needs, where it gives one; or a
/// Texture's mips.
#[derive(Debug)]
enum IoRequest {
    Chunk(Load, [i32; 2], AssetId, Option<DVec3>),
    Mips(MipRequest),
}

impl IoRequest {
    fn lane(&self) -> Lane {
        match self {
            IoRequest::Chunk(Load::Chunk, ..) => Lane::Chunk,
            IoRequest::Mips(_) => Lane::Mips,
            IoRequest::Chunk(Load::FarProxy, ..) => Lane::FarProxy,
        }
    }
}

/// Which requests the IO pool takes before which: those of an earlier lane
/// first.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Lane {
    Chunk,
    Mips,
    FarProxy,
}

/// The requests the IO pool's threads have not taken, by lane, then in the
/// order requested, until it closes.
#[derive(Default)]
struct RequestQueue {
    queued: Mutex<QueuedRequests>,
    arrived: Condvar,
}

#[derive(Default)]
struct QueuedRequests {
    requests: BTreeMap<(Lane, u64), IoRequest>,
    pushed: u64,
    /// The Chunk reads taken and not yet finished.
    chunk_reads_taken: usize,
    /// The Far proxy reads taken and not yet finished.
    far_proxy_reads_taken: usize,
    closed: bool,
}

impl QueuedRequests {
    /// Takes the first request, but a Far proxy's while a Chunk read is
    /// taken or `FAR_PROXY_READERS` threads read one.
    fn take(&mut self) -> Option<IoRequest> {
        let entry = self.requests.first_entry()?;
        let (lane, _) = *entry.key();
        match lane {
            Lane::Chunk => self.chunk_reads_taken += 1,
            Lane::Mips => {}
            Lane::FarProxy => {
                if self.chunk_reads_taken > 0 || self.far_proxy_reads_taken >= FAR_PROXY_READERS {
                    return None;
                }
                self.far_proxy_reads_taken += 1;
            }
        }
        Some(entry.remove())
    }
}

impl RequestQueue {
    /// Queues `request`, or gives it back once closed.
    fn push(&self, request: IoRequest) -> Result<(), IoRequest> {
        let mut queued = self.queued.lock().unwrap_or_else(PoisonError::into_inner);
        if queued.closed {
            return Err(request);
        }
        let key = (request.lane(), queued.pushed);
        queued.pushed += 1;
        queued.requests.insert(key, request);
        self.arrived.notify_one();
        Ok(())
    }

    /// Takes the first request it may, waiting for one: None once closed.
    /// Each request it gives is `finished` once read, or no Far proxy read
    /// is taken again.
    fn pop(&self) -> Option<IoRequest> {
        let mut queued = self.queued.lock().unwrap_or_else(PoisonError::into_inner);
        loop {
            if queued.closed {
                return None;
            }
            if let Some(request) = queued.take() {
                return Some(request);
            }
            queued = self
                .arrived
                .wait(queued)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    /// Takes the first request it may, if any, without waiting.
    #[cfg(all(test, feature = "cook"))]
    fn try_pop(&self) -> Option<IoRequest> {
        let mut queued = self.queued.lock().unwrap_or_else(PoisonError::into_inner);
        queued.take()
    }

    /// Ends a read of `lane` that `pop` gave, so a Far proxy's may be taken.
    fn finished(&self, lane: Lane) {
        let mut queued = self.queued.lock().unwrap_or_else(PoisonError::into_inner);
        match lane {
            Lane::Chunk => queued.chunk_reads_taken -= 1,
            Lane::Mips => return,
            Lane::FarProxy => queued.far_proxy_reads_taken -= 1,
        }
        self.arrived.notify_all();
    }

    /// Refuses further requests, and wakes every thread waiting, which then
    /// takes none.
    fn close(&self) {
        let mut queued = self.queued.lock().unwrap_or_else(PoisonError::into_inner);
        queued.closed = true;
        self.arrived.notify_all();
    }
}

enum Finished {
    Chunk(Load, [i32; 2], Box<anyhow::Result<DecodedChunk>>),
    Mips(MipRequest, anyhow::Result<DecodedMips>),
}

/// A few std threads that read Chunks and the Assets they name, and
/// Textures' mips, from the Cooked cache and decode them, off the main
/// thread.
struct IoPool {
    requests: Arc<RequestQueue>,
    finished: mpsc::Receiver<Finished>,
    threads: Vec<JoinHandle<()>>,
    in_flight: usize,
    /// The Chunk and Far proxy reads in flight, by coordinates, each with
    /// the number of such reads requested before it.
    chunk_reads: HashMap<(Load, [i32; 2]), u64>,
    chunk_reads_requested: u64,
    /// Loads the threads finished and the main thread has not taken, Chunks
    /// first, by coordinates.
    ready: BTreeMap<(Load, [i32; 2]), anyhow::Result<DecodedChunk>>,
    /// Mips the threads read and the main thread has not applied, by the
    /// finest requested, then Texture, with the mips requested.
    ready_mips: BTreeMap<(u32, AssetId), (Range<u32>, anyhow::Result<DecodedMips>)>,
}

impl IoPool {
    fn new(cache: Arc<CookedCache>, claimed: Claimed) -> anyhow::Result<Self> {
        let requests = Arc::new(RequestQueue::default());
        let (finished_sender, finished) = mpsc::channel();
        let threads = (0..IO_THREADS)
            .map(|_| {
                let (requests, finished, cache, claimed) = (
                    requests.clone(),
                    finished_sender.clone(),
                    cache.clone(),
                    claimed.clone(),
                );
                std::thread::Builder::new()
                    .name("IO Pool".to_string())
                    .spawn(move || {
                        while let Some(request) = requests.pop() {
                            let lane = request.lane();
                            let finished_load = match request {
                                IoRequest::Chunk(load, coordinates, id, camera) => {
                                    let decoded = decode_chunk(
                                        &cache,
                                        &claimed,
                                        load,
                                        id,
                                        coordinates,
                                        camera,
                                    );
                                    Finished::Chunk(load, coordinates, Box::new(decoded))
                                }
                                IoRequest::Mips(request) => {
                                    let decoded = decode_mips(&cache, &request);
                                    Finished::Mips(request, decoded)
                                }
                            };
                            requests.finished(lane);
                            if finished.send(finished_load).is_err() {
                                return;
                            }
                        }
                    })
                    .context("starting the IO pool")
            })
            .collect::<anyhow::Result<_>>()?;
        Ok(Self {
            requests,
            finished,
            threads,
            in_flight: 0,
            chunk_reads: HashMap::new(),
            chunk_reads_requested: 0,
            ready: BTreeMap::new(),
            ready_mips: BTreeMap::new(),
        })
    }

    fn request(&mut self, request: IoRequest) {
        let chunk_read = match &request {
            IoRequest::Chunk(load, coordinates, _, _) => Some((*load, *coordinates)),
            IoRequest::Mips(_) => None,
        };
        match self.requests.push(request) {
            Ok(()) => {
                self.in_flight += 1;
                if let Some(chunk_read) = chunk_read {
                    self.chunk_reads
                        .insert(chunk_read, self.chunk_reads_requested);
                    self.chunk_reads_requested += 1;
                }
            }
            Err(request) => tracing::error!(?request, "not requested: the IO pool stopped"),
        }
    }

    /// Whether a Chunk read among the first `requested` is still in flight.
    fn reads_in_flight_before(&self, requested: u64) -> bool {
        self.chunk_reads.values().any(|&order| order < requested)
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
            self.in_flight > 0 || !self.ready.is_empty(),
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
            Finished::Chunk(load, coordinates, decoded) => {
                self.chunk_reads.remove(&(load, coordinates));
                let mut decoded = *decoded;
                if let Ok(decoded) = &mut decoded {
                    decoded.reads_requested_before_it_came_back = self.chunk_reads_requested;
                }
                self.ready.insert((load, coordinates), decoded);
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
        self.requests.close();
        for thread in self.threads.drain(..) {
            thread.join().ok();
        }
    }
}

/// Reads and decodes the Chunk `id` at `coordinates` from `cache`, and each
/// Mesh, Material and Texture it names that `claimed` lacks, which it
/// claims; of a Far proxy, the Mesh `id` and what it names. Each Texture is
/// its Resident tail, or, of a Far proxy read for a `camera`, its mips down
/// to the one its distance from there needs.
fn decode_chunk(
    cache: &CookedCache,
    claimed: &Mutex<HashSet<AssetId>>,
    load: Load,
    id: AssetId,
    coordinates: [i32; 2],
    camera: Option<DVec3>,
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
    let chunk = match load {
        Load::Chunk => Some(
            CookedChunk::from_blob(&read(id)?).with_context(|| format!("reading Chunk {id}"))?,
        ),
        Load::FarProxy => None,
    };
    let mut assets = DecodedAssets::default();
    let (mut meshes, mut placement_materials) = (BTreeSet::new(), BTreeSet::new());
    for placement in chunk.iter().flat_map(|chunk| &chunk.placements) {
        meshes.insert(placement.mesh);
        placement_materials.extend(placement.material);
    }
    if chunk.is_none() {
        meshes.insert(id);
    }
    let mut materials: Vec<AssetId> = placement_materials.iter().copied().collect();
    let mut densest_uses = HashMap::new();
    for &mesh_id in &meshes {
        if claim(mesh_id) {
            let mesh = CookedMesh::from_blob(&read(mesh_id)?)
                .with_context(|| format!("reading Mesh {mesh_id}"))?;
            for primitive in &mesh.primitives {
                if let Some(material) = primitive.material {
                    materials.push(material);
                    let texel_density = primitive.texel_density;
                    let densest_use = DensestUse {
                        texel_density,
                        bounds: mesh.bounds,
                    };
                    keep_densest(&mut densest_uses, material, densest_use);
                }
            }
            assets.meshes.insert(mesh_id, mesh);
        }
    }
    let mut textures = Vec::new();
    for material_id in materials {
        if claim(material_id) {
            let material = CookedMaterial::from_blob(&read(material_id)?)
                .with_context(|| format!("reading Material {material_id}"))?;
            textures.extend(material.textures.iter().flatten());
            if let Some(&densest_use) = densest_uses.get(&material_id) {
                for &texture in material.textures.iter().flatten() {
                    keep_densest(&mut densest_uses, texture, densest_use);
                }
            }
            assets.materials.insert(material_id, material);
        }
    }
    let far_proxy_camera = camera.filter(|_| load == Load::FarProxy);
    for texture_id in textures {
        if claim(texture_id) {
            let blob = read(texture_id)?;
            let texture = match (far_proxy_camera, densest_uses.get(&texture_id)) {
                (
                    Some(camera),
                    Some(&DensestUse {
                        texel_density,
                        bounds,
                    }),
                ) => {
                    let transform = far_proxy_transform(coordinates);
                    CookedTexture::down_to_from_blob(&blob, |side| {
                        placement_finest_mip_needed(camera, &transform, bounds, texel_density, side)
                    })
                }
                _ => CookedTexture::resident_tail_from_blob(&blob),
            };
            let texture = texture.with_context(|| format!("reading Texture {texture_id}"))?;
            assets.textures.insert(texture_id, texture);
        }
    }
    Ok(DecodedChunk {
        id,
        coordinates,
        chunk,
        meshes: meshes.into_iter().collect(),
        materials: placement_materials.into_iter().collect(),
        assets,
        bytes_read,
        reads_requested_before_it_came_back: 0,
    })
}

/// A Material's or a Texture's use at the densest texel density a Mesh's
/// Primitive samples it at, with that Mesh's bounds.
#[derive(Clone, Copy)]
struct DensestUse {
    texel_density: f32,
    bounds: [[f32; 3]; 2],
}

/// Keeps `candidate` as `asset`'s in `densest_uses` where it is denser.
fn keep_densest(
    densest_uses: &mut HashMap<AssetId, DensestUse>,
    asset: AssetId,
    candidate: DensestUse,
) {
    let densest = densest_uses.entry(asset).or_insert(candidate);
    if candidate.texel_density > densest.texel_density {
        *densest = candidate;
    }
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
    /// alone, as the IO pool decodes a Chunk's, rather than its whole chain.
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
    pub late_far_proxies: u32,
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
    /// Whether the Far proxies' streamer, in a scene with a Far field, still
    /// waits for a Far proxy within the View distance since the camera last
    /// arrived.
    pub far_proxies_holding: Option<bool>,
    /// Whether a Fast travel's hold has ended and its Far field's fill not.
    pub far_field_filling: bool,
    /// On the frame a Fast travel's Far field's fill ends, its time from the
    /// end of its hold.
    pub far_field_fill: Option<Duration>,
}

/// Times a Fast travel: its hold, from the jump until every Chunk within the
/// Load radius has loaded, then, in a scene with a Far field, its fill, from
/// the hold's end until every Far proxy within the View distance is
/// Resident.
#[derive(Default)]
struct FastTravelClock {
    jumped: Option<Instant>,
    hold_ended: Option<Instant>,
}

impl FastTravelClock {
    /// Starts timing a Fast travel that jumped at `now`.
    fn jumped(&mut self, now: Instant) {
        self.jumped = Some(now);
        self.hold_ended = None;
    }

    /// Ends `streamed`'s frame at `now`: reports the hold on the frame that
    /// ends it and the fill on the frame that ends that, and whether the
    /// frame fills.
    fn time(&mut self, now: Instant, streamed: &mut StreamedFrame) {
        if !streamed.holding
            && let Some(jumped) = self.jumped.take()
        {
            streamed.fast_travel_hold = Some(now - jumped);
            self.hold_ended = streamed.far_proxies_holding.map(|_| now);
        }
        if streamed.far_proxies_holding == Some(false)
            && let Some(hold_ended) = self.hold_ended.take()
        {
            streamed.far_field_fill = Some(now - hold_ended);
        }
        streamed.far_field_filling = self.hold_ended.is_some();
    }
}

/// A scene's Far proxies streamed around the camera, by a Chunk streamer
/// whose radius is the View distance.
struct FarProxies {
    streamer: ChunkStreamer,
    /// Far proxies loaded into the pools that the streamer has not taken.
    awaiting_placement: Vec<LoadedChunk>,
    /// The coordinates of the Chunks drawn this frame, in order, kept for
    /// its memory.
    drawn_chunks: Vec<[i32; 2]>,
}

/// A Reference scene's Chunks streamed around the camera: the Chunk
/// streamer, served by the IO pool, keeping the World's `LoadedChunks`; and
/// the Far proxies of a scene with a Far field, served after its Chunks,
/// keeping the World's `ResidentFarProxies`.
pub(crate) struct ChunkStreaming {
    streamer: ChunkStreamer,
    far_proxies: Option<FarProxies>,
    io: IoPool,
    scene: ReferenceScene,
    shift: DVec3,
    cache: Arc<CookedCache>,
    claimed: Claimed,
    /// Assets the IO pool decoded that no load has taken yet.
    decoded: DecodedAssets,
    /// Chunks loaded into the pools and not placed, in the order loaded,
    /// each until rays may trace every Mesh it names.
    awaiting_placement: Vec<LoadedChunk>,
    /// Chunks placed this frame, by coordinates, until the Chunk streamer
    /// takes or drops them.
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
    fast_travel: FastTravelClock,
}

/// The Instances a frame's streamed Chunk loads place, as prepare and the
/// static TLAS take each one on the main thread in the frame it is placed;
/// sized in `docs/performance.md`, "Measurements behind the code".
const INSTANCES_PLACED_PER_FRAME: usize = 131_072;

/// The Chunks and Far proxies a frame loads, as each load's sizing and pool writes take the
/// main thread whatever it places or writes; sized in
/// `docs/performance.md`, "Measurements behind the code".
const LOADS_PER_FRAME: usize = 3;

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
        let view_distance_metres = scene.view_distance_metres();
        let has_far_field = view_distance_metres > scene.load_radius_metres();
        let far_proxies = has_far_field.then(|| FarProxies {
            streamer: ChunkStreamer::of_far_proxies(
                scene.chunk_coordinates(),
                view_distance_metres,
            ),
            awaiting_placement: Vec::new(),
            drawn_chunks: Vec::new(),
        });
        Ok(Self {
            streamer: ChunkStreamer::new(scene.chunk_coordinates(), load_radius_metres),
            far_proxies,
            io: IoPool::new(cache.clone(), claimed.clone())?,
            scene,
            shift,
            cache,
            claimed,
            decoded: DecodedAssets::default(),
            awaiting_placement: Vec::new(),
            finished: BTreeMap::new(),
            error: None,
            cook_chunk: None,
            load_bytes_per_frame,
            instances_placed_per_frame: INSTANCES_PLACED_PER_FRAME,
            mips: MipPriority::default(),
            fast_travel: FastTravelClock::default(),
        })
    }

    /// Runs `cook_chunk` on each Chunk it requests, on the main thread,
    /// before the IO pool reads it: a dev launch's cook on a miss.
    #[cfg(feature = "cook")]
    pub(crate) fn cook_each_chunk(&mut self, cook_chunk: CookChunk) {
        self.cook_chunk = Some(cook_chunk);
    }

    /// Runs the dev launch's cook, where there is one, on each Chunk within
    /// the hold radius of any of `cameras`, World positions, and on each Far
    /// proxy within its own, now rather than on the main thread as each is
    /// requested, so frames that stream time no cook.
    pub(crate) fn cook_ahead(
        &mut self,
        cameras: impl IntoIterator<Item = DVec3>,
    ) -> anyhow::Result<()> {
        let Some(cook_chunk) = &mut self.cook_chunk else {
            return Ok(());
        };
        let (mut chunks, mut far_proxies) = (BTreeSet::new(), BTreeSet::new());
        for camera in cameras {
            let camera = camera - self.shift;
            chunks.extend(self.streamer.within_hold_radius(camera));
            if let Some(resident) = &self.far_proxies {
                far_proxies.extend(resident.streamer.within_hold_radius(camera));
            }
        }
        let (chunks, far_proxies) = (chunks.into_iter(), far_proxies.into_iter());
        let chunks = chunks.map(|coordinates| self.scene.chunk_id(coordinates));
        let far_proxies = far_proxies.map(|coordinates| self.scene.far_proxy_id(coordinates));
        for id in chunks.chain(far_proxies) {
            cook_chunk(id)?;
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
        if self.io.in_flight == 0 {
            return Ok(());
        }
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

    /// Runs the Chunk streamer for the World's camera, then the Far proxies'
    /// as it runs the Chunks', within the same caps: loads the Chunks whose
    /// reads finished into `renderer`'s pools, nearest the camera first, up
    /// to the frame's caps on loads and load bytes; places those loaded, in
    /// the order loaded, whose Meshes' BLASes the GPU has built and
    /// compacted, up to the frame's cap on placed Instances; and keeps the
    /// World's `LoadedChunks` to what the streamer holds, and its
    /// `ResidentFarProxies` to what the Far proxies' holds, each held while
    /// its Chunk is drawn. With
    /// `settle`, waits until the frame is Settled, loading the Chunks of
    /// each wait together once all have arrived, in coordinate order and
    /// past the frame's caps, so their order never depends on the IO pool's
    /// timing, and placing every Chunk loaded, whose BLASes the frame's
    /// upload then waits for; a load that fails then fails the frame. Ends the frame's
    /// frees to make room, which it reports with the pools' refusals since
    /// the frame before, and clears the camera's `teleported`, timing the
    /// hold and the Far field's fill of the Fast travel it marks.
    pub(crate) fn update(
        &mut self,
        world: &mut World,
        renderer: &mut Renderer,
        settle: bool,
    ) -> anyhow::Result<StreamedFrame> {
        let teleported = std::mem::take(&mut world.resource_mut::<Camera>().teleported);
        if teleported {
            self.fast_travel.jumped(Instant::now());
        }
        let streamed = self.stream(world, renderer, settle, teleported);
        let frees =
            renderer.mesh_pool.take_frame_frees() + renderer.material_pool.take_frame_frees();
        let mut streamed = streamed?;
        streamed.pool_entries_freed = frees as u32;
        streamed.meshes_refused = renderer.mesh_pool.take_refusals();
        streamed.materials_refused = renderer.material_pool.take_refusals();
        self.fast_travel.time(Instant::now(), &mut streamed);
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
        let reads_far_proxy_mips = renderer.material_pool.streams_mips();
        if settle {
            renderer.mesh_pool.wait_for_blases();
        } else {
            renderer.mesh_pool.poll_blases();
        }
        loop {
            let Self {
                streamer,
                far_proxies,
                io,
                scene,
                shift,
                cache,
                claimed,
                decoded,
                awaiting_placement,
                finished,
                error,
                cook_chunk,
                load_bytes_per_frame,
                instances_placed_per_frame,
                mips,
                fast_travel: _,
            } = self;
            let mut frame_io = FrameIo {
                cook_chunk,
                io,
                load: Load::Chunk,
                reads_far_proxy_mips,
                scene: *scene,
                shift: *shift,
                camera,
                cache,
                claimed,
                decoded,
                awaiting_placement,
                finished,
                pools: renderer,
                settle,
                load_bytes_per_frame: *load_bytes_per_frame,
                instances_placed_per_frame: *instances_placed_per_frame,
                loads_per_frame: LOADS_PER_FRAME,
                loads: 0,
                written_bytes: &mut streamed.written_bytes,
                placed_instances: &mut placed_instances,
                bytes_read: &mut streamed.bytes_read,
                error,
                waiting: HashSet::new(),
                loads_ended: false,
            };
            let frame = streamer.update(camera, teleported, &mut frame_io);
            streamed.holding = frame.holding;
            streamed.late_chunks += frame.late_chunks;
            streamed.chunks_loaded += frame.loaded.len() as u32;
            streamed.chunks_not_loaded += frame.not_loaded.len() as u32;
            streamed.chunks_unloaded += frame.unloaded.len() as u32;
            let mut settled = frame.settled;
            let mut loaded_chunks = world.resource_mut::<LoadedChunks>();
            let chunks = &mut loaded_chunks.bypass_change_detection().0;
            let (finished, pools) = (&mut *frame_io.finished, &mut *frame_io.pools);
            if keep_loaded_chunks(chunks, frame, &[], finished, pools, MeshUse::DrawnAndTraced) {
                loaded_chunks.set_changed();
            }
            if let Some(far_proxies) = far_proxies {
                let drawn_chunks = &mut far_proxies.drawn_chunks;
                drawn_chunks.clear();
                let drawn = loaded_chunks.0.iter().filter(|chunk| !chunk.held);
                drawn_chunks.extend(drawn.map(|chunk| chunk.coordinates));
                drawn_chunks.sort_unstable();
                frame_io.serve_far_proxies(&mut far_proxies.awaiting_placement);
                let frame = (far_proxies.streamer).update(camera, teleported, &mut frame_io);
                streamed.late_far_proxies += frame.late_chunks;
                streamed.far_proxies_holding = Some(frame.holding);
                settled &= frame.settled;
                let (finished, pools) = (&mut *frame_io.finished, &mut *frame_io.pools);
                let mut resident = world.resource_mut::<ResidentFarProxies>();
                let proxies = &mut resident.bypass_change_detection().0;
                let mesh_use = MeshUse::FarProxy;
                if keep_loaded_chunks(proxies, frame, drawn_chunks, finished, pools, mesh_use) {
                    resident.set_changed();
                }
            }
            let drawn_chunks = far_proxies
                .as_ref()
                .map_or(&[][..], |far_proxies| &far_proxies.drawn_chunks);
            let mip_streaming = MipStreaming {
                io,
                priority: mips,
                renderer,
                settle,
                teleported,
                drawn_chunks,
                load_bytes_per_frame: *load_bytes_per_frame,
            };
            let mips_settled = mip_streaming.stream(world, &mut streamed);
            if !settle || (settled && mips_settled) {
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
    teleported: bool,
    /// The coordinates of the Chunks drawn this frame, in order.
    drawn_chunks: &'a [[i32; 2]],
    load_bytes_per_frame: u64,
}

impl MipStreaming<'_> {
    /// Applies the mips the IO pool read, finest-needed first, then in Asset
    /// ID order, up to the frame's load bytes, or, while settling, all of them once nothing is
    /// in flight, as the Chunk streamer's last take found it; then requests
    /// the mips the Placements of the drawn Chunks and Far proxies need, and
    /// of those held past their radius, from the camera a lead ahead,
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
        let far_proxies = world
            .get_resource::<ResidentFarProxies>()
            .map_or(&[][..], |far_proxies| &far_proxies.0);
        let Renderer {
            mesh_pool,
            material_pool,
            ..
        } = &*self.renderer;
        let chunk_sight = |chunk: &LoadedChunk| {
            if chunk.held {
                Sight::Ahead
            } else {
                Sight::Drawn
            }
        };
        let drawn_chunks = self.drawn_chunks;
        let far_proxy_sight = |far_proxy: &LoadedChunk| {
            let replaced = drawn_chunks.binary_search(&far_proxy.coordinates).is_ok();
            match (far_proxy.held, replaced) {
                (false, _) => Sight::Drawn,
                (true, false) => Sight::Ahead,
                (true, true) => Sight::Replaced,
            }
        };
        let chunks = chunks.iter().map(|chunk| (chunk, chunk_sight(chunk)));
        let far_proxies =
            (far_proxies.iter()).map(|far_proxy| (far_proxy, far_proxy_sight(far_proxy)));
        let (late_mips, requests) = self.priority.plan(
            camera,
            self.teleported,
            chunks.chain(far_proxies),
            mesh_pool.meshes(),
            material_pool,
        );
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

/// Moves the loads the streamer took this frame from `finished` into
/// `chunks`, removes those it unloaded, marks as held those it holds and
/// those at `also_held`, coordinates in order, and gives back the pool
/// users, of Meshes loaded for `mesh_use`, of those it unloaded or dropped.
/// Returns whether `chunks` changed.
fn keep_loaded_chunks(
    chunks: &mut Vec<LoadedChunk>,
    frame: &ChunkFrame,
    also_held: &[[i32; 2]],
    finished: &mut BTreeMap<[i32; 2], LoadedChunk>,
    renderer: &mut Renderer,
    mesh_use: MeshUse,
) -> bool {
    let mut changed = false;
    for coordinates in &frame.loaded {
        if let Some(chunk) = finished.remove(coordinates) {
            chunks.push(chunk);
            changed = true;
        }
    }
    for chunk in std::mem::take(finished).into_values() {
        remove_users(renderer, &chunk, mesh_use);
    }
    if !frame.unloaded.is_empty() {
        chunks.retain(|chunk| {
            let unloaded = frame.unloaded.contains(&chunk.coordinates);
            if unloaded {
                remove_users(renderer, chunk, mesh_use);
            }
            !unloaded
        });
        changed = true;
    }
    for chunk in chunks.iter_mut() {
        let held = frame.held.binary_search(&chunk.coordinates).is_ok()
            || also_held.binary_search(&chunk.coordinates).is_ok();
        changed |= chunk.held != held;
        chunk.held = held;
    }
    changed
}

fn remove_users(renderer: &mut Renderer, chunk: &LoadedChunk, mesh_use: MeshUse) {
    for &mesh in &chunk.meshes {
        renderer.mesh_pool.remove_user(mesh, mesh_use);
    }
    for &material in &chunk.materials {
        renderer.material_pool.remove_user(material);
    }
}

/// The IO a streamer reaches in one frame, the Chunks' or the Far proxies':
/// requests go to the IO pool, and its finished reads are loaded into
/// `Pools`, the Renderer's.
struct FrameIo<'a, Pools> {
    cook_chunk: &'a mut Option<CookChunk>,
    io: &'a mut IoPool,
    /// Which of the two the streamer it serves loads.
    load: Load,
    /// Whether a Far proxy is read with its Textures' mips down to the one
    /// its distance needs, as mips stream, rather than their tails alone.
    reads_far_proxy_mips: bool,
    scene: ReferenceScene,
    shift: DVec3,
    /// The camera's position in the scene's own placement.
    camera: DVec3,
    cache: &'a CookedCache,
    claimed: &'a Mutex<HashSet<AssetId>>,
    decoded: &'a mut DecodedAssets,
    awaiting_placement: &'a mut Vec<LoadedChunk>,
    finished: &'a mut BTreeMap<[i32; 2], LoadedChunk>,
    pools: &'a mut Pools,
    settle: bool,
    load_bytes_per_frame: u64,
    instances_placed_per_frame: usize,
    loads_per_frame: usize,
    /// The loads the frame made so far, Chunks and Far proxies, those that
    /// failed among them.
    loads: usize,
    /// The bytes the frame's loads wrote into the load staging so far.
    written_bytes: &'a mut u64,
    /// The Instances the frame's loads placed so far.
    placed_instances: &'a mut usize,
    bytes_read: &'a mut u64,
    error: &'a mut Option<anyhow::Error>,
    /// The ready Chunks that wait this frame on a read in flight.
    waiting: HashSet<[i32; 2]>,
    /// Whether no further ready Chunk loads this frame.
    loads_ended: bool,
}

/// What a frame's Chunk loads ask of the pools they load into.
trait LoadsIntoPools {
    fn waits_on_a_read(&self, decoded: &DecodedChunk) -> bool;
    fn most_written_bytes(&mut self, decoded: &DecodedChunk) -> anyhow::Result<u64>;
    /// The bytes written into the load staging so far.
    fn load_staging_written_bytes(&self) -> u64;
    fn load_into_pools(&mut self, decoded: &DecodedChunk) -> anyhow::Result<LoadedChunk>;
    /// Whether the GPU has built and compacted the BLAS of every Mesh
    /// `chunk` names.
    fn blases_ready(&self, chunk: &LoadedChunk) -> bool;
}

impl<'a, Pools> FrameIo<'a, Pools> {
    /// Serves the Far proxies' streamer for the rest of the frame, once the
    /// Chunks' has run: its loads wait in `awaiting_placement`, and count
    /// towards the frame's caps after the Chunks'.
    fn serve_far_proxies(&mut self, awaiting_placement: &'a mut Vec<LoadedChunk>) {
        self.load = Load::FarProxy;
        self.awaiting_placement = awaiting_placement;
        self.waiting.clear();
        self.loads_ended = false;
    }
}

impl<Pools> FrameIo<'_, Pools>
where
    Self: LoadsIntoPools,
{
    /// The ready load to take next: the Chunk nearest the camera, lowest
    /// coordinates first, or the lowest coordinates while settling.
    fn next_ready(&self) -> Option<[i32; 2]> {
        let mut ready = (self.io.ready.keys())
            .filter(|(load, coordinates)| *load == self.load && !self.waiting.contains(coordinates))
            .map(|&(_, coordinates)| coordinates);
        if self.settle {
            return ready.next();
        }
        ready.min_by(|&left, &right| {
            distance_metres(self.camera, left).total_cmp(&distance_metres(self.camera, right))
        })
    }

    /// Whether loading `decoded` keeps the frame's loads and load bytes within
    /// their caps: always while settling; past the bytes' cap only while the
    /// frame's loads have written none.
    fn fits(&mut self, decoded: &DecodedChunk) -> anyhow::Result<bool> {
        if self.settle {
            return Ok(true);
        }
        if self.loads >= self.loads_per_frame {
            return Ok(false);
        }
        if *self.written_bytes == 0 {
            return Ok(true);
        }
        let bytes = self.most_written_bytes(decoded)?;
        Ok(*self.written_bytes + bytes <= self.load_bytes_per_frame)
    }

    /// Loads `decoded` into the pools, counting what it read and wrote.
    fn load(&mut self, decoded: DecodedChunk) -> anyhow::Result<LoadedChunk> {
        self.loads += 1;
        *self.bytes_read += decoded.bytes_read;
        #[cfg(debug_assertions)]
        let most_written_bytes = self.most_written_bytes(&decoded)?;
        let written_before = self.load_staging_written_bytes();
        let loaded = self.load_into_pools(&decoded);
        let written_bytes = self.load_staging_written_bytes() - written_before;
        #[cfg(debug_assertions)]
        assert!(
            written_bytes <= most_written_bytes,
            "a Chunk wrote {written_bytes} bytes, past the {most_written_bytes} it was sized at"
        );
        *self.written_bytes += written_bytes;
        loaded
    }

    /// Loads the next ready Chunk into the pools, if the frame's caps allow
    /// it: its coordinates and its load, or None when no ready Chunk loads
    /// this frame.
    fn load_next_ready(&mut self) -> Option<([i32; 2], anyhow::Result<LoadedChunk>)> {
        let (coordinates, decoded) = loop {
            let coordinates = self.next_ready()?;
            let decoded = self.io.ready.remove(&(self.load, coordinates))?;
            match &decoded {
                Ok(chunk) if !self.settle && self.waits_on_a_read(chunk) => {
                    self.io.ready.insert((self.load, coordinates), decoded);
                    self.waiting.insert(coordinates);
                }
                _ => break (coordinates, decoded),
            }
        };
        let loaded = match decoded {
            Ok(decoded) => match self.fits(&decoded) {
                Ok(true) => self.load(decoded),
                Ok(false) => {
                    self.io.ready.insert((self.load, coordinates), Ok(decoded));
                    return None;
                }
                Err(error) => Err(error),
            },
            Err(error) => Err(error),
        };
        Some((coordinates, loaded))
    }

    /// Takes the first Chunk loaded and not placed whose BLASes are ready,
    /// or, while settling, the first: None when there is none, or when it
    /// would take the frame's placed Instances past their cap after another
    /// placed some.
    fn place_next(&mut self) -> Option<LoadedChunk> {
        let next = (self.awaiting_placement.iter())
            .position(|chunk| self.settle || self.blases_ready(chunk))?;
        let instances = *self.placed_instances + self.awaiting_placement[next].placements.len();
        let past_cap = instances > self.instances_placed_per_frame;
        if !self.settle && *self.placed_instances > 0 && past_cap {
            return None;
        }
        *self.placed_instances = instances;
        Some(self.awaiting_placement.remove(next))
    }
}

impl LoadsIntoPools for FrameIo<'_, Renderer> {
    /// Whether loading `decoded` needs an Asset the pools lack that a Chunk
    /// read in flight since before its own came back claimed, and so may be
    /// decoding.
    fn waits_on_a_read(&self, decoded: &DecodedChunk) -> bool {
        if !self
            .io
            .reads_in_flight_before(decoded.reads_requested_before_it_came_back)
        {
            return false;
        }
        let Renderer {
            mesh_pool,
            material_pool,
            ..
        } = &*self.pools;
        let claimed = self.claimed.lock().unwrap_or_else(PoisonError::into_inner);
        let available = |id| {
            !claimed.contains(&id)
                || mesh_pool.is_loaded(id, self.load.mesh_use())
                || material_pool.is_loaded(id)
                || material_pool.is_texture_loaded(id)
        };
        self.decoded
            .lacks(&decoded.meshes, &decoded.materials, available)
    }

    /// The most bytes loading `decoded` writes into the load staging: those of
    /// each Mesh, Material and Texture it names that the pools lack, each
    /// read into what the IO pool decoded, if it is not there, for the load.
    fn most_written_bytes(&mut self, decoded: &DecodedChunk) -> anyhow::Result<u64> {
        let mesh_use = self.load.mesh_use();
        let Renderer {
            mesh_pool,
            material_pool,
            ..
        } = &*self.pools;
        let reader = DecodedReader {
            decoded: RefCell::new(&mut *self.decoded),
            cache: self.cache,
            claimed: self.claimed,
            reads_resident_tails: true,
        };
        let mut seen = HashSet::new();
        let mut bytes = 0;
        let mut materials = decoded.materials.clone();
        for &id in &decoded.meshes {
            if mesh_pool.is_loaded(id, mesh_use) {
                continue;
            }
            let mesh = reader.mesh(id)?;
            bytes += MeshPool::written_bytes(&mesh, mesh_use);
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

    fn load_staging_written_bytes(&self) -> u64 {
        self.pools.load_staging.written_bytes()
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
        } = &mut *self.pools;
        let mut loader = PoolLoader {
            device,
            staging: load_staging,
            reader: &reader,
            mesh_pool,
            material_pool,
        };
        loader.mesh_pool.defer_blases(!self.settle);
        let (id, coordinates) = (decoded.id, decoded.coordinates);
        let chunk = match &decoded.chunk {
            Some(chunk) => load_cooked_chunk(&mut loader, id, coordinates, chunk),
            None => load_far_proxy(&mut loader, id, coordinates),
        };
        loader.mesh_pool.defer_blases(false);
        let mut chunk = chunk?;
        chunk.shift(self.shift);
        Ok(chunk)
    }

    fn blases_ready(&self, chunk: &LoadedChunk) -> bool {
        (chunk.meshes.iter())
            .all(|&mesh| (self.pools.mesh_pool).asset_blas_ready(mesh, self.load.mesh_use()))
    }
}

impl<Pools> ChunkIo for FrameIo<'_, Pools>
where
    Self: LoadsIntoPools,
{
    fn request(&mut self, coordinates: [i32; 2]) {
        let load = self.load;
        let id = match load {
            Load::Chunk => self.scene.chunk_id(coordinates),
            Load::FarProxy => self.scene.far_proxy_id(coordinates),
        };
        if let Some(cook_chunk) = self.cook_chunk
            && let Err(error) = cook_chunk(id)
        {
            let error = format!("{error:#}");
            tracing::error!(?load, ?coordinates, error, "Chunk or Far proxy not cooked");
        }
        let camera = (load == Load::FarProxy && self.reads_far_proxy_mips).then_some(self.camera);
        self.io
            .request(IoRequest::Chunk(load, coordinates, id, camera));
    }

    fn take_finished(&mut self) -> Option<FinishedLoad> {
        self.io.receive();
        for decoded in self.io.ready.values_mut().flatten() {
            self.decoded.extend(std::mem::take(&mut decoded.assets));
        }
        if self.settle && self.io.in_flight > 0 {
            return None;
        }
        while !self.loads_ended {
            let Some((coordinates, loaded)) = self.load_next_ready() else {
                self.loads_ended = true;
                break;
            };
            match loaded {
                Ok(chunk) => self.awaiting_placement.push(chunk),
                Err(error) => {
                    let load = self.load;
                    let error = error.context(format!("loading the {load:?} at {coordinates:?}"));
                    tracing::error!(
                        error = format!("{error:#}"),
                        "Chunk or Far proxy not loaded"
                    );
                    if self.settle {
                        self.error.get_or_insert(error);
                    }
                    return Some(FinishedLoad {
                        coordinates,
                        assets: None,
                    });
                }
            }
        }
        let chunk = self.place_next()?;
        let coordinates = chunk.coordinates;
        let assets = chunk
            .meshes
            .iter()
            .chain(&chunk.materials)
            .copied()
            .collect();
        self.finished.insert(coordinates, chunk);
        Some(FinishedLoad {
            coordinates,
            assets: Some(assets),
        })
    }
}

#[cfg(all(test, feature = "cook"))]
mod tests {
    use super::*;
    use crate::cooked::CookedMaterial;

    /// An IO pool without threads, with the end its requests arrive at and
    /// the end that finishes them.
    fn io_pool() -> (IoPool, Arc<RequestQueue>, mpsc::Sender<Finished>) {
        let requests = Arc::new(RequestQueue::default());
        let (finished_sender, finished) = mpsc::channel();
        let io = IoPool {
            requests: requests.clone(),
            finished,
            threads: Vec::new(),
            in_flight: 0,
            chunk_reads: HashMap::new(),
            chunk_reads_requested: 0,
            ready: BTreeMap::new(),
            ready_mips: BTreeMap::new(),
        };
        (io, requests, finished_sender)
    }

    /// The read of a Chunk without Placements at `coordinates`, as it comes
    /// back from the IO pool.
    fn read_chunk(coordinates: [i32; 2]) -> Finished {
        read(Load::Chunk, coordinates)
    }

    /// `read_chunk`, or the read of the Chunk's Far proxy.
    fn read(load: Load, coordinates: [i32; 2]) -> Finished {
        let decoded = DecodedChunk {
            id: AssetId(1),
            coordinates,
            chunk: Some(CookedChunk {
                placements: Vec::new(),
                lights: Vec::new(),
                bounds: [DVec3::ZERO; 2],
            }),
            meshes: Vec::new(),
            materials: Vec::new(),
            assets: DecodedAssets::default(),
            bytes_read: 0,
            reads_requested_before_it_came_back: 0,
        };
        Finished::Chunk(load, coordinates, Box::new(Ok(decoded)))
    }

    /// Pools that load any Chunk but the one at `refused`, without its
    /// Placements, and write nothing; the BLASes of the Chunks at `unready`
    /// are not ready.
    struct NoPools<'a> {
        refused: Option<[i32; 2]>,
        unready: &'a [[i32; 2]],
    }

    impl LoadsIntoPools for FrameIo<'_, NoPools<'_>> {
        fn waits_on_a_read(&self, _decoded: &DecodedChunk) -> bool {
            false
        }

        fn most_written_bytes(&mut self, _decoded: &DecodedChunk) -> anyhow::Result<u64> {
            Ok(0)
        }

        fn load_staging_written_bytes(&self) -> u64 {
            0
        }

        fn load_into_pools(&mut self, decoded: &DecodedChunk) -> anyhow::Result<LoadedChunk> {
            anyhow::ensure!(self.pools.refused != Some(decoded.coordinates), "refused");
            Ok(LoadedChunk {
                id: decoded.id,
                coordinates: decoded.coordinates,
                placements: Vec::new(),
                lights: Vec::new(),
                meshes: Vec::new(),
                materials: Vec::new(),
                held: false,
            })
        }

        fn blases_ready(&self, chunk: &LoadedChunk) -> bool {
            !self.pools.unready.contains(&chunk.coordinates)
        }
    }

    /// The nine Chunks around Chunk [0, 0], in coordinate order.
    fn nine_chunks() -> impl Iterator<Item = [i32; 2]> {
        (-1..=1).flat_map(|x| (-1..=1).map(move |z| [x, z]))
    }

    /// The middle of Chunk [0, 0].
    const MIDDLE: DVec3 = DVec3::new(128.0, 10.0, 128.0);

    /// What streaming `nine_chunks` leaves: the Chunks the streamer loaded on
    /// each frame, those loaded into the pools and not placed, and those
    /// placed, which the streamer took or left for their pool users to be
    /// given back.
    struct NineChunksStreamed {
        loaded: Vec<Vec<[i32; 2]>>,
        awaiting_placement: Vec<[i32; 2]>,
        placed: Vec<[i32; 2]>,
    }

    /// Streams `nine_chunks`, whose reads all come back between the first
    /// frame and the second, at most four loads a frame, over `frames`: each
    /// a camera and the Chunks whose BLASes are not ready then. The pools
    /// refuse the Chunk at `refused`.
    fn stream_nine_chunks(
        settle: bool,
        refused: Option<[i32; 2]>,
        frames: &[(DVec3, &[[i32; 2]])],
    ) -> NineChunksStreamed {
        let (mut io, _requests, finished_sender) = io_pool();
        let cache = CookedCache::new(std::env::temp_dir());
        let claimed = Mutex::new(HashSet::new());
        let mut streamer = ChunkStreamer::new(nine_chunks(), 256.0);
        let mut decoded = DecodedAssets::default();
        let mut awaiting_placement = Vec::new();
        let mut finished = BTreeMap::new();
        let mut loaded = Vec::new();
        for (frame, &(camera, unready)) in frames.iter().enumerate() {
            if frame == 1 {
                for coordinates in nine_chunks() {
                    finished_sender.send(read_chunk(coordinates)).unwrap();
                }
            }
            let mut frame_io = FrameIo {
                cook_chunk: &mut None,
                io: &mut io,
                load: Load::Chunk,
                reads_far_proxy_mips: false,
                scene: ReferenceScene::TextureWall,
                shift: DVec3::ZERO,
                camera,
                cache: &cache,
                claimed: &claimed,
                decoded: &mut decoded,
                awaiting_placement: &mut awaiting_placement,
                finished: &mut finished,
                pools: &mut NoPools { refused, unready },
                settle,
                load_bytes_per_frame: u64::MAX,
                instances_placed_per_frame: usize::MAX,
                loads_per_frame: 4,
                loads: 0,
                written_bytes: &mut 0,
                placed_instances: &mut 0,
                bytes_read: &mut 0,
                error: &mut None,
                waiting: HashSet::new(),
                loads_ended: false,
            };
            loaded.push(streamer.update(camera, false, &mut frame_io).loaded.clone());
        }
        NineChunksStreamed {
            loaded,
            awaiting_placement: (awaiting_placement.iter())
                .map(|chunk| chunk.coordinates)
                .collect(),
            placed: finished.into_keys().collect(),
        }
    }

    /// The Chunks each of four frames loads, for a camera over the middle of
    /// Chunk [0, 0], every BLAS ready.
    fn loads_by_frame(settle: bool, refused: Option<[i32; 2]>) -> Vec<Vec<[i32; 2]>> {
        let frames: [(DVec3, &[[i32; 2]]); 4] = [(MIDDLE, &[]); 4];
        stream_nine_chunks(settle, refused, &frames).loaded
    }

    /// The second frame loads four Chunks into the pools, and places none
    /// while their BLASes are not ready. The third places three of them,
    /// ready by then, in the order loaded, then the four it loads itself,
    /// ready at once. The camera is past the hold radius by the fourth,
    /// when [0, 0]'s BLASes are ready: it and [1, 1] are placed for the
    /// streamer to drop, in no frame's loads.
    #[test]
    fn a_load_is_placed_once_its_blases_are_ready_and_dropped_if_its_chunk_left_meanwhile() {
        let first_four = [[0, 0], [-1, 0], [0, -1], [0, 1]];
        let far = MIDDLE + DVec3::new(10_000.0, 0.0, 0.0);
        let frames: [(DVec3, &[[i32; 2]]); 4] = [
            (MIDDLE, &[]),
            (MIDDLE, &first_four),
            (MIDDLE, &[[0, 0]]),
            (far, &[]),
        ];
        let waiting = stream_nine_chunks(false, None, &frames[..2]);
        assert!(waiting.loaded.iter().all(Vec::is_empty));
        assert_eq!(waiting.awaiting_placement, first_four);
        assert!(waiting.placed.is_empty());

        let streamed = stream_nine_chunks(false, None, &frames);
        assert_eq!(
            streamed.loaded,
            [
                vec![],
                vec![],
                vec![[-1, 0], [0, -1], [0, 1], [1, 0], [-1, -1], [-1, 1], [1, -1]],
                vec![],
            ]
        );
        assert!(streamed.awaiting_placement.is_empty());
        assert_eq!(streamed.placed, nine_chunks().collect::<Vec<_>>());
    }

    /// Nine reads come back in one frame that loads four Chunks at most: the
    /// four nearest the camera load, and the rest on the frames after,
    /// nearest first.
    #[test]
    fn loads_past_the_frames_count_wait_for_later_frames_nearest_first() {
        assert_eq!(
            loads_by_frame(false, None),
            [
                vec![],
                vec![[0, 0], [-1, 0], [0, -1], [0, 1]],
                vec![[1, 0], [-1, -1], [-1, 1], [1, -1]],
                vec![[1, 1]],
            ]
        );
    }

    /// A load the pools refuse is one of its frame's four: the nearest
    /// Chunk fails, and three load beside it.
    #[test]
    fn a_load_that_fails_counts_towards_its_frames_loads() {
        assert_eq!(
            loads_by_frame(false, Some([0, 0])),
            [
                vec![],
                vec![[-1, 0], [0, -1], [0, 1]],
                vec![[1, 0], [-1, -1], [-1, 1], [1, -1]],
                vec![[1, 1]],
            ]
        );
    }

    /// A settling frame loads every Chunk read, past the frame's count, in
    /// coordinate order.
    #[test]
    fn a_settling_frame_loads_past_the_frames_count_in_coordinate_order() {
        let every_chunk: Vec<[i32; 2]> = nine_chunks().collect();
        assert_eq!(
            loads_by_frame(true, None),
            [vec![], every_chunk, vec![], vec![]]
        );
    }

    /// A row of seven Chunks, [-3, 0] to [3, 0], streamed with a Load radius
    /// of 256 m and a View distance of 512 m, four loads a frame, through an
    /// IO pool without threads into `NoPools`.
    struct RowOfSeven {
        io: IoPool,
        requests: Arc<RequestQueue>,
        finished_sender: mpsc::Sender<Finished>,
        cache: CookedCache,
        claimed: Mutex<HashSet<AssetId>>,
        streamer: ChunkStreamer,
        far_proxies: ChunkStreamer,
        decoded: DecodedAssets,
        awaiting_placement: Vec<LoadedChunk>,
        far_proxies_awaiting: Vec<LoadedChunk>,
    }

    impl RowOfSeven {
        fn new() -> Self {
            let (io, requests, finished_sender) = io_pool();
            let row = || (-3..=3).map(|x| [x, 0]);
            Self {
                io,
                requests,
                finished_sender,
                cache: CookedCache::new(std::env::temp_dir()),
                claimed: Mutex::new(HashSet::new()),
                streamer: ChunkStreamer::new(row(), 256.0),
                far_proxies: ChunkStreamer::new(row(), 512.0),
                decoded: DecodedAssets::default(),
                awaiting_placement: Vec::new(),
                far_proxies_awaiting: Vec::new(),
            }
        }

        /// A frame for the camera at `camera` once `reads` came back: the
        /// Chunks' streamer's, then the Far proxies'.
        fn frame(
            &mut self,
            camera: DVec3,
            teleported: bool,
            reads: &[(Load, [i32; 2])],
        ) -> (ChunkFrame, ChunkFrame) {
            for &(load, coordinates) in reads {
                self.finished_sender.send(read(load, coordinates)).unwrap();
            }
            let mut frame_io = FrameIo {
                cook_chunk: &mut None,
                io: &mut self.io,
                load: Load::Chunk,
                reads_far_proxy_mips: true,
                scene: ReferenceScene::World,
                shift: DVec3::ZERO,
                camera,
                cache: &self.cache,
                claimed: &self.claimed,
                decoded: &mut self.decoded,
                awaiting_placement: &mut self.awaiting_placement,
                finished: &mut BTreeMap::new(),
                pools: &mut NoPools {
                    refused: None,
                    unready: &[],
                },
                settle: false,
                load_bytes_per_frame: u64::MAX,
                instances_placed_per_frame: usize::MAX,
                loads_per_frame: 4,
                loads: 0,
                written_bytes: &mut 0,
                placed_instances: &mut 0,
                bytes_read: &mut 0,
                error: &mut None,
                waiting: HashSet::new(),
                loads_ended: false,
            };
            let chunks = self.streamer.update(camera, teleported, &mut frame_io);
            let chunks = chunks.clone();
            frame_io.serve_far_proxies(&mut self.far_proxies_awaiting);
            let far = self.far_proxies.update(camera, teleported, &mut frame_io);
            (chunks, far.clone())
        }
    }

    /// A row of seven Chunks, a Load radius of 256 m, a View distance of
    /// 512 m, four loads a frame. The first frame requests the three Chunks
    /// within the Load radius, then the five Far proxies within the View
    /// distance, nearest first, each read for the camera's distance and no
    /// Chunk so; the hold ends with no Far proxy read; a jump three Chunks
    /// east releases the two past the View distance's edge.
    #[test]
    fn far_proxies_are_requested_after_the_chunks_and_released_an_edge_past_the_view_distance() {
        let mut row = RowOfSeven::new();
        let near = [[0, 0], [-1, 0], [1, 0]];
        let chunk_reads = near.map(|chunk| (Load::Chunk, chunk));
        let within_view = [[0, 0], [-1, 0], [1, 0], [-2, 0], [2, 0]];
        let far_proxy_reads = within_view.map(|chunk| (Load::FarProxy, chunk));

        let (chunks, _) = row.frame(MIDDLE, false, &[]);
        let queued = std::mem::take(&mut row.requests.queued.lock().unwrap().requests);
        let requested: Vec<(Load, [i32; 2])> = queued
            .into_values()
            .map(|request| match request {
                IoRequest::Chunk(load, coordinates, _, camera) => {
                    assert_eq!(camera.is_some(), load == Load::FarProxy);
                    (load, coordinates)
                }
                IoRequest::Mips(_) => unreachable!(),
            })
            .collect();
        assert_eq!(requested, [&chunk_reads[..], &far_proxy_reads[..]].concat());
        assert!(chunks.holding);

        let (chunks, far) = row.frame(MIDDLE, false, &chunk_reads);
        assert_eq!(chunks.loaded, near);
        assert!(!chunks.holding && chunks.settled);
        assert!(far.loaded.is_empty() && !far.settled);

        let (_, far) = row.frame(MIDDLE, false, &far_proxy_reads);
        assert_eq!(far.loaded, [[0, 0], [-1, 0], [1, 0], [-2, 0]]);
        let (_, far) = row.frame(MIDDLE, false, &[]);
        assert_eq!(far.loaded, [[2, 0]]);
        assert!(far.settled);

        let three_east = MIDDLE + DVec3::X * 768.0;
        let (_, far) = row.frame(three_east, true, &[]);
        assert_eq!(far.unloaded, [[-2, 0], [-1, 0]]);
        assert_eq!(far.held, [[0, 0]]);
    }

    /// `RowOfSeven` settles over [0, 0], then jumps three Chunks east, where
    /// [2, 0] and [3, 0] are within the Load radius and [3, 0]'s Far proxy
    /// alone is not Resident. The hold ends on the frame both Chunks load;
    /// the fill, timed from that frame's end, on the frame [3, 0]'s Far
    /// proxy loads, two frames later. Before the jump the far field fills
    /// untimed, as no Fast travel came first.
    #[test]
    fn the_far_fields_fill_is_timed_from_the_holds_end_to_the_last_far_proxy_within_the_view_distance()
     {
        let mut row = RowOfSeven::new();
        let mut clock = FastTravelClock::default();
        let start = Instant::now();
        let mut frame = |frame_number: u64, camera, teleported, reads: &[(Load, [i32; 2])]| {
            if teleported {
                clock.jumped(start + Duration::from_millis(frame_number * 10));
            }
            let (chunks, far) = row.frame(camera, teleported, reads);
            let mut streamed = StreamedFrame {
                holding: chunks.holding,
                far_proxies_holding: Some(far.holding),
                ..StreamedFrame::default()
            };
            clock.time(
                start + Duration::from_millis(frame_number * 10 + 5),
                &mut streamed,
            );
            (far, streamed)
        };
        let settling = [
            (Load::Chunk, [0, 0]),
            (Load::Chunk, [-1, 0]),
            (Load::Chunk, [1, 0]),
            (Load::FarProxy, [0, 0]),
            (Load::FarProxy, [-1, 0]),
            (Load::FarProxy, [1, 0]),
            (Load::FarProxy, [-2, 0]),
            (Load::FarProxy, [2, 0]),
        ];
        for (number, reads) in [&[][..], &settling, &[], &[]].into_iter().enumerate() {
            let (_, streamed) = frame(number as u64, MIDDLE, false, reads);
            let timed = (streamed.fast_travel_hold, streamed.far_field_fill);
            assert_eq!(timed, (None, None));
            assert!(!streamed.far_field_filling);
        }

        let three_east = MIDDLE + DVec3::X * 768.0;
        let (_, jump) = frame(4, three_east, true, &[]);
        assert!(jump.holding && jump.fast_travel_hold.is_none());
        let chunk_reads = [(Load::Chunk, [2, 0]), (Load::Chunk, [3, 0])];
        let (_, hold_ends) = frame(5, three_east, false, &chunk_reads);
        assert!(!hold_ends.holding);
        assert_eq!(hold_ends.fast_travel_hold, Some(Duration::from_millis(15)));
        assert!(hold_ends.far_field_filling && hold_ends.far_field_fill.is_none());
        let (_, filling) = frame(6, three_east, false, &[]);
        assert!(filling.far_field_filling && filling.far_field_fill.is_none());
        let (far, fill_ends) = frame(7, three_east, false, &[(Load::FarProxy, [3, 0])]);
        assert_eq!(far.loaded, [[3, 0]]);
        assert!(!fill_ends.far_field_filling);
        assert_eq!(fill_ends.far_field_fill, Some(Duration::from_millis(20)));
        let (_, after) = frame(8, three_east, false, &[]);
        assert_eq!((after.fast_travel_hold, after.far_field_fill), (None, None));
    }

    /// A Chunk waits only on the Chunk reads in flight when its own came back:
    /// any later read found what it names claimed and skipped it.
    #[test]
    fn a_chunk_waits_only_on_the_reads_in_flight_when_its_own_came_back() {
        let (mut io, _requests, finished_sender) = io_pool();
        let finish = |coordinates| finished_sender.send(read_chunk(coordinates)).unwrap();
        let reads_before = |io: &IoPool, coordinates| match &io.ready[&(Load::Chunk, coordinates)] {
            Ok(decoded) => io.reads_in_flight_before(decoded.reads_requested_before_it_came_back),
            Err(_) => unreachable!(),
        };

        io.request(IoRequest::Chunk(Load::Chunk, [0, 0], AssetId(1), None));
        io.request(IoRequest::Chunk(Load::Chunk, [1, 0], AssetId(1), None));
        finish([1, 0]);
        io.receive();
        assert!(reads_before(&io, [1, 0]));

        io.request(IoRequest::Chunk(Load::Chunk, [2, 0], AssetId(1), None));
        assert!(reads_before(&io, [1, 0]));
        finish([0, 0]);
        io.receive();
        assert!(!reads_before(&io, [1, 0]));
        assert!(reads_before(&io, [0, 0]));
    }

    /// The IO pool takes Chunk reads first, then mips, then Far proxy reads,
    /// each in the order requested, a Far proxy's only once no Chunk read is
    /// taken and while fewer than all threads but one read one, so a Far
    /// proxy's read of its mips never holds back a Chunk's; once closed it
    /// refuses and gives nothing.
    #[test]
    fn requests_are_taken_chunks_first_then_mips_then_far_proxies() {
        let queue = RequestQueue::default();
        let chunk = |load, x| IoRequest::Chunk(load, [x, 0], AssetId(1), None);
        let mips = |texture| {
            IoRequest::Mips(MipRequest {
                texture: AssetId(texture),
                levels: 0..3,
            })
        };
        let requests = [
            chunk(Load::FarProxy, 1),
            mips(10),
            chunk(Load::Chunk, 2),
            chunk(Load::FarProxy, 3),
            mips(11),
            chunk(Load::Chunk, 4),
            chunk(Load::FarProxy, 5),
            chunk(Load::FarProxy, 7),
        ];
        for request in requests {
            queue.push(request).unwrap();
        }
        let take_all = || {
            let taken = std::iter::from_fn(|| queue.try_pop());
            let named = taken.map(|request| match request {
                IoRequest::Chunk(Load::Chunk, [x, _], ..) => format!("chunk {x}"),
                IoRequest::Chunk(Load::FarProxy, [x, _], ..) => format!("far proxy {x}"),
                IoRequest::Mips(request) => format!("mips {}", request.texture.0),
            });
            named.collect::<Vec<_>>()
        };
        assert_eq!(take_all(), ["chunk 2", "chunk 4", "mips 10", "mips 11"]);
        queue.finished(Lane::Chunk);
        assert!(take_all().is_empty());
        queue.finished(Lane::Chunk);
        assert_eq!(FAR_PROXY_READERS, 3);
        assert_eq!(take_all(), ["far proxy 1", "far proxy 3", "far proxy 5"]);
        queue.finished(Lane::FarProxy);
        assert_eq!(take_all(), ["far proxy 7"]);

        queue.push(mips(12)).unwrap();
        queue.close();
        assert!(queue.push(mips(13)).is_err());
        assert!(queue.pop().is_none());
    }

    /// A wait for loads fails when the IO pool holds none, and returns when it
    /// holds reads a frame's second streamer took before its first loaded them.
    #[test]
    fn a_wait_for_loads_finds_the_reads_taken_since_they_were_requested() {
        let (mut io, _requests, finished_sender) = io_pool();
        assert!(io.receive_all().is_err());
        io.request(IoRequest::Chunk(Load::Chunk, [0, 0], AssetId(1), None));
        finished_sender.send(read_chunk([0, 0])).unwrap();
        io.receive();
        io.receive_all().unwrap();
    }

    /// Waiting for reads returns at once when the IO pool holds none, as when
    /// it received every read requested before the wait.
    #[test]
    fn waiting_for_reads_with_none_in_flight_returns() {
        let cache = Arc::new(CookedCache::new(std::env::temp_dir()));
        let mut streaming =
            ChunkStreaming::new(ReferenceScene::TextureWall, DVec3::ZERO, 1.0, 1, cache).unwrap();

        streaming.wait_for_reads().unwrap();
    }

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

    /// A Chunk lacks an Asset when one it names, through its Meshes'
    /// Materials to their Textures, is neither available nor decoded.
    #[test]
    fn a_chunk_lacks_an_asset_neither_available_nor_decoded() {
        let (mesh, material, texture, available_mesh) =
            (AssetId(1), AssetId(2), AssetId(3), AssetId(4));
        let mut decoded = DecodedAssets::default();
        decoded.meshes.insert(
            mesh,
            CookedMesh {
                vertices: Vec::new(),
                indices: Vec::new(),
                primitives: vec![crate::cooked::CookedPrimitive {
                    first_index: 0,
                    index_count: 0,
                    material: Some(material),
                    texel_density: 0.0,
                    thin_radius_metres: None,
                    meshlets: Vec::new(),
                }],
                coarser_lods: Vec::new(),
                bounds: [[0.0; 3]; 2],
            },
        );
        let mut decoded_material = CookedMaterial {
            base_color: [1.0; 4],
            metallic: 0.0,
            roughness: 1.0,
            emissive: [0.0; 3],
            normal_scale: 1.0,
            textures: [None; 5],
        };
        decoded_material.textures[0] = Some(texture);
        decoded.materials.insert(material, decoded_material);
        let available = |id| id == available_mesh;

        assert!(decoded.lacks(&[mesh, available_mesh], &[], available));
        decoded.textures.insert(
            texture,
            CookedTexture {
                format: wgpu::TextureFormat::Rgba8Unorm,
                width: 1,
                height: 1,
                first_level: 0,
                levels: vec![vec![0; 4]],
            },
        );
        assert!(!decoded.lacks(&[mesh, available_mesh], &[], available));
        assert!(decoded.lacks(&[mesh], &[AssetId(5)], available));
        assert!(!decoded.lacks(&[mesh], &[material], available));
        let available = |id| id == material;
        decoded.textures.clear();
        assert!(!decoded.lacks(&[], &[material], available));
    }

    /// A Far proxy read for a camera decodes its atlas down to the mip its
    /// distance needs, never coarser than its Resident tail: a 2,048² BC7
    /// atlas, tail from mip 3, sampled at one texel per pixel at a metre,
    /// needs mip 1 at 2.5 m and its tail at 100 m, and read for no camera
    /// it is its tail.
    #[test]
    fn a_far_proxy_read_for_a_camera_decodes_its_atlas_down_to_the_mip_its_distance_needs() {
        use crate::cooked::{CookedPrimitive, CookedVertex, chunk_origin, level_bytes};

        let directory =
            std::env::temp_dir().join(format!("praxis-io-far-proxy-mips-{}", std::process::id()));
        let cache = CookedCache::new(&directory);
        let (mesh_id, material_id, texture_id) = (AssetId(1), AssetId(2), AssetId(3));
        let one_texel_per_pixel_at_one_metre = 1.7138 / 0.000_575_3;
        let side = 2048;
        let triangle = [[0, 1, 2]];
        let mesh = CookedMesh {
            vertices: (0..3)
                .map(|vertex| CookedVertex {
                    position: [0.0; 3],
                    uv: [vertex as f32, 0.0],
                    qtangent: [0, 0, 0, i16::MAX],
                })
                .collect(),
            indices: vec![0, 1, 2],
            primitives: vec![CookedPrimitive {
                first_index: 0,
                index_count: 3,
                material: Some(material_id),
                texel_density: (one_texel_per_pixel_at_one_metre / f64::from(side)) as f32,
                thin_radius_metres: None,
                meshlets: crate::cooked::CookedMeshlet::split(&triangle, [1]).unwrap(),
            }],
            coarser_lods: Vec::new(),
            bounds: [[0.0; 3]; 2],
        };
        let mut material = CookedMaterial {
            base_color: [1.0; 4],
            metallic: 0.0,
            roughness: 1.0,
            emissive: [0.0; 3],
            normal_scale: 1.0,
            textures: [None; 5],
        };
        material.textures[0] = Some(texture_id);
        let format = wgpu::TextureFormat::Bc7RgbaUnormSrgb;
        let texture = CookedTexture {
            format,
            width: side,
            height: side,
            first_level: 0,
            levels: (0..12)
                .map(|level| vec![level as u8; level_bytes(format, side, side, level)])
                .collect(),
        };
        cache
            .write(mesh_id, "key", &mesh.to_blob().unwrap())
            .unwrap();
        cache
            .write(material_id, "key", &material.to_blob())
            .unwrap();
        let texture_blob = texture.to_blob(1).unwrap();
        cache.write(texture_id, "key", &texture_blob).unwrap();
        let coordinates = [1, 2];
        let centre = chunk_origin(coordinates) + DVec3::new(128.0, 0.0, 128.0);
        let first_level_read = |camera: Option<DVec3>| {
            let claimed = Mutex::new(HashSet::new());
            let decoded = decode_chunk(
                &cache,
                &claimed,
                Load::FarProxy,
                mesh_id,
                coordinates,
                camera,
            );
            let mut decoded = decoded.unwrap();
            let read = decoded.assets.textures.remove(&texture_id).unwrap();
            let first_level = read.first_level as usize;
            assert_eq!(read.levels, texture.levels[first_level..]);
            first_level
        };

        let from = |metres| Some(centre + DVec3::new(metres, 0.0, 0.0));
        assert_eq!(
            [from(2.5), from(100.0), None].map(first_level_read),
            [1, 3, 3]
        );
        std::fs::remove_dir_all(&directory).ok();
    }
}
