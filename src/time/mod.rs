use std::time::Duration;

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::{LogLevel, ScheduleBuildSettings};

use crate::threading::single_threaded_schedule;

#[derive(Resource, Default)]
pub struct Time {
    // Seconds since the previous frame, zero on the first. Injected through
    // `State::update`: the window measures it, a capture fixes it. Per frame
    // only: the fixed schedule never reads it (ADR 0022).
    pub delta_time: f32,
}

/// Longest frame delta that advances the State (ADR 0022's clamp), so a
/// stalled event loop resumes without a jump.
pub const MAX_FRAME_DELTA: Duration = Duration::from_millis(250);

/// One Tick, 1/64 s, exact in nanoseconds and in binary.
pub const TICK: Duration = Duration::from_nanos(15_625_000);

/// The per-Tick delta systems in the fixed schedule integrate with.
pub const TICK_SECONDS: f64 = TICK.as_secs_f64();

/// Ticks run per frame at most; accumulated time past them is dropped.
pub const MAX_TICKS_PER_FRAME: u32 = 4;

/// The wall-clock time since the previous frame, as the State may advance by.
pub fn clamp_frame_delta(elapsed: Duration) -> Duration {
    elapsed.min(MAX_FRAME_DELTA)
}

/// The fixed loop's clock: simulated time is the Tick counter, never a sum
/// of seconds.
#[derive(Resource, Default)]
pub struct FixedTime {
    /// Ticks run so far; inside the fixed schedule, the running Tick's index.
    pub tick: u64,
    /// How far time has moved past the newest Tick, in Ticks, in [0, 1): the
    /// weight α of the newest Tick's state in the frame's blend.
    pub blend_weight: f32,
    accumulator: Duration,
}

impl FixedTime {
    /// Adds `frame_delta` and takes out the whole Ticks it completes. Returns
    /// the Ticks to run, at most `MAX_TICKS_PER_FRAME`, and the time of the
    /// Ticks past the cap, which is dropped.
    fn advance(&mut self, frame_delta: Duration) -> (u32, Duration) {
        self.accumulator += frame_delta;
        let due = self.accumulator.as_nanos() / TICK.as_nanos();
        let ticks = due.min(MAX_TICKS_PER_FRAME.into()) as u32;
        let remainder =
            Duration::from_nanos((self.accumulator.as_nanos() % TICK.as_nanos()) as u64);
        let dropped = self.accumulator - remainder - TICK * ticks;
        self.accumulator = remainder;
        self.blend_weight = (remainder.as_secs_f64() / TICK_SECONDS) as f32;
        (ticks, dropped)
    }
}

/// An empty fixed schedule, run once per Tick by `run_ticks`. Its build
/// fails on any pair of conflicting systems left unordered, so game state
/// never depends on the executor's choice of order.
pub fn fixed_schedule() -> Schedule {
    let mut schedule = single_threaded_schedule();
    schedule.set_build_settings(ScheduleBuildSettings {
        ambiguity_detection: LogLevel::Error,
        ..Default::default()
    });
    schedule
}

/// Advances the World's `FixedTime` by `frame_delta` and runs `schedule`
/// once per Tick due. Returns the time dropped past the Tick cap.
pub fn run_ticks(schedule: &mut Schedule, world: &mut World, frame_delta: Duration) -> Duration {
    let (ticks, dropped) = world.resource_mut::<FixedTime>().advance(frame_delta);
    for _ in 0..ticks {
        schedule.run(world);
        world.resource_mut::<FixedTime>().tick += 1;
    }
    dropped
}

#[cfg(test)]
mod tests {
    use winit::keyboard::KeyCode;

    use super::*;
    use crate::input::{Input, clear_just_pressed};

    /// 60 Hz, as the window measures it in whole nanoseconds.
    const SIXTY_HERTZ: Duration = Duration::from_nanos(16_666_667);

    fn world_with_fixed_time() -> World {
        let mut world = World::new();
        world.insert_resource(FixedTime::default());
        world
    }

    #[derive(Resource, Default)]
    struct Counter(u32);

    fn increment(mut counter: ResMut<Counter>) {
        counter.0 += 1;
    }

    fn double(mut counter: ResMut<Counter>) {
        counter.0 *= 2;
    }

    #[test]
    fn an_unordered_conflicting_pair_fails_the_build() {
        let mut world = world_with_fixed_time();
        world.insert_resource(Counter::default());

        let mut unordered = fixed_schedule();
        unordered.add_systems((increment, double));
        assert!(unordered.initialize(&mut world).is_err());

        let mut ordered = fixed_schedule();
        ordered.add_systems((increment, double).chain());
        assert!(ordered.initialize(&mut world).is_ok());
    }

    /// A body integrated per Tick in f64, and its position after every Tick.
    #[derive(Resource, Default)]
    struct Body {
        position: glam::DVec3,
        velocity: glam::DVec3,
        positions: Vec<glam::DVec3>,
    }

    /// The input replayed at `tick`: a thrust that is a fixed function of the
    /// Tick, uneven enough that every step rounds.
    fn thrust_at(tick: u64) -> glam::DVec3 {
        glam::dvec3(
            (tick % 7) as f64 * 0.37 - 1.1,
            (tick * 13 % 5) as f64 * 0.11,
            if tick % 64 < 32 { 1.3 } else { -0.9 },
        )
    }

