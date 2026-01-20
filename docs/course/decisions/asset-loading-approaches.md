# Decision Tree: Asset Loading Approaches

```
┌──────────────────────────────────────────────────┐
│ How should I handle asset loading in my engine? │
└──────────────────────────────────────────────────┘
                        │
                        ▼
        ┌───────────────────────────────────┐
        │ Can you afford frame stuttering?  │
        └───────────────────────────────────┘
                /                   \
               /                     \
        Yes (prototyping)      No (production)
              │                       │
              ▼                       ▼
    ┌──────────────────┐      ┌─────────────────┐
    │ Synchronous      │      │ More questions →│
    │ (simple)         │      └─────────────────┘
    └──────────────────┘              │
                                      ▼
                        ┌───────────────────────────┐
                        │ Do you have open world   │
                        │ with dynamic loading?     │
                        └───────────────────────────┘
                              /              \
                             /                \
                           Yes                 No
                            │                  │
                            ▼                  ▼
                    ┌──────────────┐    ┌─────────────┐
                    │  Streaming   │    │ Asynchronous│
                    │  (advanced)  │    │   (good)    │
                    └──────────────┘    └─────────────┘
```

## Quick Decision Matrix

| Approach | Complexity | Stuttering | Memory Use | Best For |
|----------|------------|------------|------------|----------|
| **Synchronous** | ✅ Simple | ❌ Freezes | ⚠️ All at once | Prototypes, small games |
| **Asynchronous** | ⚠️ Moderate | ✅ Smooth | ⚠️ All at once | Most games |
| **Streaming** | ❌ Complex | ✅ Smooth | ✅ Minimal | Open worlds, large games |
| **On-demand** | ⚠️ Moderate | ⚠️ Micro-stutters | ✅ Minimal | Procedural, variable content |
| **Preloading** | ✅ Simple | ⚠️ Load screen | ❌ High | Level-based games |

## Detailed Analysis

### Approach 1: Synchronous Loading (Blocking)

**How it works:**
```rust
fn load_level() {
    let model = load_model("level.gltf");    // Blocks 2s
    let texture = load_texture("tex.png");   // Blocks 500ms
    let audio = load_audio("music.ogg");     // Blocks 1s
    // Total: 3.5s freeze
}
```

Game freezes while loading.

#### Choose Synchronous If:

**✅ High Priority:**
- **Prototyping** (want simplest code)
- **Small assets** (<100ms load time)
- **Loading screens** (freeze acceptable)
- **Learning project** (avoid async complexity)
- **Web games** (initial load only)

**Example Use Cases:**
- Game jam projects
- Prototypes and MVPs
- Small indie games
- Level transitions with loading screen

**Pros:**
- **Simplicity**: Straightforward code flow
- **No threading**: No race conditions
- **Predictable**: Know exactly when assets ready
- **Debugging**: Easy to trace
- **Immediate**: Asset available after function returns

**Cons:**
- **Freezing**: Game unresponsive during load
- **Poor UX**: Users see frozen screen
- **Wasted time**: CPU idle during I/O
- **Scalability**: Doesn't work for large assets
- **No progress**: Can't show loading bar easily

**Example:**
```rust
fn main() {
    let mut world = World::new();
    
    // Game freezes here
    let level = load_level_sync("assets/level1.gltf");
    spawn_level(&mut world, level);
    
    // Now game can start
    game_loop(world);
}

fn load_level_sync(path: &str) -> Level {
    // Blocks until complete
    let bytes = std::fs::read(path).unwrap();
    parse_level(bytes)
}
```

**When acceptable:**
- Behind loading screen
- Assets < 100ms load time
- Single-player turn-based games
- Initial load only (then keep in memory)

### Approach 2: Asynchronous Loading (Non-blocking)

