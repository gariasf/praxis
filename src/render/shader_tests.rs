//! Every shader module is checked here without a GPU, as wgpu checks it when
//! it creates a shader module (backend code generation aside), every WGSL
//! file under `src/` is a part of some module, each binding is visible to
//! exactly the stages that use it, and each struct whose bytes the CPU
//! writes is compared with its Rust twin.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use wgpu::naga;

use crate::assets::material::MaterialData;
use crate::assets::{MeshData, MeshletData};
use crate::cooked::CookedVertex;
use crate::gpu::{floor_downlevel_flags, floor_features, floor_limits};
use crate::render::bend_pass::BendData;
use crate::render::bind_groups::{
    BindGroupDefinition, Binding, Writer, read_only_storage, uniform,
};
use crate::render::shaders::{MODULES, Part, ShaderModule};
use crate::render::{CameraUniform, InstanceData, LightData, LightUniform};

#[test]
fn every_module_parses_and_validates() {
    let errors: Vec<String> = MODULES
        .iter()
        .filter_map(|module| validate(module).err())
        .collect();
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

/// wgpu checks only that a stage using a binding can see it, and only on a
/// GPU; this also fails a binding visible to a stage that never uses it.
#[test]
fn every_binding_is_visible_to_exactly_the_stages_that_use_it() {
    let problems = stage_problems(&MODULES);
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// At submit, wgpu-core 30.0.1 walks a TLAS's dependencies, one BLAS per
/// Instance, for each pass that binds a group holding it, so only the pass
/// that traces rays and the markers that time the dynamic TLAS's build
/// declare one.
#[test]
fn only_the_shadow_mask_pass_and_the_dynamic_tlas_markers_declare_a_tlas() {
    let declaring: Vec<&str> = MODULES
        .iter()
        .filter(|module| {
            module
                .bind_groups
                .iter()
                .flat_map(|group| group.bindings)
                .any(|binding| {
                    matches!(
                        binding.binding_type,
                        wgpu::BindingType::AccelerationStructure { .. }
                    )
                })
        })
        .map(|module| module.name)
        .collect();
    assert_eq!(declaring, ["shadow_mask", "dynamic_tlas_marker"]);
}

/// A group's layout serves every module that declares it, so a binding is
/// judged by the stages that use it in any of them.
#[test]
fn a_binding_seen_by_a_stage_that_does_not_use_it_or_hidden_from_one_that_does_fails() {
    const VALUES: BindGroupDefinition = BindGroupDefinition {
        index: 0,
        label: "Values",
        bindings: &[
            uniform("exact", "vec4<f32>", wgpu::ShaderStages::VERTEX),
            uniform("both", "vec4<f32>", wgpu::ShaderStages::VERTEX_FRAGMENT),
            uniform("seen", "vec4<f32>", wgpu::ShaderStages::VERTEX_FRAGMENT),
            uniform("hidden", "vec4<f32>", wgpu::ShaderStages::VERTEX),
        ],
    };
    const VERTEX: Part = Part {
        path: "vertex.wgsl",
        source: "
            @group(0) @binding(4) var<uniform> stray: vec4<f32>;

            @vertex
            fn vertex_main() -> @builtin(position) vec4<f32> {
                return exact + both + seen + hidden + stray;
            }
        ",
    };
    const FRAGMENT: Part = Part {
        path: "fragment.wgsl",
        source: "
            @fragment
            fn fragment_main() -> @location(0) vec4<f32> {
                return both + hidden;
            }
        ",
    };
    const PASSES: [ShaderModule; 2] = [
        ShaderModule {
            name: "vertex",
            enables: &[],
            bind_groups: &[&VALUES],
            parts: &[VERTEX],
        },
        ShaderModule {
            name: "fragment",
            enables: &[],
            bind_groups: &[&VALUES],
            parts: &[FRAGMENT],
        },
    ];
    assert_eq!(
        stage_problems(&PASSES),
        [
            "vertex module: stray: declared in a part, not by its bind group's definition",
            "Values: hidden: visible to ShaderStages(VERTEX), used by ShaderStages(VERTEX | FRAGMENT)",
            "Values: seen: visible to ShaderStages(VERTEX | FRAGMENT), used by ShaderStages(VERTEX)",
        ]
    );
}

#[test]
fn every_wgsl_file_is_a_part_of_some_module() {
    let parts: BTreeSet<&Path> = MODULES
        .iter()
        .flat_map(|module| module.parts)
        .map(|part| Path::new(part.path))
        .collect();
    let unused: Vec<String> = wgsl_files()
        .iter()
        .filter(|file| !parts.contains(file.as_path()))
        .map(|file| format!("{}: no shader module uses it", file.display()))
        .collect();
    assert!(unused.is_empty(), "{}", unused.join("\n"));
}

/// The floor requires ray queries (ADRs 0020, 0021), so a module that
/// traces them validates.
#[test]
fn a_module_with_ray_queries_validates() {
    const SHADOW: Part = Part {
        path: "shadow.wgsl",
        source: "
            @group(0) @binding(0) var scene: acceleration_structure;

            fn occluded(origin: vec3<f32>, direction: vec3<f32>) -> bool {
                var query: ray_query;
                rayQueryInitialize(
                    &query,
                    scene,
                    RayDesc(RAY_FLAG_TERMINATE_ON_FIRST_HIT, 0xFFu, 0.001, 100.0, origin, direction)
                );
                rayQueryProceed(&query);
                return rayQueryGetCommittedIntersection(&query).kind != RAY_QUERY_INTERSECTION_NONE;
            }
        ",
    };
    let module = ShaderModule {
        name: "shadow",
        enables: &["wgpu_ray_query"],
        bind_groups: &[],
        parts: &[SHADOW],
    };
    assert_eq!(validate(&module).err(), None);
}

/// A parse error, and a validation error found after parsing, each name
/// their module, their part and their line within it, past the module's
/// `enable` directives and earlier parts.
#[test]
fn an_error_names_its_module_part_and_line() {
    const LIGHT: Part = Part {
        path: "light.wgsl",
        source: "struct Light {\n    color: vec4<f32>,\n}\n",
    };
    const MISSPELT: Part = Part {
        path: "intensity.wgsl",
        source: "// A misspelt member, line 4.\n\nfn intensity(light: Light) -> f32 {\n    return light.colour.a;\n}\n",
    };
    const UNALIGNED_UNIFORM: Part = Part {
        path: "counts.wgsl",
        source: "// A uniform array with a 4-byte stride, line 3.\n\n\
                 @group(0) @binding(0) var<uniform> counts: array<u32>;\n",
    };
    let parse_error = validate(&ShaderModule {
        name: "test",
        enables: &["wgpu_binding_array"],
        bind_groups: &[],
        parts: &[LIGHT, MISSPELT],
    })
    .unwrap_err();
    let validation_error = validate(&ShaderModule {
        name: "test",
        enables: &["wgpu_binding_array"],
        bind_groups: &[],
        parts: &[LIGHT, UNALIGNED_UNIFORM],
    })
    .unwrap_err();
    assert!(
        parse_error.starts_with("test module: intensity.wgsl:4:"),
        "{parse_error}"
    );
    assert!(
        validation_error.starts_with("test module: counts.wgsl:3:"),
        "{validation_error}"
    );
}

/// The `Twin` of `$type`, matched to the WGSL struct of the same name. The
/// field list builds a `$type` literal, so a field added to `$type` fails to
/// compile until it is listed.
macro_rules! twin {
    ($type:ident { $($field:ident),+ $(,)? }) => {{
        let value = $type {
            $($field: bytemuck::Zeroable::zeroed()),+
        };
        Twin {
            wgsl_name: stringify!($type),
            size: size_of::<$type>(),
            fields: vec![$(Field {
                name: stringify!($field),
                offset: std::mem::offset_of!($type, $field),
                size: size_of_val(&value.$field),
                scalar: scalar_of(&value.$field),
            }),+],
        }
    }};
}

#[test]
fn structs_shared_with_the_cpu_match_their_rust_twins() {
    let modules: Vec<(ShaderModule, naga::Module)> = MODULES
        .iter()
        .map(|module| {
            let (parsed, _) = validate(module).unwrap_or_else(|error| panic!("{error}"));
            (*module, parsed)
        })
        .collect();
    let twins = [
        twin!(CameraUniform {
            view_proj,
            anchor,
            exposure,
            offset,
            _pad_offset,
            tlas_anchor,
            dynamic_tlas_entries,
        }),
        twin!(LightUniform {
            ambient_illuminance,
            directional_light_count,
            point_light_count,
            _pad,
        }),
        twin!(LightData {
            anchor,
            range,
            offset,
            _pad_offset,
            intensity,
            _pad_intensity,
        }),
        twin!(InstanceData {
            rotation_scale,
            normal_matrix,
            anchor,
            material_id,
            offset,
            mesh,
        }),
        twin!(MeshData {
            first_meshlet,
            meshlet_count,
            first_triangle,
            triangle_count,
            bounds_min,
            bounds_max,
        }),
        twin!(MeshletData {
            first_vertex,
            first_triangle,
            triangle_count,
            material_id,
        }),
        twin!(MaterialData {
            base_color,
            metallic_roughness,
            emissive,
            layers,
            extra,
        }),
        twin!(CookedVertex {
            position,
            uv,
            qtangent
        }),
        twin!(BendData {
            rest_first_vertex,
            first_vertex,
            vertex_count,
            azimuth,
            curvature,
        }),
    ];
    let problems = layout_problems(&modules, &twins);
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn layout_check_reports_every_kind_of_mismatch() {
    const SOURCE: &str = "
        struct Light { color: vec3<f32>, count: u32, direction: vec4<f32> }
        struct Material { base_color: vec4<f32>, roughness: f32 }
        struct Orphan { value: f32 }
        @group(0) @binding(0) var<uniform> light: Light;
        @group(0) @binding(1) var<storage, read> materials: array<Material>;
        @group(0) @binding(2) var<storage, read> orphans: array<Orphan>;
    ";
    #[repr(C)]
    struct Light {
        color: [f32; 3],
        count: f32,
        direction: [f32; 4],
    }
    #[repr(C)]
    struct Material {
        base_color: [f32; 4],
        roughness: f32,
    }
    let twins = [
        twin!(Light {
            color,
            count,
            direction
        }),
        twin!(Material {
            base_color,
            roughness
        }),
        Twin {
            wgsl_name: "Missing",
            ..twin!(Material {
                base_color,
                roughness
            })
        },
    ];
    const PART: Part = Part {
        path: "test.wgsl",
        source: SOURCE,
    };
    let module = ShaderModule {
        name: "test",
        enables: &[],
        bind_groups: &[],
        parts: &[PART],
    };
    let (parsed, _) = validate(&module).unwrap_or_else(|error| panic!("{error}"));

    let mut problems = layout_problems(&[(module, parsed)], &twins);
    problems.sort();
    assert_eq!(
        problems,
        [
            "Missing: no shader module shares it",
            "test module: Light: Rust `count` at offset 12: 4 bytes of f32",
            "test module: Light: WGSL `count` at offset 12: 4 bytes of u32",
            "test module: Material: Rust size: 20 bytes",
            "test module: Material: WGSL size: 32 bytes",
            "test module: Orphan: shared with the CPU but has no twin",
        ]
    );
}

/// The twin check covers what the CPU writes, so a struct only the GPU
/// writes needs no Rust twin, while one the CPU writes still does.
#[test]
fn a_struct_only_the_gpu_writes_needs_no_twin() {
    const COUNTS: BindGroupDefinition = BindGroupDefinition {
        index: 0,
        label: "Counts",
        bindings: &[
            read_only_storage("requests", "array<Request>", wgpu::ShaderStages::COMPUTE),
            Binding {
                writer: Writer::Gpu,
                ..read_only_storage("counts", "array<Count>", wgpu::ShaderStages::COMPUTE)
            },
        ],
    };
    const PART: Part = Part {
        path: "test.wgsl",
        source: "
            struct Request { value: f32 }
            struct Count { drawn: u32 }

            @compute @workgroup_size(1)
            fn count() {
                let total = requests[0].value + f32(counts[0].drawn);
            }
        ",
    };
    let module = ShaderModule {
        name: "test",
        enables: &[],
        bind_groups: &[&COUNTS],
        parts: &[PART],
    };
    let (parsed, _) = validate(&module).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        layout_problems(&[(module, parsed)], &[]),
        ["test module: Request: shared with the CPU but has no twin"]
    );
}

#[test]
fn a_bind_group_past_the_device_limit_fails_validation() {
    const PART: Part = Part {
        path: "test.wgsl",
        source: "\n@group(4) @binding(0) var<uniform> value: vec4<f32>;\n",
    };
    assert_eq!(
        validate(&ShaderModule {
            name: "test",
            enables: &[],
            bind_groups: &[],
            parts: &[PART],
        })
        .err()
        .as_deref(),
        Some("test module: test.wgsl:2:23: @group(4) exceeds the device's max_bind_groups of 4")
    );
}

/// Parses and validates `module` as wgpu does on a device that requests
/// exactly the Capability floor, as `GpuContext` does, so a shader that needs
/// more fails until the floor gains it (ADR 0021), and returns it with
/// naga's analysis of it. An error names the module, and the part and line
/// it is in.
fn validate(module: &ShaderModule) -> Result<(naga::Module, naga::valid::ModuleInfo), String> {
    let source = module.source();
    let label = format!("{} module", module.name);
    let error = |location: Option<naga::SourceLocation>, message: &str| match location
        .and_then(|location| module.locate(location.offset as usize))
    {
        Some((path, line, column)) => format!("{label}: {path}:{line}:{column}: {message}"),
        None => format!("{label}: {message}"),
    };
    let capabilities =
        wgpu_naga_bridge::features_to_naga_capabilities(floor_features(), floor_downlevel_flags());
    let mut options = naga::front::wgsl::Options::new();
    options.capabilities = capabilities;
    let parsed = naga::front::wgsl::Frontend::new_with_options(options)
        .parse(&source)
        .map_err(|parse_error| error(parse_error.location(&source), parse_error.message()))?;
    let max_bind_groups = floor_limits().max_bind_groups;
    for (handle, global) in parsed.global_variables.iter() {
        if let Some(binding) = &global.binding
            && binding.group >= max_bind_groups
        {
            let span = parsed.global_variables.get_span(handle);
            return Err(error(
                Some(span.location(&source)),
                &format!(
                    "@group({}) exceeds the device's max_bind_groups of {max_bind_groups}",
                    binding.group
                ),
            ));
        }
    }
    let info = naga::valid::Validator::new(naga::valid::ValidationFlags::all(), capabilities)
        .validate(&parsed)
        .map_err(|validation_error| {
            // The outermost error names the item; its sources say why.
            let causes: Vec<String> = std::iter::successors(
                Some(validation_error.as_inner() as &dyn std::error::Error),
                |cause| cause.source(),
            )
            .map(ToString::to_string)
            .collect();
            error(validation_error.location(&source), &causes.join(": "))
        })?;
    Ok((parsed, info))
}

/// Each binding a part of `modules` declares rather than its bind group's
/// definition, as `module: binding: problem`; then each binding visible to
/// a stage whose entry points, in every module that declares it, never use
/// it, or used by a stage it is not visible to, per naga's analysis, as
/// `group: binding: problem`.
fn stage_problems(modules: &[ShaderModule]) -> Vec<String> {
    let mut problems = Vec::new();
    // Each defined binding's stages, and the stages that use it.
    let mut stages = BTreeMap::new();
    for module in modules {
        let (parsed, info) = validate(module).unwrap_or_else(|error| panic!("{error}"));
        for (handle, global) in parsed.global_variables.iter() {
            let Some(resource) = &global.binding else {
                continue;
            };
            let name = global.name.as_deref().unwrap_or_default();
            let Some((group, binding)) =
                defined_binding(module, resource).filter(|(_, binding)| binding.name == name)
            else {
                problems.push(format!(
                    "{} module: {name}: declared in a part, not by its bind group's definition",
                    module.name
                ));
                continue;
            };
            let used_by = (0..)
                .zip(&parsed.entry_points)
                .filter(|&(index, _)| !info.get_entry_point(index)[handle].is_empty())
                .fold(wgpu::ShaderStages::NONE, |stages, (_, entry_point)| {
                    stages | wgpu_naga_bridge::map_naga_stage(entry_point.stage)
                });
            stages
                .entry((group.label, binding.name))
                .or_insert((binding.stages, wgpu::ShaderStages::NONE))
                .1 |= used_by;
        }
    }
    problems.extend(
        stages
            .into_iter()
            .filter(|(_, (visible_to, used_by))| visible_to != used_by)
            .map(|((group, name), (visible_to, used_by))| {
                format!("{group}: {name}: visible to {visible_to:?}, used by {used_by:?}")
            }),
    );
    problems
}

/// The bind group and the definition of the binding `resource` names, among
/// `module`'s bind groups.
fn defined_binding(
    module: &ShaderModule,
    resource: &naga::ResourceBinding,
) -> Option<(&'static BindGroupDefinition, &'static Binding)> {
    let group = module
        .bind_groups
        .iter()
        .copied()
        .find(|bind_group| bind_group.index == resource.group)?;
    Some((group, group.bindings.get(resource.binding as usize)?))
}

/// Every `.wgsl` file under `src/`, by its path from the crate root.
fn wgsl_files() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    let mut directories = vec![root.join("src")];
    while let Some(directory) = directories.pop() {
        let entries = std::fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()));
        for entry in entries {
            let path = entry.expect("read a directory entry").path();
            if path.is_dir() {
                directories.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "wgsl")
            {
                let relative = path.strip_prefix(root).expect("under the crate root");
                files.push(relative.to_path_buf());
            }
        }
    }
    files.sort();
    files
}

