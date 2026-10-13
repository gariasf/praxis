//! The compute pool is process-global, so this binary owns its process.

#[test]
fn compute_pool_leaves_the_main_thread_a_core() {
    let cores = std::thread::available_parallelism().unwrap().get();
    let pool = praxis::init_compute_pool();
    assert_eq!(pool.thread_num(), (cores - 1).max(1));
}
