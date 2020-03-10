//! This module contains the Css3dRenderer, a struct used to render CSS3D elements.

use crate::prelude::*;

use crate::display;
use crate::display::camera::Camera2d;
use crate::display::camera::camera2d::Projection;
use crate::system::web::dom::html::{Css3dObject, Css3dSystem};
use crate::system::gpu::data::JsBufferView;
use crate::system::web;
use crate::system::web::Result;
use crate::system::web::dyn_into;
use crate::system::web::NodeInserter;
use crate::system::web::NodeRemover;
use crate::system::web::StyleSetter;
use crate::system::web::dom::DomContainer;
use crate::system::web::dom::ResizeCallback;
use crate::system::web::get_element_by_id;
use super::css3d_object::Css3dOrder;

use nalgebra::Vector2;
use nalgebra::Matrix4;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use web_sys::HtmlElement;
use web_sys::HtmlDivElement;
use js_sys::Object;


// ===================
// === Js Bindings ===
// ===================

mod js {
    use super::*;
    #[wasm_bindgen(inline_js = "
        function arr_to_css_matrix3d(a) {
            return 'matrix3d(' + a.join(',') + ')'
        }

        export function set_object_transform(dom, matrix_array) {
            let css = arr_to_css_matrix3d(matrix_array);
            dom.style.transform = 'translate(-50%, -50%)' + css;
        }

        export function setup_perspective(dom, perspective) {
            dom.style.perspective = perspective + 'px';
        }

        export function setup_camera_orthographic(dom, matrix_array) {
            dom.style.transform = arr_to_css_matrix3d(matrix_array);
        }

        export function setup_camera_perspective
        (dom, near, matrix_array) {
            let translateZ  = 'translateZ(' + near + 'px)';
            let matrix3d    = arr_to_css_matrix3d(matrix_array);
            let transform   = translateZ + matrix3d;
            dom.style.transform = transform;
        }
    ")]
    extern "C" {
        /// Setup perspective CSS 3D projection on DOM.
        #[allow(unsafe_code)]
        pub fn setup_perspective(dom: &JsValue, znear: &JsValue);

        /// Setup Camera orthographic projection on DOM.
        #[allow(unsafe_code)]
        pub fn setup_camera_orthographic(dom:&JsValue, matrix_array:&JsValue);

        /// Setup Camera perspective projection on DOM.
        #[allow(unsafe_code)]
        pub fn setup_camera_perspective(dom:&JsValue, near:&JsValue, matrix_array:&JsValue);

        /// Sets object's CSS 3D transform.
        #[allow(unsafe_code)]
        pub fn set_object_transform(dom:&JsValue, matrix_array:&Object);
    }
}

#[allow(unsafe_code)]
fn set_object_transform(dom:&JsValue, matrix:&Matrix4<f32>) {
    // Views to WASM memory are only valid as long the backing buffer isn't
    // resized. Check documentation of IntoFloat32ArrayView trait for more
    // details.
    unsafe {
        let matrix_array = matrix.js_buffer_view();
        js::set_object_transform(&dom,&matrix_array);
    }
}


#[allow(unsafe_code)]
fn setup_camera_perspective(dom:&JsValue, near:f32, matrix:&Matrix4<f32>) {
    // Views to WASM memory are only valid as long the backing buffer isn't
    // resized. Check documentation of IntoFloat32ArrayView trait for more
    // details.
    unsafe {
        let matrix_array = matrix.js_buffer_view();
        js::setup_camera_perspective(
            &dom,
            &near.into(),
            &matrix_array
        )
    }
}

#[allow(unsafe_code)]
fn setup_camera_orthographic(dom:&JsValue, matrix:&Matrix4<f32>) {
    // Views to WASM memory are only valid as long the backing buffer isn't
    // resized. Check documentation of IntoFloat32ArrayView trait for more
    // details.
    unsafe {
        let matrix_array = matrix.js_buffer_view();
        js::setup_camera_orthographic(&dom, &matrix_array)
    }
}



// =============
// === Utils ===
// =============

