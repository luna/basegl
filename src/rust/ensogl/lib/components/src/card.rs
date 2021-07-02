use crate::prelude::*;

use crate::shadow;

use ensogl_core::display;
use ensogl_core::display::shape::*;
use ensogl_core::display::object::{ObjectOps, Instance};
use ensogl_theme as theme;



const SHADOW_PX:f32 = 10.0;

ensogl_core::define_shape_system! {
    (style:Style,corner_radius:f32) {
        let sprite_width  : Var<Pixels> = "input_size.x".into();
        let sprite_height : Var<Pixels> = "input_size.y".into();
        let width         = sprite_width - SHADOW_PX.px() * 2.0;
        let height        = sprite_height - SHADOW_PX.px() * 2.0;
        let color         = style.get_color(theme::application::file_browser::background);
        let rect          = Rect((&width,&height)).corners_radius(corner_radius);
        let shape         = rect.fill(color);

        let shadow  = shadow::from_shape(rect.into(),style);

        (shadow + shape).into()
    }
}

#[derive(Debug,Clone,CloneRef)]
pub struct Card(View);

impl Card {
    pub fn new() -> Self {
        Card(View::new(Logger::new("Card")))
    }

    pub fn resize(&self,size:Vector2) {
        let shadow_margin = Vector2(SHADOW_PX,SHADOW_PX);
        self.0.size.set(size+2.0*shadow_margin);
    }

    pub fn set_corner_radius(&self,radius:f32) {
        self.0.corner_radius.set(radius);
    }
}

impl display::Object for Card {
    fn display_object(&self) -> &Instance {
        self.0.display_object()
    }
}
