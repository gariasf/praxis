use std::fmt;

/// The adapter, device and queue that every GPU path shares. Created for a
/// window's surface, or headless for offscreen work.
pub struct GpuContext {
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
}

impl GpuContext {
    /// A context with no window or surface.
    pub async fn headless() -> anyhow::Result<Self> {
        let adapter = request_adapter(&create_wgpu_instance(), None).await?;
        Self::request_device(adapter, None).await
    }

    /// A context whose adapter can present to `target`, the surface for it,
    /// and the surface format the main pass renders into.
    pub(crate) async fn with_surface(
        target: impl Into<wgpu::SurfaceTarget<'static>>,
    ) -> anyhow::Result<(Self, wgpu::Surface<'static>, wgpu::TextureFormat)> {
        let wgpu_instance = create_wgpu_instance();
        let surface = wgpu_instance.create_surface(target)?;
        let adapter = request_adapter(&wgpu_instance, Some(&surface)).await?;
        let surface_format = choose_surface_format(&surface.get_capabilities(&adapter))?;
        let context = Self::request_device(adapter, Some(surface_format)).await?;
        Ok((context, surface, surface_format))
    }

    /// Requests a device with exactly the Capability floor, after checking
    /// the adapter against all of it, `surface_format` included.
    async fn request_device(
        adapter: wgpu::Adapter,
        surface_format: Option<wgpu::TextureFormat>,
    ) -> anyhow::Result<Self> {
        let attachment_formats: Vec<_> = MAIN_PASS_FORMATS
            .into_iter()
            .chain(surface_format)
            .map(|format| (format, adapter.get_texture_format_features(format).flags))
            .collect();
        check_floor(
            adapter.features(),
            adapter.get_downlevel_capabilities().flags,
            &adapter.limits(),
            &attachment_formats,
        )?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: floor_features(),
                // SAFETY: ray queries are an experimental wgpu feature, and
                // enabling them accepts that a wgpu bug may cause undefined
                // behaviour from safe calls. ADR 0020's ray-traced shadows
                // and probes need them, so the floor (ADR 0021) takes that
                // risk.
                experimental_features: unsafe { wgpu::ExperimentalFeatures::enabled() },
                required_limits: floor_limits(),
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

    /// Whether the adapter is a discrete, integrated, or software (CPU) GPU.
    pub fn device_type(&self) -> wgpu::DeviceType {
        self.adapter.get_info().device_type
    }
}

/// The capability report of the adapter a headless context would use. It
/// requests no device, so it also runs on adapters below the Capability
/// floor, to diagnose them.
pub async fn capability_report() -> anyhow::Result<String> {
    let adapter = request_adapter(&create_wgpu_instance(), None).await?;
    Ok(adapter_report(&adapter))
}

/// The adapter, its backend, and whether the adapter supports each
/// feature the roadmap plans on, with the related features and limits.
/// One line per entry, followed by the Capability floor's limits, 4x MSAA on
/// the main pass's attachment formats, resolved on colour ones, and
/// anisotropic filtering.
fn adapter_report(adapter: &wgpu::Adapter) -> String {
    let info = adapter.get_info();
    let features = adapter.features();
    let limits = adapter.limits();
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
            "indirect first instance",
            wgpu::Features::INDIRECT_FIRST_INSTANCE,
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
        "{:<24} {} ({:?})\n{:<24} {}\n",
        "adapter", info.name, info.device_type, "backend", info.backend
    );
    for (label, feature, details) in planned_features {
        let line = format!("{label:<24} {:<3}  {details}", yes_or_no(feature));
        report.push_str(line.trim_end());
        report.push('\n');
    }
    let msaa_x4: Vec<_> = MAIN_PASS_FORMATS
        .into_iter()
        .map(|format| {
            let flags = adapter.get_texture_format_features(format).flags;
            let missing: Vec<_> = (floor_format_features(format) - flags)
                .iter_names()
                .map(|(name, _)| name)
                .collect();
            if missing.is_empty() {
                format!("{format:?} yes")
            } else {
                format!("{format:?} no ({})", missing.join(", "))
            }
        })
        .collect();
    report.push_str(&format!(
        "{:<24} {} bytes\n{:<24} max {} per stage\n{:<24} {}\n{:<24} {}\n",
        "max buffer size",
        limits.max_buffer_size,
        "acceleration structures",
        limits.max_acceleration_structures_per_shader_stage,
        "4x MSAA",
        msaa_x4.join(", "),
        "anisotropic filtering",
        if adapter
            .get_downlevel_capabilities()
            .flags
            .contains(wgpu::DownlevelFlags::ANISOTROPIC_FILTERING)
        {
            "yes"
        } else {
            "no"
        },
    ));
    report
}