**How it works:**
```rust
async fn load_level() {
    // All load in parallel, game continues
    let model_future = async_load_model("level.gltf");
    let texture_future = async_load_texture("tex.png");
    let audio_future = async_load_audio("music.ogg");
    
    // Wait for all (in parallel)
    let (model, texture, audio) = 
        futures::join!(model_future, texture_future, audio_future);
    // Total: 2s (longest single asset), not 3.5s!
}
```

Game continues running, assets load in background.

#### Choose Asynchronous If:

**✅ High Priority (MOST GAMES):**
- **Production game** (smooth experience required)
- **Medium-large assets** (100ms+ load time)
- Want **loading progress** UI
- Target is **desktop/console** (good I/O)
- Need **responsive game**

**Example Use Cases:**
- Most commercial games
- Desktop/console games
- Games with loading screens
- Multiplayer games (load while connecting)

**Pros:**
- **No freezing**: Game remains responsive
- **Parallel I/O**: Load multiple assets at once
- **Progress**: Can show loading bar
- **Better UX**: Players see activity, not freeze
- **Throughput**: Maximize I/O bandwidth

**Cons:**
- **Complexity**: Async code harder to write/debug
- **Timing**: Must handle "not ready yet" state
- **Synchronization**: Need to coordinate when assets ready
- **Placeholders**: Need fallbacks for missing assets
- **Memory spikes**: All assets loaded at once (even if off-screen)

**Praxis Implementation:**
```rust
use tokio::fs;

// Async asset loading
pub async fn load_model_async(path: &str) -> Result<Model> {
    // Non-blocking I/O
    let bytes = fs::read(path).await?;
    
    // Parse on background thread (blocking operation)
    tokio::task::spawn_blocking(move || {
        parse_gltf(&bytes)
    }).await?
}

// Main game loop continues while loading
pub struct AssetManager {
    pending_loads: Vec<JoinHandle<Asset>>,
}

impl AssetManager {
    pub fn start_load(&mut self, path: String) -> AssetHandle {
        let handle = AssetHandle::new();
        let future = async move {
            load_model_async(&path).await
        };
        self.pending_loads.push(tokio::spawn(future));
        handle
    }
    
    pub fn update(&mut self) {
        // Check if any loads completed
        self.pending_loads.retain(|handle| !handle.is_finished());
    }
}
```

**Usage pattern:**
```rust
// Frame 1: Start load
let handle = assets.start_load("big_model.gltf");

// Frames 2-60: Loading in background, game runs normally
// Show loading spinner or placeholder

// Frame 61: Asset ready
if let Some(model) = assets.get(handle) {
    spawn_model(world, model);
}
```

**Performance:**
```
Synchronous:
  Model:   2000ms (blocked)
  Texture: 500ms (blocked)
  Audio:   1000ms (blocked)
  Total:   3500ms FREEZE

Asynchronous:
  All parallel:  2000ms (longest)
  Game responsive throughout
```

### Approach 3: Streaming (Progressive Loading)

**How it works:**
```rust
// Load assets as needed based on proximity/visibility
fn update_streaming(player_pos: Vec3, assets: &mut StreamingManager) {
    // Unload distant chunks
    for chunk in get_distant_chunks(player_pos, 1000.0) {
        assets.unload_chunk(chunk);
    }
    
    // Load nearby chunks
    for chunk in get_nearby_chunks(player_pos, 500.0) {
        if !assets.is_loaded(chunk) {
            assets.stream_chunk(chunk);
        }
    }
}
```

#### Choose Streaming If:

