#![feature(bool_to_option)]
#![feature(drain_filter)]
#![feature(trait_alias)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unsafe_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

pub mod node;

use prelude::*;

use ensogl::display;
use ensogl::display::Sprite;
use ensogl::display::shape::*;
use ensogl::display::object::traits::*;
use ensogl::traits::*;
use logger::Logger;
use enso_prelude::std_reexports::fmt::{Formatter, Error};
use nalgebra::Vector2;

pub mod prelude {
    pub use enso_prelude::*;
}

pub use node::Node;
use ensogl::display::shape::system::ShapeSystem;
use ensogl::display::shape::*;
use std::any::TypeId;
use ensogl::display::world::World;




use ensogl::prelude::*;

use ensogl::display::object::traits::*;
use ensogl::display::shape::text::glyph::font::FontHandle;
use ensogl::display::shape::text::glyph::system::GlyphSystem;
use ensogl::display::shape::text::text_field::content::TextFieldContent;
use ensogl::display::shape::text::text_field::cursor::Cursor;
use ensogl::display::shape::text::text_field::cursor::Cursors;
use ensogl::display::shape::text::text_field::render::assignment::GlyphLinesAssignment;
use ensogl::display::shape::text::text_field::render::assignment::LineFragment;
use ensogl::display::shape::text::text_field::render::selection::SelectionSpritesGenerator;
use ensogl::display::shape::text::text_field::TextFieldProperties;
use ensogl::display::shape::*;

use ensogl::math::topology::unit::PixelDistance;
use ensogl::display::Glsl;



// =========================
// === Library Utilities ===
// =========================

pub trait HasSprite {
    fn set_sprite(&self, sprite:&Sprite);
}

/// Registers Node shape.
pub fn register_shapes(world:&World) {
   let node_shape   = Rect(Vector2::new(50.0.px(),50.0.px()));;
   let shape_system = ShapeSystem::new(world,&node_shape);
   world.scene().register_shape(TypeId::of::<Node>(),shape_system.clone());
}



// =============
// === Graph ===
// =============

#[derive(Default)]
pub struct OnEditCallbacks {
    node_added : Option<Box<dyn Fn(&node::Node) + 'static>>,
}

impl Debug for OnEditCallbacks {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str("graph::OnEditCallbacks")
    }
}

#[derive(Debug,Default)]
struct GraphData {
    nodes : Vec<node::Node>,
}

#[derive(Debug)]
pub struct Graph {
    data           : Rc<RefCell<GraphData>>,
    display_object : display::object::Node,
    callbacks      : Rc<RefCell<OnEditCallbacks>>,
    logger         : Logger,
}

impl Graph {
    pub fn new(world:&World) -> Self {
        let logger         = Logger::new("graph");
        let data           = default();
        let display_object = display::object::Node::new(&logger);
        let callbacks      = default();
        register_shapes(world);
        Self {data,display_object,callbacks,logger}
    }

    pub fn set_on_edit_callbacks(&self, callbacks: OnEditCallbacks) {
        *self.callbacks.borrow_mut() = callbacks
    }
}

impl<'a> From<&'a Graph> for &'a display::object::Node {
    fn from(graph: &'a Graph) -> Self {
        &graph.display_object
    }
}


// === Interface for library users ===

impl Graph {
    pub fn add_node(&self, new_node:node::Node) {
        self.logger.warning(|| format!("Add new node with label {}", new_node.label()));
        self.display_object.add_child(&new_node);
        self.data.borrow_mut().nodes.push(new_node);
    }

    pub fn clear_graph(&self) {
        self.logger.warning("Clear graph");
        let mut data = self.data.borrow_mut();
        for node in &data.nodes {
            self.display_object.remove_child(node);
        }
        data.nodes.clear();
    }
}


// === Interface for GUI events ===

impl Graph {
    pub fn gui_add_node(&self, new_node:node::Node) {
        self.gui_add_node(new_node.clone());
        if let Some(callback) = &self.callbacks.borrow().node_added {
            callback(&new_node)
        }
    }
}