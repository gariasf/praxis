// The marker after the BLAS refits: one invocation that reads their scratch
// and keeps what it read, so the GPU runs it once the refits are done, and a
// timestamp at its start ends their time (src/render/blas_refit.rs).
@compute @workgroup_size(1)
fn mark_blas_refit() {
    marker_result = refit_scratch[0];
}
