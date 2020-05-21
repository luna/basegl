//! Definition of the Edge component.

#![allow(missing_docs)]
// WARNING! UNDER HEAVY DEVELOPMENT. EXPECT DRASTIC CHANGES.

use crate::prelude::*;

use enso_frp;
use enso_frp as frp;
use ensogl::data::color;
use ensogl::display::Attribute;
use ensogl::display::Buffer;
use ensogl::display::Sprite;
use ensogl::display::scene::Scene;
use ensogl::display::shape::*;
use ensogl::display::traits::*;
use ensogl::display;
use ensogl::gui::component;

use super::node;



// ============
// === Edge ===
// ============

/// Canvas node shape definition.
pub mod shape {
    use super::*;

    ensogl::define_shape_system! {
        (radius:f32, start_angle:f32, angle:f32) {
            let radius = 1.px() * radius;
            let width  = LINE_WIDTH.px();
            let width2 = width / 2.0;
            let ring   = Circle(&radius + &width2) - Circle(radius-width2);
            let right : Var<f32> = (std::f32::consts::PI/2.0).into();
            let rot    = right - &angle/2.0;
            let mask   = Plane().cut_angle_fast(angle).rotate(rot);
            let shape  = ring * mask;
            let shape  = shape.fill(color::Rgba::from(color::Lcha::new(0.6,0.5,0.76,1.0)));
            shape.into()
        }
    }
}


/// Canvas node shape definition.
pub mod line {
    use super::*;

    ensogl::define_shape_system! {
        () {
            let width  = LINE_WIDTH.px();
            let height : Var<Distance<Pixels>> = "input_size.y".into();
            let shape  = Rect((width,height));
            let shape  = shape.fill(color::Rgba::from(color::Lcha::new(0.6,0.5,0.76,1.0)));
            shape.into()
        }
    }
}

/// Canvas node shape definition.
pub mod port_line {
    use super::*;

    ensogl::define_shape_system! {
        () {
            let width  = LINE_WIDTH.px();
            let height : Var<Distance<Pixels>> = "input_size.y".into();
            let shape  = Rect((width,height));
            let shape  = shape.fill(color::Rgba::from(color::Lcha::new(0.6,0.5,0.76,1.0)));
            shape.into()
        }
    }
}


/// Canvas node shape definition.
pub mod helper {
    use super::*;

    ensogl::define_shape_system! {
        () {
            let shape = Circle(2.px());
            let shape = shape.fill(color::Rgba::new(1.0,0.0,0.0,1.0));
            shape.into()
        }
    }
}


const LINE_WIDTH : f32 = 4.0;
const PADDING    : f32 = 5.0;



// ============
// === Edge ===
// ============

/// Edge definition.
#[derive(AsRef,Clone,CloneRef,Debug,Deref)]
pub struct Edge {
    data : Rc<EdgeData>,
}

impl AsRef<Edge> for Edge {
    fn as_ref(&self) -> &Self {
        self
    }
}


#[derive(Clone,CloneRef,Debug)]
pub struct InputEvents {
    pub network         : frp::Network,
    pub source_width    : frp::Source<f32>,
    pub target_position : frp::Source<frp::Position>,
}

impl InputEvents {
    pub fn new() -> Self {
        frp::new_network! { network
            def source_width    = source();
            def target_position = source();
        }
        Self {network,source_width,target_position}
    }
}

impl Default for InputEvents {
    fn default() -> Self {
        Self::new()
    }
}


pub fn sort_hack_1(scene:&Scene) {
    let logger = Logger::new("hack");
    component::ShapeView::<shape::Shape>::new(&logger,scene);
    component::ShapeView::<line::Shape>::new(&logger,scene);
}

pub fn sort_hack_2(scene:&Scene) {
    let logger = Logger::new("hack");
    component::ShapeView::<port_line::Shape>::new(&logger,scene);
}


