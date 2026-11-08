//! The Chunk streamer: which Chunks of a Game world to request, keep and
//! unload around the camera, decided each frame from the camera's World
//! position and the loads its IO finished. It reads no GPU state; `io`
//! serves it from the IO pool.

pub(crate) mod chunks;
mod io;
mod mips;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::RangeInclusive;

use glam::DVec3;

#[cfg(feature = "cook")]
pub(crate) use chunks::load_chunk;
pub(crate) use chunks::{LoadedChunk, LoadedChunks};
pub(crate) use io::ChunkStreaming;

use crate::cooked::{AssetId, CHUNK_EDGE_METRES};

/// What the Chunk streamer reaches IO through.
pub(crate) trait ChunkIo {
    /// Starts loading the Chunk at `coordinates`, x then z.
    fn request(&mut self, coordinates: [i32; 2]);

    /// A load that finished since the last call, if any. A load finishes
    /// once its Chunk's content is applied and the Assets it names are
    /// Resident: the lead and the Settled report count to that point.
    fn take_finished(&mut self) -> Option<FinishedLoad>;
}

/// A Chunk whose load finished, and the Mesh and Material Asset IDs its
/// Placements name, in Placement order.
pub(crate) struct FinishedLoad {
    pub coordinates: [i32; 2],
    pub assets: Vec<AssetId>,
}

/// What one frame's update decided. Coordinates are x then z.
#[derive(Clone, Debug, Default)]
pub(crate) struct ChunkFrame {
    /// Chunks requested this frame, nearest first.
    pub requested: Vec<[i32; 2]>,
    /// Chunks whose loads finished this frame, in the order IO gave them.
    pub loaded: Vec<[i32; 2]>,
    /// Loaded Chunks outside the Load radius: kept, but neither drawn nor
    /// traced.
    pub held: Vec<[i32; 2]>,
    /// Chunks unloaded this frame.
    pub unloaded: Vec<[i32; 2]>,
    /// The Assets whose last loaded user unloaded this frame, in the order
    /// of `unloaded`, then of each Chunk's Asset IDs.
    pub last_users: Vec<AssetId>,
    /// Chunks the camera came within the Load radius of this frame before
    /// they loaded, arrivals after a teleport aside.
    pub late_chunks: u32,
    /// Whether the camera, since it last arrived, still waits for a Chunk
    /// within the Load radius to load.
    pub holding: bool,
    /// Whether every Chunk this frame requests, now or on an earlier frame,
    /// has loaded.
    pub settled: bool,
}

pub(crate) struct ChunkStreamer {
    chunks: HashSet<[i32; 2]>,
    /// The lowest and highest coordinates of `chunks`, x then z.
    bounds: [RangeInclusive<i32>; 2],
    load_radius_metres: f64,
    loaded: BTreeMap<[i32; 2], Vec<AssetId>>,
    /// How many times loaded Chunks name each Asset.
    users: HashMap<AssetId, u32>,
    /// Each requested Chunk not yet loaded, and the frame it was requested
    /// on.
    in_flight: HashMap<[i32; 2], u64>,
    /// Chunks counted as Late Chunks whose loads have not finished.
    counted_late: HashSet<[i32; 2]>,
    within_load_radius: HashSet<[i32; 2]>,
    within_load_radius_before: HashSet<[i32; 2]>,
    camera_before: Option<DVec3>,
    frame_number: u64,
    /// Frames from request to load of the latest load. A load finishes on
    /// the frame after IO completes it, so the next may take one frame more.
    load_frames: u64,
    requests: Vec<(f64, [i32; 2])>,
    frame: ChunkFrame,
}

impl ChunkStreamer {
    /// A streamer over the Chunks at `chunks`, loading those within
    /// `load_radius_metres` of the camera.
    pub(crate) fn new(chunks: impl IntoIterator<Item = [i32; 2]>, load_radius_metres: f64) -> Self {
        let chunks: HashSet<[i32; 2]> = chunks.into_iter().collect();
        let along = |axis: usize| {
            let coordinates = chunks.iter().map(|chunk| chunk[axis]);
            let lowest = coordinates.clone().min().unwrap_or(0);
            let highest = coordinates.max().unwrap_or(-1);
            lowest..=highest
        };
        Self {
            bounds: [along(0), along(1)],
            chunks,
            load_radius_metres,
            loaded: BTreeMap::new(),
            users: HashMap::new(),
            in_flight: HashMap::new(),
            counted_late: HashSet::new(),
            within_load_radius: HashSet::new(),
            within_load_radius_before: HashSet::new(),
            camera_before: None,
            frame_number: 0,
            load_frames: 0,
            requests: Vec::new(),
            frame: ChunkFrame::default(),
        }
    }