/// The attachment formats the main pass uses apart from the window's
/// surface format: depth, and the capture target.
const MAIN_PASS_FORMATS: [wgpu::TextureFormat; 2] =
    [crate::render::DEPTH_FORMAT, crate::capture::TARGET_FORMAT];

/// The Capability floor's features (ADR 0021): exactly what every device
/// requests.
pub(crate) fn floor_features() -> wgpu::Features {
    // ADR 0017: a Layer index varies per Instance.
    wgpu::Features::TEXTURE_BINDING_ARRAY
        | wgpu::Features::PARTIALLY_BOUND_BINDING_ARRAY
        | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
        // ADR 0018 cooks BC only.
        | wgpu::Features::TEXTURE_COMPRESSION_BC
        // ADRs 0019 and 0020.
        | wgpu::Features::EXPERIMENTAL_RAY_QUERY
        // ADR 0013's Budgets are GPU milliseconds.
        | wgpu::Features::TIMESTAMP_QUERY
}

/// The Capability floor's downlevel flags (ADR 0021), which an adapter has
/// or lacks with no request.
pub(crate) fn floor_downlevel_flags() -> wgpu::DownlevelFlags {
    // ADR 0017 keeps hardware anisotropic filtering, and Material Channels
    // sample at 16x (#144); without the flag, wgpu silently clamps every
    // sampler's anisotropy to 1.
    wgpu::DownlevelFlags::ANISOTROPIC_FILTERING
        // The vertex shader reads Instances and pulls vertices from the
        // vertex Arena as storage buffers (ADRs 0005, 0019).
        | wgpu::DownlevelFlags::VERTEX_STORAGE
        // A compute pass builds the frame's draw, which the main pass draws
        // with one indirect call (ADR 0019).
        | wgpu::DownlevelFlags::COMPUTE_SHADERS
        | wgpu::DownlevelFlags::INDIRECT_EXECUTION
}

/// The binding-array elements per shader stage the Capability floor
/// guarantees (ADR 0021), which the Material Channels share.
pub(crate) const FLOOR_BINDING_ARRAY_ELEMENTS_PER_STAGE: u32 = 65_536;

/// The Capability floor's limits (ADR 0021): the defaults, raised only where
/// an ADR needs it.
pub(crate) fn floor_limits() -> wgpu::Limits {
    wgpu::Limits {
        // ADR 0017's arenas.
        max_buffer_size: (1 << 31) - 1,
        max_binding_array_elements_per_shader_stage: FLOOR_BINDING_ARRAY_ELEMENTS_PER_STAGE,
        // ADR 0020's static and dynamic TLAS.
        max_acceleration_structures_per_shader_stage: 2,
        ..wgpu::Limits::default().using_minimum_supported_acceleration_structure_values()
    }
}

/// Checks an adapter's `features`, `downlevel_flags`, `limits`, and the flags
/// of each main-pass attachment format against the Capability floor, and
/// names every gap.
fn check_floor(
    features: wgpu::Features,
    downlevel_flags: wgpu::DownlevelFlags,
    limits: &wgpu::Limits,
    attachment_formats: &[(wgpu::TextureFormat, wgpu::TextureFormatFeatureFlags)],
) -> Result<(), BelowCapabilityFloor> {
    let mut limit_gaps = Vec::new();
    floor_limits().check_limits_with_fail_fn(limits, false, |name, required, adapter| {
        limit_gaps.push(LimitGap {
            name,
            required,
            adapter,
        });
    });
    let gaps = BelowCapabilityFloor {
        missing_features: floor_features() - features,
        missing_downlevel_flags: floor_downlevel_flags() - downlevel_flags,
        missing_format_features: attachment_formats
            .iter()
            .map(|&(format, flags)| (format, floor_format_features(format) - flags))
            .filter(|(_, missing)| !missing.is_empty())
            .collect(),
        limits: limit_gaps,
    };
    if gaps.missing_features.is_empty()
        && gaps.missing_downlevel_flags.is_empty()
        && gaps.missing_format_features.is_empty()
        && gaps.limits.is_empty()
    {
        Ok(())
    } else {
        Err(gaps)
    }
}

