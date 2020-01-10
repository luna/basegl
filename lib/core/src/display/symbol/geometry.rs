//! This is the root module for all geometry types, including polygon meshes, NURBS surfaces, and
//! volumes. It also contains compound geometry, predefined more complex shapes.

pub mod compound;
pub mod primitive;
pub mod sprite;


// =================
// === Reexports ===
// =================

pub mod types {
    use super::*;
    pub use primitive::types::*;
}
pub use types::*;
