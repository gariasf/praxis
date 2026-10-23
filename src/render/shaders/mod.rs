//! The WGSL shader modules, each assembled from ordered parts in this
//! directory, so passes share structs and functions without copying them,
//! after the constants Rust shares with WGSL, the Render switches' overrides
//! and its bindings' declarations, all generated from Rust (`assemble`,
//! `render::bind_groups`). A function under an oracle lives in a shared
//! part, which the oracle harness assembles with its kernel alone.

use crate::cooked::{MESHLET_MAX_TRIANGLES, MESHLET_MAX_VERTICES, QTANGENT_SMALLEST_W};
use crate::render::bend_pass::BEND_WORKGROUP_SIZE;
use crate::render::bind_groups::{
    BENDS, BindGroupDefinition, DEPTH_PYRAMID_LEVEL, DYNAMIC_TLAS_MARKERS, FRAME, FRAME_DRAW,
    FROXEL_LISTS, LIGHT_COUNT_INPUT, MAIN_PASS_INPUTS, MATERIALS, SCENE_GEOMETRY,
    SHADOW_MASK_BUILD, TONEMAPPING_INPUT,
};
use crate::render::culling_pass::SCAN_BLOCK_SIZE;
use crate::render::depth_pyramid_pass::DEPTH_PYRAMID_TILE_SIZE;
use crate::render::froxel_pass::{
    FIRST_FROXEL_SLICE_DEPTH, FROXEL_COLUMNS, FROXEL_LIGHT_CAPACITY, FROXEL_ROWS, FROXEL_SLICES,
    FROXEL_SLICES_PER_OCTAVE,
};
use crate::render::instance::{ANCHOR_CELL_SIZE, INSTANCE_CAPACITY, NO_MATERIAL, NO_MESH};
use crate::render::light_count_pass::LIGHT_COUNT_TILE_SIZE;
use crate::render::renderer::RenderSwitches;
use crate::render::shadow_mask_pass::{SAMPLE_OFFSETS, SHADOW_MASK_LIGHTS, SHADOW_MASK_TILE_SIZE};
use crate::render::targets::LUMINANCE_COEFFICIENTS;
use crate::render::tonemapping_pass::DITHER_MASK_SIDE;

// A packed index holds a Meshlet vertex in whole bits.
const _: () = assert!(MESHLET_MAX_VERTICES.is_power_of_two());

/// One WGSL file of this directory.
#[derive(Clone, Copy)]
pub struct Part {
    /// Its path from the crate root, as errors name it.
    #[cfg_attr(not(test), allow(dead_code))]
    pub path: &'static str,
    pub source: &'static str,
}

/// The part in `$file`, a file of this directory.
macro_rules! part {
    ($file:literal) => {
        Part {
            path: concat!("src/render/shaders/", $file),
            source: include_str!($file),
        }
    };
}

/// The structs more than one pass reads, the Anchor composition, the packed
/// index and the vertex decode.
const COMMON: Part = part!("common.wgsl");

/// The shading terms.
const SHADING: Part = part!("shading.wgsl");

/// The Froxel grid: its slices, and where a Froxel's list starts.
const FROXEL_GRID: Part = part!("froxel_grid.wgsl");

/// Exposed colour as the HDR target stores it.
const HDR: Part = part!("hdr.wgsl");

/// GT7's tone mapping, under Polyphony's MIT notice.
const GT7: Part = part!("gt7.wgsl");

/// What turns exposed colour into display colour, around GT7.
const DISPLAY: Part = part!("display.wgsl");

/// A WGSL module: the extensions it enables, the bind groups whose bindings
/// it declares, then its parts in order.
#[derive(Clone, Copy)]
pub struct ShaderModule {
    pub name: &'static str,
    pub enables: &'static [&'static str],
    pub bind_groups: &'static [&'static BindGroupDefinition],
    pub parts: &'static [Part],
}

/// The main pass.
pub const MAIN: ShaderModule = ShaderModule {
    name: "main",
    // Each Channel is a binding array indexed by Layer (ADR 0017), and the
    // scene geometry holds both TLAS (ADR 0020).
    enables: &["wgpu_binding_array", "wgpu_ray_query"],
    bind_groups: &[&FRAME, &MATERIALS, &SCENE_GEOMETRY, &MAIN_PASS_INPUTS],
    parts: &[COMMON, SHADING, HDR, FROXEL_GRID, part!("main.wgsl")],
};

