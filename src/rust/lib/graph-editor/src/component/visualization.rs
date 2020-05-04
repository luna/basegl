//! This module defines the visualization widgets and related functionality.
//!
//! At the core of this functionality is the `Visualisation` that takes in data and renders an
//! output visualisation which is displayed in a `Container`. The `Container` provides generic UI
//! elements that facilitate generic interactions, for example, visualisation selection. The
//! `Container` also provides the FRP API that allows internal interaction with the
//! `Visualisation`. Data for a visualisation has to be provided wrapped in the `Data` struct.
//!
use crate::prelude::*;
use ensogl::display::traits::*;

use crate::frp;

use ensogl::display::DomSymbol;
use ensogl::display;
use ensogl::system::web;
use web::StyleSetter;


// ============================================
// === Wrapper for Visualisation Input Data ===
// ============================================

/// Wrapper for data that can be consumed by a visualisation.
/// TODO replace with better typed data wrapper.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub enum Data {
    JSON { content : String },
    Empty,
}

impl Data {
    /// Render the data as JSON.
    pub fn as_json(&self) -> String {
        match &self {
            Data::JSON { content } => content.clone(),
            Data::Empty => { "{}".to_string() },
        }
    }
}

impl Default for Data{
    fn default() -> Self {
        Data::Empty
    }
}



// =============================================
// === Internal Visualisation Representation ===
// =============================================

/// Inner representation of a visualisation.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Visualization {
    content : Rc<DomSymbol>
}

impl display::Object  for Visualization {
    fn display_object(&self) -> &display::object::Instance {
        &self.content.display_object()
    }
}

impl Visualization {
    /// Update the visualisation with the given data.
    // TODO remove dummy functionality and use an actual visualisation
    pub fn set_data(&self, data:Data){
                self.content.dom().set_inner_html(
                    &format!(r#"
<svg>
  <circle style="fill: #69b3a2" stroke="black" cx=50 cy=50 r={}></circle>
</svg>
"#, data.as_json()));
     }

    /// Set whether the visualisation should be visible or not.
    pub fn set_visibility(&self, is_visible:bool) {
        if is_visible {
            self.content.dom().set_style_or_panic("visibility", "visible");
        } else {
            self.content.dom().set_style_or_panic("visibility", "hidden");
        }
    }
}

impl From<DomSymbol> for Visualization {
    fn from(symbol: DomSymbol) -> Self {
        Visualization { content : Rc::new(symbol) }
    }
}

impl From<Rc<DomSymbol>> for Visualization {
    fn from(symbol: Rc<DomSymbol>) -> Self {
        Visualization { content : symbol }
    }
}



// ============================
// === Visualization Events ===
// ============================

/// Visualization events.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Events {
    pub network           : frp::Network,
    pub set_visibility    : frp::Source<bool>,
    pub toggle_visibility : frp::Source,
    pub set_visualization : frp::Source<Option<Visualization>>,
    pub set_data          : frp::Source<Data>,
}

impl Default for Events {
    fn default() -> Self {
        frp::new_network! { visualization_events
            def set_visibility    = source::<bool>                  ();
            def toggle_visibility = source::<()>                    ();
            def set_visualization = source::<Option<Visualization>> ();
            def set_data          = source::<Data>                  ();
        };
        let network = visualization_events;
        Self {network,set_visibility,set_visualization,toggle_visibility,set_data }
    }
}



// ================================
// === Visualizations Container ===
// ================================

/// Container that wraps a `Visualisation` for rendering and interaction in the GUI.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Container {
    pub data : Rc<ContainerData>
}

/// Weak version of `Container`.
#[derive(Clone,CloneRef,Debug)]
pub struct WeakContainer {
    data : Weak<ContainerData>
}

/// Internal data of a `Container`.
#[derive(Debug,Clone)]
#[allow(missing_docs)]
pub struct ContainerData {
    pub logger : Logger,
    pub events : Events,

    node          : display::object::Instance,
    size          : Cell<Vector2<f32>>,
    is_visible    : Cell<bool>,
    visualization : RefCell<Option<Visualization>>,
}

