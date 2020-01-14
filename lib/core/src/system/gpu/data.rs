//! This module defines data types representing attributes and uniforms.
//!   - Uniforms are per-primitive parameters (constant during an entire draw call).
//!   - Attributes are per-vertex parameters (typically positions, normals, colors, UVs, ...).
//!
//! To learn more about these concepts, follow the link:
//! https://www.khronos.org/opengl/wiki/Type_Qualifier_(GLSL)

pub mod attribute;
pub mod shader_default;
pub mod gl_enum;
pub mod texture;
pub mod uniform;


// =================
// === Reexports ===
// =================

pub use attribute::*;
pub use super::buffer::item::*;
pub use shader_default::*;
pub use uniform::*;

/// ...
pub mod types {
    use super::*;
    pub use uniform::Uniform;
    pub use uniform::UniformScope;
    pub use attribute::Buffer;
    pub use attribute::Attribute;
    pub use attribute::AttributeScope;
}
pub use types::*;
