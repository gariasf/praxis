//! The WGSL shader modules, each assembled from ordered parts in this
//! directory, so passes share structs and functions without copying them,
//! after the constants Rust shares with WGSL, the Render switches' overrides
//! and its bindings' declarations, all generated from Rust (`assemble`,
//! `render::bind_groups`). A function under an oracle lives in a shared
//! part, which the oracle harness assembles with its kernel alone.

use crate::cooked::{MESHLET_MAX_TRIANGLES, MESHLET_MAX_VERTICES};
use crate::render::bind_groups::{
    BindGroupDefinition, FRAME, FRAME_DRAW, MATERIALS, MESHLET_RECORDS, SCENE_GEOMETRY,
};
use crate::render::culling_pass::SCAN_BLOCK_SIZE;
use crate::render::instance::{ANCHOR_CELL_SIZE, INSTANCE_CAPACITY, NO_MATERIAL, NO_MESH};
use crate::render::renderer::RenderSwitches;

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
    // Each Channel is a binding array indexed by Layer (ADR 0017).
    enables: &["wgpu_binding_array"],
    bind_groups: &[&FRAME, &MATERIALS, &SCENE_GEOMETRY, &MESHLET_RECORDS],
    parts: &[COMMON, SHADING, part!("main.wgsl")],
};

/// The culling pass.
pub const CULLING: ShaderModule = ShaderModule {
    name: "culling",
    enables: &[],
    bind_groups: &[&FRAME, &SCENE_GEOMETRY, &FRAME_DRAW],
    parts: &[COMMON, part!("culling.wgsl")],
};

/// The shared parts alone, which the oracle harness appends its kernel to.
#[cfg(test)]
pub const SHARED: ShaderModule = ShaderModule {
    name: "shared",
    enables: &[],
    bind_groups: &[],
    parts: &[COMMON, SHADING],
};

/// Every module, which `shader_tests` validates.
#[cfg(test)]
pub const MODULES: [ShaderModule; 3] = [SHARED, MAIN, CULLING];

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
             const INSTANCE_CAPACITY: u32 = {INSTANCE_CAPACITY}u;\n",
            ANCHOR_CELL_SIZE as f32,
            MESHLET_MAX_VERTICES.ilog2()
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
