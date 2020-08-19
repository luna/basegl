//! A debug scene which shows the Select Component. The chosen entries are logged in console.

use crate::prelude::*;

use ensogl_core::system::web;
use ensogl_core::application::Application;
use ensogl_core::display::object::ObjectOps;
use ensogl_core::display::shape::*;
use ensogl_core::display::style::theme;
use ensogl_core::data::color;
use ensogl_core::gui;
use ensogl_text_msdf_sys::run_once_initialized;
use ensogl_select as select;
use logger::enabled::Logger;
use wasm_bindgen::prelude::*;
use ensogl_core::display::Scene;
use ensogl_text::buffer::data::unit::Bytes;



// ===================
// === Entry Point ===
// ===================

/// An entry point.
#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_select_component() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    web::set_stack_trace_limit();
    run_once_initialized(|| {
        let app = Application::new(&web::get_html_element_by_id("root").unwrap());
        init(&app);
        mem::forget(app);
    });
}



// ====================
// === Mock Entries ===
// ====================

mod icon {
    use super::*;
    ensogl_core::define_shape_system! {
        (style:Style,id:f32) {
            let width  = select::entry::ICON_SIZE.px();
            let height = select::entry::ICON_SIZE.px();
            let color  : Var<color::Rgba> = "rgba(input_id/16.0,0.0,0.0,1.0)".into();
            Rect((&width,&height)).fill(color).into()
        }
    }
}


#[derive(Clone,Debug)]
struct MockEntries {
    logger        : Logger,
    scene         : Scene,
    entries_count : usize,
}

impl MockEntries {
    fn new(app:&Application, entries_count:usize) -> Self {
        let logger  = Logger::new("MockEntries");
        let scene   = app.display.scene().clone_ref();
        Self {logger,scene,entries_count}
    }
}

impl select::entry::ModelProvider for MockEntries {
    fn entry_count(&self) -> usize { self.entries_count }

    fn get(&self, id:usize) -> select::entry::Model {
        let icon = gui::component::ShapeView::<icon::Shape>::new(&self.logger,&self.scene);
        icon.shape.sprite.size.set(Vector2(select::entry::ICON_SIZE,select::entry::ICON_SIZE));
        icon.shape.id.set(id as f32);
        let model = select::entry::Model::new(iformat!("Entry {id}")).with_icon(icon);
        if id == 10 { model.highlight(std::iter::once((Bytes(1)..Bytes(3)).into())) }
        else        { model }
    }
}



// ========================
// === Init Application ===
// ========================

fn init(app:&Application) {

    let mut dark = theme::Theme::new();
    dark.insert("application.background.color", color::Lcha::new(0.13,0.013,0.18,1.0));
    dark.insert("select.background.color", color::Lcha::new(0.2,0.013,0.18,1.0));
    dark.insert("select.selection.color", color::Lcha::new(0.72,0.5,0.22,1.0));
    dark.insert("animation.duration", 0.5);
    dark.insert("mouse.pointer.color", color::Rgba::new(0.3,0.3,0.3,1.0));

    app.themes.register("dark",dark);
    app.themes.set_enabled(&["dark"]);

    let select                                   = app.new_view::<select::component::Select>();
    let provider:select::entry::AnyModelProvider = MockEntries::new(app,13000).into();
    select.frp.resize(Vector2(100.0,160.0));
    select.frp.set_entries(provider);
    app.display.add_child(&select);

    let logger  = Logger::new("SelectDebugScene");
    let network = enso_frp::Network::new();
    enso_frp::extend! {network
        eval select.chosen_entry([logger](entry) {
            info!(logger, "Chosen entry {entry:?}")
        });
    }

    std::mem::forget(select);
    std::mem::forget(network);
}
