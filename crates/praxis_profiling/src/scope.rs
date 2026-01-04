//! Profiling scope management for tracking execution time.

use parking_lot::Mutex;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

/// Unique identifier for a profiling scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub(crate) u64);

/// Data about a completed profiling scope.
#[derive(Debug, Clone)]
pub struct ScopeData {
    /// Unique identifier for this scope
    #[allow(dead_code)]
    pub id: ScopeId,
    /// Name of the scope
    pub name: String,
    /// Start time of the scope
    pub start_time: Instant,
    /// Duration of the scope
    pub duration: Duration,
    /// Thread ID where the scope executed
    pub thread_id: std::thread::ThreadId,
    /// Parent scope ID, if any
    #[allow(dead_code)]
    pub parent_id: Option<ScopeId>,
    /// Depth level in the scope hierarchy
    pub depth: u32,
}

/// Callback function for completed scopes.
type ScopeCallback = Arc<dyn Fn(ScopeData) + Send + Sync>;

/// Global scope tracking state.
#[derive(Default)]
struct ScopeState {
    /// Next scope ID to assign
    next_id: u64,
    /// Callback to invoke when scopes complete
    callback: Option<ScopeCallback>,
    /// Current scope stack per thread
    scope_stack: std::collections::HashMap<std::thread::ThreadId, Vec<ScopeId>>,
}

static SCOPE_STATE: OnceLock<Mutex<ScopeState>> = OnceLock::new();

/// Sets the global callback for completed profiling scopes.
///
/// This callback will be invoked whenever a `ProfileScope` is dropped,
/// allowing the profiler to collect timing data.
pub fn set_scope_callback<F>(callback: F)
where
    F: Fn(ScopeData) + Send + Sync + 'static,
{
    let mut state = SCOPE_STATE.get_or_init(|| Mutex::new(ScopeState::default())).lock();
    state.callback = Some(Arc::new(callback));
}

/// Clears the global scope callback.
pub fn clear_scope_callback() {
    let mut state = SCOPE_STATE.get_or_init(|| Mutex::new(ScopeState::default())).lock();
    state.callback = None;
}

/// Allocates a new unique scope ID.
fn allocate_scope_id() -> ScopeId {
    let mut state = SCOPE_STATE.get_or_init(|| Mutex::new(ScopeState::default())).lock();
    let id = state.next_id;
    state.next_id += 1;
    ScopeId(id)
}

/// Gets the current parent scope ID for the calling thread.
fn get_parent_scope_id() -> Option<ScopeId> {
    let state = SCOPE_STATE.get_or_init(|| Mutex::new(ScopeState::default())).lock();
    let thread_id = std::thread::current().id();
    state
        .scope_stack
        .get(&thread_id)
        .and_then(|stack| stack.last().copied())
}

/// Pushes a scope ID onto the current thread's stack.
fn push_scope_id(id: ScopeId) {
    let mut state = SCOPE_STATE.get_or_init(|| Mutex::new(ScopeState::default())).lock();
    let thread_id = std::thread::current().id();
    state
        .scope_stack
        .entry(thread_id)
        .or_default()
        .push(id);
}

/// Pops a scope ID from the current thread's stack.
fn pop_scope_id() {
    let mut state = SCOPE_STATE.get_or_init(|| Mutex::new(ScopeState::default())).lock();
    let thread_id = std::thread::current().id();
    if let Some(stack) = state.scope_stack.get_mut(&thread_id) {
        stack.pop();
    }
}

/// A RAII profiling scope that automatically tracks execution time.
///
/// When created, it records the start time. When dropped, it calculates
/// the elapsed time and reports it to the global profiler.
///
/// # Example
///
/// ```rust,ignore
/// {
///     let _scope = ProfileScope::new("physics_update");
///     // Code to profile
/// } // Automatically reports timing when _scope is dropped
/// ```
pub struct ProfileScope {
    id: ScopeId,
    name: String,
    start_time: Instant,
    thread_id: std::thread::ThreadId,
    parent_id: Option<ScopeId>,
    depth: u32,
}

impl ProfileScope {
    /// Creates a new profiling scope with the given name.
    ///
    /// The scope begins timing immediately upon creation.
    pub fn new(name: impl Into<String>) -> Self {
        let id = allocate_scope_id();
        let parent_id = get_parent_scope_id();
        let thread_id = std::thread::current().id();
        
        // Calculate depth based on parent
        let depth = if parent_id.is_some() {
            let state = SCOPE_STATE.get_or_init(|| Mutex::new(ScopeState::default())).lock();
            state
                .scope_stack
                .get(&thread_id)
                .map(|stack| stack.len() as u32)
                .unwrap_or(0)
        } else {
            0
        };

        push_scope_id(id);

        Self {
            id,
            name: name.into(),
            start_time: Instant::now(),
            thread_id,
            parent_id,
            depth,
        }
    }
}

impl Drop for ProfileScope {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        pop_scope_id();

        let state = SCOPE_STATE.get_or_init(|| Mutex::new(ScopeState::default())).lock();
        if let Some(callback) = &state.callback {
            let data = ScopeData {
                id: self.id,
                name: self.name.clone(),
                start_time: self.start_time,
                duration,
                thread_id: self.thread_id,
                parent_id: self.parent_id,
                depth: self.depth,
            };
            callback(data);
        }
    }
}

/// Macro for creating a profiling scope with automatic naming.
///
/// # Example
///
/// ```rust,ignore
/// fn update_physics() {
///     profile_scope!(); // Creates scope named "update_physics"
///     // Code to profile
/// }
/// ```
#[macro_export]
macro_rules! profile_scope {
    () => {
        let _profile_scope = $crate::ProfileScope::new(
            ::core::concat!(::core::module_path!(), "::", ::core::line!())
        );
    };
    ($name:expr) => {
        let _profile_scope = $crate::ProfileScope::new($name);
    };
}