/// The format features the floor requires of a main-pass attachment format:
/// 4x MSAA, and on a colour format its resolve, since the lit pass resolves
/// its multisampled colour into the target (ADR 0019).
fn floor_format_features(format: wgpu::TextureFormat) -> wgpu::TextureFormatFeatureFlags {
    let msaa_x4 = wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X4;
    if format.is_depth_stencil_format() {
        msaa_x4
    } else {
        msaa_x4 | wgpu::TextureFormatFeatureFlags::MULTISAMPLE_RESOLVE
    }
}

/// Every way an adapter falls short of the Capability floor, so one run
/// names them all.
#[derive(Debug, PartialEq)]
pub struct BelowCapabilityFloor {
    pub missing_features: wgpu::Features,
    pub missing_downlevel_flags: wgpu::DownlevelFlags,
    /// Each main-pass attachment format with the floor's format features it
    /// lacks.
    pub missing_format_features: Vec<(wgpu::TextureFormat, wgpu::TextureFormatFeatureFlags)>,
    pub limits: Vec<LimitGap>,
}

/// A limit the adapter supports worse than the Capability floor requires.
#[derive(Debug, PartialEq)]
pub struct LimitGap {
    pub name: &'static str,
    pub required: u64,
    pub adapter: u64,
}

impl fmt::Display for BelowCapabilityFloor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut gaps: Vec<String> = self
            .missing_features
            .iter_names()
            .map(|(name, _)| format!("missing feature {name}"))
            .collect();
        gaps.extend(
            self.missing_downlevel_flags
                .iter_names()
                .map(|(name, _)| format!("missing downlevel flag {name}")),
        );
        gaps.extend(
            self.missing_format_features
                .iter()
                .flat_map(|(format, missing)| {
                    missing.iter_names().map(move |(name, _)| {
                        format!("missing format feature {name} on {format:?}")
                    })
                }),
        );
        gaps.extend(self.limits.iter().map(|gap| {
            format!(
                "{} {} required, adapter has {}",
                gap.name, gap.required, gap.adapter
            )
        }));
        write!(
            formatter,
            "adapter below the Capability floor (ADR 0021): {}",
            gaps.join("; ")
        )
    }
}

impl std::error::Error for BelowCapabilityFloor {}

async fn request_adapter(
    wgpu_instance: &wgpu::Instance,
    compatible_surface: Option<&wgpu::Surface<'_>>,
) -> anyhow::Result<wgpu::Adapter> {
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
    Ok(adapter)
}

