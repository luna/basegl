//! The `Registry` provides a mechanism to store `Factory`s for all available visualizations. It
//! provides functionality to register new factories, as well as get suitable factories for
//! a specific data type.
//!
//! Example
//! --------
//! ```
//! use graph_editor::component::visualization::Registry;
//! use graph_editor::component::visualization::EnsoType;
//! use graph_editor::component::visualization::JsSourceFactory;
//!
//! // Instantiate a pre-populated registry.
//! let registry = Registry::with_default_visualisations();
//! // Add a new factory that creates visualisations defined in JS.
//! registry.register_factory(JsSourceFactory::from_js_source_raw(r#"
//! class BubbleVisualisation {
//!     onDataReceived(root, data) {}
//!     setSize(root, size) {}
//! }
//! return new BubbleVisualisation();
//! "#.into()));
//!
//! // Get all factories that can render  visualisation for the type `[[float;3]]`.
//! let target_type:EnsoType = "[[float;3]]".to_string().into();
//! assert!(registry.valid_sources(&target_type).len() > 0);
//! ```

use crate::prelude::*;

use crate::component::visualization::EnsoType;
use crate::component::visualization::Factory;
use crate::component::visualization::Metadata;
use crate::component::visualization::NativeConstructorFactory;
use crate::component::visualization::Visualization;
use crate::component::visualization::renderer::example::js::constructor_sample_js_bubble_chart;
use crate::component::visualization::renderer::example::native::BubbleChart;

use ensogl::display::scene::Scene;



// ==============================
// === Visualization Registry ===
// ==============================

/// HashMap that contains the mapping from `EnsoType`s to a `Vec` of `Factories. This is meant to
/// map a `EnsoType` to all `Factory`s that support visualising that type.
type RegistryTypeMap = HashMap<EnsoType, Vec<Rc<dyn Factory>>>;

/// The registry struct. For more information see the module description.
#[derive(Clone,CloneRef,Default,Debug)]
#[allow(missing_docs)]
pub struct Registry {
    entries : Rc<RefCell<RegistryTypeMap>>,
}

impl Registry {
    /// Return an empty `Registry`.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Return a `Registry` prepopulated with default visualizations.
    pub fn with_default_visualisations() -> Self {
        let registry = Self::empty();
        // FIXME use proper enso types here.
        registry.register_factory(NativeConstructorFactory::new(
            Metadata {
                name        : "Bubble Visualisation (native)".to_string(),
                input_types : vec!["[[float;3]]".to_string().into()],
            },
            Rc::new(|scene:&Scene| Ok(Visualization::new(BubbleChart::new(scene))))
        ));
        registry.register_factory(NativeConstructorFactory::new(
            Metadata {
                name        : "Bubble Visualisation (JS)".to_string(),
                input_types : vec!["[[float;3]]".to_string().into()],
            },
            Rc::new(|scene:&Scene| {
                let renderer = constructor_sample_js_bubble_chart();
                renderer.set_dom_layer(&scene.dom.layers.front);
                Ok(Visualization::new(renderer))
            })
        ));

        registry
    }

    /// Register a new visualisation factory with the registry.
    pub fn register_factory<T:Factory + 'static>(&self, factory:T) {
        self.register_factory_rc(Rc::new(factory));
    }

    /// Register a new visualisation factory that's pre-wrapped in an `Rc` with the registry.
    pub fn register_factory_rc(&self, factory:Rc<dyn Factory>) {
        let spec = factory.metadata();
        for dtype in &spec.input_types {
            let mut entries = self.entries.borrow_mut();
            let entry_vec = entries.entry(dtype.clone()).or_insert_with(default);
            entry_vec.push(Rc::clone(&factory));
        }

    }

    /// Return all `Factory`s that can create a visualisation for the given datatype.
    pub fn valid_sources(&self, dtype:&EnsoType) -> Vec<Rc<dyn Factory>>{
        let entries       = self.entries.borrow();
        entries.get(dtype).cloned().unwrap_or_else(default)
    }
}
