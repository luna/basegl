#![allow(missing_docs)]

use crate::prelude::*;

pub use crate::display::symbol::registry::SymbolId;

use crate::closure;
use crate::control::callback::CallbackHandle;
use crate::control::callback::DynEvent;
use crate::control::io::mouse::MouseFrpCallbackHandles;
use crate::control::io::mouse::MouseManager;
use crate::control::io::mouse;
use crate::data::dirty::traits::*;
use crate::data::dirty;
use crate::debug::stats::Stats;
use crate::display;
use crate::display::camera::Camera2d;
use crate::display::object::DisplayObjectOps;
use crate::display::render::RenderComposer;
use crate::display::render::RenderPipeline;
use crate::display::symbol::registry::SymbolRegistry;
use crate::display::symbol::Symbol;
use crate::system::gpu::data::uniform::Uniform;
use crate::system::gpu::data::uniform::UniformScope;
use crate::system::gpu::shader::Context;
use crate::system::gpu::types::*;
use crate::system::web::dom::html::Css3dRenderer;
use crate::system::web::dyn_into;
use crate::system::web::resize_observer::ResizeObserver;
use crate::system::web::StyleSetter;
use crate::system::web;
use crate::system::web::NodeInserter;

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsValue;
use web_sys::HtmlElement;



// =============
// === Error ===
// =============

#[derive(Debug, Fail, From)]
pub enum Error {
    #[fail(display = "Web Platform error: {}", error)]
    WebError { error: web::Error },
}



// =============
// === Shape ===
// =============

// === Shape ===

#[derive(Clone,Debug)]
pub struct Shape {
    rc: Rc<RefCell<ShapeData>>
}

impl Shape {
    pub fn new(width:f32, height:f32) -> Shape {
        let rc = Rc::new(RefCell::new(ShapeData::new(width,height)));
        Self {rc}
    }

    pub fn from_element(element:&HtmlElement) -> Self {
        let bounding_box = element.get_bounding_client_rect();
        let width        = bounding_box.width() as f32;
        let height       = bounding_box.height() as f32;
        Self::new(width,height)
    }

    pub fn from_window(window:&web_sys::Window) -> Self {
        let width  = window.inner_width().unwrap().as_f64().unwrap() as f32;
        let height = window.inner_height().unwrap().as_f64().unwrap() as f32;
        Self::new(width,height)
    }


    pub fn screen_shape(&self) -> ShapeData {
        *self.rc.borrow()
    }

    pub fn canvas_shape(&self) -> ShapeData {
        let mut shape = self.screen_shape();
        shape.width  *= shape.pixel_ratio;
        shape.height *= shape.pixel_ratio;
        shape
    }

    pub fn set_screen_dimension(&self, width:f32, height:f32) {
        self.rc.borrow_mut().set_screen_dimension(width,height);
    }

    pub fn pixel_ratio(&self) -> f32 {
        self.rc.borrow().pixel_ratio
    }
}

impl CloneRef for Shape {}


// === ShapeData ===

#[derive(Clone,Copy,Debug)]
pub struct ShapeData {
    pub width       : f32,
    pub height      : f32,
    pub pixel_ratio : f32
}

impl ShapeData {
    pub fn new(width:f32, height:f32) -> ShapeData {
        let pixel_ratio = web::device_pixel_ratio().unwrap_or(1.0) as f32;
        Self{width,height,pixel_ratio}
    }

    pub fn set_screen_dimension(&mut self, width:f32, height:f32) {
        self.width  = width;
        self.height = height;
    }
}



// ======================
// === Mouse Handling ===
// ======================

pub trait MouseEventFn      = Fn(JsValue) + 'static;
pub type  MouseEventClosure = Closure<dyn Fn(JsValue)>;

fn mouse_event_closure<F:MouseEventFn>(f:F) -> MouseEventClosure {
    Closure::wrap(Box::new(f))
}

