//! Provides generic operations that can be applied to UI components.
use crate::prelude::*;

use ensogl::display::DomScene;
use ensogl::display::DomSymbol;
use ensogl::display::Scene;
use ensogl::display::Symbol;
use ensogl::display::scene::View;
use ensogl::display::traits::*;
use ensogl::display;
use ensogl::frp;
use ensogl::gui::component::animation;


/// Indicates the required target layer.
// FIXME this is a layer management hack. Remove this once we have nicer scene layer management.
#[derive(Debug)]
pub enum SymbolType {
    /// A symbol that goes onto the `Main` layer.
    Main (Symbol),
    /// A visualisation symbol that goes above the `Main` layer, but below the cursor.
    Visualisation (Symbol),
}

// ==================================
// === UI Component Helper Traits ===
// ==================================

/// Should be implemented by UI component that consist of `Symbol`. Provides access to the shapes
/// and some helper methods for working with those shapes.
pub trait NativeUiElement {
    /// Return all `Symbol`s that make up this component.
    fn symbols(&self) -> Vec<SymbolType>;

    /// Change the scene layer of all `Symbol`s.
    fn set_layer(&self, layer:&View) {
        self.symbols().iter().for_each(|symbol| {
            match symbol{
                SymbolType::Main(symbol)
                | SymbolType::Visualisation(symbol)
                => layer.add(symbol),
            }
        })
    }

    /// Remove the `Symbol`s from all scene layers.
    fn unset_layers_all(&self, scene:&Scene) {
        self.symbols().iter().for_each(|symbol|   match symbol{
            SymbolType::Main(symbol)
            | SymbolType::Visualisation(symbol)
            =>scene.views.remove_symbol(symbol),
        })
    }
}

/// Should be implemented by UI component that consist of `DomSymbol`s. Provides access to the
/// symbols some helper methods for working with them.
pub trait HtmlUiElement {
    /// Return all `DomSymbol`s that make up this component.
    fn elements(&self) -> Vec<DomSymbol>;

    /// Change the `DomScene` of all `DomSymbol`s.
    fn set_layer(&self, scene:&DomScene) {
        self.elements().iter().for_each(|element| scene.manage(&element))
    }
}

/// Provides information and functionality to resize an element. A complex UI component should
/// implement this and propagate size and layout changes to all its sub-components.
pub trait Resizable {
    /// Set the size for the UI component.
    fn set_size(&self, size:Vector3<f32>);
    /// Return the size of the UI element.
    fn size(&self) -> Vector3<f32>;
    /// Return an FRP endpoint to set the size.
    fn set_size_frp(&self) -> &frp::Source<Option<Vector2<f32>>> { unimplemented!() }
}

/// A component that owns a Network. Can be used to create external animations.
pub trait Networked {
    /// Return a reference to the components network.
    fn network(&self) -> &frp::Network;
}

// ==================================
// === Fullscreen Operator Handle ===
// ==================================

/// FullscreenOperatorCellHandle is a helper type that wraps a `FullscreenOperator` and applies an
/// undos the operator on the inner component. This can be used to ensure that only a single
/// component is made fullscreen at any time.
#[derive(Debug,CloneRef,Derivative)]
#[derivative(Clone(bound=""))]
#[derivative(Default(bound=""))]
pub struct FullscreenOperatorHandle<T> {
    operator: Rc<RefCell<Option<FullscreenOperator<T>>>>
}


impl<T:display::Object+Resizable+NativeUiElement+CloneRef+Networked+'static> FullscreenOperatorHandle<T> {
    /// returns whether there is a component that is in fullscreen mode.
    pub fn is_active(&self) -> bool {
        self.operator.borrow().is_some()
    }

    /// Enables fullscreen mode for the given component. If there is another component already in
    /// fullscreen mode, it disables fullscreen for that component.
    pub fn set_fullscreen(&self, target:T, scene:Scene) {
        self.disable_fullscreen();
        let operator = FullscreenOperator::apply(target,scene);
        self.operator.set(operator);
    }

    /// Disables fullscreen mode for the given component.
    pub fn disable_fullscreen(&self) {
        if let Some(old) = self.operator.borrow_mut().take() {
            old.undo();
        }
    }

    /// Return a ref clone of the fullscreen element.
    pub fn get_element(&self) -> Option<T>{
        self.operator.borrow().as_ref().map(|op| op.target.clone_ref())
    }
}



// ===============================
// === Layer Management Helper ===
// ===============================