/// The shadow mask pass.
pub const SHADOW_MASK: ShaderModule = ShaderModule {
    name: "shadow_mask",
    // It traces rays against both TLAS.
    enables: &["wgpu_ray_query"],
    bind_groups: &[&SCENE_GEOMETRY, &SHADOW_MASK_BUILD],
    parts: &[COMMON, FROXEL_GRID, part!("shadow_mask.wgsl")],
};

/// The culling pass.
pub const CULLING: ShaderModule = ShaderModule {
    name: "culling",
    // The scene geometry holds both TLAS.
    enables: &["wgpu_ray_query"],
    bind_groups: &[&FRAME, &SCENE_GEOMETRY, &FRAME_DRAW],
    parts: &[COMMON, part!("culling.wgsl")],
};

/// The Froxel pass.
pub const FROXEL: ShaderModule = ShaderModule {
    name: "froxel",
    enables: &[],
    bind_groups: &[&FROXEL_LISTS],
    parts: &[COMMON, FROXEL_GRID, part!("froxel.wgsl")],
};

/// The light count pass, with the `count_point_lights` Render switch on.
pub const LIGHT_COUNT: ShaderModule = ShaderModule {
    name: "light_count",
    enables: &[],
    bind_groups: &[&LIGHT_COUNT_INPUT],
    parts: &[COMMON, FROXEL_GRID, part!("light_count.wgsl")],
};

/// The depth pyramid pass.
pub const DEPTH_PYRAMID: ShaderModule = ShaderModule {
    name: "depth_pyramid",
    enables: &[],
    bind_groups: &[&DEPTH_PYRAMID_LEVEL],
    parts: &[part!("depth_pyramid.wgsl")],
};

/// The tonemapping pass.
pub const TONEMAPPING: ShaderModule = ShaderModule {
    name: "tonemapping",
    enables: &[],
    bind_groups: &[&TONEMAPPING_INPUT],
    parts: &[HDR, GT7, DISPLAY, part!("tonemapping.wgsl")],
};

/// The dynamic TLAS's markers, which time its build.
pub const DYNAMIC_TLAS_MARKER: ShaderModule = ShaderModule {
    name: "dynamic_tlas_marker",
    enables: &["wgpu_ray_query"],
    bind_groups: &[&DYNAMIC_TLAS_MARKERS],
    parts: &[part!("dynamic_tlas_marker.wgsl")],
};

/// The bend pass.
pub const BEND: ShaderModule = ShaderModule {
    name: "bend",
    enables: &[],
    bind_groups: &[&BENDS],
    parts: &[COMMON, part!("bend.wgsl")],
};

/// The shared parts alone, which the oracle harness appends its kernel to.
#[cfg(test)]
pub const SHARED: ShaderModule = ShaderModule {
    name: "shared",
    enables: &[],
    bind_groups: &[],
    parts: &[COMMON, SHADING, HDR, GT7, DISPLAY],
};

/// Every module, which `shader_tests` validates.
#[cfg(test)]
pub const MODULES: [ShaderModule; 10] = [
    SHARED,
    MAIN,
    SHADOW_MASK,
    CULLING,
    FROXEL,
    LIGHT_COUNT,
    DEPTH_PYRAMID,
    TONEMAPPING,
    DYNAMIC_TLAS_MARKER,
    BEND,
];