#[derive(Debug)]
struct Mouse {
    mouse_manager   : MouseManager,
    position        : Uniform<Vector2<i32>>,
    hover_ids       : Uniform<Vector4<u32>>,
    button0_pressed : Uniform<bool>,
    button1_pressed : Uniform<bool>,
    button2_pressed : Uniform<bool>,
    button3_pressed : Uniform<bool>,
    button4_pressed : Uniform<bool>,
    last_hover_ids  : Vector4<u32>,
    handles         : Vec<CallbackHandle>,
}

impl Mouse {
    pub fn new(shape:&Shape, variables:&UniformScope) -> Self {

        let empty_hover_ids = Vector4::<u32>::new(0,0,0,0);
        let position        = variables.add_or_panic("mouse_position",Vector2::new(0,0));
        let hover_ids       = variables.add_or_panic("mouse_hover_ids",empty_hover_ids);
        let button0_pressed = variables.add_or_panic("mouse_button0_pressed",false);
        let button1_pressed = variables.add_or_panic("mouse_button1_pressed",false);
        let button2_pressed = variables.add_or_panic("mouse_button2_pressed",false);
        let button3_pressed = variables.add_or_panic("mouse_button3_pressed",false);
        let button4_pressed = variables.add_or_panic("mouse_button4_pressed",false);
        let last_hover_ids  = empty_hover_ids;
        let document        = web::document().unwrap();
        let mouse_manager   = MouseManager::new(&document);

        let shape_ref       = shape.clone_ref();
        let position_ref    = position.clone_ref();
        let on_move_handle  = mouse_manager.on_move.add(move |event:&mouse::event::OnMove| {
            let pixel_ratio = shape_ref.pixel_ratio() as i32;
            let screen_x    = event.offset_x();
            let screen_y    = shape_ref.screen_shape().height as i32 - event.offset_y();
            let canvas_x    = pixel_ratio * screen_x;
            let canvas_y    = pixel_ratio * screen_y;
            position_ref.set(Vector2::new(canvas_x,canvas_y))
        });

        let button0_pressed_ref = button0_pressed.clone_ref();
        let button1_pressed_ref = button1_pressed.clone_ref();
        let button2_pressed_ref = button2_pressed.clone_ref();
        let button3_pressed_ref = button3_pressed.clone_ref();
        let button4_pressed_ref = button4_pressed.clone_ref();
        let on_down_handle      = mouse_manager.on_down.add(move |event:&mouse::event::OnDown| {
            match event.button() {
                mouse::Button0 => button0_pressed_ref.set(true),
                mouse::Button1 => button1_pressed_ref.set(true),
                mouse::Button2 => button2_pressed_ref.set(true),
                mouse::Button3 => button3_pressed_ref.set(true),
                mouse::Button4 => button4_pressed_ref.set(true),
            }
        });

        let button0_pressed_ref = button0_pressed.clone_ref();
        let button1_pressed_ref = button1_pressed.clone_ref();
        let button2_pressed_ref = button2_pressed.clone_ref();
        let button3_pressed_ref = button3_pressed.clone_ref();
        let button4_pressed_ref = button4_pressed.clone_ref();
        let on_up_handle        = mouse_manager.on_up.add(move |event:&mouse::event::OnUp| {
            match event.button() {
                mouse::Button0 => button0_pressed_ref.set(false),
                mouse::Button1 => button1_pressed_ref.set(false),
                mouse::Button2 => button2_pressed_ref.set(false),
                mouse::Button3 => button3_pressed_ref.set(false),
                mouse::Button4 => button4_pressed_ref.set(false),
            }
        });

        let handles = vec![on_move_handle,on_down_handle,on_up_handle];

        Self {mouse_manager,position,hover_ids,button0_pressed,button1_pressed,button2_pressed,button3_pressed
             ,button4_pressed,last_hover_ids,handles}
    }
}



// ===========
// === Dom ===
// ===========