/// A Rust struct whose bytes the GPU reads, and the WGSL struct it must match.
struct Twin {
    wgsl_name: &'static str,
    size: usize,
    fields: Vec<Field>,
}

struct Field {
    name: &'static str,
    offset: usize,
    size: usize,
    scalar: naga::Scalar,
}

/// The scalar WGSL reads a Rust field as: `f32` for `[[f32; 4]; 4]`.
trait ShaderScalar {
    const SCALAR: naga::Scalar;
}

impl ShaderScalar for f32 {
    const SCALAR: naga::Scalar = naga::Scalar::F32;
}

impl ShaderScalar for i32 {
    const SCALAR: naga::Scalar = naga::Scalar::I32;
}

impl ShaderScalar for u32 {
    const SCALAR: naga::Scalar = naga::Scalar::U32;
}

/// WGSL has no 16-bit integers, so a shader reads them in pairs, as u32s.
impl ShaderScalar for i16 {
    const SCALAR: naga::Scalar = naga::Scalar::U32;
}

impl<T: ShaderScalar, const LENGTH: usize> ShaderScalar for [T; LENGTH] {
    const SCALAR: naga::Scalar = T::SCALAR;
}

fn scalar_of<T: ShaderScalar>(_: &T) -> naga::Scalar {
    T::SCALAR
}