    /// Decides this frame's requests, loads and unloads for the camera at
    /// `camera`, which arrives without travelling when it `teleported` or
    /// on the first frame.
    pub(crate) fn update(
        &mut self,
        camera: DVec3,
        teleported: bool,
        io: &mut impl ChunkIo,
    ) -> &ChunkFrame {
        let load_radius_metres = self.load_radius_metres;
        let hold_radius_metres = self.hold_radius_metres();
        let frame = &mut self.frame;
        frame.requested.clear();
        frame.loaded.clear();
        frame.held.clear();
        frame.unloaded.clear();
        frame.last_users.clear();
        frame.late_chunks = 0;
        let metres_per_frame = match self.camera_before {
            Some(before) if !teleported => (camera.x - before.x).hypot(camera.z - before.z),
            _ => 0.0,
        };
        let arrived = teleported || self.camera_before.is_none();

        while let Some(load) = io.take_finished() {
            let Some(requested_on) = self.in_flight.remove(&load.coordinates) else {
                continue;
            };
            self.counted_late.remove(&load.coordinates);
            self.load_frames = self.frame_number - requested_on;
            if distance_metres(camera, load.coordinates) > hold_radius_metres {
                continue;
            }
            for &asset in &load.assets {
                *self.users.entry(asset).or_default() += 1;
            }
            frame.loaded.push(load.coordinates);
            self.loaded.insert(load.coordinates, load.assets);
        }

        let users = &mut self.users;
        self.loaded.retain(|&coordinates, assets| {
            if distance_metres(camera, coordinates) <= hold_radius_metres {
                return true;
            }
            frame.unloaded.push(coordinates);
            for asset in assets.iter() {
                let Some(count) = users.get_mut(asset) else {
                    continue;
                };
                *count -= 1;
                if *count == 0 {
                    users.remove(asset);
                    frame.last_users.push(*asset);
                }
            }
            false
        });

        let lead_metres = metres_per_frame * (self.load_frames + 1) as f64;
        let request_radius_metres = (load_radius_metres + lead_metres).min(hold_radius_metres);
        std::mem::swap(
            &mut self.within_load_radius,
            &mut self.within_load_radius_before,
        );
        self.within_load_radius.clear();
        let mut every_chunk_within_load_radius_loaded = true;
        frame.settled = true;
        for coordinates in chunks_within(camera, request_radius_metres, &self.bounds) {
            if !self.chunks.contains(&coordinates) {
                continue;
            }
            let metres = distance_metres(camera, coordinates);
            if metres > request_radius_metres {
                continue;
            }
            let loaded = self.loaded.contains_key(&coordinates);
            frame.settled &= loaded;
            if metres <= load_radius_metres {
                self.within_load_radius.insert(coordinates);
                every_chunk_within_load_radius_loaded &= loaded;
                let entered = !self.within_load_radius_before.contains(&coordinates);
                if !arrived && !loaded && entered && self.counted_late.insert(coordinates) {
                    frame.late_chunks += 1;
                }
            }
            if !loaded && !self.in_flight.contains_key(&coordinates) {
                self.requests.push((metres, coordinates));
            }
        }
        frame.holding = (frame.holding || arrived) && !every_chunk_within_load_radius_loaded;
        self.requests
            .sort_unstable_by(|(left_metres, left), (right_metres, right)| {
                left_metres.total_cmp(right_metres).then(left.cmp(right))
            });
        for (_, coordinates) in self.requests.drain(..) {
            io.request(coordinates);
            self.in_flight.insert(coordinates, self.frame_number);
            frame.requested.push(coordinates);
        }

        frame.held.extend(
            self.loaded
                .keys()
                .filter(|&&coordinates| distance_metres(camera, coordinates) > load_radius_metres),
        );
        self.camera_before = Some(camera);
        self.frame_number += 1;
        frame
    }
}

impl ChunkStreamer {
    fn hold_radius_metres(&self) -> f64 {
        self.load_radius_metres + CHUNK_EDGE_METRES
    }

    /// The Chunks within the hold radius of `camera`: the most it requests
    /// or keeps from there.
    pub(crate) fn within_hold_radius(&self, camera: DVec3) -> impl Iterator<Item = [i32; 2]> + '_ {
        let hold_radius_metres = self.hold_radius_metres();
        chunks_within(camera, hold_radius_metres, &self.bounds).filter(move |coordinates| {
            self.chunks.contains(coordinates)
                && distance_metres(camera, *coordinates) <= hold_radius_metres
        })
    }
}