**✅ High Priority:**
- **Open world** game (can't load entire world)
- **Large game** (>10GB assets)
- **Memory constrained** (consoles, mobile)
- Need **seamless exploration** (no load screens)
- **Dynamic content** (player-driven)

**Example Use Cases:**
- Open-world RPGs (GTA, Skyrim)
- Flight simulators
- MMOs
- Procedural worlds (Minecraft)

**Pros:**
- **Memory efficient**: Only load what's visible/near
- **Scalability**: Supports huge worlds
- **No load screens**: Seamless experience
- **Dynamic**: Adapts to player movement
- **Future-proof**: Works with limited memory

**Cons:**
- **Very complex**: Hardest to implement correctly
- **Pop-in**: Visible assets appearing
- **Stutter risk**: If loading too slow
- **Predictive loading**: Need to anticipate player movement
- **Bug-prone**: Many edge cases
- **Testing**: Hard to test all scenarios

**Implementation (conceptual):**
```rust
pub struct StreamingManager {
    loaded_chunks: HashMap<ChunkId, Chunk>,
    loading_queue: VecDeque<ChunkId>,
    chunk_size: f32,
}

impl StreamingManager {
    pub fn update(&mut self, player_pos: Vec3) {
        let current_chunk = self.position_to_chunk(player_pos);
        let load_radius = 3; // Load 3 chunks in each direction
        
        // Determine which chunks should be loaded
        let desired_chunks = self.get_chunks_in_radius(current_chunk, load_radius);
        
        // Unload far chunks
        self.loaded_chunks.retain(|chunk_id, _| {
            desired_chunks.contains(chunk_id)
        });
        
        // Queue nearby chunks for loading
        for chunk_id in desired_chunks {
            if !self.loaded_chunks.contains_key(&chunk_id) 
                && !self.loading_queue.contains(&chunk_id) 
            {
                self.loading_queue.push_back(chunk_id);
            }
        }
        
        // Load one chunk per frame (budget)
        if let Some(chunk_id) = self.loading_queue.pop_front() {
            self.start_loading_chunk(chunk_id);
        }
    }
    
    fn start_loading_chunk(&mut self, chunk_id: ChunkId) {
        // Async load chunk
        // When ready, insert into loaded_chunks
    }
}
```

**Streaming strategies:**

1. **Distance-based:**
```rust
// Load/unload based on distance from player
if distance(player, chunk.center) < LOAD_RADIUS {
    load(chunk);
} else if distance(player, chunk.center) > UNLOAD_RADIUS {
    unload(chunk);
}
```

2. **View-frustum-based:**
```rust
// Load visible chunks + some off-screen buffer
if frustum.contains(chunk) || distance(player, chunk) < BUFFER {
    load(chunk);
}
```

3. **Priority-based:**
```rust
// Sort chunks by importance
chunks.sort_by_key(|chunk| {
    let dist = distance(player, chunk);
    let visible = frustum.contains(chunk);
    (visible, -dist) // Visible first, then by distance
});
```

**Challenges:**

**Pop-in (assets appearing suddenly):**
```rust
// Solution: LOD system
// Load low-res version first, then high-res
chunk.load_lod0_async(); // Low quality, fast
chunk.load_lod1_async(); // Medium quality
chunk.load_lod2_async(); // High quality, slow
```

**Loading speed vs movement speed:**
```rust
// Must load faster than player can move
const PLAYER_MAX_SPEED: f32 = 20.0; // m/s
const LOAD_BUFFER: f32 = 100.0; // meters
const TIME_TO_REACH_BUFFER: f32 = LOAD_BUFFER / PLAYER_MAX_SPEED; // 5s

// Must load chunk in < 5s or player sees pop-in
```

**Memory budget:**
```rust
struct StreamingBudget {
    max_chunks_loaded: usize,
    max_memory_mb: usize,
    current_memory_mb: usize,
}

impl StreamingBudget {
    fn can_load(&self, chunk: &Chunk) -> bool {
        self.current_memory_mb + chunk.size_mb() < self.max_memory_mb
    }
}
```

### Approach 4: On-Demand Loading

**How it works:**
```rust
// Load asset only when first accessed
let texture = asset_manager.get_or_load("texture.png");
// If not loaded, loads now (might stutter)
// If loaded, returns immediately
```

#### Choose On-Demand If:

**✅ High Priority:**
- **Unpredictable usage** (user-driven content)
- **Many small assets** (textures, sounds)
- **Editor tools** (lazy loading)
- **Procedural content** (generate on-demand)

**Example Use Cases:**
- Game editors
- User-generated content
- Mod support
- Development/debugging

**Pros:**
- **Memory efficient**: Only load what's used
- **Simple**: Don't predict what's needed
- **Flexible**: Handles dynamic scenarios
- **No waste**: Never load unused assets

**Cons:**
- **Stutter**: First access causes frame drop
- **Unpredictable**: Hard to guarantee smooth performance
- **Cache complexity**: Need eviction strategy
- **Duplicates**: Might load same asset multiple times

**Implementation:**
```rust
pub struct AssetCache<T> {
    cache: HashMap<String, Arc<T>>,
    load_fn: Box<dyn Fn(&str) -> T>,
}

impl<T> AssetCache<T> {
    pub fn get_or_load(&mut self, path: &str) -> Arc<T> {
        if let Some(asset) = self.cache.get(path) {
            // Cache hit - instant
            Arc::clone(asset)
        } else {
            // Cache miss - load now (might stutter!)
            let asset = (self.load_fn)(path);
            let arc = Arc::new(asset);
            self.cache.insert(path.to_string(), Arc::clone(&arc));
            arc
        }
    }
}
```

**With async fallback:**
```rust
pub enum AssetState<T> {
    NotLoaded,
    Loading(JoinHandle<T>),
    Ready(Arc<T>),
}

impl<T> AssetCache<T> {
    pub fn get_or_start_load(&mut self, path: &str) -> Option<Arc<T>> {
        match self.cache.entry(path.to_string()) {
            Entry::Occupied(e) => {
                match e.get() {
                    AssetState::Ready(asset) => Some(Arc::clone(asset)),
                    AssetState::Loading(_) => None, // Still loading
                    AssetState::NotLoaded => unreachable!(),
                }
            }
            Entry::Vacant(e) => {
                // Start async load
                let handle = tokio::spawn(async_load(path));
                e.insert(AssetState::Loading(handle));
                None
            }
        }
    }
}

// Usage: show placeholder while loading
let texture = cache.get_or_start_load("tex.png")
    .unwrap_or(default_texture);
```

### Approach 5: Preloading (Level-based)

**How it works:**
```rust
// Load entire level before starting
show_loading_screen();
let level = load_level_complete("level1.gltf").await;
hide_loading_screen();
start_level(level);
```

#### Choose Preloading If:

**✅ High Priority:**
- **Level-based** game (discrete levels)
- **Small levels** (< 1GB per level)
- **Fast storage** (SSD, console)
- Want **guaranteed smooth gameplay**
- **Linear games** (known progression)

**Example Use Cases:**
- Linear action games (Uncharted)
- Puzzle games (Portal)
- Fighting games (Street Fighter)
- Racing games (single track)

**Pros:**
- **Smooth gameplay**: Everything loaded before play
- **Predictable**: Know exact load time
- **Simple**: Just load everything upfront
- **No pop-in**: All assets ready
- **Testing**: Easy to verify all assets present

**Cons:**
- **Load screens**: Players wait before playing
- **Memory**: Must fit entire level in RAM
- **Wasted memory**: Load assets that might not be used
- **Long waits**: Large levels = long load screens
- **Inflexible**: Can't adapt to player choices

**Implementation:**
```rust
pub async fn load_level_complete(path: &str) -> Level {
    show_loading_screen("Loading level...");
    
    // Load all assets in parallel
    let (
        geometry,
        textures,
        audio,
        scripts,
    ) = futures::join!(
        load_all_geometry(path),
        load_all_textures(path),
        load_all_audio(path),
        load_all_scripts(path),
    );
    
    hide_loading_screen();
    
    Level {
        geometry,
        textures,
        audio,
        scripts,
    }
}
```

**With progress bar:**
```rust
pub async fn load_level_with_progress(path: &str) -> Level {
    let total_assets = count_assets(path);
    let mut loaded = 0;
    
    show_loading_screen("Loading...");
    
    for asset in list_assets(path) {
        load_asset(asset).await;
        loaded += 1;
        update_progress_bar(loaded as f32 / total_assets as f32);
    }
    
    hide_loading_screen();
}
```

## Hybrid Approaches

### Async + Streaming (Common)

```rust
// Preload nearby, stream distant
pub struct HybridLoader {
    immediate_radius: f32,  // Preload everything
    streaming_radius: f32,  // Stream as needed
}

impl HybridLoader {
    pub fn update(&mut self, player_pos: Vec3) {
        // Preload everything in immediate radius
        for chunk in chunks_in_radius(player_pos, self.immediate_radius) {
            if !self.is_loaded(chunk) {
                // Async load (must complete before entering)
                self.async_load(chunk);
            }
        }
        
        // Stream in streaming radius
        for chunk in chunks_in_radius(player_pos, self.streaming_radius) {
            if !self.is_loading(chunk) && !self.is_loaded(chunk) {
                // Low priority streaming
                self.stream_load(chunk);
            }
        }
    }
}
```

### Preload + On-Demand (Praxis Approach)

```rust
// Preload common assets, load rare ones on-demand
pub struct AssetManager {
    preloaded: HashMap<String, Asset>,  // Loaded at startup
    cache: HashMap<String, Asset>,      // Loaded on-demand
}

impl AssetManager {
    pub fn preload_common_assets(&mut self) {
        // Load frequently-used assets
        self.preloaded.insert("default_texture.png", load(...));
        self.preloaded.insert("ui_font.ttf", load(...));
        // ...
    }
    
    pub fn get(&mut self, path: &str) -> &Asset {
        // Check preloaded first
        if let Some(asset) = self.preloaded.get(path) {
            return asset;
        }
        
        // Fall back to on-demand
        self.cache.entry(path)
            .or_insert_with(|| load(path))
    }
}
```

## Platform Considerations

### Desktop (SSD)
**Recommendation: Async or Streaming**

NVMe SSDs are very fast:
- Read speed: 3-7 GB/s
- Latency: <0.1ms

Can load assets quickly, streaming works well.

### Console (PS5, Xbox Series X)
**Recommendation: Streaming**

Custom fast I/O:
- PS5: 5.5 GB/s (raw)
- Xbox Velocity Architecture

Designed for streaming, eliminate load screens.

### Mobile
**Recommendation: Preload or minimal async**

Mobile storage varies:
- eMMC: Slow (100-400 MB/s)
- UFS: Fast (500-2000 MB/s)

Battery concerns limit background loading.

### Web
**Recommendation: Progressive loading with caching**

Network bottleneck:
- Download speed varies (1-100+ Mbps)
- Latency (50-200ms)

**Strategy:**
```javascript
// 1. Load minimal bundle first
loadCore().then(() => {
    // 2. Show something quickly
    startGame();
    
    // 3. Load additional assets in background
    loadAdditionalAssets();
});
```

## Common Pitfalls

### Pitfall 1: Loading Too Much

```rust
// Bad: Load entire game at startup
load_all_levels();        // 10 GB
load_all_textures();      // 5 GB
load_all_audio();         // 3 GB
// Out of memory!

// Good: Load what's needed
load_current_level();     // 500 MB
load_common_assets();     // 100 MB
```

### Pitfall 2: Blocking the Main Thread

```rust
// Bad: Heavy parsing on main thread
let bytes = async_read_file(path).await;  // Async I/O - good
let model = parse_gltf(&bytes);           // Sync CPU work - blocks!

// Good: Parse on background thread
let bytes = async_read_file(path).await;
let model = tokio::task::spawn_blocking(move || {
    parse_gltf(&bytes)  // Off main thread
}).await?;
```

### Pitfall 3: No Loading Budget

```rust
// Bad: Load unlimited assets per frame
while let Some(request) = load_queue.pop() {
    load_sync(request); // Might take 100ms!
}

// Good: Time budget per frame
let budget_ms = 5.0;
let start = Instant::now();
while let Some(request) = load_queue.pop() {
    if start.elapsed().as_secs_f32() * 1000.0 > budget_ms {
        break; // Defer to next frame
    }
    load_sync(request);
}
```

### Pitfall 4: Not Handling Missing Assets

```rust
// Bad: Crash if asset missing
let texture = assets.get("texture.png").unwrap(); // Panic!

// Good: Fallback
let texture = assets.get("texture.png")
    .unwrap_or(assets.get_default_texture());
```

### Pitfall 5: Memory Leaks

```rust
// Bad: Never unload assets
fn load_level(assets: &mut AssetManager) {
    assets.load("level.gltf"); // Stays in memory forever
}

// Good: Unload when done
fn load_level(assets: &mut AssetManager) {
    assets.load("level.gltf");
}

fn unload_level(assets: &mut AssetManager) {
    assets.unload_level_assets(); // Free memory
}
```

## Loading Performance

### Load Time Examples (SSD)

**Small asset (texture 2K):**
```
File I/O:     5ms
Decompression: 3ms
GPU upload:   2ms
Total:        10ms (acceptable per frame at 60 FPS)
```

**Medium asset (model):**
```
File I/O:     50ms
Parsing:      100ms
GPU upload:   20ms
Total:        170ms (must async or causes stutter)
```

**Large asset (level):**
```
File I/O:     500ms
Parsing:      2000ms
GPU upload:   300ms
Total:        2800ms (must async + show loading screen)
```

### Async vs Sync Comparison

**Scenario: Load 10 models**

**Synchronous:**
```
Model 1: 200ms (freeze)
Model 2: 200ms (freeze)
...
Model 10: 200ms (freeze)
Total: 2000ms frozen game
```

**Asynchronous (parallel):**
```
All 10 models: 200ms (game responsive)
Speedup: 10x
```

## Decision Checklist

| Question | Sync | Async | Streaming | On-Demand | Preload |
|----------|------|-------|-----------|-----------|---------|
| Can afford stuttering? | ✓ | | | | |
| Need smooth experience? | | ✓ | ✓ | | ✓ |
| Open world? | | | ✓ | | |
| Level-based? | | ✓ | | | ✓ |
| Limited memory? | | | ✓ | ✓ | |
| Unpredictable usage? | | | | ✓ | |
| Prototyping? | ✓ | | | | |
| Production? | | ✓ | ⚠️ | ⚠️ | ✓ |

## Recommended Reading

- **General:**
  - [Fast Loading Times](https://www.gdcvault.com/play/1025402/Technical-Art-Smackdown-Fast-Loading)
  - [I/O Best Practices](https://developer.nvidia.com/blog/best-practices-for-io-in-games/)

- **Streaming:**
  - [GTA V Streaming](https://www.adriancourreges.com/blog/2015/11/02/gta-v-graphics-study/)
  - [PS5 I/O Architecture](https://www.youtube.com/watch?v=erxUR9SI4F0)

- **Async Rust:**
  - [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
  - Praxis: `crates/praxis_assets/README.md`

## Conclusion

**TL;DR:**
- **Prototype? → Synchronous (simple)**
- **Most games? → Asynchronous (smooth + reasonable complexity)**
- **Open world? → Streaming (complex but necessary)**
- **Level-based? → Preloading (simple + guaranteed smooth)**
- **Unpredictable? → On-demand (flexible)**

**Praxis Choice: Asynchronous + Preload Common**

Reasons:
1. **Async I/O**: Using `tokio` for non-blocking loads
2. **Preload**: Common assets (shaders, default textures) at startup
3. **Background parsing**: Heavy parsing off main thread
4. **Progressive**: Can add streaming later if needed

**Implementation path:**
```rust
// 1. Start with sync (prototype)
let model = load_model_sync("model.gltf");

// 2. Add async (production)
let model = load_model_async("model.gltf").await;

// 3. Add streaming (if needed for scale)
streaming_manager.update(player_pos);
```

**Key takeaway:** Start simple (sync or async), add complexity (streaming) only when needed. Most games don't need streaming!
