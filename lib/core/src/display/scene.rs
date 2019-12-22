pub mod mesh_registry;

use crate::prelude::*;

use crate::display::camera::Camera2D;
use crate::display::symbol::DisplayObjectData;
use basegl_system_web::Logger;


// =============
// === Scene ===
// =============

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Scene {
    pub root   : DisplayObjectData,
    pub camera : Camera2D
}

// === Implementation ===


impl Scene {
    pub fn new(logger:Logger) -> Self {
        let root   = DisplayObjectData::new(logger.sub("root"));
        let camera = Camera2D::new(logger.sub("camera"));
        Self {root,camera}
    }
}

