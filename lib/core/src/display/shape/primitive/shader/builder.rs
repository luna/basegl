//! This module contains GLSL code builder. It allows translating complex vector shapes to the GLSL
//! code.

use crate::prelude::*;

use super::canvas::Canvas;
use super::super::class::Shape;
use crate::display::shape::primitive::def::sdf;
use crate::display::symbol::shader::builder::CodeTemplete;
use crate::display::shape::primitive::shader::overload;

const MATH            :&str = include_str!("../glsl/math.glsl");
const COLOR           :&str = include_str!("../glsl/color.glsl");
const DEBUG           :&str = include_str!("../glsl/debug.glsl");
const SHAPE           :&str = include_str!("../glsl/shape.glsl");
const FRAGMENT_RUNNER :&str = include_str!("../glsl/fragment_runner.glsl");


pub fn header(label:&str) -> String {
    let border_len = label.len() + 8;
    let border     = "=".repeat(border_len);
    iformat!("// {border}\n// === {label} ===\n// {border}")
}

/// GLSL code builder.
pub struct Builder {}

impl Builder {
    /// Returns the final GLSL code.
    pub fn run<S:Shape>(shape:&S) -> CodeTemplete {
        let sdf_defs     = sdf::all_shapes_glsl_definitions();
        let mut canvas   = Canvas::default();
        let shape_ref    = shape.draw(&mut canvas);
        let defs_header  = header("SDF Primitives");
        let shape_header = header("Shape Definition");
        canvas.add_current_function_code_line(iformat!("return {shape_ref.getter()};"));
        canvas.submit_shape_constructor("run");
        let defs = iformat!("{defs_header}\n\n{sdf_defs}\n\n\n\n{shape_header}\n\n{canvas.to_glsl()}");

        let redirections = overload::builtin_redirections();

        let math         = overload::allow_overloading(MATH);
        let color        = overload::allow_overloading(COLOR);
        let debug        = overload::allow_overloading(DEBUG);
        let shape      = overload::allow_overloading(SHAPE);

        let defs = overload::allow_overloading(&defs);

        let code = format!("{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}",redirections,math,color,debug,shape,defs);


        CodeTemplete::new(code,FRAGMENT_RUNNER.to_string(),default())
    }
}