/// DOM element manager
#[derive(Debug)]
pub struct Dom {
    /// Root DOM element of the scene.
    pub root : web_sys::HtmlDivElement,
    /// Layers of the scene.
    pub layers : Layers,
}

impl Dom {
    /// Constructor.
    pub fn new(logger:&Logger) -> Self {
        let root   = web::create_div();
        let layers = Layers::new(&logger,&root);
        root.set_style_or_panic("height"  , "100vh");
        root.set_style_or_panic("width"   , "100vw");
        root.set_style_or_panic("display" , "block");
        Self {root,layers}
    }
}



// ==============
// === Layers ===
// ==============

/// DOM Layers of the scene. It contains a 2 CSS 3D layers and a canvas layer in the middle. The
/// CSS layers are used to manage DOM elements and to simulate depth-sorting of DOM and canvas
/// elements.
#[derive(Debug)]
pub struct Layers {
    /// Front DOM scene layer.
    pub dom_front : Css3dRenderer,
    /// The WebGL scene layer.
    pub canvas : web_sys::HtmlCanvasElement,
    /// Back DOM scene layer.
    pub dom_back : Css3dRenderer,
}

impl Layers {
    /// Constructor.
    pub fn new(logger:&Logger, dom:&web_sys::HtmlDivElement) -> Self {
        let canvas    = web::create_canvas();
        let dom_front = Css3dRenderer::new(&logger);
        let dom_back  = Css3dRenderer::new(&logger);
        canvas.set_style_or_panic("height"  , "100vh");
        canvas.set_style_or_panic("width"   , "100vw");
        canvas.set_style_or_panic("display" , "block");
        dom.append_or_panic(&dom_front.dom);
        dom.append_or_panic(&canvas);
        dom.append_or_panic(&dom_back.dom);
        dom_back.set_z_index(-1);
        Self {dom_front,canvas,dom_back}
    }
}



// =============
// === Scene ===
// =============

