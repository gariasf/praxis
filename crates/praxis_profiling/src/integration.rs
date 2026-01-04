//! Integration helpers for profiling with various engine systems.

use crate::{Profiler, SystemProfiler};
use bevy_ecs::system::Resource;
use std::sync::Arc;

/// Resource wrapper for the profiler in ECS.
#[derive(Resource)]
pub struct ProfilerResource {
    profiler: Arc<Profiler>,
}

impl ProfilerResource {
    /// Creates a new profiler resource.
    pub fn new(profiler: Profiler) -> Self {
        Self {
            profiler: Arc::new(profiler),
        }
    }

    /// Gets a reference to the profiler.
    pub fn profiler(&self) -> &Arc<Profiler> {
        &self.profiler
    }
}

/// Resource wrapper for the system profiler in ECS.
#[derive(Resource, Clone)]
pub struct SystemProfilerResource {
    profiler: Arc<SystemProfiler>,
}

impl SystemProfilerResource {
    /// Creates a new system profiler resource.
    pub fn new(profiler: Arc<SystemProfiler>) -> Self {
        Self { profiler }
    }

    /// Gets a reference to the system profiler.
    pub fn profiler(&self) -> &Arc<SystemProfiler> {
        &self.profiler
    }
}

/// Macro for profiling a system with automatic name inference.
#[macro_export]
macro_rules! profile_system {
    ($profiler:expr, $body:expr) => {{
        let _scope = $crate::ProfileScope::new(::core::any::type_name::<_>());
        let _system_scope = $crate::system_profiler::SystemProfileScope::new(
            $profiler,
            ::core::any::type_name::<_>(),
        );
        $body
    }};
}
