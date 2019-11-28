pub mod font;

use crate::prelude::*;

use crate::Color;
use crate::text::font::MsdfTexture;
use crate::display::world::Workspace;
use basegl_backend_webgl::{Context, compile_shader, link_program, Program, };
use web_sys::{WebGlRenderingContext, WebGlBuffer, WebGlTexture};
use font::FontRenderInfo;
use js_sys::Float32Array;
use nalgebra::{Vector2, Point2};

struct CachingIterator<T:Clone, It:Iterator<Item=T>> {
    last : Option<T>,
    iter : It
}

impl<T:Clone, It:Iterator<Item=T>> Iterator for CachingIterator<T, It> {
    type Item = (Option<T>, T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|value| {
            let new_last = Some(value.clone());
            let old_last = std::mem::replace(&mut self.last, new_last);
            (old_last, value)
        })
    }
}

trait IntoCachingIterator {
    type Item : Clone;
    type Iter : Iterator<Item = Self::Item>;
    fn cache_last_value(self) -> CachingIterator<Self::Item, Self::Iter>;
}

impl<T : Clone, It : Iterator<Item=T>> IntoCachingIterator for It {
    type Item = T;
    type Iter = Self;

    fn cache_last_value(self) -> CachingIterator<Self::Item, Self::Iter> {
        CachingIterator { last : None, iter : self }
    }
}



pub struct TextComponentBuilder<'a> {
    pub text             : String,
    pub font             : &'a mut FontRenderInfo,
    pub x                : f32,
    pub y                : f32,
    pub size             : f32,
    pub color            : Color<f32>,
}

#[derive(Debug)]
pub struct TextComponent {
    gl_context       : WebGlRenderingContext,
    gl_program       : Program,
    gl_vertex_buf    : WebGlBuffer,
    gl_tex_coord_buf : WebGlBuffer,
    gl_msdf_texture  : WebGlTexture,
    buffers_size     : usize,
}

impl<'a> TextComponentBuilder<'a> {
    pub fn build(mut self, workspace  : &Workspace) -> TextComponent {
        let gl_context       = workspace.context.clone();
        let gl_program       = self.create_program(&gl_context);

        for ch in self.text.chars() {
            self.font.get_or_create_char_info(ch);
        }

        let gl_vertex_buf    = self.create_vertex_buf(&gl_context);
        let gl_tex_coord_buf = self.create_tex_coord_buf(&gl_context);
        let gl_msdf_texture  =
            self.create_msdf_texture(&gl_context, &gl_program);

        self.setup_uniforms(&gl_context, &gl_program);

        TextComponent {
            gl_context,
            gl_program,
            gl_vertex_buf,
            gl_tex_coord_buf,
            gl_msdf_texture,
            buffers_size: self.text.len()
        }
    }

    fn create_program(&self, gl_context : &Context) -> Program {
        gl_context.get_extension("OES_standard_derivatives")
            .unwrap().unwrap();

        let vert_shader = compile_shader(
            &gl_context,
            WebGlRenderingContext::VERTEX_SHADER,
            include_str!("msdf_vert.glsl")
        ).unwrap();

        let frag_shader = compile_shader(
            &gl_context,
            WebGlRenderingContext::FRAGMENT_SHADER,
            include_str!("msdf_frag.glsl")
        ).unwrap();

        link_program(&gl_context, &vert_shader, &frag_shader).unwrap()
    }

