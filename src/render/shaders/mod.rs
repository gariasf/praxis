//! The WGSL shader modules, each assembled from ordered parts in this
//! directory, so passes share structs and functions without copying them. A
//! function under an oracle lives in a shared part, which the oracle harness
//! assembles with its kernel alone.

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

/// The structs the CPU writes, the Anchor composition and the vertex
/// decode.
const COMMON: Part = part!("common.wgsl");

/// The shading terms.
const SHADING: Part = part!("shading.wgsl");

/// A WGSL module: the extensions it enables, then its parts in order.
#[derive(Clone, Copy)]
pub struct ShaderModule {
    pub name: &'static str,
    pub enables: &'static [&'static str],
    pub parts: &'static [Part],
}

/// The main pass.
pub const MAIN: ShaderModule = ShaderModule {
    name: "main",
    // Each Channel is a binding array indexed by Layer (ADR 0017).
    enables: &["wgpu_binding_array"],
    parts: &[COMMON, SHADING, part!("main.wgsl")],
};

/// The shared parts alone, which the oracle harness appends its kernel to.
#[cfg(test)]
pub const SHARED: ShaderModule = ShaderModule {
    name: "shared",
    enables: &[],
    parts: &[COMMON, SHADING],
};

/// Every module, which `shader_tests` validates.
#[cfg(test)]
pub const MODULES: [ShaderModule; 2] = [SHARED, MAIN];

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
    /// 1-based line and column of `offset` within that part; `None` in the
    /// `enable` directives.
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
    /// directives precede every declaration, then each part on lines of its
    /// own; and the byte offset where each part starts.
    fn assemble(&self) -> (String, Vec<usize>) {
        let mut source: String = self
            .enables
            .iter()
            .map(|extension| format!("enable {extension};\n"))
            .collect();
        let mut starts = Vec::with_capacity(self.parts.len());
        for part in self.parts {
            starts.push(source.len());
            source.push_str(part.source);
            source.push('\n');
        }
        (source, starts)
    }
}
