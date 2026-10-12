//! Every WGSL file under `src/` is checked here without a GPU, as wgpu checks
//! it when it creates a shader module (backend code generation aside), and
//! each struct whose bytes the CPU writes is compared with its Rust twin.

use std::collections::BTreeSet;
use std::path::Path;

use wgpu::naga;

use crate::assets::MaterialData;
use crate::render::{CameraUniform, InstanceData, LightUniform, Vertex};

/// Shader capabilities Praxis requires of every adapter. None today: a
/// shader that needs one fails validation until it is added here, so a new
/// hardware requirement is a visible decision.
const CAPABILITIES: naga::valid::Capabilities = naga::valid::Capabilities::empty();

#[test]
fn every_wgsl_file_parses_and_validates() {
    let files = wgsl_files();
    assert!(!files.is_empty(), "no .wgsl file under src/");
    let errors: Vec<String> = files
        .iter()
        .filter_map(|(path, source)| validate(path, source).err())
        .collect();
    assert!(errors.is_empty(), "{}", errors.join("\n"));
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
            vertex_buffer: None,
        }
    }};
}

#[test]
fn structs_shared_with_the_cpu_match_their_rust_twins() {
    let modules: Vec<(String, naga::Module)> = wgsl_files()
        .into_iter()
        .map(|(path, source)| {
            let module = validate(&path, &source).unwrap_or_else(|error| panic!("{error}"));
            (path, module)
        })
        .collect();
    let twins = [
        twin!(CameraUniform {
            view_proj,
            position
        }),
        twin!(LightUniform {
            direction,
            color,
            ambient,
            point_positions,
            point_colors,
            num_point_lights,
        }),
        twin!(InstanceData {
            model,
            normal_matrix,
            material_id,
            _pad
        }),
        twin!(MaterialData {
            base_color,
            metallic_roughness,
            emissive,
            texture_indices,
            extra,
        }),
        Twin {
            wgsl_name: "VertexInput",
            vertex_buffer: Some(Vertex::desc()),
            ..twin!(Vertex {
                position,
                normal,
                uv,
                tangent
            })
        },
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
        struct VertexInput {
            @builtin(vertex_index) index: u32,
            @location(0) position: vec3<f32>,
            @location(1) uv: vec2<f32>,
            @location(2) weight: f32,
        }
        @group(0) @binding(0) var<uniform> light: Light;
        @group(0) @binding(1) var<storage, read> materials: array<Material>;
        @group(0) @binding(2) var<storage, read> orphans: array<Orphan>;
        @vertex fn main(input: VertexInput) -> @builtin(position) vec4<f32> {
            return vec4<f32>(input.position, 1.0);
        }
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
    #[repr(C)]
    struct TestVertex {
        uv: [f32; 2],
        position: [f32; 3],
    }
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Unorm16x4];
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
            wgsl_name: "VertexInput",
            vertex_buffer: Some(wgpu::VertexBufferLayout {
                array_stride: size_of::<TestVertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &ATTRIBUTES,
            }),
            ..twin!(TestVertex { uv, position })
        },
        Twin {
            wgsl_name: "Missing",
            ..twin!(TestVertex { uv, position })
        },
    ];
    let module = validate("test.wgsl", SOURCE).unwrap_or_else(|error| panic!("{error}"));

    let mut problems = layout_problems(&[("test.wgsl".to_string(), module)], &twins);
    problems.sort();
    assert_eq!(
        problems,
        [
            "Missing: no WGSL file shares it",
            "test.wgsl: Light: Rust `count` at offset 12: 4 bytes of f32",
            "test.wgsl: Light: WGSL `count` at offset 12: 4 bytes of u32",
            "test.wgsl: Material: Rust size: 20 bytes",
            "test.wgsl: Material: WGSL size: 32 bytes",
            "test.wgsl: Orphan: shared with the CPU but has no twin",
            "test.wgsl: VertexInput: Rust `position` at offset 8: 12 bytes of f32",
            "test.wgsl: VertexInput: Rust `uv` at offset 0: 8 bytes of f32",
            "test.wgsl: VertexInput: WGSL `position` at offset 0: 12 bytes of f32",
            "test.wgsl: VertexInput: WGSL `uv` at offset 12: 8 bytes of f32",
            "test.wgsl: VertexInput: `uv` is fed by a Unorm16x4 vertex attribute",
            "test.wgsl: VertexInput: no vertex attribute at location 2 (`weight`)",
        ]
    );
}

#[test]
fn a_bind_group_past_the_device_limit_fails_validation() {
    let source = "@group(4) @binding(0) var<uniform> value: vec4<f32>;";
    assert_eq!(
        validate("test.wgsl", source).err().as_deref(),
        Some("test.wgsl: @group(4) exceeds the device's max_bind_groups of 4")
    );
}

/// Parses and validates `source` as wgpu does, or returns the error after
/// the path, which some naga diagnostics do not mention.
fn validate(path: &str, source: &str) -> Result<naga::Module, String> {
    let mut options = naga::front::wgsl::Options::new();
    options.capabilities = CAPABILITIES;
    let module = naga::front::wgsl::Frontend::new_with_options(options)
        .parse(source)
        .map_err(|error| format!("{path}: {}", error.emit_to_string_with_path(source, path)))?;
    // `GpuContext` requests the default limits.
    let max_bind_groups = wgpu::Limits::default().max_bind_groups;
    for (_, global) in module.global_variables.iter() {
        if let Some(binding) = &global.binding
            && binding.group >= max_bind_groups
        {
            return Err(format!(
                "{path}: @group({}) exceeds the device's max_bind_groups of {max_bind_groups}",
                binding.group
            ));
        }
    }
    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), CAPABILITIES)
        .validate(&module)
        .map_err(|error| format!("{path}: {}", error.emit_to_string_with_path(source, path)))?;
    Ok(module)
}

