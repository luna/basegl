use crate::prelude::*;

use enso_frp as frp;
use ensogl::application;
use ensogl::application::{Application, shortcut};
use ensogl::display;
use ensogl_gui_list_view::ListView;

pub use ensogl_gui_list_view::entry;
use enso_frp::io::keyboard::Key;

ensogl::def_command_api!( Commands
    /// Pick the selected suggestion and add it to the current input.
    pick_suggestion,
);

ensogl_text::define_endpoints! {
    Commands { Commands }
    Input {
        resize           (Vector2<f32>),
        set_entries      (entry::AnyModelProvider),
    }
    Output {
        selected_entry    (Option<entry::Id>),
        picked_entry      (Option<entry::Id>),
        editing_committed (),
        size              (Vector2<f32>),
    }
}

#[derive(Clone,CloneRef,Debug)]
struct Model {
    logger         : Logger,
    display_object : display::object::Instance,
    list           : ListView,
}

impl Model {
    pub fn new(app:&Application) -> Self {
        let logger         = Logger::new("SearcherView");
        let display_object = display::object::Instance::new(&logger);
        let list           = app.new_view::<ListView>();
        display_object.add_child(&list);
        Self{logger,display_object,list}
    }
}

#[derive(Clone,CloneRef,Debug)]
pub struct View {
    model   : Model,
    pub frp : Frp,
}

impl Deref for View {
    type Target = Frp;
    fn deref(&self) -> &Self::Target { &self.frp }
}

impl View {
    pub fn new(app:&Application) -> Self {
        let model = Model::new(app);
        let frp   = Frp::new_network();
        Self{model,frp}.init()
    }

    fn init(self) -> Self {
        let network = &self.frp.network;
        let model   = &self.model;
        let frp     = &self.frp;
        let source  = &self.frp.source;

        frp::extend! { network
            eval frp.resize      ((size)    model.list.resize(size));
            eval frp.set_entries ((entries) model.list.set_entries(entries));
            source.selected_entry <+ model.list.selected_entry;
            source.size           <+ model.list.size;

            is_selected         <- model.list.selected_entry.map(|e| e.is_some());
            opt_picked_entry    <- model.list.selected_entry.sample(&frp.pick_suggestion);
            source.picked_entry <+ opt_picked_entry.gate(&is_selected);
            // Order of the two below is important: we want pick the entry first, and then commit
            // editing.
            source.picked_entry <+ model.list.chosen_entry.gate(&is_selected);
            source.editing_committed <+ model.list.chosen_entry.gate(&is_selected).constant(());
            trace model.list.chosen_entry;
            trace source.picked_entry;
            trace source.editing_committed;
        }

        self
    }
}
impl display::Object for View {
    fn display_object(&self) -> &display::object::Instance { &self.model.display_object }
}

impl application::command::FrpNetworkProvider for View {
    fn network(&self) -> &frp::Network {
        &self.frp.network
    }
}

impl application::command::CommandApi for View {
    fn command_api_docs() -> Vec<application::command::EndpointDocs> {
        Commands::command_api_docs()
    }

    fn command_api(&self) -> Vec<application::command::CommandEndpoint> {
        self.frp.input.command.command_api()
    }
}

impl application::command::Provider for View {
    fn label() -> &'static str {
        "Searcher"
    }
}

impl application::View for View {
    fn new(app: &Application) -> Self {
        Self::new(app)
    }
}

impl application::shortcut::DefaultShortcutProvider for View {
    fn default_shortcuts() -> Vec<shortcut::Shortcut> {
        use enso_frp::io::mouse;
        vec!
        [ Self::self_shortcut(shortcut::Action::press   (&[Key::Tab], &[]) , "pick_suggestion"),
        ]
    }
}