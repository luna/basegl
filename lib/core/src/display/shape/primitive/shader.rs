//! Shader primitives used to render vector shapes on GPU.

use crate::prelude::*;

use std::include_str;
use inflector::Inflector;
use crate::display::symbol::geometry::primitive::mesh::buffer::item::Item;
use nalgebra::Vector2;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;


const GLSL_DEFS:&str = include_str!("shader/defs.glsl");


fn mk_bb_name    <S:Str> (s:S) -> String { format!("{}_bb"    , s.as_ref()) }
fn mk_id_name    <S:Str> (s:S) -> String { format!("{}_id"    , s.as_ref()) }
fn mk_cd_name    <S:Str> (s:S) -> String { format!("{}_cd"    , s.as_ref()) }
fn mk_sdf_name   <S:Str> (s:S) -> String { format!("{}_sdf"   , s.as_ref()) }
fn mk_shape_name <S:Str> (s:S) -> String { format!("shape_{}" , s.as_ref()) }

//defCdC = Color.rgb [1,0,0,1]
//defCd  = "rgb2lch(#{GLSL.toCode defCdC})"



// ================
// === GlslItem ===
// ================

/// Trait describing all types which can be converted to GLSL expressions.
///
/// Please note that conversion from string is defined, allowing dirty injection of GLSL code
/// easily. For example, when moving a shape, you can write `s1.translate("a","b")`, where `a` and
/// `b` refer to variables defined in the GLSL shader. Such operation is not checked during
/// compilation, so be careful when using it, please.

pub trait GlslItem {
    /// Checks if the value is zero.
    fn is_zero (&self) -> bool;

    /// Converts the value to GLSL code.
    fn to_glsl (&self) -> String;
}


// === Instances ===

impl GlslItem for str {
    fn is_zero (&self) -> bool   { self == "0" || self == "0.0" }
    fn to_glsl (&self) -> String { self.into() }
}

impl GlslItem for &str {
    fn is_zero (&self) -> bool   { (*self).is_zero() }
    fn to_glsl (&self) -> String { (*self).to_glsl()    }
}

impl GlslItem for f32 {
    fn is_zero (&self) -> bool   { *self == 0.0 }
    fn to_glsl (&self) -> String {
        let is_int = self.fract() == 0.0;
        if is_int { iformat!("{self}.0") }
        else      { iformat!("{self}")   }
    }
}



// ===================
// === CanvasShape ===
// ===================

/// Reference to a shape defined on `Canvas`.
#[derive(Clone,Debug)]
pub struct CanvasShape {
    shape_num : usize,
    ids       : Vec<usize>,
    name      : String,
}

impl CanvasShape {
    /// Constructor.
    pub fn new(shape_num:usize) -> Self {
        let ids  = default();
        let name = mk_shape_name(shape_num.to_string());
        Self {shape_num,ids,name}
    }

    /// Adds new id enclosed in this shape.
    pub fn add_id(&mut self, id:usize) {
        self.ids.push(id);
    }

    /// Add multiple ids enclosed in this shape.
    pub fn add_ids(&mut self, ids:&Vec<usize>) {
        self.ids.extend(ids)
    }
}



// ==============
// === Canvas ===
// ==============

// === Definition ===

/// Canvas for drawing vector graphics.
///
/// The API is stateful, similar to the API of HTML5 canvas element.
/// It uses GLSL and signed distance fields under the hood.

#[derive(Debug,Default)]
pub struct Canvas {
    shape_num  : usize,
    last_id    : usize,
    code_lines : Vec<String>,
}


// === ID Management ===

impl Canvas {
    /// Generates a new unique shape's ID.
    fn get_new_id(&mut self) -> usize {
        let id = self.last_id;
        self.last_id += 1;
        id
    }

    /// Generate a new unique shape number.
    pub fn get_new_shape_num(&mut self) -> usize {
        let out = self.shape_num;
        self.shape_num += 1;
        out
    }
}


// === GLSL Modification ===

impl Canvas {
    /// Adds new code line to the GLSL code.
    fn add_code_line(&mut self, line:String) {
        self.code_lines.push(line);
    }

