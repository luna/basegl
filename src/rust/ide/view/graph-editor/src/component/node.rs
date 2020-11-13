//! Definition of the Node component.

#![allow(missing_docs)]
// WARNING! UNDER HEAVY DEVELOPMENT. EXPECT DRASTIC CHANGES.

#[deny(missing_docs)]
pub mod action_bar;
pub mod expression;
pub mod input;
pub mod output;

pub use expression::Expression;

use crate::prelude::*;

use enso_frp as frp;
use enso_frp;
use ensogl::application::Application;
use ensogl::data::color;
use ensogl::display::shape::*;
use ensogl::display::traits::*;
use ensogl::display;
use ensogl::gui::component::DEPRECATED_Animation;
use ensogl::gui::component;
use ensogl_text::Text;
use ensogl_theme;

use crate::Type;
use crate::component::visualization;

use super::edge;



// =================
// === Constants ===
// =================

pub const ACTION_BAR_HEIGHT : f32 = 15.0;
pub const CORNER_RADIUS     : f32 = 14.0;
pub const HEIGHT            : f32 = 28.0;
pub const PADDING           : f32 = 40.0;
pub const RADIUS            : f32 = 14.0;
pub const SHADOW_SIZE       : f32 = 10.0;
pub const TEXT_OFF          : f32 = 10.0;



// ============
// === Node ===
// ============

/// Canvas node shape definition.
pub mod shape {
    use super::*;

    ensogl::define_shape_system! {
        (style:Style, selection:f32, bg_color:Vector4 ) {
            use ensogl_theme::graph_editor::node as node_theme;

            let bg_color      = Var::<color::Rgba>::from(bg_color);
            let border_size_f = 16.0;

            let width  = Var::<Pixels>::from("input_size.x");
            let height = Var::<Pixels>::from("input_size.y");
            let width  = width  - PADDING.px() * 2.0;
            let height = height - PADDING.px() * 2.0;
            let radius = RADIUS.px();
            let shape  = Rect((&width,&height)).corners_radius(radius);
            let shape  = shape.fill(bg_color);


            // === Shadow ===

            let shadow_size   = SHADOW_SIZE.px();
            let shadow_width  = &width  + &shadow_size * 2.0;
            let shadow_height = &height + &shadow_size * 2.0;
            let shadow_radius = &shadow_height / 2.0;
            let shadow        = Rect((shadow_width,shadow_height)).corners_radius(shadow_radius);
            let base_color    = style.get_color(node_theme::shadow);
            let fading_color  = style.get_color(node_theme::shadow::fading);
            let exponent      = style.get_number_or(node_theme::shadow::exponent,2.0);
            let shadow_color  = color::LinearGradient::new()
                .add(0.0,color::Rgba::from(fading_color).into_linear())
                .add(1.0,color::Rgba::from(base_color).into_linear());
            let shadow_color = color::SdfSampler::new(shadow_color)
                .max_distance(border_size_f)
                .slope(color::Slope::Exponent(exponent));
            let shadow        = shadow.fill(shadow_color);


            // === Selection ===

            let sel_color = style.get_color(ensogl_theme::graph_editor::node::selection);
            let sel_size  = style.get_number_or(ensogl_theme::graph_editor::node::selection::size,9.0);

            let sel_offset  = 5.px();
            let sel_size    = sel_size.px();
            let sel_width   = &width  - 2.px() + &sel_offset * 2.0 * &selection;
            let sel_height  = &height - 2.px() + &sel_offset * 2.0 * &selection;
            let sel_radius  = &sel_height / 2.0;
            let select      = Rect((&sel_width,&sel_height)).corners_radius(&sel_radius);

            let sel2_width  = &width  - 2.px() + &sel_size * 2.0 * &selection;
            let sel2_height = &height - 2.px() + &sel_size * 2.0 * &selection;
            let sel2_radius = &sel2_height / 2.0;
            let select2     = Rect((&sel2_width,&sel2_height)).corners_radius(&sel2_radius);

            let select = select2 - select;
            let select = select.fill(color::Rgba::from(sel_color));


            // === Final Shape ===

            let out = select + shadow + shape;
            out.into()
        }
    }
}

pub mod drag_area {
    use super::*;

