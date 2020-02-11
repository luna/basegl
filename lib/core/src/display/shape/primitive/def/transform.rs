//! This module contains definitions of all primitive shapes transformations, like translation, or
//! rotation.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::prelude::*;

use crate::display::shape::primitive::def::class::Owned;
use crate::display::shape::primitive::def::class::AsOwned;
use crate::display::shape::primitive::def::class::IntoOwned;
use crate::display::shape::primitive::def::class::Shape;
use crate::display::shape::primitive::def::class::ShapeRef;
use crate::display::shape::primitive::shader::canvas::Canvas;
use crate::display::shape::primitive::shader::canvas::CanvasShape;
use crate::display::shape::primitive::shader::data::ShaderData;
use crate::system::gpu::shader::glsl::Glsl;



// ========================================
// === Compound Shape Definition Macros ===
// ========================================

/// Defines compound canvas shapes.
///
/// For the following input:
/// ```compile_fail
/// define_compound_shapes! {
///    Translate(child)(x:f32,y:f32)
/// }
/// ```
///
/// The macro generates:
/// ```compile_fail
/// pub mod mutable {
///     use super::*;
///
///     pub struct Translate<child> {
///         pub child : child,
///         pub x     : Glsl,
///         pub y     : Glsl,
///     }
///
///     impl<child:Shape> Translate<child> {
///         pub fn new<x:ShaderData<f32>,y:ShaderData<f32>>(child:&child,x:x,y:y) -> Self {
///             let child = child.clone();
///             let x     = x.into();
///             let y     = y.into();
///             Self {child,x,y}
///         }
///     }
/// }
///
/// pub mod immutable {
///     use super::*;
///
///     pub type Translate<child> = ShapeRef<mutable::Translate<child>>;
///     pub fn Translate<child:Shape,x:ShaderData<f32>,y:ShaderData<f32>>
///     (child:&child,x:x,y:y) -> Translate<child> {
///         ShapeRef::new(mutable::Translate::new(child,x,y))
///     }
/// }
/// ```

macro_rules! define_compound_shapes {
    ( $($name:ident $shapes:tt $fields:tt)* ) => {
        /// Contains mutable shapes definitions.
        pub mod mutable {
            use super::*;
            $(_define_compound_shape_data! {$name $shapes $fields})*
        }

        /// Contains immutable shapes definitions.
        pub mod immutable {
            use super::*;
            $(_define_compound_shape! {$name $shapes $fields})*
        }
    }
}

macro_rules! _define_compound_shape_data {
    ($name:ident ($($shape_field:ident),*$(,)?) ($($field:ident : $field_type:ty),*$(,)?)) => {

        /// Shape type definition.
        #[allow(missing_docs)]
        #[derive(Debug)]
        pub struct $name<$($shape_field),*> {
            $(pub $shape_field : $shape_field),*,
            $(pub $field       : Glsl),*
        }
        impl<$($shape_field),*> $name<$($shape_field),*> {
            /// Constructor.
            pub fn new<$($field:ShaderData<$field_type>),*>
            ($($shape_field:$shape_field),*,$($field:$field),*) -> Self {
                $(let $field = $field.into();)*
                Self {$($shape_field),*,$($field),*}
            }
        }

        impl<$($shape_field),*> AsOwned for $name<$($shape_field),*> { type Owned = $name<$($shape_field),*>; }

    }
}

macro_rules! _define_compound_shape {
    ($name:ident ($($shape_field:ident),*$(,)?) ($($field:ident : $field_type:ty),*$(,)?)) => {
        /// Shape type definition.
        pub type $name<$($shape_field),*> =
            ShapeRef<mutable::$name<$($shape_field),*>>;

        /// Smart constructor.
        pub fn $name<$($shape_field:IntoOwned),*,$($field:ShaderData<$field_type>),*>
        ( $($shape_field:$shape_field),*,$($field:$field),*) -> $name<$(Owned<$shape_field>),*> {
            ShapeRef::new(mutable::$name::new($($shape_field.into()),*,$($field),*))
        }

        impl<$($shape_field),*> AsOwned for $name<$($shape_field),*> { type Owned = $name<$($shape_field),*>; }
    }
}



// =======================
// === Compound Shapes ===
// =======================

use immutable::*;

define_compound_shapes! {
    Translate(child)(x:f32,y:f32)
    Rotation(child)(angle:f32)
    Union(child1,child2)()
    Difference(child1,child2)()
    Intersection(child1,child2)()
    Fill(child)(color:dyn Any)
}


impl<Child:Shape> Shape for Translate<Child> {
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
        let s1 = self.child.draw(canvas);
        canvas.translate(self.id(),s1,&self.x,&self.y)
    }
}

impl<Child:Shape> Shape for Rotation<Child> {
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
        let s1 = self.child.draw(canvas);
        canvas.rotation(self.id(),s1,&self.angle)
    }
}

impl<Child1:Shape,Child2:Shape> Shape for Union<Child1,Child2> {
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
        let s1 = self.child1.draw(canvas);
        let s2 = self.child2.draw(canvas);
        canvas.union(self.id(),s1,s2)
    }
}

impl<Child1:Shape,Child2:Shape> Shape for Difference<Child1,Child2> {
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
        let s1 = self.child1.draw(canvas);
        let s2 = self.child2.draw(canvas);
        canvas.difference(self.id(),s1,s2)
    }
}

impl<Child1:Shape,Child2:Shape> Shape for Intersection<Child1,Child2> {
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
        let s1 = self.child1.draw(canvas);
        let s2 = self.child2.draw(canvas);
        canvas.intersection(self.id(),s1,s2)
    }
}

impl<Child:Shape> Shape for Fill<Child> {
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
        let s = self.child.draw(canvas);
        canvas.fill(self.id(),s,&self.color)
    }
}