    /// Adds new indented code line to the GLSL code.
    pub fn add_indented_code_line(&mut self, line:String) {
        self.add_code_line(format!("    {}",line));
    }

    /// Defines a new variable in the GLSL code.
    fn define<E:Str>(&mut self, ty:&str, name:&str, expr:E) {
        let max_type_length = 7;
        let max_name_length = 13;
        let ty              = format!("{:1$}" , ty   , max_type_length);
        let name            = format!("{:1$}" , name , max_name_length);
        self.add_indented_code_line(iformat!("{ty} {name} = {expr.as_ref()};"));
    }

    /// Get the final GLSL code.
    fn to_glsl(&self) -> String {
        self.code_lines.join("\n")
    }
}


// === Shape Modification ===

impl Canvas {
    /// Creates a new `CanvasShape` object. The shape is not assigned with any id and is not
    /// represented in the GLSL code yet.
    fn new_canvas_shape(&mut self) -> CanvasShape {
        let num = self.get_new_shape_num();
        CanvasShape::new(num)
    }

    /// Defines a new shape with a new id and associated parameters, like color.
    fn define_shape(&mut self, sdf:&str, cd:Option<&str>) -> CanvasShape {
        let color     = "rgb2lch(vec3(1.0,0.0,0.0)";
        let mut shape = self.new_canvas_shape();
        let id        = self.get_new_id();
        let id_name   = mk_id_name  (&shape.name);
        let cd_name   = mk_cd_name  (&shape.name);
        let sdf_name  = mk_sdf_name (&shape.name);
        self.define("color" , &cd_name    , iformat!("{color}"));
        self.define("sdf"   , &sdf_name   , iformat!("{sdf}"));
        self.define("id"    , &id_name    , iformat!("new_id_layer({sdf_name},{id})"));
        self.define("shape" , &shape.name , iformat!("shape({id_name},{cd_name},{sdf_name})"));
        shape.add_id(id);
        shape
    }

    /// Define a new shape from the provided GLSL expression.
    pub fn new_shape_from_expr(&mut self, expr:&str) -> CanvasShape {
        let shape = self.new_canvas_shape();
        self.define("shape",&shape.name,expr);
        shape
    }

    /// Create a union shape from the provided shape components.
    pub fn union(&mut self, s1:CanvasShape, s2:CanvasShape) -> CanvasShape {
        let mut shape = self.new_shape_from_expr(&iformat!("union({s1.name},{s2.name})"));
        shape.add_ids(&s1.ids);
        shape.add_ids(&s2.ids);
        shape
    }

    /// Translate the current canvas origin.
    pub fn translate(&mut self, x:f32, y:f32) {
        let expr = iformat!("sdf_translate(position, vec2({x.to_glsl()},{y.to_glsl()}))");
        self.define("","position",expr);
    }

}


// =================
// === PrimShape ===
// =================

/// Class of primitive shapes. Primitive shapes are described by a SDF field.
pub trait PrimShape {
    fn to_sdf_code(&self) -> String;
}


// ===============================
// === Shape Definition Macros ===
// ===============================