impl Container {
    /// Constructor.
    pub fn new() -> Self {
        let logger   = Logger::new("visualization");
        let events   = default();
        let content  = default();
        let size     = Cell::new(Vector2::new(100.0, 100.0));
        let is_visible  = Cell::new(true);
        let node     = display::object::Instance::new(&logger);

        let data     = ContainerData {logger,events,visualization: content,size,is_visible,node};
        let data     = Rc::new(data);
        Self {data} . init_frp()
    }

    /// Update the content properties with the values from the `ContainerData`.
    ///
    /// Needs to called when a visualisation has been set.
    fn update_visualisation_properties(&self) {
        let size       = self.data.size.get();
        let position   = self.data.node.position();

        if let Some(vis) = self.data.visualization.borrow().as_ref() {
            vis.content.set_size(size);
            vis.content.set_position(position);
        };
    }

    /// Set the visualization content.
    pub fn set_visualisation(&self, visualization: Visualization) {
        self.display_object().add_child(visualization.display_object());
        self.data.visualization.replace(Some(visualization));
        self.update_visualisation_properties();
    }

    fn init_frp(self) -> Self {
        let network = &self.data.events.network;

        frp::extend! { network
            let weak_vis = self.downgrade();
            def _f_hide = self.data.events.set_visibility.map(move |is_visible| {
                if let Some(vis) = weak_vis.upgrade() {
                    vis.set_visibility(*is_visible)
               }
            });

            let weak_vis = self.downgrade();
            def _f_toggle = self.data.events.toggle_visibility.map(move |_| {
                if let Some(vis) = weak_vis.upgrade() {
                    vis.toggle_visibility()
               }
            });

            let weak_vis = self.downgrade();
            def _f_hide = self.data.events.set_visualization.map(move |content| {
                if let Some(vis) = weak_vis.upgrade() {
                    if let Some(content) = content.clone() {
                        vis.set_visualisation(content);
                    }
                }
            });

            let weak_vis = self.downgrade();
            def _f_hide = self.data.events.set_data.map(move |data| {
                if let Some(vis) = weak_vis.upgrade() {
                    vis.set_data(data.clone());
                }
            });
        }
        self
    }

    /// Set whether the visualisation should be visible or not.
    pub fn set_visibility(&self, is_visible:bool) {
        self.data.is_visible.set(is_visible)  ;
        if let Some(vis) = self.data.visualization.borrow().as_ref() {
            // TODO use display object functionality
            vis.set_visibility(is_visible)
        }
    }

    /// Indicates whether the visualisation is visible.
    pub fn is_visible(&self) -> bool {
        self.data.is_visible.get()
    }

    /// Toggle visibility.
    pub fn toggle_visibility(&self) {
        self.set_visibility(!self.is_visible())
    }

    /// Update the data in the inner visualisation.
    pub fn set_data(&self, data: Data) {
        if let Some(vis) = self.data.visualization.borrow().as_ref() {
            vis.set_data(data)
        }
    }
}

impl Default for Container {
    fn default() -> Self {
        Container::new()
    }
}

impl StrongRef for Container {
    type WeakRef = WeakContainer;
    fn downgrade(&self) -> WeakContainer {
        WeakContainer {data:Rc::downgrade(&self.data)}
    }
}

impl WeakRef for WeakContainer {
    type StrongRef = Container;
    fn upgrade(&self) -> Option<Container> {
        self.data.upgrade().map(|data| Container {data})
    }
}

impl display::Object for Container {
    fn display_object(&self) -> &display::object::Instance {
        &self.data.node
    }
}

/// Dummy content for testing.
// FIXME remove this when actual content is available.
pub(crate) fn default_content() -> DomSymbol {
    let div = web::create_div();
    div.set_style_or_panic("width","100px");
    div.set_style_or_panic("height","100px");

    let content = web::create_element("div");
    content.set_inner_html(
        r#"<svg>
  <circle style="fill: #69b3a2" stroke="black" cx=50 cy=50 r=20></circle>
</svg>
"#);
    content.set_attribute("width","100%").unwrap();
    content.set_attribute("height","100%").unwrap();

    div.append_child(&content).unwrap();

    let r          = 102_u8;
    let g          = 153_u8;
    let b          = 194_u8;
    let color      = iformat!("rgb({r},{g},{b})");
    div.set_style_or_panic("background-color",color);

    let symbol = DomSymbol::new(&div);
    symbol.dom().set_attribute("id","vis").unwrap();
    symbol.dom().style().set_property("overflow","hidden").unwrap();
    symbol

}