    fn integrate(fixed_time: Res<FixedTime>, mut body: ResMut<Body>) {
        body.velocity += thrust_at(fixed_time.tick) * TICK_SECONDS;
        let velocity = body.velocity;
        body.position += velocity * TICK_SECONDS;
        let position = body.position;
        body.positions.push(position);
    }

    /// The body's position after each of the first `ticks` Ticks, with the
    /// frame deltas `next_frame_delta` returns.
    fn replay(ticks: usize, mut next_frame_delta: impl FnMut() -> Duration) -> Vec<glam::DVec3> {
        let mut world = world_with_fixed_time();
        world.insert_resource(Body::default());
        let mut schedule = fixed_schedule();
        schedule.add_systems(integrate);
        while world.resource::<Body>().positions.len() < ticks {
            run_ticks(&mut schedule, &mut world, next_frame_delta());
        }
        let mut positions = world.remove_resource::<Body>().unwrap().positions;
        positions.truncate(ticks);
        positions
    }

    #[test]
    fn the_same_inputs_reach_the_same_state_whatever_the_frame_timing() {
        const TICKS: usize = 1280;
        let steady = replay(TICKS, || SIXTY_HERTZ);

        // 1 to 40 ms frames from a fixed-seed generator, and every 97th frame
        // a stall the clamp cuts to 250 ms and the cap drops most of.
        let mut seed: u64 = 0x5eed;
        let mut frame = 0;
        let jittery = replay(TICKS, || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            frame += 1;
            if frame % 97 == 0 {
                clamp_frame_delta(Duration::from_secs(1))
            } else {
                Duration::from_micros(1000 + (seed >> 33) % 39_000)
            }
        });

        assert_eq!(steady.len(), TICKS);
        for (tick, (steady, jittery)) in steady.iter().zip(&jittery).enumerate() {
            assert_eq!(
                steady.to_array().map(f64::to_bits),
                jittery.to_array().map(f64::to_bits),
                "positions differ after Tick {tick}"
            );
        }
    }

    #[test]
    fn a_stall_is_clamped_to_250_ms_and_time_past_four_ticks_is_dropped() {
        let mut world = world_with_fixed_time();
        let mut schedule = fixed_schedule();

        let frame_delta = clamp_frame_delta(Duration::from_secs(1));
        assert_eq!(frame_delta, Duration::from_millis(250));
        // 16 Ticks are due; 4 run and the other 12 are dropped.
        let dropped = run_ticks(&mut schedule, &mut world, frame_delta);
        assert_eq!(dropped, Duration::from_micros(187_500));
        assert_eq!(world.resource::<FixedTime>().blend_weight, 0.0);
    }

    #[test]
    fn a_long_frame_under_the_clamp_keeps_its_partial_tick() {
        let mut world = world_with_fixed_time();
        let mut schedule = fixed_schedule();

        // 6.4 Ticks: 4 run, 2 are dropped, 0.4 of a Tick carries over.
        let frame_delta = clamp_frame_delta(Duration::from_millis(100));
        assert_eq!(frame_delta, Duration::from_millis(100));
        let dropped = run_ticks(&mut schedule, &mut world, frame_delta);
        assert_eq!(dropped, Duration::from_micros(31_250));
        assert_eq!(world.resource::<FixedTime>().blend_weight, 0.4);
    }

    #[test]
    fn steady_frames_drop_nothing() {
        let mut world = world_with_fixed_time();
        let mut schedule = fixed_schedule();

        for _ in 0..600 {
            let frame_delta = clamp_frame_delta(SIXTY_HERTZ);
            assert_eq!(frame_delta, SIXTY_HERTZ);
            assert_eq!(
                run_ticks(&mut schedule, &mut world, frame_delta),
                Duration::ZERO
            );
        }
        // 10.0000002 s of 1/64 s Ticks.
        assert_eq!(world.resource::<FixedTime>().tick, 640);
    }

    #[test]
    fn a_frame_shorter_than_a_tick_runs_none_and_keeps_its_time() {
        let mut world = world_with_fixed_time();
        world.insert_resource(Counter::default());
        let mut schedule = fixed_schedule();
        schedule.add_systems(increment);

        run_ticks(&mut schedule, &mut world, TICK / 2);
        assert_eq!(world.resource::<Counter>().0, 0);
        assert_eq!(world.resource::<FixedTime>().blend_weight, 0.5);
        run_ticks(&mut schedule, &mut world, TICK / 2);
        assert_eq!(world.resource::<Counter>().0, 1);
        assert_eq!(world.resource::<FixedTime>().tick, 1);
    }

    #[derive(Resource, Default)]
    struct Presses(u32);

    fn count_presses(input: Res<Input>, mut presses: ResMut<Presses>) {
        presses.0 += input.just_pressed.len() as u32;
    }

    #[test]
    fn a_press_in_a_frame_that_runs_no_tick_is_consumed_by_the_next_tick() {
        let mut world = world_with_fixed_time();
        world.insert_resource(Input::default());
        world.insert_resource(Presses::default());
        let mut schedule = fixed_schedule();
        schedule.add_systems((count_presses, clear_just_pressed).chain());

        // Pressed and released within a frame too short for a Tick.
        world
            .resource_mut::<Input>()
            .just_pressed
            .insert(KeyCode::KeyG);
        run_ticks(&mut schedule, &mut world, TICK / 2);
        assert_eq!(world.resource::<Presses>().0, 0);
        // The next frame runs two Ticks; only the first sees the press.
        run_ticks(&mut schedule, &mut world, TICK * 3 / 2);
        assert_eq!(world.resource::<Presses>().0, 1);
    }
}
