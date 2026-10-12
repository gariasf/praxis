/// The adapter, device and queue that every GPU path shares. Created for a
/// window's surface, or headless for offscreen work and capability queries.
pub struct GpuContext {
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
}

impl GpuContext {
    /// A context with no window or surface.
    pub async fn headless() -> anyhow::Result<Self> {
        Self::request(&create_wgpu_instance(), None).await
    }

    /// A context whose adapter can present to `target`, and the surface for it.
    pub(crate) async fn with_surface(
        target: impl Into<wgpu::SurfaceTarget<'static>>,
    ) -> anyhow::Result<(Self, wgpu::Surface<'static>)> {
        let wgpu_instance = create_wgpu_instance();
        let surface = wgpu_instance.create_surface(target)?;
        let context = Self::request(&wgpu_instance, Some(&surface)).await?;
        Ok((context, surface))
    }

    async fn request(
        wgpu_instance: &wgpu::Instance,
        compatible_surface: Option<&wgpu::Surface<'_>>,
    ) -> anyhow::Result<Self> {
        let adapter = wgpu_instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface,
                force_fallback_adapter: false,
                apply_limit_buckets: false,
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
                // Timestamp queries time render passes for frame stats.
                required_features: adapter.features() & wgpu::Features::TIMESTAMP_QUERY,
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

    /// The graphics API the adapter runs on.
    pub fn backend(&self) -> wgpu::Backend {
        self.adapter.get_info().backend
    }

    /// The adapter, its backend, and whether the adapter supports each
    /// feature the roadmap plans on, with the related features and limits.
    /// One line per entry.
    pub fn capability_report(&self) -> String {
        let info = self.adapter.get_info();
        let features = self.adapter.features();
        let limits = self.adapter.limits();
        let yes_or_no = |feature| {
            if features.contains(feature) {
                "yes"
            } else {
                "no"
            }
        };

        // Each planned feature: its label, the wgpu feature that decides yes
        // or no, and the related features and limits.
        let planned_features = [
            (
                "binding arrays",
                wgpu::Features::TEXTURE_BINDING_ARRAY,
                format!(
                    "non-uniform indexing {}, partially bound {}, max {} per stage",
                    yes_or_no(
                        wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                    ),
                    yes_or_no(wgpu::Features::PARTIALLY_BOUND_BINDING_ARRAY),
                    limits.max_binding_array_elements_per_shader_stage,
                ),
            ),
            (
                "mesh shaders",
                wgpu::Features::EXPERIMENTAL_MESH_SHADER,
                format!(
                    "max {} output vertices, {} output primitives",
                    limits.max_mesh_output_vertices, limits.max_mesh_output_primitives,
                ),
            ),
            (
                "ray query",
                wgpu::Features::EXPERIMENTAL_RAY_QUERY,
                format!("max {} TLAS instances", limits.max_tlas_instance_count),
            ),
            (
                "timestamp queries",
                wgpu::Features::TIMESTAMP_QUERY,
                format!(
                    "inside encoders {}, inside passes {}",
                    yes_or_no(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS),
                    yes_or_no(wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES),
                ),
            ),
            (
                "BC compression",
                wgpu::Features::TEXTURE_COMPRESSION_BC,
                String::new(),
            ),
            (
                "subgroups",
                wgpu::Features::SUBGROUP,
                format!(
                    "barrier {}, size {} to {}",
                    yes_or_no(wgpu::Features::SUBGROUP_BARRIER),
                    info.subgroup_min_size,
                    info.subgroup_max_size,
                ),
            ),
            (
                "immediates",
                wgpu::Features::IMMEDIATES,
                format!("max {} bytes", limits.max_immediate_size),
            ),
        ];

        let mut report = format!(
            "{:<18} {} ({:?})\n{:<18} {}\n",
            "adapter", info.name, info.device_type, "backend", info.backend
        );
        for (label, feature, details) in planned_features {
            let line = format!("{label:<18} {:<3}  {details}", yes_or_no(feature));
            report.push_str(line.trim_end());
            report.push('\n');
        }
        report
    }
}

fn create_wgpu_instance() -> wgpu::Instance {
    wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: None,
    })
}
