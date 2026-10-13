//! The compute pool and schedule executor of ADR 0016.

use std::num::NonZeroUsize;

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ExecutorKind;
use bevy_tasks::{ComputeTaskPool, TaskPoolBuilder};

/// Creates the process-global `ComputeTaskPool` with one worker per core
/// but the main thread's, which joins scopes itself. Idempotent: a pool
/// already created is returned unchanged.
pub fn init_compute_pool() -> &'static ComputeTaskPool {
    ComputeTaskPool::get_or_init(|| {
        let cores = std::thread::available_parallelism().map_or(1, NonZeroUsize::get);
        let workers = cores.saturating_sub(1).max(1);
        tracing::debug!(workers, cores, "compute pool");
        TaskPoolBuilder::new()
            .num_threads(workers)
            .thread_name("Compute Task Pool".to_string())
            .build()
    })
}

/// A schedule on the single-threaded executor, which `multi_threaded` would
/// otherwise replace: per-system overhead outweighs the gain until a
/// Baseline shows otherwise (ADR 0016).
pub(crate) fn single_threaded_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    schedule
}