/// Defines primitive shapes and appropriate shape wrappers.
///
/// Primitive shapes are defined in the `prim_shapes` module, while the shape wrappers are placed in
/// the `shapes` module. The shape definition accepted by this macro is similar to both a struct and
/// a function definition. It's body should be defined as a valid GLSL code.
///
/// For the following input:
/// ```
/// define_shapes! {
///     Circle (radius:f32) {
///         return sdf(length(position)-radius, bbox_center(radius,radius));
///     }
/// ```
///
/// The following output will be generated:
/// ```
/// pub mod prim_shapes {
///     use super::*;
///
///     #[derive(Debug,Clone)]
///     pub struct Circle {
///         pub glsl_name : String,
///         pub radius    : String,
///     }
///
///     impl Circle {
///         pub fn new<radius:GlslItem>(radius:radius) -> Self {
///             let glsl_name = "circle".to_string();
///             let radius    = radius.to_glsl();
///             Self {glsl_name,radius}
///         }
///
///         pub fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
///             let args = vec!["position".to_string(), self.radius.to_glsl()].join(",");
///             let code = format!("{}({})",self.glsl_name,args);
///             canvas.define_shape(&code,None)
///         }
///     }
///
///     impl PrimShape for Circle {
///            fn to_sdf_code(&self) -> String {
///                let body = "return sdf(length(position)-radius, bbox_center(radius,radius));";
///                let args = vec![
///                    "vec2 position".to_string(),
///                    format!("{} {}", <$f32 as Item>::gpu_type_name(), "radius")
///                    ].join(", ");
///                format!("sdf {} ({}) {{ {} }}",self.glsl_name,args,body)
///            }
///        }
///
///        impl ShapeDef for Circle {
///            fn render_glsl(&self, renderer:&mut GlslRenderer) -> CanvasShape {
///                self.draw(&mut renderer.canvas)
///            }
///        }
/// }
///
/// pub mod shapes {
///     use super::*;
///
///     pub type Circle = Immutable<prim_shapes::Circle>;
///     pub fn Circle<radius:GlslItem>(radius:radius) -> Circle {
///         Shape::new(prim_shapes::Circle::new(radius))
///     }
/// }
/// ```

macro_rules! define_shapes {
    ( $($name:ident $args:tt $body:tt)* ) => {
        pub mod prim_shapes {
            use super::*;
            $(define_prim_shape! {$name $args $body} )*
        }

        pub mod shapes {
            use super::*;
            $(define_shape_wrappers! {$name $args $body} )*
        }
    };
}

/// See the docs of `define_shapes`.
macro_rules! define_shape_wrappers {
    ( $name:ident ( $($field:ident : $field_type:ty),* $(,)? ) { $($code:tt)* } ) => {

        /// Smart shape type.
        pub type $name = Immutable<prim_shapes::$name>;

        /// Smart shape constructor.
        pub fn $name <$($field:GlslItem),*> ( $($field : $field),* ) -> $name {
            Immutable::new(prim_shapes::$name::new($($field),*))
        }
    }
}

/// See the docs of `define_shapes`.
macro_rules! define_prim_shape {
    ( $name:ident ( $($field:ident : $field_type:ty),* $(,)? ) { $($code:tt)* } ) => {
        /// The shape definition.
        #[derive(Debug,Clone)]
        pub struct $name {
            pub glsl_name : String,
            $(pub $field  : String),*
        }

        impl $name {
            /// Constructor.
            pub fn new <$($field:GlslItem),*> ( $($field : $field),* ) -> Self {
                let glsl_name = stringify!($name).to_snake_case();
                $(let $field = $field.to_glsl();)*
                Self {glsl_name,$($field),*}
            }

            /// Draws the shape on the provided canvas. Shapes are always drawn in the center of
            /// the canvas. In order to move them somewhere, use the canvas moving API.
            pub fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
                let args = vec!["position".to_string(), $(self.$field.to_glsl()),* ].join(",");
                let code = format!("{}({})",self.glsl_name,args);
                canvas.define_shape(&code,None)
            }
        }

        impl PrimShape for $name {
            fn to_sdf_code(&self) -> String {
                let body = stringify!($($code)*);
                let args = vec!["vec2 position".to_string(), $(
                    format!("{} {}", <$field_type as Item>::gpu_type_name(), stringify!($field))
                ),*].join(", ");
                format!("sdf {} ({}) {{ {} }}",self.glsl_name,args,body)
            }
        }

        impl ShapeDef for $name {
            fn render_glsl(&self, renderer:&mut GlslRenderer) -> CanvasShape {
                self.draw(&mut renderer.canvas)
            }
        }
    };
}



// =============
// === HasId ===
// =============

/// Each shape definition has to be assigned with an unique id in order for the renderer to
/// implement results cache. For example, we can create a circle as `s1` and then move it right,
/// which will result in the `s2` object. We can merge them together creating `s3` object. The
/// renderer needs to discover that `s3` was in fact created from two `s1` under the hood.
///
/// This trait should not be implemented manually. It is implemented by `Immutable`, which wraps
/// every shape definition.
pub trait HasId {
    fn id(&self) -> usize;
}