/// The first sRGB format the surface offers, else its first format: the
/// main pass writes linear colour and relies on an sRGB target to encode it,
/// so a non-sRGB surface comes out darker.
fn choose_surface_format(
    surface_capabilities: &wgpu::SurfaceCapabilities,
) -> anyhow::Result<wgpu::TextureFormat> {
    let formats = &surface_capabilities.formats;
    let surface_format = formats
        .iter()
        .find(|format| format.is_srgb())
        .or_else(|| formats.first())
        .copied()
        .ok_or_else(|| anyhow::anyhow!("the surface offers no formats on the selected adapter"))?;
    tracing::debug!(
        ?surface_format,
        srgb = surface_format.is_srgb(),
        "surface format chosen"
    );
    Ok(surface_format)
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

#[cfg(test)]
mod tests {
    use super::*;

    const MSAA_X4: wgpu::TextureFormatFeatureFlags =
        wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X2
            .union(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X4)
            .union(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_RESOLVE);

    /// The Apple M4 Pro's features on wgpu 30.0.1: the floor's, plus the
    /// optional ones the capability report prints.
    fn m4_pro_features() -> wgpu::Features {
        floor_features()
            | wgpu::Features::EXPERIMENTAL_MESH_SHADER
            | wgpu::Features::SUBGROUP
            | wgpu::Features::SUBGROUP_BARRIER
            | wgpu::Features::IMMEDIATES
    }

    /// Metal reports anisotropic filtering, vertex-stage storage buffers and
    /// compute shaders on every adapter, and indirect execution on the M4
    /// Pro's GPU family (wgpu-hal 30.0.1).
    const M4_PRO_DOWNLEVEL_FLAGS: wgpu::DownlevelFlags =
        wgpu::DownlevelFlags::ANISOTROPIC_FILTERING
            .union(wgpu::DownlevelFlags::VERTEX_STORAGE)
            .union(wgpu::DownlevelFlags::COMPUTE_SHADERS)
            .union(wgpu::DownlevelFlags::INDIRECT_EXECUTION);

    /// The Apple M4 Pro's limits on wgpu 30.0.1.
    fn m4_pro_limits() -> wgpu::Limits {
        wgpu::Limits {
            max_texture_dimension_1d: 16384,
            max_texture_dimension_2d: 16384,
            max_texture_dimension_3d: 2048,
            max_texture_array_layers: 2048,
            max_bind_groups: 8,
            max_bind_groups_plus_vertex_buffers: 24,
            max_bindings_per_bind_group: 4294967295,
            max_dynamic_uniform_buffers_per_pipeline_layout: 29,
            max_dynamic_storage_buffers_per_pipeline_layout: 29,
            max_sampled_textures_per_shader_stage: 96,
            max_samplers_per_shader_stage: 16,
            max_storage_buffers_per_shader_stage: 29,
            max_storage_textures_per_shader_stage: 32,
            max_uniform_buffers_per_shader_stage: 29,
            max_binding_array_elements_per_shader_stage: 1000000,
            max_binding_array_acceleration_structure_elements_per_shader_stage: 0,
            max_binding_array_sampler_elements_per_shader_stage: 500000,
            max_uniform_buffer_binding_size: 4294967295,
            max_storage_buffer_binding_size: 4294967292,
            max_vertex_buffers: 16,
            max_buffer_size: 4294967295,
            max_vertex_attributes: 31,
            max_vertex_buffer_array_stride: 2048,
            max_inter_stage_shader_variables: 31,
            min_uniform_buffer_offset_alignment: 256,
            min_storage_buffer_offset_alignment: 32,
            max_color_attachments: 8,
            max_color_attachment_bytes_per_sample: 64,
            max_compute_workgroup_storage_size: 32768,
            max_compute_invocations_per_workgroup: 1024,
            max_compute_workgroup_size_x: 1024,
            max_compute_workgroup_size_y: 1024,
            max_compute_workgroup_size_z: 1024,
            max_compute_workgroups_per_dimension: 65535,
            max_immediate_size: 256,
            max_non_sampler_bindings: 4294967295,
            max_task_workgroup_total_count: 4294967295,
            max_task_workgroups_per_dimension: 4294967295,
            max_mesh_workgroup_total_count: 1048576,
            max_mesh_workgroups_per_dimension: 1048576,
            max_task_invocations_per_workgroup: 1024,
            max_task_invocations_per_dimension: 1024,
            max_mesh_invocations_per_workgroup: 1024,
            max_mesh_invocations_per_dimension: 1024,
            max_task_payload_size: 16352,
            max_mesh_output_vertices: 256,
            max_mesh_output_primitives: 256,
            max_mesh_output_layers: 2048,
            max_mesh_multiview_view_count: 0,
            max_blas_primitive_count: 268435456,
            max_blas_geometry_count: 16777216,
            max_tlas_instance_count: 16777216,
            max_acceleration_structures_per_shader_stage: 29,
            max_buffers_and_acceleration_structures_per_shader_stage: 29,
            max_multiview_view_count: 8,
            max_ray_dispatch_count: 0,
            max_ray_recursion_depth: 0,
        }
    }

    /// The window's main-pass formats on the M4 Pro, each with 4x MSAA.
    fn m4_pro_attachment_formats() -> [(wgpu::TextureFormat, wgpu::TextureFormatFeatureFlags); 3] {
        [
            (wgpu::TextureFormat::Depth32Float, MSAA_X4),
            (wgpu::TextureFormat::Rgba8UnormSrgb, MSAA_X4),
            (wgpu::TextureFormat::Bgra8UnormSrgb, MSAA_X4),
        ]
    }

    #[test]
    fn the_m4_pro_meets_the_floor() {
        assert_eq!(
            check_floor(
                m4_pro_features(),
                M4_PRO_DOWNLEVEL_FLAGS,
                &m4_pro_limits(),
                &m4_pro_attachment_formats()
            ),
            Ok(())
        );
    }

    #[test]
    fn every_gap_is_named_in_one_error() {
        let limits = wgpu::Limits {
            max_buffer_size: 1 << 28,
            ..m4_pro_limits()
        };
        let mut formats = m4_pro_attachment_formats();
        formats[0].1 = wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X2;
        formats[1].1 = MSAA_X4 - wgpu::TextureFormatFeatureFlags::MULTISAMPLE_RESOLVE;

        let error = check_floor(
            m4_pro_features() - wgpu::Features::TIMESTAMP_QUERY,
            wgpu::DownlevelFlags::empty(),
            &limits,
            &formats,
        )
        .unwrap_err();

        assert_eq!(
            error,
            BelowCapabilityFloor {
                missing_features: wgpu::Features::TIMESTAMP_QUERY,
                missing_downlevel_flags: wgpu::DownlevelFlags::COMPUTE_SHADERS
                    | wgpu::DownlevelFlags::INDIRECT_EXECUTION
                    | wgpu::DownlevelFlags::VERTEX_STORAGE
                    | wgpu::DownlevelFlags::ANISOTROPIC_FILTERING,
                missing_format_features: vec![
                    (
                        wgpu::TextureFormat::Depth32Float,
                        wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X4
                    ),
                    (
                        wgpu::TextureFormat::Rgba8UnormSrgb,
                        wgpu::TextureFormatFeatureFlags::MULTISAMPLE_RESOLVE
                    ),
                ],
                limits: vec![LimitGap {
                    name: "max_buffer_size",
                    required: 2_147_483_647,
                    adapter: 268_435_456,
                }],
            }
        );
        assert_eq!(
            error.to_string(),
            "adapter below the Capability floor (ADR 0021): \
             missing feature TIMESTAMP_QUERY; missing downlevel flag COMPUTE_SHADERS; \
             missing downlevel flag INDIRECT_EXECUTION; missing downlevel flag VERTEX_STORAGE; \
             missing downlevel flag ANISOTROPIC_FILTERING; \
             missing format feature MULTISAMPLE_X4 on Depth32Float; \
             missing format feature MULTISAMPLE_RESOLVE on Rgba8UnormSrgb; \
             max_buffer_size 2147483647 required, adapter has 268435456"
        );
    }

    /// Every Texture a Blob holds is one the floor's devices can create.
    #[test]
    fn the_largest_cooked_texture_fits_the_floor() {
        assert!(crate::cooked::MAX_TEXTURE_SIDE <= floor_limits().max_texture_dimension_2d);
    }

    /// The vertex shader reads the whole vertex Arena through one storage
    /// binding (ADR 0019), which the floor holds to the default 128 MiB:
    /// Vulkan's minimum and lavapipe's `maxStorageBufferRange`.
    #[test]
    fn the_vertex_arena_fits_one_storage_binding_of_the_floor() {
        assert!(
            crate::assets::ARENA_CAPACITY_BYTES <= floor_limits().max_storage_buffer_binding_size
        );
    }

    #[test]
    fn limits_exactly_at_the_floor_pass() {
        let limits = wgpu::Limits {
            max_buffer_size: 2_147_483_647,
            max_binding_array_elements_per_shader_stage: 65_536,
            max_acceleration_structures_per_shader_stage: 2,
            ..wgpu::Limits::default().using_minimum_supported_acceleration_structure_values()
        };
        assert_eq!(
            check_floor(
                floor_features(),
                floor_downlevel_flags(),
                &limits,
                &m4_pro_attachment_formats()
            ),
            Ok(())
        );
    }
}