/// FIXME This is an ugly hack for layer management.
/// FIXME Needs to be removed as soon as we have something better.
pub fn set_layers_normal<T:NativeUiElement>(target:&T, scene:&Scene){
    target.unset_layers_all(&scene);
    for symbol in target.symbols() {
        match symbol {
            SymbolType::Main(symbol)          => scene.views.main.add(&symbol),
            SymbolType::Visualisation(symbol) => scene.views.visualisation.add(&symbol),
        }
    }
}

/// FIXME This is an ugly hack for layer management.
/// FIXME Needs to be removed as soon as we have something better.
pub fn set_layers_fullscreen<T:NativeUiElement>(target:&T, scene:&Scene) {
    target.unset_layers_all(&scene);
    for symbol in target.symbols() {
        match symbol {
            SymbolType::Main(symbol)          => scene.views.overlay.add(&symbol) ,
            SymbolType::Visualisation(symbol) => scene.views.overlay_visualisation.add(&symbol) ,
        }
    }
}



// ===========================
// === Fullscreen Operator ===
// ===========================

/// A `FullscreenOperator` can be used to apple fullscreen mode to a UI element as well as undo the
/// undo the fullscreen operation and restore the previous state. The  `FullscreenOperator` can be
/// applied to any target that implements `display::Object`, `Resizable` and `NativeUiElement`.
// TODO consider incorporating these traits into display::Object or another common "SceneElement"
// type. But it important that complex UI components can provide information about their
// sub-components (for example, multiple sub-shapes or HTML components).
#[derive(Debug)]
pub struct FullscreenOperator<T> {
    target            : T,
    scene             : Scene,
    size_original     : Vector3<f32>,
    position_original : Vector3<f32>,
    parent_original   : Option<display::object::Instance>,
}

impl<T:display::Object+Resizable+NativeUiElement+Networked+CloneRef+'static> FullscreenOperator<T> {

    /// Make the provided target fullscreen within the given scene and return the
    /// `FullscreenOperator`.
    pub fn apply(target:T, scene:Scene) -> Self {
        let size_original     = target.size();
        let position_original = target.position();
        let parent_original   = target.display_object().rc.parent();
        FullscreenOperator {target,scene,size_original,position_original,parent_original}.init()
    }

    fn init(self) -> Self {
        let original_pos = self.target.display_object().global_position();

        // Change parent
        self.target.display_object().set_parent(self.scene.display_object());
        self.target.unset_layers_all(&self.scene);
        set_layers_fullscreen(&self.target, &self.scene);

        let margin = 0.1;
        let scene_shape = self.scene.shape();
        let size_new    = Vector3::new(scene_shape.width(), scene_shape.height(),0.0) * (1.0 - margin);

        // TODO Currently we assume objects are center aligned, but this needs to be properly
        // accounted for here.

        let frp_network      = &self.target.network().clone_ref();
        let target_pos       = size_new/2.0;
        let original_size    = self.size_original;
        let target_size      = size_new;
        let target           = self.target.clone_ref();
        let resize_animation = animation(frp_network, move |value| {
            let pos  = original_pos  * (1.0 - value) + target_pos * value;
            let size = original_size * (1.0 - value) + target_size * value;
            target.set_position(pos);
            target.set_size(size);
        });
        resize_animation.set_target_position(1.0);

        self.scene.views.toggle_overlay_cursor();

        self
    }

    /// Undo the fullscreen operation and restore the previous state exactly as it was.
    pub fn undo(self) {
        let global_pos_start = self.target.global_position();

        self.target.unset_layers_all(&self.scene);
        set_layers_normal(&self.target, &self.scene);

        if let Some(parent) = self.parent_original.as_ref() {
            self.target.display_object().set_parent(&parent);
        }

        let parent_pos     = self.parent_original.map(|p| p.global_position());
        let parent_pos     = parent_pos.unwrap_or_else(Vector3::zero);
        let mut source_pos = self.target.position();
        source_pos        += global_pos_start ;
        source_pos        -= parent_pos ;
        source_pos        -= self.target.size() / 2.0;
        let source_pos     = source_pos;

        self.target.set_position(source_pos);

        let original_pos     = self.target.position();
        let target_pos       = self.position_original;
        let original_size    = self.target.size();
        let target_size      = self.size_original;
        let target           = self.target.clone_ref();
        let frp_network      = &self.target.network().clone_ref();
        let resize_animation = animation(frp_network, move |value| {
            let pos  = original_pos  * (1.0 - value) + target_pos * value;
            let size = original_size * (1.0 - value) + target_size * value;
            target.set_position(pos);
            target.set_size(size);
        });
        resize_animation.set_target_position(1.0);

        self.scene.views.toggle_overlay_cursor();
    }
}