// =================
// === Immutable ===
// =================

/// Wrapper for primitive shapes. It makes them both immutable as well as assigns each shape with
/// an unique id.
#[derive(Debug,Derivative,Shrinkwrap)]
#[derivative(Clone(bound=""))]
pub struct Immutable<T> {
    rc:Rc<T>
}

impl<T> Immutable<T> {
    pub fn new(t:T) -> Self {
        Self {rc:Rc::new(t)}
    }
}

impl<T> HasId for Immutable<T> {
    fn id(&self) -> usize {
        Rc::downgrade(&self.rc).as_raw() as *const() as usize
    }
}



trait ShapeDef {
    fn render_glsl(&self, renderer:&mut GlslRenderer) -> CanvasShape;
}

trait IsShape = ShapeDef + HasId;

impl<T:ShapeDef> ShapeDef for Immutable<T> {
    fn render_glsl(&self, renderer:&mut GlslRenderer) -> CanvasShape {
        self.rc.render_glsl(renderer)
    }
}




// === Translate ===

pub struct Translate<S> {
    shape : S,
    x     : f32,
    y     : f32,
}

impl<S:Clone> Translate<S> {
    pub fn new(shape:&S,x:f32,y:f32) -> Self {
        Self {shape:shape.clone(),x,y}
    }
}

impl<S:IsShape> ShapeDef for Translate<S> {
    fn render_glsl(&self, renderer:&mut GlslRenderer) -> CanvasShape {
        renderer.with_new_tx_ctx(|r| {
            r.canvas.translate(self.x,self.y);
            r.render_shape(&self.shape)
        })
    }
}



// === Union ===

pub struct Union<S1,S2> {
    shape1 : S1,
    shape2 : S2
}

impl<S1:Clone,S2:Clone> Union<S1,S2> {
    pub fn new(shape1:&S1,shape2:&S2) -> Self {
        Self {shape1:shape1.clone(),shape2:shape2.clone()}
    }
}

impl<S1:IsShape,S2:IsShape> ShapeDef for Union<S1,S2> {
    fn render_glsl(&self, renderer:&mut GlslRenderer) -> CanvasShape {
        let s1 = renderer.render_shape(&self.shape1);
        let s2 = renderer.render_shape(&self.shape2);
        renderer.canvas.union(s1,s2)
    }
}




#[derive(Debug,Default)]
struct GlslRenderer {
    canvas      : Canvas,
    done        : HashMap<(usize,usize), CanvasShape>,
    tx_ctx      : usize,
    last_tx_ctx : usize,
}

impl GlslRenderer {
    pub fn get_new_tx_ctx(&mut self) -> usize {
        self.last_tx_ctx += 1;
        self.last_tx_ctx
    }

    pub fn with_new_tx_ctx<F:FnOnce(&mut Self)->T,T>(&mut self, f:F) -> T {
        let old_ctx = self.tx_ctx;
        let new_ctx = self.get_new_tx_ctx();
        self.tx_ctx = new_ctx;
        self.canvas.define("vec2",&iformat!("position_{new_ctx}"),"position");
        let out = f(self);
        self.canvas.define("","position",iformat!("position_{new_ctx}"));
        self.tx_ctx = old_ctx;
        out
    }

    pub fn render_shape<S:IsShape>(&mut self, shape:&S) -> CanvasShape {
        let shape_ptr    = shape.id();
        let canvas_shape = self.done.get(&(shape_ptr,self.tx_ctx));
        match canvas_shape {
            Some(s) => s.clone(),
            None    => {
                let canvas_shape = shape.render_glsl(self);
                self.done.insert((shape_ptr,self.tx_ctx), canvas_shape.clone());
                canvas_shape
            }
        }
    }

    pub fn render<S:IsShape>(&mut self, shape:&S) -> String {
        let canvas_shape = self.render_shape(shape);
        iformat!("shape main(vec2 position) {{\n{self.canvas.to_glsl()}\n    return {canvas_shape.name};\n}}")
    }
}