/// Each way a struct shared with the CPU in `modules`, each parsed, disagrees
/// with its twin, as `module: struct: problem`; empty when all agree.
fn layout_problems(modules: &[(ShaderModule, naga::Module)], twins: &[Twin]) -> Vec<String> {
    let mut problems = Vec::new();
    let mut matched = BTreeSet::new();
    for (module, parsed) in modules {
        let label = format!("{} module", module.name);
        for struct_type in structs_shared_with_the_cpu(module, parsed) {
            let name = parsed.types[struct_type]
                .name
                .as_deref()
                .unwrap_or_default();
            let Some(twin) = twins.iter().find(|twin| twin.wgsl_name == name) else {
                problems.push(format!(
                    "{label}: {name}: shared with the CPU but has no twin"
                ));
                continue;
            };
            matched.insert(name);
            problems.extend(
                twin_problems(parsed, struct_type, twin)
                    .into_iter()
                    .map(|problem| format!("{label}: {name}: {problem}")),
            );
        }
    }
    for twin in twins {
        if !matched.contains(twin.wgsl_name) {
            problems.push(format!("{}: no shader module shares it", twin.wgsl_name));
        }
    }
    problems
}

/// Structs whose bytes the CPU writes, in `module` as `parsed`: those in
/// uniform, storage and immediate data, alone or as array elements, but for
/// bindings whose definition says only the GPU writes them.
fn structs_shared_with_the_cpu(
    module: &ShaderModule,
    parsed: &naga::Module,
) -> BTreeSet<naga::Handle<naga::Type>> {
    let mut types = BTreeSet::new();
    for (_, global) in parsed.global_variables.iter() {
        let only_the_gpu_writes = global
            .binding
            .as_ref()
            .and_then(|resource| defined_binding(module, resource))
            .is_some_and(|(_, binding)| binding.writer == Writer::Gpu);
        if !only_the_gpu_writes
            && matches!(
                global.space,
                naga::AddressSpace::Uniform
                    | naga::AddressSpace::Storage { .. }
                    | naga::AddressSpace::Immediate
            )
        {
            types.insert(match parsed.types[global.ty].inner {
                naga::TypeInner::Array { base, .. } => base,
                _ => global.ty,
            });
        }
    }
    types.retain(|&handle| matches!(parsed.types[handle].inner, naga::TypeInner::Struct { .. }));
    types
}