/// The horizontal distance from `camera` to the nearest point of the Chunk
/// at `coordinates`.
pub(crate) fn distance_metres(camera: DVec3, [x, z]: [i32; 2]) -> f64 {
    let along = |position: f64, coordinate: i32| {
        let lowest = coordinate as f64 * CHUNK_EDGE_METRES;
        (lowest - position)
            .max(position - lowest - CHUNK_EDGE_METRES)
            .max(0.0)
    };
    along(camera.x, x).hypot(along(camera.z, z))
}

/// The coordinates within `bounds`, x then z, of every Chunk that overlaps
/// the square of half-size `radius_metres` around `camera`.
fn chunks_within(
    camera: DVec3,
    radius_metres: f64,
    bounds: &[RangeInclusive<i32>; 2],
) -> impl Iterator<Item = [i32; 2]> + use<> {
    let range = |position: f64, bounds: &RangeInclusive<i32>| {
        let chunk = |metres: f64| (metres / CHUNK_EDGE_METRES).floor() as i32;
        let lowest = chunk(position - radius_metres).max(*bounds.start());
        let highest = chunk(position + radius_metres).min(*bounds.end());
        lowest..=highest
    };
    let columns = range(camera.x, &bounds[0]);
    let rows = range(camera.z, &bounds[1]);
    columns.flat_map(move |x| rows.clone().map(move |z| [x, z]))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every Reference scene but the world draws all its Chunks on each
    /// frame of a reference bench, so each keeps its Goldens and Baselines.
    #[test]
    fn each_reference_scene_holds_every_chunk_within_its_load_radius() {
        for scene in crate::scenes::ReferenceScene::ALL {
            if scene == crate::scenes::ReferenceScene::World {
                continue;
            }
            let chunks = scene.chunk_coordinates();
            for frame in 0..2048 {
                let camera = scene.camera_at(frame, DVec3::ZERO).position;
                let farthest = chunks
                    .iter()
                    .map(|&chunk| distance_metres(camera, chunk))
                    .fold(0.0, f64::max);
                assert!(
                    farthest <= scene.load_radius_metres(),
                    "{} frame {frame}: a Chunk {farthest} m away",
                    scene.name()
                );
            }
        }
    }

    const LOAD_RADIUS_METRES: f64 = 256.0;

    /// The Mesh Asset ID every Chunk in column `x` names.
    fn mesh(x: i32) -> AssetId {
        AssetId((1000 + x) as u128)
    }

    /// The Material Asset ID every Chunk in row `z` names.
    fn material(z: i32) -> AssetId {
        AssetId((2000 + z) as u128)
    }

    /// Loads that finish a set number of frames after their request, all
    /// in `load_frames` unless `delays` names the Chunk.
    struct ScriptedIo {
        load_frames: u32,
        delays: HashMap<[i32; 2], u32>,
        frame: u32,
        in_flight: Vec<(u32, [i32; 2])>,
    }

    impl ScriptedIo {
        fn new(load_frames: u32) -> Self {
            Self {
                load_frames,
                delays: HashMap::new(),
                frame: 0,
                in_flight: Vec::new(),
            }
        }
    }

    impl ChunkIo for ScriptedIo {
        fn request(&mut self, coordinates: [i32; 2]) {
            let load_frames = self
                .delays
                .get(&coordinates)
                .copied()
                .unwrap_or(self.load_frames);
            self.in_flight.push((self.frame + load_frames, coordinates));
        }

        fn take_finished(&mut self) -> Option<FinishedLoad> {
            let position = self
                .in_flight
                .iter()
                .position(|&(finishes, _)| finishes <= self.frame)?;
            let (_, coordinates @ [x, z]) = self.in_flight.remove(position);
            Some(FinishedLoad {
                coordinates,
                assets: vec![mesh(x), material(z)],
            })
        }
    }

    /// A streamer over the Chunks from -4 to 40 in x and -4 to 4 in z.
    fn streamer() -> ChunkStreamer {
        let chunks = (-4..=40).flat_map(|x| (-4..=4).map(move |z| [x, z]));
        ChunkStreamer::new(chunks, LOAD_RADIUS_METRES)
    }

    /// The camera over the middle of Chunk `[x, z]`.
    fn middle_of([x, z]: [i32; 2]) -> DVec3 {
        DVec3::new(
            (x as f64 + 0.5) * CHUNK_EDGE_METRES,
            10.0,
            (z as f64 + 0.5) * CHUNK_EDGE_METRES,
        )
    }

    /// Runs one frame and steps the IO's clock past it.
    fn step(
        streamer: &mut ChunkStreamer,
        io: &mut ScriptedIo,
        camera: DVec3,
        teleported: bool,
    ) -> ChunkFrame {
        let frame = streamer.update(camera, teleported, io).clone();
        io.frame += 1;
        frame
    }

    #[test]
    fn chunks_load_within_the_load_radius_stay_held_to_the_hold_radius_and_unload_past_it() {
        let (mut streamer, mut io) = (streamer(), ScriptedIo::new(1));

        let first = step(&mut streamer, &mut io, middle_of([0, 0]), false);
        let around_origin = [
            [0, 0],
            [-1, 0],
            [0, -1],
            [0, 1],
            [1, 0],
            [-1, -1],
            [-1, 1],
            [1, -1],
            [1, 1],
        ];
        assert_eq!(first.requested, around_origin);
        assert!(first.loaded.is_empty());

        let second = step(&mut streamer, &mut io, middle_of([0, 0]), false);
        assert!(second.requested.is_empty());
        assert_eq!(second.loaded, around_origin);
        assert!(second.held.is_empty());

        // Two Chunks east: column 1 is 128 m away, column 0 is 384 m (held)
        // and column -1 is 640 m, past the 512 m hold radius.
        let jump = step(&mut streamer, &mut io, middle_of([2, 0]), true);
        assert_eq!(
            jump.requested,
            [[2, 0], [2, -1], [2, 1], [3, 0], [3, -1], [3, 1]]
        );
        assert_eq!(jump.held, [[0, -1], [0, 0], [0, 1]]);
        assert_eq!(jump.unloaded, [[-1, -1], [-1, 0], [-1, 1]]);

        let after = step(&mut streamer, &mut io, middle_of([2, 0]), false);
        assert_eq!(
            after.loaded,
            [[2, 0], [2, -1], [2, 1], [3, 0], [3, -1], [3, 1]]
        );
        assert_eq!(after.held, [[0, -1], [0, 0], [0, 1]]);
        assert!(after.unloaded.is_empty());

        // Back to the middle: the held column is drawn again without IO.
        let back = step(&mut streamer, &mut io, middle_of([1, 0]), true);
        assert!(back.requested.is_empty());
        assert_eq!(back.held, [[3, -1], [3, 0], [3, 1]]);
        assert!(back.unloaded.is_empty());
    }

    /// The camera at `frame` of a path that waits 10 frames over the middle
    /// of Chunk [0, 0], then flies east at 50 m/s, 60 frames a second.
    fn flying_east(frame: u32) -> DVec3 {
        middle_of([0, 0]) + DVec3::X * frame.saturating_sub(10) as f64 * 50.0 / 60.0
    }

    /// Late Chunks over 60 s of `flying_east`, as the streamer counts them.
    fn late_chunks_flying_east(mut io: ScriptedIo) -> u32 {
        let mut streamer = streamer();
        (0..3600)
            .map(|frame| step(&mut streamer, &mut io, flying_east(frame), false).late_chunks)
            .sum()
    }

    #[test]
    fn chunks_ahead_are_requested_a_load_time_early_at_50_metres_a_second() {
        let (mut streamer, mut io) = (streamer(), ScriptedIo::new(6));
        let frames: Vec<ChunkFrame> = (0..170)
            .map(|frame| step(&mut streamer, &mut io, flying_east(frame), false))
            .collect();

        // The column at x = 512 m comes within 256 m and 7 frames at 50/60 m
        // (the load's 6 and one more) on frame 10 + 147, and within 256 m on
        // frame 10 + 154.
        let requested_at = |chunk: [i32; 2]| {
            frames
                .iter()
                .position(|frame| frame.requested.contains(&chunk))
        };
        assert_eq!(requested_at([2, 0]), Some(157));
        assert!(frames[163].loaded.contains(&[2, 0]));
        assert_eq!(late_chunks_flying_east(ScriptedIo::new(6)), 0);
    }

    #[test]
    fn a_chunk_whose_load_is_delayed_past_its_arrival_counts_as_late_once() {
        let mut io = ScriptedIo::new(6);
        io.delays.insert([5, 0], 600);
        io.delays.insert([10, 0], 600);
        assert_eq!(late_chunks_flying_east(io), 2);
    }

    #[test]
    fn a_teleport_holds_until_the_last_chunk_in_the_load_radius_loads() {
        let (mut streamer, mut io) = (streamer(), ScriptedIo::new(1));
        io.delays.insert([21, 1], 5);
        let cameras = [
            [0, 0],
            [0, 0],
            [20, 0],
            [20, 0],
            [20, 0],
            [20, 0],
            [20, 0],
            [20, 0],
        ];
        let frames: Vec<ChunkFrame> = cameras
            .iter()
            .enumerate()
            .map(|(frame, &chunk)| step(&mut streamer, &mut io, middle_of(chunk), frame == 2))
            .collect();

        let holding: Vec<bool> = frames.iter().map(|frame| frame.holding).collect();
        assert_eq!(holding, [true, false, true, true, true, true, true, false]);
        assert_eq!(frames[7].loaded, [[21, 1]]);
        assert!(frames.iter().all(|frame| frame.late_chunks == 0));
    }

    #[test]
    fn a_frame_is_settled_when_every_chunk_it_requested_has_loaded() {
        let (mut streamer, mut io) = (streamer(), ScriptedIo::new(6));
        let unsettled: Vec<u32> = (0..210)
            .filter(|&frame| !step(&mut streamer, &mut io, flying_east(frame), false).settled)
            .collect();

        // The first frame's Chunks load on frame 6. Flying east, [2, 0] is
        // requested on frame 157 and [2, -1] and [2, 1], 128 m off the
        // camera's line, on frame 197, once within 256 m and 7 frames at
        // 50/60 m of it; each loads 6 frames later.
        let expected: Vec<u32> = (0..6).chain(157..163).chain(197..203).collect();
        assert_eq!(unsettled, expected);
    }

    #[test]
    fn an_unload_reports_the_assets_it_was_the_last_loaded_user_of_in_order() {
        let (mut streamer, mut io) = (streamer(), ScriptedIo::new(1));
        let cameras = [
            ([0, 0], false),
            ([0, 0], false),
            ([2, 0], true),
            ([2, 0], false),
        ];
        for (chunk, teleported) in cameras {
            let frame = step(&mut streamer, &mut io, middle_of(chunk), teleported);
            if teleported {
                // Column -1 unloads; held column 0 and column 1 still name
                // every row's Material.
                assert_eq!(frame.last_users, [mesh(-1)]);
            }
        }

        // Columns 0 to 3 unload in coordinate order, each Chunk naming its
        // column's Mesh, then its row's Material.
        let jump = step(&mut streamer, &mut io, middle_of([20, 0]), true);
        assert_eq!(jump.unloaded.len(), 12);
        assert_eq!(
            jump.last_users,
            [
                mesh(0),
                mesh(1),
                mesh(2),
                material(-1),
                material(0),
                mesh(3),
                material(1)
            ]
        );
    }

    #[test]
    fn a_chunk_the_camera_reaches_twice_before_it_loads_is_one_late_chunk() {
        let (mut streamer, mut io) = (streamer(), ScriptedIo::new(1));
        io.delays.insert([2, 0], 100);
        // Column 2 starts at x = 512 m: 255 m from x = 257 m, 262 m from 250.
        let late: Vec<u32> = [128.0, 128.0, 257.0, 250.0, 257.0]
            .into_iter()
            .map(|x| {
                let camera = DVec3::new(x, 10.0, 128.0);
                step(&mut streamer, &mut io, camera, false).late_chunks
            })
            .collect();
        assert_eq!(late, [0, 0, 1, 0, 0]);
    }

    #[test]
    fn a_load_finishing_past_the_hold_radius_is_dropped_and_one_in_flight_is_not_requested_again() {
        let (mut streamer, mut io) = (streamer(), ScriptedIo::new(1));
        io.delays.insert([1, 1], 3);
        let cameras = [
            ([0, 0], false),
            ([20, 0], true),
            ([0, 0], true),
            ([0, 0], false),
        ];
        let frames: Vec<ChunkFrame> = cameras
            .into_iter()
            .map(|(chunk, teleported)| step(&mut streamer, &mut io, middle_of(chunk), teleported))
            .collect();

        for frame in &frames[1..3] {
            assert!(frame.loaded.is_empty());
            assert!(frame.held.is_empty());
            assert!(frame.unloaded.is_empty());
        }
        let around_origin_but_in_flight = [
            [0, 0],
            [-1, 0],
            [0, -1],
            [0, 1],
            [1, 0],
            [-1, -1],
            [-1, 1],
            [1, -1],
        ];
        assert_eq!(frames[2].requested, around_origin_but_in_flight);
        assert_eq!(frames[3].loaded[0], [1, 1]);
        assert_eq!(frames[3].loaded[1..], around_origin_but_in_flight);
        assert!(!frames[3].holding);
    }
}