pub trait ShapeOps
where Self:Sized+Clone {
    fn translate(&self,x:f32,y:f32) -> Immutable<Translate<Self>> {
        Immutable::new(Translate::new(self,x,y))
    }

    fn union<S:Clone>(&self,that:S) -> Immutable<Union<Self,S>> {
        Immutable::new(Union::new(self,&that))
    }
}

impl<T> ShapeOps for Immutable<T> {}



impl<T,S:Clone> std::ops::Add<S> for Immutable<T> {
    type Output = Immutable<Union<Immutable<T>,S>>;
    fn add(self, that:S) -> Self::Output {
        self.union(that)
    }
}




define_shapes! {

    // === Infinite ===

    Plane () {
        return sdf(FLOAT_MIN,bbox_center(0.0,0.0));
    }

    HalfPlane () {
        return sdf(position.y, bbox_center(0.0,0.0))
    }

    Line (width:f32) {
        return sdf(abs(position.y)-width, bbox_center(0.0,width));
    }


    // === Ellipse ===

    Circle (radius:f32) {
        return sdf(length(position)-radius, bbox_center(radius,radius));
    }

    Ellipse (x_radius:f32, y_radius:f32) {
        float a2   = x_radius * x_radius;
        float b2   = y_radius * y_radius;
        float px2  = position.x * position.x;
        float py2  = position.y * position.y;
        float dist = (b2 * px2 + a2 * py2 - a2 * b2) / (a2 * b2);
        return sdf(dist, bbox_center(x_radius,y_radius));
    }


    // === Rectangle ===

    SharpRect (width:f32, height:f32) {
        vec2 size = vec2(width,height);
        return max_el(abs(position) - size);
    }

    Rect (width:f32, height:f32) {
        vec2  size = vec2(width,height);
        vec2  dir  = abs(position) - size;
        float dist = max_el(min(dir,0.0)) + length(max(dir,0.0));
        return sdf(dist,bbox_center(width,height));
    }

    RoundedRectByCorner
    (width:f32, height:f32, top_left:f32, top_right:f32, bottom_left:f32, bottom_right:f32) {
        vec2 size = vec2(width,height);
        size /= 2.0;

        float tl = top_left;
        float tr = top_right;
        float bl = bottom_left;
        float br = bottom_right;

        bool is_top_left     = position.x <  - size.x + tl && position.y >   size.y - tl;
        bool is_top_right    = position.x >    size.x - tr && position.y >   size.y - tr;
        bool is_bottom_left  = position.x <  - size.x + bl && position.y < - size.y + bl;
        bool is_bottom_right = position.x >    size.x - br && position.y < - size.y + br;

        if      is_top_left     {return length(position - vec2(- size.x + tl,   size.y - tl)) - tl;}
        else if is_top_right    {return length(position - vec2(  size.x - tr,   size.y - tr)) - tr;}
        else if is_bottom_left  {return length(position - vec2(- size.x + bl, - size.y + bl)) - bl;}
        else if is_bottom_right {return length(position - vec2(  size.x - br, - size.y + br)) - br;}
        else {
            vec2 dir = abs(position) - size;
            return min(max(dir.x,dir.y),0.0) + length(max(dir,0.0));
        }
    }


    // === Triangle ===

    Triangle(width:f32, height:f32) {
        vec2  norm = normalize(vec2(height,width/2.0));
        float dist = max(abs(position).x*norm.x + position.y*norm.y - height*norm.y, -position.y);
        return sdf(dist,bbox_center(width,height/2.0));
    }
}


pub fn main() {
    use shapes::*;

    let mut r:GlslRenderer = default();
    let canvas = &mut r.canvas;

//    let c1 = Circle::new("10.0");
//    let c2 = Circle::new("10.0");
//    let s1 = c1.draw(canvas);
//    let s2 = c2.draw(canvas);
//    canvas.union(s1,s2);

    let s1 = Circle(10.0);
    let s2 = s1.translate(1.0,2.0);
    let s3 = s1 + s2;

    println!("{}", r.render(&s3));

//
//    println!("{}", c1.to_sdf_code());
}