    ensogl::define_shape_system! {
        (style:Style) {
            let width  : Var<Pixels> = "input_size.x".into();
            let height : Var<Pixels> = "input_size.y".into();
            let width  = width  - PADDING.px() * 2.0;
            let height = height - PADDING.px() * 2.0;
            let radius = 14.px();
            let shape  = Rect((&width,&height)).corners_radius(radius);
            let shape  = shape.fill(color::Rgba::new(0.0,0.0,0.0,0.000_001));

            let out = shape;
            out.into()
        }
    }
}



// ===========
// === Frp ===
// ===========

ensogl::define_endpoints! {
    Input {
        select              (),
        deselect            (),
        set_visualization   (Option<visualization::Definition>),
        set_dimmed          (bool),
        set_input_connected (span_tree::Crumbs,bool),
        set_expression      (Expression),
        /// Set the expression USAGE type. This is not the definition type, which can be set with
        /// `set_expression` instead. In case the usage type is set to None, ports still may be
        /// colored if the definition type was present.
        set_expression_usage_type ((ast::Id,Option<Type>)),
    }
    Output {
        /// Press event. Emitted when user clicks on non-active part of the node, like its
        /// background. In edit mode, the whole node area is considered non-active.
        background_press (),
        expression (Text),
        skip       (bool),
        freeze     (bool),
    }
}



// ============
// === Node ===
// ============

/// Internal data of `Node`
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Node {
    pub model : Rc<NodeModel>,
    pub frp   : Frp,
}

impl AsRef<Node> for Node {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Deref for Node {
    type Target = Frp;
    fn deref(&self) -> &Self::Target {
        &self.frp
    }
}

/// Internal data of `Node`
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct NodeModel {
    pub app            : Application,
    pub display_object : display::object::Instance,
    pub logger         : Logger,
    pub main_area      : component::ShapeView<shape::Shape>,
    pub drag_area      : component::ShapeView<drag_area::Shape>,
    pub input          : input::Area,
    pub output         : output::Area,
    pub visualization  : visualization::Container,
    pub action_bar     : action_bar::ActionBar,
}


impl NodeModel {
    /// Constructor.
    pub fn new
    ( app            : &Application
    , registry       : visualization::Registry
    , inputs_enabled : &frp::Sampler<bool>
    ) -> Self {
        let scene  = app.display.scene();
        let logger = Logger::new("node");
        edge::depth_sort_hack_1(scene);

        output::area::depth_sort_hack(&scene);
        let main_logger = Logger::sub(&logger,"main_area");
        let drag_logger = Logger::sub(&logger,"drag_area");
        let main_area   = component::ShapeView::<shape::Shape>::new(&main_logger,scene);
        let drag_area   = component::ShapeView::<drag_area::Shape>::new(&drag_logger,scene);
        edge::depth_sort_hack_2(scene);

        input::area::depth_sort_hack(scene); // FIXME hack for sorting

        let display_object  = display::object::Instance::new(&logger);
        display_object.add_child(&drag_area);
        display_object.add_child(&main_area);

        // Disable shadows to allow interaction with the output port.
        let shape_system = scene.shapes.shape_system(PhantomData::<shape::Shape>);
        shape_system.shape_system.set_pointer_events(false);

        let input = input::Area::new(&logger,app,inputs_enabled);
        let scene = scene.clone_ref();
        let visualization = visualization::Container::new(&logger,&app,registry);
        visualization.mod_position(|t| {
            t.x = 60.0;
            t.y = -120.0;
        });

        display_object.add_child(&visualization);

        input.mod_position(|p| {
            p.x = TEXT_OFF;
            p.y = HEIGHT/2.0;
        });
        display_object.add_child(&input);

        let action_bar = action_bar::ActionBar::new(&app);
        display_object.add_child(&action_bar);
        action_bar.frp.show_icons();

        let output = output::Area::new(&scene);
        display_object.add_child(&output);

        let app = app.clone_ref();
        Self {app,display_object,logger,main_area,drag_area,output,input
             ,visualization,action_bar} . init()
    }

    fn init(self) -> Self {
        self.set_expression(Expression::debug_from_str("empty"));
        self
    }

    pub fn width(&self) -> f32 {
        self.input.width.value() + TEXT_OFF * 2.0
    }

    pub fn height(&self) -> f32 {
        HEIGHT
    }

    fn set_expression(&self, expr:impl Into<Expression>) {
        let expr = expr.into();
        self.output.set_pattern_span_tree(&expr.output_span_tree);
        self.input.set_expression(expr);
    }