/// Inverts Matrix Y coordinates. It's equivalent to scaling by (1.0, -1.0, 1.0).
pub fn invert_y(mut m: Matrix4<f32>) -> Matrix4<f32> {
    // Negating the second column to invert Y.
    m.row_part_mut(1, 4).iter_mut().for_each(|a| *a = -*a);
    m
}



// =========================
// === Css3dRendererData ===
// =========================

#[derive(Debug)]
struct Css3dRendererData {
    pub front_dom                 : HtmlDivElement,
    pub back_dom                  : HtmlDivElement,
    pub front_dom_view_projection : HtmlDivElement,
    pub back_dom_view_projection  : HtmlDivElement,
    logger                        : Logger
}

impl Css3dRendererData {
    pub fn new
    ( front_dom                 : HtmlDivElement
    , back_dom                  : HtmlDivElement
    , front_dom_view_projection : HtmlDivElement
    , back_dom_view_projection  : HtmlDivElement
    , logger                    : Logger) -> Self {
        Self {logger,front_dom,back_dom, front_dom_view_projection, back_dom_view_projection }
    }

    fn set_dimensions(&self, dimensions:Vector2<f32>) {
        let width  = format!("{}px", dimensions.x);
        let height = format!("{}px", dimensions.y);
        let doms   = vec![&self.front_dom, &self.back_dom, &self.front_dom_view_projection, &self.back_dom_view_projection];
        for dom in doms {
            dom.set_style_or_warn("width" , &width, &self.logger);
            dom.set_style_or_warn("height", &height, &self.logger);
        }
    }
}



// =====================
// === Css3dRenderer ===
// =====================

/// `Css3dRenderer` is a renderer for `Css3dObject`s. It integrates with other rendering contexts,
/// such as WebGL, by placing two HtmlElements in front and behind of the Canvas element,
/// allowing the move `Css3dObject`s between these two layers, mimicking z-index ordering.
///
/// To make use of its functionalities, the API user can create a `Css3dSystem` by using
/// the `Css3dRenderer::new_system` method which creates and manages instances of
/// `Css3dObject`s.
#[derive(Clone,Debug)]
pub struct Css3dRenderer {
    container : DomContainer,
    data      : Rc<Css3dRendererData>
}

impl Css3dRenderer {
    /// Creates a Css3dRenderer inside an element.
    pub fn from_element_or_panic(logger:&Logger, element:HtmlElement) -> Self {
        let logger    = logger.sub("Css3dRenderer");
        let container = DomContainer::from_element(element);
        let (front_dom , front_dom_view_projection) = Self::create_layer(&logger);
        let (back_dom  , back_dom_view_projection)  = Self::create_layer(&logger);

        back_dom.set_style_or_warn("z-index","-1",&logger);
        container.dom.append_or_warn(&front_dom,&logger);
        container.dom.append_or_warn(&back_dom,&logger);

        let data = Css3dRendererData::new
            (front_dom,back_dom,front_dom_view_projection,back_dom_view_projection,logger);
        let data = Rc::new(data);
        Self{container,data}.init()
    }

    fn create_layer(logger:&Logger) -> (HtmlDivElement,HtmlDivElement) {
        let dom                 = web::create_div();
        let dom_view_projection = web::create_div();

        dom.set_style_or_warn("position"       , "absolute" , &logger);
        dom.set_style_or_warn("top"            , "0px"      , &logger);
        dom.set_style_or_warn("overflow"       , "hidden"   , &logger);
        dom.set_style_or_warn("overflow"       , "hidden"   , &logger);
        dom.set_style_or_warn("width"          , "100%"     , &logger);
        dom.set_style_or_warn("height"         , "100%"     , &logger);
        dom.set_style_or_warn("pointer-events" , "none"     , &logger);

        dom_view_projection.set_style_or_warn("width"           , "100%"        , &logger);
        dom_view_projection.set_style_or_warn("height"          , "100%"        , &logger);
        dom_view_projection.set_style_or_warn("transform-style" , "preserve-3d" , &logger);

        dom.append_or_warn(&dom_view_projection,&logger);
        return (dom,dom_view_projection);
    }

