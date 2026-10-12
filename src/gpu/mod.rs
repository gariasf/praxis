/// The adapter, device and queue that every GPU path shares. Created for a
/// window's surface, or headless for offscreen work and capability queries.
pub struct GpuContext {
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GpuContext {
    /// A context with no window or surface.
    pub async fn headless() -> anyhow::Result<Self> {
        Self::request(&create_instance(), None).await
    }

    /// A context whose adapter can present to `target`, and the surface for it.
    pub async fn with_surface(
        target: impl Into<wgpu::SurfaceTarget<'static>>,
    ) -> anyhow::Result<(Self, wgpu::Surface<'static>)> {
        let instance = create_instance();
        let surface = instance.create_surface(target)?;
        let context = Self::request(&instance, Some(&surface)).await?;
        Ok((context, surface))
    }

    async fn request(
        instance: &wgpu::Instance,
        compatible_surface: Option<&wgpu::Surface<'_>>,
    ) -> anyhow::Result<Self> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface,
                force_fallback_adapter: false,
            })
            .await?;

        let adapter_info = adapter.get_info();
        tracing::info!(
            name = %adapter_info.name,
            backend = ?adapter_info.backend,
            device_type = ?adapter_info.device_type,
            "adapter selected"
        );

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        Ok(Self {
            adapter,
            device,
            queue,
        })
    }

    /// The adapter, its backend, and whether the adapter supports each
    /// feature the roadmap plans on, with the limits that size it. One line
    /// per entry.
    pub fn capability_report(&self) -> String {
        let info = self.adapter.get_info();
        let features = self.adapter.features();
        let limits = self.adapter.limits();
        let supported = |feature| {
            if features.contains(feature) {
                "yes"
            } else {
                "no"
            }
        };

        let entries = [
            ("adapter", format!("{} ({:?})", info.name, info.device_type)),
            ("backend", info.backend.to_string()),
            (
                "binding arrays",
                format!(
                    "{}  non-uniform indexing {}, partially bound {}, max {} per stage",
                    supported(wgpu::Features::TEXTURE_BINDING_ARRAY),
                    supported(
                        wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                    ),
                    supported(wgpu::Features::PARTIALLY_BOUND_BINDING_ARRAY),
                    limits.max_binding_array_elements_per_shader_stage,
                ),
            ),
            (
                "mesh shaders",
                format!(
                    "{}  max {} output vertices, {} output primitives",
                    supported(wgpu::Features::EXPERIMENTAL_MESH_SHADER),
                    limits.max_mesh_output_vertices,
                    limits.max_mesh_output_primitives,
                ),
            ),
            (
                "ray query",
                format!(
                    "{}  max {} TLAS instances",
                    supported(wgpu::Features::EXPERIMENTAL_RAY_QUERY),
                    limits.max_tlas_instance_count,
                ),
            ),
            (
                "timestamp queries",
                format!(
                    "{}  inside encoders {}, inside passes {}",
                    supported(wgpu::Features::TIMESTAMP_QUERY),
                    supported(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS),
                    supported(wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES),
                ),
            ),
            (
                "BC compression",
                supported(wgpu::Features::TEXTURE_COMPRESSION_BC).to_string(),
            ),
            (
                "subgroups",
                format!(
                    "{}  barrier {}, size {} to {}",
                    supported(wgpu::Features::SUBGROUP),
                    supported(wgpu::Features::SUBGROUP_BARRIER),
                    info.subgroup_min_size,
                    info.subgroup_max_size,
                ),
            ),
            (
                "immediates",
                format!(
                    "{}  max {} bytes",
                    supported(wgpu::Features::IMMEDIATES),
                    limits.max_immediate_size,
                ),
            ),
        ];

        entries
            .iter()
            .map(|(label, value)| format!("{label:<18} {value}\n"))
            .collect()
    }
}

fn create_instance() -> wgpu::Instance {
    wgpu::Instance::new(wgpu::InstanceDescriptor {
        // `WGPU_BACKEND` (e.g. `vulkan`, `metal`) overrides the choice.
        backends: wgpu::Backends::PRIMARY.with_env(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: None,
    })
}