    fn set_width(&self, width:f32) {
        let height = self.height();
        let width  = width + TEXT_OFF * 2.0;
        let size   = Vector2::new(width+PADDING*2.0, height+PADDING*2.0);
        self.main_area.shape.sprite.size.set(size);
        self.drag_area.shape.sprite.size.set(size);
        self.main_area.mod_position(|t| t.x = width/2.0);
        self.main_area.mod_position(|t| t.y = height/2.0);
        self.drag_area.mod_position(|t| t.x = width/2.0);
        self.drag_area.mod_position(|t| t.y = height/2.0);

        self.output.frp.set_size.emit(size);
        self.output.mod_position(|t| t.x = width/2.0);
        self.output.mod_position(|t| t.y = height/2.0);

        self.action_bar.mod_position(|t| {
            t.x = width/2.0 + CORNER_RADIUS;
            t.y = height + ACTION_BAR_HEIGHT;
        });
        self.action_bar.frp.set_size(Vector2::new(width,ACTION_BAR_HEIGHT));
    }

    pub fn visualization(&self) -> &visualization::Container {
        &self.visualization
    }
}

impl Node {
    pub fn new
    ( app            : &Application
    , registry       : visualization::Registry
    , inputs_enabled : &frp::Sampler<bool>
    ) -> Self {
        let frp       = Frp::new();
        let network   = &frp.network;
        let inputs    = &frp.input;
        let out       = &frp.output;
        let model     = Rc::new(NodeModel::new(app,registry,inputs_enabled));
        let selection = DEPRECATED_Animation::<f32>::new(network);

        let bg_color_anim = color::Animation::new(network);
        let style         = StyleWatch::new(&app.display.scene().style_sheet);
        let actions       = &model.action_bar.frp;

        frp::extend! { network
            animations <- source::<()>();
            eval_ animations([bg_color_anim]{
                let _bg_color_anim = &bg_color_anim;
            });

            eval  selection.value ((v) model.main_area.shape.selection.set(*v));
            eval_ inputs.select   (selection.set_target_value(1.0));
            eval_ inputs.deselect (selection.set_target_value(0.0));

            model.input.set_connected       <+ inputs.set_input_connected;
            model.input.set_expression_usage_type <+ inputs.set_expression_usage_type;
            eval inputs.set_expression ((expr) model.set_expression(expr));

            eval inputs.set_visualization ((content)
                model.visualization.frp.set_visualization.emit(content)
            );

            eval model.input.frp.width ((w) model.set_width(*w));

            out.source.background_press <+ model.drag_area.events.mouse_down;

            eval_ model.drag_area.events.mouse_over (model.input.set_hover(true));
            eval_ model.drag_area.events.mouse_out  (model.input.set_hover(false));

            out.source.expression <+ model.input.frp.expression.map(|t|t.clone_ref());

            eval actions.action_visbility ((visible){
                model.visualization.frp.set_visibility.emit(visible);
            });

            out.source.skip   <+ actions.action_skip;
            out.source.freeze <+ actions.action_freeze;


            // === Color Handling ===

            bg_color <- inputs.set_dimmed.map(f!([model,style](should_dim) {
                model.input.frp.set_dimmed.emit(*should_dim);
                let bg_color_path = ensogl_theme::graph_editor::node::background;
                if *should_dim {
                   style.get_color_dim(bg_color_path)
                 } else {
                   style.get_color(bg_color_path)
                 }
            }));
            trace bg_color;

            bg_color_anim.target <+ bg_color;
            trace bg_color_anim.value;
            eval bg_color_anim.value ((c)
                model.main_area.shape.bg_color.set(color::Rgba::from(c).into())
            );


            trace model.drag_area.events.mouse_over;
            trace model.drag_area.events.mouse_out;


            // === Action Bar ===

            // eval_ model.main_area.events.mouse_over  ( actions.show_icons() );
            // eval_ model.main_area.events.mouse_out   ( actions.hide_icons() );
            // eval_ model.drag_area.events.mouse_over  ( actions.show_icons() );
            // eval_ model.drag_area.events.mouse_out   ( actions.hide_icons() );

            // FIXME[WD]: this is strange! Input.hover is a very misleading name
            is_hovered <- model.input.frp.port_hover.map(|t|t.is_on() );
            eval is_hovered ((hovered) actions.icon_visibility(hovered) );
        }

        model.action_bar.frp.hide_icons.emit(());
        frp.set_dimmed.emit(false);

        Self {frp,model}
    }
}

impl display::Object for Node {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.display_object
    }
}
