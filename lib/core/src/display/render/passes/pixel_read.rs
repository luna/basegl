//! Pass reading pixels from a previously created framebuffer.

use crate::prelude::*;

use crate::display::render::pipeline::*;
use crate::system::gpu::*;

use js_sys::ArrayBuffer;
use web_sys::WebGlBuffer;
use web_sys::WebGlSync;



// =========================
// === PixelReadPassData ===
// =========================

/// Internal state for the `PixelReadPass`.
#[derive(Clone,Debug)]
pub struct PixelReadPassData {
    uniform : Uniform<Vector4<i32>>,
    buffer  : WebGlBuffer,
}

impl PixelReadPassData {
    /// Constructor.
    pub fn new(uniform:Uniform<Vector4<i32>>, buffer:WebGlBuffer) -> Self {
        Self {uniform,buffer}
    }
}



// =====================
// === PixelReadPass ===
// =====================

/// Reads the pixel color and stores it in the 'pass_pixel_color' variable.
#[derive(Clone,Derivative)]
#[derivative(Debug)]
pub struct PixelReadPass {
    data     : Option<PixelReadPassData>,
    sync     : Option<WebGlSync>,
    position : Uniform<Vector2<i32>>,
    #[derivative(Debug="ignore")]
    callback : Option<Rc<dyn Fn(Vector4<i32>)>>,
}

impl PixelReadPass {
    /// Constructor.
    pub fn new(position:&Uniform<Vector2<i32>>) -> Self {
        let data     = default();
        let sync     = default();
        let position = position.clone_ref();
        let callback = default();
        Self {data,sync,position,callback}
    }

    pub fn set_callback<F:Fn(Vector4<i32>)+'static>(&mut self, f:F) {
        self.callback = Some(Rc::new(f));
    }

    fn init_if_fresh(&mut self, context:&Context, variables:&UniformScope) {
        if self.data.is_none() {
            let buffer  = context.create_buffer().unwrap();
            let array   = ArrayBuffer::new(4);
            let target  = Context::PIXEL_PACK_BUFFER;
            let usage   = Context::DYNAMIC_READ;
            let uniform = variables.get_or_add("pass_output_pixel_color",Vector4::new(0,0,0,0)).unwrap();
            context.bind_buffer(target,Some(&buffer));
            context.buffer_data_with_opt_array_buffer(target,Some(&array),usage);
            self.data = Some(PixelReadPassData::new(uniform,buffer));
        }
    }

    fn run_not_synced(&mut self, context:&Context) {
        let data     = self.data.as_ref().unwrap();
        let position = self.position.get();
        let width    = 1;
        let height   = 1;
        let format   = Context::RGBA;
        let typ      = Context::UNSIGNED_BYTE;
        let target   = Context::PIXEL_PACK_BUFFER;
        let offset   = 0;
        context.bind_buffer(target,Some(&data.buffer));
        context.read_pixels_with_i32(position.x,position.y,width,height,format,typ,offset).unwrap();
        let condition = Context::SYNC_GPU_COMMANDS_COMPLETE;
        let flags     = 0;
        let sync      = context.fence_sync(condition,flags).unwrap();
        self.sync     = Some(sync);
        context.flush();
    }

    fn check_and_handle_sync(&mut self, context:&Context, sync:&WebGlSync) {
        let data   = self.data.as_ref().unwrap();
        let status = context.get_sync_parameter(sync,Context::SYNC_STATUS);
        if status == Context::SIGNALED {
            context.delete_sync(Some(sync));
            self.sync          = None;
            let target         = Context::PIXEL_PACK_BUFFER;
            let offset         = 0;
            let mut raw_result = vec![0,0,0,0];
            context.bind_buffer(target,Some(&data.buffer));
            context.get_buffer_sub_data_with_i32_and_u8_array(target,offset,&mut raw_result);
            let result = Vector4::from_iterator(raw_result.iter().map(|t| *t as i32));
            data.uniform.set(result);
            if let Some(f) = &self.callback {
                f(result);
            }
//            println!("GOT: {:?}", result);
        }
    }
}

impl RenderPass for PixelReadPass {
    fn run(&mut self, context:&Context, variables:&UniformScope) {
        self.init_if_fresh(context,variables);


        let texture = match variables.get("pass_id").unwrap() {
            uniform::AnyUniform::Texture(t) => t,
            _ => panic!("!!!")
        };

        let gl_texture = texture.gl_texture();

        let framebuffer      = context.create_framebuffer().unwrap();
        let target           = Context::FRAMEBUFFER;
        let texture_target   = Context::TEXTURE_2D;
        let attachment_point = Context::COLOR_ATTACHMENT0;
        let gl_texture       = Some(&gl_texture);
        let level            = 0;
        context.bind_framebuffer(target,Some(&framebuffer));
        context.framebuffer_texture_2d(target,attachment_point,texture_target,gl_texture,level);



        if let Some(sync) = self.sync.clone() {
            self.check_and_handle_sync(context,&sync);
        }
        if self.sync.is_none() {
            self.run_not_synced(context);
        }
    }
}
