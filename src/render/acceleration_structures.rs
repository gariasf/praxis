/// What acceleration structures take in GPU memory, as measured on one
/// adapter: estimates, since wgpu exposes no acceleration-structure size,
/// until M4 sets exact ones (decision on #17, item 15).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AccelerationStructureBytes {
    pub per_blas_triangle: f64,
    pub per_compacted_blas_triangle: f64,
}

impl AccelerationStructureBytes {
    /// The M4 Pro's on Metal: `currentAllocatedSize` around creating 2.1M
    /// triangles of BLAS (`docs/research/global-illumination-and-lighting.md`).
    const M4_PRO: Self = Self {
        per_blas_triangle: 94.0,
        per_compacted_blas_triangle: 45.5,
    };

    /// lavapipe's on Vulkan (Mesa 25.2.8): wgpu's allocator report around
    /// building and compacting BLASes of 2,048 to 131,072 triangles, which
    /// compaction leaves as they were (#231).
    const LAVAPIPE: Self = Self {
        per_blas_triangle: 104.0,
        per_compacted_blas_triangle: 104.0,
    };

    /// The estimate for `backend`: lavapipe's on Vulkan, software or not,
    /// and the M4 Pro's on any other, as the Budgets' reference machine
    /// (ADR 0013).
    pub fn estimate(backend: wgpu::Backend) -> Self {
        match backend {
            wgpu::Backend::Vulkan => Self::LAVAPIPE,
            _ => Self::M4_PRO,
        }
    }

    /// Bytes of BLASes of `compacted_triangles` compacted and
    /// `uncompacted_triangles` not.
    pub fn blases(&self, compacted_triangles: u64, uncompacted_triangles: u64) -> u64 {
        (compacted_triangles as f64 * self.per_compacted_blas_triangle
            + uncompacted_triangles as f64 * self.per_blas_triangle)
            .round() as u64
    }
}