shared! { Scene
#[derive(Derivative)]
#[derivative(Debug)]
pub struct SceneData {
    display_object : display::object::Node,
    dom            : Dom,
    context        : Context,
    symbols        : SymbolRegistry,
    symbols_dirty  : SymbolRegistryDirty,
    camera         : Camera2d,
    shape          : Shape,
    shape_dirty    : ShapeDirty,
    logger         : Logger,
    listeners      : Listeners,
    variables      : UniformScope,
    pipeline       : RenderPipeline,
    composer       : RenderComposer,
    stats          : Stats,
    pixel_ratio    : Uniform<f32>,
    zoom_uniform   : Uniform<f32>,
    zoom_callback  : CallbackHandle,
    mouse          : Mouse,
    #[derivative(Debug="ignore")]
    on_resize: Option<Box<dyn Fn(&Shape)>>,
}

impl {
    /// Create new instance with the provided on-dirty callback.
    pub fn new<OnMut:Fn()+Clone+'static>
    (parent_dom:&HtmlElement, logger:Logger, stats:&Stats, on_mut:OnMut) -> Self {
        logger.trace("Initializing.");
        let display_object  = display::object::Node::new(&logger);
        let dom             = Dom::new(&logger);
        parent_dom.append_child(&dom.root).unwrap();
        let context         = web::get_webgl2_context(&dom.layers.canvas).unwrap();
        let sub_logger      = logger.sub("shape_dirty");
        let shape_dirty     = ShapeDirty::new(sub_logger,Box::new(on_mut.clone()));
        let sub_logger      = logger.sub("symbols_dirty");
        let dirty_flag      = SymbolRegistryDirty::new(sub_logger,Box::new(on_mut));
        let on_change       = symbols_on_change(dirty_flag.clone_ref());
        let sub_logger      = logger.sub("symbols");
        let variables       = UniformScope::new(logger.sub("global_variables"),&context);
        let symbols         = SymbolRegistry::new(&variables,&stats,&context,sub_logger,on_change);
        let shape           = Shape::from_element(&dom.root);
        let screen_shape    = shape.screen_shape();
        let width           = screen_shape.width;
        let height          = screen_shape.height;
        let listeners       = Self::init_listeners(&logger,&dom.layers.canvas,&shape,&shape_dirty);
        let symbols_dirty   = dirty_flag;
        let camera          = Camera2d::new(logger.sub("camera"),width,height);
        let zoom_uniform    = variables.add_or_panic("zoom", 1.0);
        let on_resize       = default();
        let stats           = stats.clone();
        let pixel_ratio     = variables.add_or_panic("pixel_ratio", shape.pixel_ratio());
        let mouse           = Mouse::new(&shape,&variables);
        let zoom_uniform_cp = zoom_uniform.clone();
        let zoom_callback   = camera.add_zoom_update_callback(
            move |zoom:&f32| zoom_uniform_cp.set(*zoom)
        );

        context.enable(Context::BLEND);
        // To learn more about the blending equations used here, please see the following articles:
        // - http://www.realtimerendering.com/blog/gpus-prefer-premultiplication
        // - https://www.khronos.org/opengl/wiki/Blending#Colors
        context.blend_equation_separate ( Context::FUNC_ADD, Context::FUNC_ADD );
        context.blend_func_separate     ( Context::ONE , Context::ONE_MINUS_SRC_ALPHA
                                        , Context::ONE , Context::ONE_MINUS_SRC_ALPHA );

        let pipeline = default();
        let width    = shape.canvas_shape().width  as i32;
        let height   = shape.canvas_shape().height as i32;
        let composer = RenderComposer::new(&pipeline,&context,&variables,width,height);

        Self { pipeline,composer,display_object,dom,context,symbols,camera,symbols_dirty,shape,shape_dirty
             , logger,listeners,variables,on_resize,stats,pixel_ratio,mouse,zoom_uniform
             ,zoom_callback }
    }

    pub fn symbol_registry(&self) -> SymbolRegistry {
        self.symbols.clone_ref()
    }

    pub fn css3d_renderer(&self) -> Css3dRenderer {
        self.dom.layers.dom_front.clone()
    }

    pub fn canvas(&self) -> web_sys::HtmlCanvasElement {
        self.dom.layers.canvas.clone()
    }

    pub fn context(&self) -> Context {
        self.context.clone()
    }

    pub fn variables(&self) -> UniformScope {
        self.variables.clone_ref()
    }

    pub fn mouse_position_uniform(&self) -> Uniform<Vector2<i32>> {
        self.mouse.position.clone_ref()
    }

    pub fn mouse_hover_ids(&self) -> Uniform<Vector4<u32>> {
        self.mouse.hover_ids.clone_ref()
    }

    pub fn set_render_pipeline<P:Into<RenderPipeline>>(&mut self, pipeline:P) {
        self.pipeline = pipeline.into();
        self.init_composer();
    }

    pub fn init_composer(&mut self) {
        let width    = self.shape.canvas_shape().width  as i32;
        let height   = self.shape.canvas_shape().height as i32;
        self.composer = RenderComposer::new(&self.pipeline,&self.context,&self.variables,width,height);
    }

    pub fn render2(&self) {
        self.symbols.render2()
    }


    pub fn render(&mut self) {
        let mouse_hover_ids = self.mouse.hover_ids.get();
        if mouse_hover_ids != self.mouse.last_hover_ids {
            self.mouse.last_hover_ids = mouse_hover_ids;
            let is_not_background = mouse_hover_ids.w != 0;
            if is_not_background {
                let symbol_id = mouse_hover_ids.x;
                let symbol = self.symbols.index(symbol_id as usize);
                symbol.dispatch(&DynEvent::new(()));
                // println!("{:?}",self.mouse.hover_ids.get());
                // TODO: finish events sending, including OnOver and OnOut.
            }
        }

        group!(self.logger, "Updating.", {
            if self.shape_dirty.check_all() {
                let screen = self.shape.screen_shape();
                self.resize_canvas(&self.shape);
                self.camera.set_screen(screen.width, screen.height);
                self.init_composer();
                self.shape_dirty.unset_all();
            }
            if self.symbols_dirty.check_all() {
                self.symbols.update();
                self.symbols_dirty.unset_all();
            }
            self.logger.info("Rendering meshes.");
            let camera_changed = self.camera.update();
            if camera_changed {
                self.symbols.render(&self.camera);
                self.dom.layers.dom_front.update(&self.camera);
                self.dom.layers.dom_back.update(&self.camera);
            }
            self.composer.run();
        })
    }

    /// Bind FRP graph to mouse js events.
    pub fn bind_frp_to_mouse_events(&self, frp:&enso_frp::Mouse) -> MouseFrpCallbackHandles {
        mouse::bind_frp_to_mouse(&self.shape,frp,&self.mouse.mouse_manager)
    }

    /// Check dirty flags and update the state accordingly.
    pub fn update(&mut self) {
        self.display_object.update();
        self.render();
    }

    pub fn camera(&self) -> Camera2d {
        self.camera.clone_ref()
    }

    pub fn stats(&self) -> Stats {
        self.stats.clone_ref()
    }

    pub fn index(&self, ix:usize) -> Symbol {
        self.symbols.index(ix)
    }

    /// Create a new `Symbol` instance.
    pub fn new_symbol(&self) -> Symbol {
        self.symbols.new_symbol()
    }
}}