    fn create_buffer(
        gl_context : &Context,
        vertices   : &[f32]
    ) -> WebGlBuffer {
        let buffer = gl_context.create_buffer().unwrap();
        gl_context.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&buffer)
        );

        unsafe { // Note [unsafe buffer_data]
            let float_32_array = Float32Array::view(&vertices);
            gl_context.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &float_32_array,
                WebGlRenderingContext::STATIC_DRAW,
            );
        }

        buffer
    }

    /* Note [unsafe buffer_data]
     *
     * The Float32Array::view is safe as long there are no allocations done
     * until it is destroyed. This way of creating buffers were taken from
     * wasm-bindgen examples
     * (https://rustwasm.github.io/wasm-bindgen/examples/webgl.html)
     */

    const BASE_GLYPH_VERTICES_LAYOUT : &'static[(f32, f32)] = &[
        (0.0, 0.0),
        (0.0, 1.0),
        (1.0, 0.0),
        (1.0, 0.0),
        (0.0, 1.0),
        (1.0, 1.0),
    ];

    fn create_vertex_buf(&mut self, gl_context : &Context) -> WebGlBuffer {
        let to_window = nalgebra::Similarity2::new(
            Vector2::new(self.x, self.y),
            0.0,
            self.size
        );
        let mut position = nalgebra::Translation2::new(0.0, 0.0);
        let font = &mut self.font;
        let vertices = self.text.chars().cache_last_value().map(|(last_ch, ch)| {
            let kerning = last_ch.map(|lc| font.get_or_retrieve_kerning(lc, ch) ).unwrap_or(0.0);
            let ch_info = font.get_or_create_char_info(ch);
            let kerning_trans = nalgebra::Translation2::new(kerning, 0.0);
            let transformation = to_window*position*ch_info.points_transformation*kerning_trans;
            position = nalgebra::Translation2::new(ch_info.advance+kerning, 0.0)*position;
            Self::BASE_GLYPH_VERTICES_LAYOUT.iter()
                .map(|(x, y)| Point2::new(*x, *y))
                .map(|p| transformation.transform_point(&p))
                .map(|p| p.iter().cloned().collect::<Vec<f32>>())
                .flatten().collect::<Vec<f32>>()
        }).flatten().collect::<Box<[f32]>>();

        Self::create_buffer(gl_context, vertices.as_ref())
    }


    fn create_tex_coord_buf(&mut self, gl_context : &Context) -> WebGlBuffer {
        let font = &mut self.font;
        let vertices = self.text.chars().map(|c| {
            let msdf_rows = font.msdf_texture.rows() as f32;
            let info = font.get_or_create_char_info(c);
            let y_min = info.msdf_texture_rows.start as f32 / msdf_rows;
            let y_max = info.msdf_texture_rows.end as f32 / msdf_rows;
            vec![0.0, y_min, 0.0, y_max, 1.0, y_min,
                 1.0, y_min, 0.0, y_max, 1.0, y_max]
        }).flatten().collect::<Box<[f32]>>();

        Self::create_buffer(gl_context, vertices.as_ref())
    }

    fn create_msdf_texture(&self, gl_context : &Context, gl_program : &Program)
        -> WebGlTexture {

        let msdf_texture = gl_context.create_texture().unwrap();
        gl_context.bind_texture(Context::TEXTURE_2D, Some(&msdf_texture));

        gl_context.tex_parameteri(
            Context::TEXTURE_2D,
            Context::TEXTURE_WRAP_S,
            Context::CLAMP_TO_EDGE as i32
        );
        gl_context.tex_parameteri(
            Context::TEXTURE_2D,
            Context::TEXTURE_WRAP_T,
            Context::CLAMP_TO_EDGE as i32
        );
        gl_context.tex_parameteri(
            Context::TEXTURE_2D,
            Context::TEXTURE_MIN_FILTER,
            Context::LINEAR as i32
        );

        gl_context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            Context::TEXTURE_2D,
            0,
            Context::RGB as i32,
            MsdfTexture::WIDTH as i32,
            self.font.msdf_texture.rows() as i32,
            0,
            Context::RGB,
            Context::UNSIGNED_BYTE,
            Some(self.font.msdf_texture.data.as_slice())
        ).unwrap();

        let msdf_loc = gl_context.get_uniform_location(gl_program, "msdf");
        let msdf_size_loc =
            gl_context.get_uniform_location(gl_program, "msdfSize");

        gl_context.use_program(Some(gl_program));
        gl_context.uniform1i(msdf_loc.as_ref(), 0);
        gl_context.uniform2f(
            msdf_size_loc.as_ref(),
            MsdfTexture::WIDTH as f32,
            self.font.msdf_texture.rows() as f32
        );

        msdf_texture
    }

    fn setup_uniforms(&self, gl_context : &Context, gl_program : &Program) {
        let fg_color_loc =
            gl_context.get_uniform_location(gl_program, "fgColor");
        let px_range_loc =
            gl_context.get_uniform_location(gl_program, "pxRange");

        gl_context.use_program(Some(gl_program));
        gl_context.uniform4f(
            fg_color_loc.as_ref(),
            self.color.r,
            self.color.g,
            self.color.b,
            self.color.a,
        );
        gl_context.uniform1f(
            px_range_loc.as_ref(),
            FontRenderInfo::MSDF_PARAMS.range as f32
        );
    }
}

impl TextComponent {

    pub fn display(&self) {
        let gl = &self.gl_context;
        let program = &self.gl_program;

        gl.use_program(Some(&self.gl_program));

        let position_location = gl.get_attrib_location(program, "position");
        gl.enable_vertex_attrib_array(position_location as u32);
        gl.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&self.gl_vertex_buf)
        );
        gl.vertex_attrib_pointer_with_i32(
            position_location as u32,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0
        );

        gl.bind_texture(Context::TEXTURE_2D, Some(&self.gl_msdf_texture));

        let tex_coord_location = gl.get_attrib_location(program, "texCoord");
        assert!(tex_coord_location >= 0);
        gl.enable_vertex_attrib_array(tex_coord_location as u32);
        gl.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&self.gl_tex_coord_buf)
        );
        gl.vertex_attrib_pointer_with_i32(
            tex_coord_location as u32,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0
        );

        gl.enable(Context::BLEND);
        gl.blend_func_separate(
            Context::SRC_ALPHA,
            Context::ONE_MINUS_SRC_ALPHA,
            Context::ZERO,
            Context::ONE
        );

        gl.draw_arrays(
            WebGlRenderingContext::TRIANGLES,
            0,
            (self.buffers_size*6) as i32,
        );
    }
}