/// Every `.wgsl` file under `src/`, as (path relative to the crate, source).
fn wgsl_files() -> Vec<(String, String)> {
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
                let source = std::fs::read_to_string(&path)
                    .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
                let relative = path.strip_prefix(root).expect("under the crate root");
                files.push((relative.display().to_string(), source));
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
    /// How the struct feeds a vertex shader's `@location` inputs; `None` for
    /// a struct in a uniform or storage buffer.
    vertex_buffer: Option<wgpu::VertexBufferLayout<'static>>,
}

struct Field {
    name: &'static str,
    offset: usize,
    size: usize,
    scalar: naga::Scalar,
}

/// The scalar a Rust field is built from: `f32` for `[[f32; 4]; 4]`.
trait ShaderScalar {
    const SCALAR: naga::Scalar;
}

impl ShaderScalar for f32 {
    const SCALAR: naga::Scalar = naga::Scalar::F32;
}

impl ShaderScalar for u32 {
    const SCALAR: naga::Scalar = naga::Scalar::U32;
}

impl<T: ShaderScalar, const LENGTH: usize> ShaderScalar for [T; LENGTH] {
    const SCALAR: naga::Scalar = T::SCALAR;
}

fn scalar_of<T: ShaderScalar>(_: &T) -> naga::Scalar {
    T::SCALAR
}

/// Each way a struct shared with the CPU in `modules` disagrees with its
/// twin, as `path: struct: problem`; empty when all agree.
fn layout_problems(modules: &[(String, naga::Module)], twins: &[Twin]) -> Vec<String> {
    let mut problems = Vec::new();
    let mut matched = BTreeSet::new();
    for (path, module) in modules {
        for struct_type in structs_shared_with_the_cpu(module) {
            let name = module.types[struct_type]
                .name
                .as_deref()
                .unwrap_or_default();
            let Some(twin) = twins.iter().find(|twin| twin.wgsl_name == name) else {
                problems.push(format!(
                    "{path}: {name}: shared with the CPU but has no twin"
                ));
                continue;
            };
            matched.insert(name);
            problems.extend(
                twin_problems(module, struct_type, twin)
                    .into_iter()
                    .map(|problem| format!("{path}: {name}: {problem}")),
            );
        }
    }
    for twin in twins {
        if !matched.contains(twin.wgsl_name) {
            problems.push(format!("{}: no WGSL file shares it", twin.wgsl_name));
        }
    }
    problems
}

/// Structs whose bytes the CPU writes: those in uniform, storage and
/// immediate data (alone or as array elements) and vertex shader inputs.
fn structs_shared_with_the_cpu(module: &naga::Module) -> BTreeSet<naga::Handle<naga::Type>> {
    let mut types = BTreeSet::new();
    for (_, global) in module.global_variables.iter() {
        if matches!(
            global.space,
            naga::AddressSpace::Uniform
                | naga::AddressSpace::Storage { .. }
                | naga::AddressSpace::Immediate
        ) {
            types.insert(match module.types[global.ty].inner {
                naga::TypeInner::Array { base, .. } => base,
                _ => global.ty,
            });
        }
    }
    for entry_point in &module.entry_points {
        if entry_point.stage == naga::ShaderStage::Vertex {
            for argument in &entry_point.function.arguments {
                types.insert(argument.ty);
            }
        }
    }
    types.retain(|&handle| matches!(module.types[handle].inner, naga::TypeInner::Struct { .. }));
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
        // A vertex input has no memory layout in WGSL: its bytes sit where
        // the vertex buffer layout puts its location.
        let offset = match (&member.binding, &twin.vertex_buffer) {
            (Some(naga::Binding::BuiltIn(_)), _) => continue,
            (Some(naga::Binding::Location { location, .. }), Some(vertex_buffer)) => {
                let Some(attribute) = vertex_buffer
                    .attributes
                    .iter()
                    .find(|attribute| attribute.shader_location == *location)
                else {
                    problems.push(format!(
                        "no vertex attribute at location {location} (`{name}`)"
                    ));
                    continue;
                };
                // Vertex data is unpacked f32s today, read as stored. A packed
                // format needs the Rust field's type to pick its row.
                let stored_format = match (wgsl_scalar(module, member.ty), size) {
                    (Some(naga::Scalar::F32), 4) => Some(wgpu::VertexFormat::Float32),
                    (Some(naga::Scalar::F32), 8) => Some(wgpu::VertexFormat::Float32x2),
                    (Some(naga::Scalar::F32), 12) => Some(wgpu::VertexFormat::Float32x3),
                    (Some(naga::Scalar::F32), 16) => Some(wgpu::VertexFormat::Float32x4),
                    _ => None,
                };
                if stored_format != Some(attribute.format) {
                    problems.push(format!(
                        "`{name}` is fed by a {:?} vertex attribute",
                        attribute.format
                    ));
                }
                attribute.offset as usize
            }
            _ => member.offset as usize,
        };
        wgsl.insert(describe(name, offset, size, wgsl_scalar(module, member.ty)));
    }
    let wgsl_size = twin
        .vertex_buffer
        .as_ref()
        .map_or(span as usize, |vertex_buffer| {
            vertex_buffer.array_stride as usize
        });
    wgsl.insert(format!("size: {wgsl_size} bytes"));
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

fn describe(name: &str, offset: usize, size: usize, scalar: Option<naga::Scalar>) -> String {
    let scalar = scalar
        .and_then(naga::common::wgsl::TryToWgsl::try_to_wgsl)
        .unwrap_or("?");
    format!("`{name}` at offset {offset}: {size} bytes of {scalar}")
}