impl Into<display::object::Node> for &SceneData {
    fn into(self) -> display::object::Node {
        self.display_object.clone()
    }
}

impl Into<display::object::Node> for &Scene {
    fn into(self) -> display::object::Node {
        let data:&SceneData = &self.rc.borrow();
        data.into()
    }
}


// === Types ===

pub type ShapeDirty          = dirty::SharedBool<Box<dyn Fn()>>;
pub type SymbolRegistryDirty = dirty::SharedBool<Box<dyn Fn()>>;


// === Callbacks ===

closure! {
fn symbols_on_change(dirty:SymbolRegistryDirty) -> OnSymbolRegistryChange {
    || dirty.set()
}}


// === Implementation ===

#[derive(Debug)]
pub struct Listeners {
    resize: ResizeObserver,
}

impl Scene {
    pub fn tmp_borrow_mut(&self) -> RefMut<'_,SceneData> {
        self.rc.borrow_mut()
    }
}

impl SceneData {
    /// Initialize all listeners and attach them to DOM elements.
    fn init_listeners
    (logger:&Logger, canvas:&web_sys::HtmlCanvasElement, shape:&Shape, dirty:&ShapeDirty)
    -> Listeners {
        let logger = logger.clone();
        let shape  = shape.clone();
        let dirty  = dirty.clone();
        let on_resize = Closure::new(move |width, height| {
            group!(logger, "Resize observer event.", {
                shape.set_screen_dimension(width as f32,height as f32);
                dirty.set();
            })
        });
        let resize = ResizeObserver::new(canvas,on_resize);
        Listeners {resize}
    }



    /// Resize the underlying canvas. This function should rather not be called
    /// directly. If you want to change the canvas size, modify the `shape` and
    /// set the dirty flag.
    fn resize_canvas(&self, shape:&Shape) {
        let screen = shape.screen_shape();
        let canvas = shape.canvas_shape();
        self.logger.group(fmt!("Resized to {}px x {}px.", screen.width, screen.height), || {
            self.dom.layers.canvas.set_attribute("width",  &canvas.width.to_string()).unwrap();
            self.dom.layers.canvas.set_attribute("height", &canvas.height.to_string()).unwrap();
            self.context.viewport(0,0,canvas.width as i32, canvas.height as i32);
            self.on_resize.iter().for_each(|f| f(shape));
        });
    }
}