impl ShaderModule {
    /// The descriptor that compiles the module, labelled with its name.
    pub fn descriptor(&self) -> wgpu::ShaderModuleDescriptor<'static> {
        wgpu::ShaderModuleDescriptor {
            label: Some(self.name),
            source: wgpu::ShaderSource::Wgsl(self.source().into()),
        }
    }

    /// The module's WGSL.
    pub fn source(&self) -> String {
        self.assemble().0
    }

    /// The part that byte `offset` of the module's WGSL falls in, and the
    /// 1-based line and column of `offset` within that part; `None` before
    /// the first part, in what `assemble` generates.
    #[cfg(test)]
    pub fn locate(&self, offset: usize) -> Option<(&'static str, usize, usize)> {
        let (_, starts) = self.assemble();
        let index = starts.iter().rposition(|&start| start <= offset)?;
        let part = self.parts[index];
        let before = &part.source[..(offset - starts[index]).min(part.source.len())];
        let line_start = before.rfind('\n').map_or(0, |newline| newline + 1);
        Some((
            part.path,
            before.matches('\n').count() + 1,
            before.len() - line_start + 1,
        ))
    }

    /// The module's WGSL, an `enable` directive per extension first, since
    /// directives precede every declaration, then the constants Rust shares,
    /// an `override` per Render switch, which has no default so a pipeline
    /// that uses it must set it, and the declarations of its bind groups'
    /// bindings, then each part on lines of its own; and the byte offset
    /// where each part starts.
    fn assemble(&self) -> (String, Vec<usize>) {
        let mut source: String = self
            .enables
            .iter()
            .map(|extension| format!("enable {extension};\n"))
            .collect();
        source.push_str(&format!(
            "const ANCHOR_CELL_SIZE: f32 = {:?};\n\
             const MESHLET_VERTEX_BITS: u32 = {}u;\n\
             const MESHLET_MAX_TRIANGLES: u32 = {MESHLET_MAX_TRIANGLES}u;\n\
             const NO_MESH: u32 = {NO_MESH}u;\n\
             const NO_MATERIAL: u32 = {NO_MATERIAL}u;\n\
             const SCAN_BLOCK_SIZE: u32 = {SCAN_BLOCK_SIZE}u;\n\
             const INSTANCE_CAPACITY: u32 = {INSTANCE_CAPACITY}u;\n\
             const DEPTH_PYRAMID_TILE_SIZE: u32 = {DEPTH_PYRAMID_TILE_SIZE}u;\n\
             const LIGHT_COUNT_TILE_SIZE: u32 = {LIGHT_COUNT_TILE_SIZE}u;\n\
             const LUMINANCE_COEFFICIENTS: vec3<f32> = vec3<f32>({:?}, {:?}, {:?});\n\
             const DITHER_MASK_SIDE: u32 = {DITHER_MASK_SIDE}u;\n\
             const FROXEL_COLUMNS: u32 = {FROXEL_COLUMNS}u;\n\
             const FROXEL_ROWS: u32 = {FROXEL_ROWS}u;\n\
             const FROXEL_SLICES: u32 = {FROXEL_SLICES}u;\n\
             const FIRST_FROXEL_SLICE_DEPTH: f32 = {FIRST_FROXEL_SLICE_DEPTH:?};\n\
             const FROXEL_SLICES_PER_OCTAVE: f32 = {FROXEL_SLICES_PER_OCTAVE:?};\n\
             const FROXEL_LIGHT_CAPACITY: u32 = {FROXEL_LIGHT_CAPACITY}u;\n\
             const BEND_WORKGROUP_SIZE: u32 = {BEND_WORKGROUP_SIZE}u;\n\
             const QTANGENT_SMALLEST_W: f32 = {QTANGENT_SMALLEST_W:?};\n\
             const SHADOW_MASK_TILE_SIZE: u32 = {SHADOW_MASK_TILE_SIZE}u;\n\
             const SHADOW_MASK_LIGHTS: u32 = {SHADOW_MASK_LIGHTS}u;\n",
            ANCHOR_CELL_SIZE as f32,
            MESHLET_MAX_VERTICES.ilog2(),
            LUMINANCE_COEFFICIENTS[0],
            LUMINANCE_COEFFICIENTS[1],
            LUMINANCE_COEFFICIENTS[2]
        ));
        let sample_offsets: Vec<String> = SAMPLE_OFFSETS
            .iter()
            .map(|[x, y]| format!("vec2<f32>({x:?}, {y:?})"))
            .collect();
        source.push_str(&format!(
            "const SAMPLE_OFFSETS: array<vec2<f32>, {}> = array<vec2<f32>, {0}>({});\n",
            SAMPLE_OFFSETS.len(),
            sample_offsets.join(", ")
        ));
        for (switch, _) in RenderSwitches::default().constants() {
            source.push_str(&format!("override {switch}: bool;\n"));
        }
        for bind_group in self.bind_groups {
            source.push_str(&bind_group.declarations());
        }
        let mut starts = Vec::with_capacity(self.parts.len());
        for part in self.parts {
            starts.push(source.len());
            source.push_str(part.source);
            source.push('\n');
        }
        (source, starts)
    }
}