/// How WGSL struct `struct_type` and `twin` disagree, one line per difference.
fn twin_problems(
    module: &naga::Module,
    struct_type: naga::Handle<naga::Type>,
    twin: &Twin,
) -> Vec<String> {
    let naga::TypeInner::Struct { ref members, span } = module.types[struct_type].inner else {
        unreachable!("only structs are shared with the CPU");
    };
    let mut problems = Vec::new();
    let mut wgsl = BTreeSet::new();
    for member in members {
        let name = member.name.as_deref().unwrap_or_default();
        let size = module.types[member.ty].inner.size(module.to_ctx()) as usize;
        let scalar = wgsl_scalar(module, member.ty);
        wgsl.insert(describe(name, member.offset as usize, size, scalar));
    }
    wgsl.insert(format!("size: {span} bytes"));
    let rust: BTreeSet<String> = twin
        .fields
        .iter()
        // An underscored field is explicit padding, absent from WGSL.
        .filter(|field| !field.name.starts_with('_'))
        .map(|field| describe(field.name, field.offset, field.size, Some(field.scalar)))
        .chain([format!("size: {} bytes", twin.size)])
        .collect();
    problems.extend(wgsl.difference(&rust).map(|line| format!("WGSL {line}")));
    problems.extend(rust.difference(&wgsl).map(|line| format!("Rust {line}")));
    problems
}

/// The scalar a WGSL type is built from: `f32` for `array<vec4<f32>, 4>`.
fn wgsl_scalar(
    module: &naga::Module,
    type_handle: naga::Handle<naga::Type>,
) -> Option<naga::Scalar> {
    match module.types[type_handle].inner {
        naga::TypeInner::Array { base, .. } => wgsl_scalar(module, base),
        ref inner => inner.scalar(),
    }
}

/// A field's place and type.
fn describe(name: &str, offset: usize, size: usize, scalar: Option<naga::Scalar>) -> String {
    let scalar = scalar.map_or("?".to_string(), |scalar| {
        let kind = match scalar.kind {
            naga::ScalarKind::Sint => "i",
            naga::ScalarKind::Uint => "u",
            naga::ScalarKind::Float => "f",
            _ => "?",
        };
        format!("{kind}{}", u32::from(scalar.width) * 8)
    });
    format!("`{name}` at offset {offset}: {size} bytes of {scalar}")
}