/// Internal data of `Edge`
#[derive(Debug)]
#[allow(missing_docs)]
pub struct EdgeData {
    pub object          : display::object::Instance,
    pub logger          : Logger,
    pub events          : InputEvents,
    pub corner          : component::ShapeView<shape::Shape>,
    pub side_line       : component::ShapeView<line::Shape>,
    pub main_line       : component::ShapeView<line::Shape>,
    pub port_line       : component::ShapeView<port_line::Shape>,
    pub source_width    : Rc<Cell<f32>>,
    pub target_position : Rc<Cell<frp::Position>>,
}

impl Edge {
    /// Constructor.
    pub fn new(scene:&Scene) -> Self {
        let logger    = Logger::new("edge");
        let object    = display::object::Instance::new(&logger);

        let corner    = component::ShapeView::<shape::Shape>::new(&logger.sub("corner"),scene);
        let side_line = component::ShapeView::<line::Shape>::new(&logger.sub("side_line"),scene);
        let main_line = component::ShapeView::<line::Shape>::new(&logger.sub("main_line"),scene);
        let port_line = component::ShapeView::<port_line::Shape>::new(&logger.sub("port_line"),scene);
        object.add_child(&corner);
        object.add_child(&side_line);
        object.add_child(&main_line);
        object.add_child(&port_line);
        side_line.mod_rotation(|r| r.z = std::f32::consts::PI/2.0);

        let input = InputEvents::new();
        let network = &input.network;

        let source_width : Rc<Cell<f32>> = default();
        let target_position = Rc::new(Cell::new(frp::Position::default()));
        source_width.set(100.0);

        let port_line_height = node::NODE_HEIGHT/2.0 + node::SHADOW_SIZE;
        port_line.shape.sprite.size().set(Vector2::new(10.0,port_line_height));


        frp::extend! { network
            eval input.target_position ((t) target_position.set(*t));
            eval input.source_width    ((t) source_width.set(*t));
            on_change <-_ [input.source_width,input.target_position];
            eval_ on_change ([source_width,target_position,object,side_line,main_line,port_line,corner] {
                let target = target_position.get();
                let target = Vector2::new(target.x - object.position().x, target.y - object.position().y + port_line_height);
                let radius = 14.0;
                let width  = source_width.get() / 2.0;

                let side_circle_x = width - radius;
                let side          = target.x.signum();
                let target        = Vector2::new(target.x.abs(),target.y);

                let corner_grow   = ((target.x - width) * 0.6).max(0.0);
                let corner_radius = 20.0 + corner_grow;
                let corner_radius = corner_radius.min(target.y.abs());
                let corner_x      = target.x - corner_radius;


                let x = (corner_x - side_circle_x).clamp(-corner_radius,radius);
                let y = (radius*radius + corner_radius*corner_radius - x*x).sqrt();


                let angle1        = f32::atan2(y,x);
                let angle2        = f32::atan2(radius,corner_radius);
                let corner_angle  = std::f32::consts::PI - angle1 - angle2;
                let angle_overlap = if corner_x > width { 0.0 } else { 0.1 };

                corner.shape.angle.set((corner_angle + angle_overlap) * side);


                let corner_y    = - y;
                let corner_side = (corner_radius + PADDING) * 2.0;
                corner.shape.sprite.size().set(Vector2::new(corner_side,corner_side));
                corner.shape.radius.set(corner_radius);
                corner.mod_position(|t| t.x = corner_x * side);
                corner.mod_position(|t| t.y = corner_y);

                let line_overlap = 2.0;
                side_line.shape.sprite.size().set(Vector2::new(10.0,corner_x - width + line_overlap));
                side_line.mod_position(|p| p.x = side*(width + corner_x)/2.0);

                let main_line_x = side * target.x;
                main_line.shape.sprite.size().set(Vector2::new(10.0,corner_y - target.y + line_overlap));
                main_line.mod_position(|p| {
                    p.x = main_line_x;
                    p.y = (target.y + corner_y) / 2.0;
                });

                port_line.mod_position(|p| {
                    p.x = main_line_x;
                    p.y = target.y - port_line_height / 2.0;
                });
            });
        }

        let events = input;
        let data = Rc::new(EdgeData {object,logger,events,corner,side_line,main_line,port_line
                                          ,source_width,target_position});
        Self {data}
    }
}

impl display::Object for Edge {
    fn display_object(&self) -> &display::object::Instance {
        &self.object
    }
}