    /// Creates a Css3dRenderer.
    pub fn new(logger:&Logger, dom_id:&str) -> Result<Self> {
        Ok(Self::from_element_or_panic(logger,dyn_into(get_element_by_id(dom_id)?)?))
    }

    pub(super) fn new_system(&self) -> Css3dSystem {
        let css3d_renderer = self.clone();
        let logger         = self.data.logger.sub("Css3dSystem");
        let display_object = display::object::Node::new(&logger);
        Css3dSystem {display_object,css3d_renderer,logger}
    }

    fn init(mut self) -> Self {
        let data = self.data.clone();
        self.add_resize_callback(move |dimensions:&Vector2<f32>| {
            data.set_dimensions(*dimensions);
        });
        self
    }

    /// Creates a new instance of Css3dObject and adds it to parent.
    pub(super) fn new_instance
    (&self,object:&Css3dObject) {
        let front_layer = self.data.front_dom_view_projection.clone();
        let back_layer  = self.data.back_dom_view_projection.clone();
        let display_object : display::object::Node = object.into();
        display_object.set_on_updated(enclose!((object) move |t| {
            let object_dom    = object.dom();
            let mut transform = t.matrix();
            transform.iter_mut().for_each(|a| *a = eps(*a));

            let layer = match object.css3d_order() {
                Css3dOrder::Front => &front_layer,
                Css3dOrder::Back  => &back_layer
            };

            let parent_node = object.dom().parent_node();
            if !layer.is_same_node(parent_node.as_ref()) {
//                display_object.with_logger(|logger| {
                    let logger = Logger::new("tmp");
                    object_dom.remove_from_parent_or_warn(&logger);
                    layer.append_or_warn(&object_dom,&logger);
//                });
            }

            set_object_transform(&object_dom, &transform);
        }));
    }

    /// Update the objects to match the new camera's point of view. This function should be called
    /// only after camera position change.
    pub fn update(&self, camera:&Camera2d) {
        let trans_cam  = camera.transform().matrix().try_inverse();
        let trans_cam  = trans_cam.expect("Camera's matrix is not invertible.");
        let trans_cam  = trans_cam.map(eps);
        let trans_cam  = invert_y(trans_cam);
        let half_dim   = self.container.dimensions() / 2.0;
        let fovy_slope = camera.half_fovy_slope();
        let near       = half_dim.y / fovy_slope;

        match camera.projection() {
            Projection::Perspective{..} => {
                js::setup_perspective(&self.data.front_dom , &near.into());
                js::setup_perspective(&self.data.back_dom  , &near.into());
                setup_camera_perspective(&self.data.front_dom_view_projection , near, &trans_cam);
                setup_camera_perspective(&self.data.back_dom_view_projection  , near, &trans_cam);
            },
            Projection::Orthographic => {
                setup_camera_orthographic(&self.data.front_dom_view_projection , &trans_cam);
                setup_camera_orthographic(&self.data.back_dom_view_projection  , &trans_cam);
            }
        }
    }

    /// Adds a ResizeCallback.
    pub fn add_resize_callback<T:ResizeCallback>(&mut self, callback:T) {
        self.container.add_resize_callback(callback);
    }

    /// Sets Css3dRenderer's container dimensions.
    pub fn set_dimensions(&mut self, dimensions:Vector2<f32>) {
        self.data.set_dimensions(dimensions);
        self.container.set_dimensions(dimensions);
    }
}


// === Getters ===

impl Css3dRenderer {
    /// Gets Css3dRenderer's container.
    pub fn container(&self) -> &DomContainer {
        &self.container
    }

    /// Gets Css3dRenderer's DOM.
    pub fn dom(&self) -> &HtmlElement {
        &self.data.front_dom
    }

    /// Gets the Css3dRenderer's dimensions.
    pub fn dimensions(&self) -> Vector2<f32> {
        self.container.dimensions()
    }
}



// =============
// === Utils ===
// =============

/// eps is used to round very small values to 0.0 for numerical stability
pub fn eps(value: f32) -> f32 {
    if value.abs() < 1e-10 { 0.0 } else { value }
}
