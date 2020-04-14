#![allow(missing_docs)]

//! NOTE
//! This file is under a heavy development. It contains commented lines of code and some code may
//! be of poor quality. Expect drastic changes.

#![feature(associated_type_defaults)]
#![feature(drain_filter)]
#![feature(overlapping_marker_traits)]
#![feature(specialization)]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#![feature(unboxed_closures)]
#![feature(weak_into_raw)]
#![feature(fn_traits)]

#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unsafe_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

#![recursion_limit="512"]

pub mod app;

#[warn(missing_docs)]
pub mod component;

/// Common types and functions usable in all modules of this crate.
pub mod prelude {
    pub use ensogl::prelude::*;
}

use app::App;

use ensogl::prelude::*;
use ensogl::traits::*;

use ensogl::display;
use ensogl::display::world::*;
use ensogl::system::web;
use crate::component::node::Node;
use crate::component::node::WeakNode;
use crate::component::cursor::Cursor;
use nalgebra::Vector2;
use enso_frp as frp;
use enso_frp::Position;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use ensogl::display::object::{Id, Instance};
use ensogl::system::web::StyleSetter;


use ensogl::control::io::keyboard::listener::KeyboardFrpBindings;
use enso_frp::io::keyboard;
use enso_frp::io::keyboard::Keyboard;



macro_rules! f {
    (($($name:ident),*) ($($args:tt)*) $($expr:tt)*) => {
        {
            $(let $name = $name.clone_ref();)*
            move |$($args)*| $($expr)*
        }
    };
}

macro_rules! f_ {
    (($($name:ident),*) $($expr:tt)*) => {
        f! { ($($name),*) (_) $($expr)*  }
    };
}


macro_rules! gen_api {
    (
        $name:ident {
            $($field_vis:vis $field_name:ident : [ $($($field_ty:tt)+)? ]),* $(,)?
        }

    ) => {
        #[derive(Debug,Clone,CloneRef)]
        pub struct $name {
            $($field_vis $field_name : frp::Source $(<$($field_ty)+>)?),*
        }

        impl $name {
            pub fn new(network:&frp::Network) -> Self {
                frp::extend_network! { network
                    $(def $field_name = source();)*
                }
                Self {$($field_name),*}
            }

            $( gen_api_fn! { $field_vis $field_name ($($($field_ty)+)?) } )*
        }
    };
}

macro_rules! gen_api_fn {
    ( $vis:vis $name:ident () ) => {
        $vis fn $name(&self) {
            self.$name.emit(());
        }
    };

    ( $vis:vis $name:ident ($($arg:tt)*) ) => {
        $vis fn $name<T:AsRef<$($arg)*>>(&self, arg:T) {
            self.$name.emit(arg.as_ref());
        }
    };
}



#[derive(Clone,CloneRef,Debug,Default)]
pub struct NodeSet {
    data : Rc<RefCell<HashMap<Id,Node>>>
}

impl NodeSet {
    pub fn borrow(&self) -> Ref<HashMap<Id,Node>> {
        self.data.borrow()
    }

    pub fn take(&self) -> HashMap<Id,Node> {
        mem::take(&mut *self.data.borrow_mut())
    }

    pub fn insert(&self, node:Node) {
        self.data.borrow_mut().insert(node.id(),node);
    }

    pub fn remove(&self, node:&Node) {
        self.data.borrow_mut().remove(&node.id());
    }

    pub fn contains(&self, node:&Node) -> bool {
        self.get(node.id()).is_some()
    }

    pub fn get(&self, id:Id) -> Option<Node> {
        self.data.borrow().get(&id).map(|t| t.clone_ref())
    }

    pub fn clear(&self) {
        self.data.borrow_mut().clear();
    }
}



#[derive(Clone,CloneRef,Debug,Default)]
pub struct WeakNodeSet {
    data : Rc<RefCell<HashMap<Id,WeakNode>>>
}

impl WeakNodeSet {
    pub fn borrow(&self) -> Ref<HashMap<Id,WeakNode>> {
        self.data.borrow()
    }

    pub fn take(&self) -> HashMap<Id,WeakNode> {
        mem::take(&mut *self.data.borrow_mut())
    }

    pub fn for_each_taken<F:Fn(Node)>(&self,f:F) {
        self.take().into_iter().for_each(|(_,node)| { node.upgrade().for_each(|n| f(n)) })
    }

    pub fn for_each<F:Fn(Node)>(&self,f:F) {
        self.data.borrow().iter().for_each(|(_,node)| { node.upgrade().for_each(|n| f(n)) })
    }

    pub fn insert(&self, node:&Node) {
        self.data.borrow_mut().insert(node.id(),node.downgrade());
    }

    pub fn contains(&self, node:&Node) -> bool {
        self.get(node.id()).is_some()
    }

    pub fn get(&self, id:Id) -> Option<Node> {
        self.data.borrow().get(&id).and_then(|t| t.upgrade())
    }
}


#[derive(Clone,CloneRef,Debug,Default,Shrinkwrap)]
pub struct WeakNodeSelectionSet {
    data : WeakNodeSet
}

impl WeakNodeSelectionSet {
    pub fn clear(&self) {
        self.for_each_taken(|node| node.events.deselect.emit(()));
    }
}



#[derive(Debug,Clone,CloneRef)]
pub struct GraphEditorFrp {
    pub network : frp::Network,
    pub inputs  : FrpInputs,
    pub status  : FrpStatus,
}

impl Deref for GraphEditorFrp {
    type Target = FrpInputs;
    fn deref(&self) -> &FrpInputs {
        &self.inputs
    }
}


#[derive(Debug,Clone,CloneRef)]
pub struct FrpStatus {
    pub is_active : frp::Sampler<bool>,
    pub is_empty  : frp::Sampler<bool>,
}

gen_api! { NodesFrpInputs {
    register               : [Node],
    pub add_at             : [Position],
    pub add_at_cursor      : [],
    pub select             : [Option<WeakNode>],
    pub translate_selected : [Position],
    pub remove_selected    : [],
    pub remove_all         : [],
}}

#[derive(Debug,Clone,CloneRef)]
pub struct FrpInputs {
    pub nodes : NodesFrpInputs,
}

impl FrpInputs {
    pub fn new(network:&frp::Network) -> Self {
        let nodes = NodesFrpInputs::new(&network);
        Self {nodes}
    }
}

impl app::module::NetworkProvider for GraphEditor {
    fn network(&self) -> &frp::Network {
        &self.frp.network
    }
}

impl app::module::CommandProvider for GraphEditor {
    fn command_api() -> Vec<app::module::CommandDefinition<Self>> {
        vec! [ (app::module::CommandDefinition::new("remove_all_nodes"      , "remove all nodes"      , |t:&Self| &t.frp.inputs.nodes.remove_all))
             , (app::module::CommandDefinition::new("remove_selected_nodes" , "remove selected nodes" , |t:&Self| &t.frp.inputs.nodes.remove_selected))
             , (app::module::CommandDefinition::new("add_node_at_cursor"    , "add node at cursor position" , |t:&Self| &t.frp.inputs.nodes.add_at_cursor))
        ]
    }
}

impl app::module::StatusProvider for GraphEditor {
    fn status_api() -> Vec<app::module::StatusDefinition<Self>> {
        vec! [ (app::module::StatusDefinition::new("is_active" , "checks whether this graph editor instance is active" , |t:&Self| &t.frp.status.is_active))
             , (app::module::StatusDefinition::new("is_empty"  , "checks whether this graph editor instance is empty"  , |t:&Self| &t.frp.status.is_empty))
        ]
    }
}




#[derive(Debug,Clone,CloneRef,Default)]
pub struct NodeState {
    pub set      : NodeSet,
    pub selected : WeakNodeSelectionSet,
}

#[derive(Debug,Clone,CloneRef)]
pub struct GraphEditor {
    pub logger         : Logger,
    pub display_object : display::object::Instance,
    pub nodes          : NodeState,
    pub frp            : GraphEditorFrp,
}

#[derive(Debug,CloneRef,Derivative)]
#[derivative(Clone(bound=""))]
pub struct TouchNetwork<T:frp::Data> {
    pub down     : frp::Source<T>,
    pub up       : frp::Stream<T>,
    pub is_down  : frp::Stream<bool>,
    pub selected : frp::Stream<T>
}

impl<T:frp::Data> TouchNetwork<T> {
    pub fn new(network:&frp::Network, mouse:&frp::io::Mouse) -> Self {
        frp::extend_network! { network
            def down          = source::<T> ();
            def down_bool     = down.map(|_| true);
            def up_bool       = mouse.release.map(|_| false);
            def is_down       = down_bool.merge(&up_bool);
            def was_down      = is_down.previous();
            def mouse_up      = mouse.release.gate(&was_down);
            def pos_on_down   = mouse.position.sample(&down);
            def pos_on_up     = mouse.position.sample(&mouse_up);
            def should_select = pos_on_up.map3(&pos_on_down,&mouse.distance,Self::check);
            def up            = down.sample(&mouse_up);
            def selected      = up.gate(&should_select);
        }
        Self {down,up,is_down,selected}
    }

    fn check(end:&Position, start:&Position, diff:&f32) -> bool {
        (end-start).length() <= diff * 2.0
    }
}

#[derive(Debug,Clone,CloneRef)]
pub struct TouchState {
    pub nodes      : TouchNetwork::<Option<WeakNode>>,
    pub bg : TouchNetwork::<()>,
}

impl TouchState {
    pub fn new(network:&frp::Network, mouse:&frp::io::Mouse) -> Self {
        let nodes      = TouchNetwork::<Option<WeakNode>>::new(&network,mouse);
        let bg = TouchNetwork::<()>::new(&network,mouse);
        Self {nodes,bg}
    }
}


impl GraphEditor {

    pub fn add_node(&self) -> WeakNode {
        let node = Node::new();
        self.frp.inputs.nodes.register(&node);
        let weak_node = node.downgrade();
        weak_node
    }

    pub fn remove_node(&self, node:WeakNode) {
        if let Some(node) = node.upgrade() {
            self.nodes.set.remove(&node);
        }
    }
}

impl app::Module for GraphEditor {
    const LABEL : &'static str = "GraphEditor";

    fn new(app:&App) -> Self {
        let logger = Logger::new("GraphEditor");
        let scene  = app.world.scene();
        let cursor = Cursor::new();
        web::body().set_style_or_panic("cursor","none");
        app.world.add_child(&cursor);


        let display_object = display::object::Instance::new(logger.clone());
        let mouse          = &scene.mouse.frp;
        let network        = frp::Network::new();
        let inputs         = FrpInputs::new(&network);
        let nodes          = NodeState::default();
        let touch          = TouchState::new(&network,mouse);


        frp::extend_network! { network

        // === Cursor ===

        def mouse_on_down_position = mouse.position.sample(&mouse.press);
        def selection_zero         = source::<Position>();
        def selection_size_down    = mouse.position.map2(&mouse_on_down_position,|m,n|{m-n});
        def selection_size_if_down = selection_size_down.gate(&touch.bg.is_down);
        def selection_size_on_down = selection_zero.sample(&mouse.press);
        def selection_size         = selection_size_if_down.merge(&selection_size_on_down);

        def _cursor_size = selection_size.map(f!((cursor)(p) {
            cursor.set_selection_size(Vector2::new(p.x as f32,p.y as f32));
        }));

        def _cursor_press = mouse.press.map(f!((cursor)(_) {
            cursor.events.press.emit(());
        }));

        def _cursor_release = mouse.release.map(f!((cursor)(_) {
            cursor.events.release.emit(());
        }));

        def _cursor_position = mouse.position.map(f!((cursor)(p) {
            cursor.set_position(Vector2::new(p.x as f32,p.y as f32));
        }));


        // === Generic Selection ===

        def mouse_down_target  = mouse.press.map(f_!((scene) scene.mouse.target.get()));
        def _mouse_down_target = mouse_down_target.map(f!((touch,scene)(target) {
            match target {
                display::scene::Target::Background => {
                    touch.bg.down.emit(());
                }
                display::scene::Target::Symbol {instance_id,..} => {
                    scene.shapes.get_mouse_target(&(*instance_id as usize)).for_each(|target| {
                        target.mouse_down().for_each(|t| t.emit(()));
                    })
                }
            }
        }));


        // === Selection ===

        def _deselect_all_on_bg_press = touch.bg.selected.map(f_!((nodes) nodes.selected.clear()));
        def select_unified            = inputs.nodes.select.merge(&touch.nodes.selected);
        def _select_pressed           = select_unified.map(f!((nodes)(opt_node) {
            opt_node.for_each_ref(|weak_node| {
                weak_node.upgrade().map(|node| {
                    nodes.selected.clear();
                    node.events.select.emit(());
                    nodes.selected.insert(&node);
                })
            })
        }));


        // === Add Node ===

        def add_node_at_cursor_pos = inputs.nodes.add_at_cursor.map2(&mouse.position,|_,p|{*p});
        def add_node               = inputs.nodes.add_at.merge(&add_node_at_cursor_pos);
        def _add_new_node          = add_node.map(f!((inputs)(pos) {
            let node = Node::new();
            inputs.nodes.register(&node);
            node.mod_position(|t| {
                t.x += pos.x as f32;
                t.y += pos.y as f32;
            });
        }));

        def _new_node = inputs.nodes.register.map(f!((network,nodes,touch,display_object)(node) {
            let weak_node = node.downgrade();
            frp::new_subnetwork! { [network,node.view.events.network]
                def foo_ = node.view.events.mouse_down.map(f_!((touch) {
                    touch.nodes.down.emit(Some(weak_node.clone_ref()))
                }));
            }
            display_object.add_child(node);
            nodes.set.insert(node.clone_ref());
        }));


        // === Remove Node ===

        def _remove_all      = inputs.nodes.remove_all.map(f!((nodes)(()) nodes.set.clear()));
        def _remove_selected = inputs.nodes.remove_selected.map(f!((nodes,nodes)(_) {
            nodes.selected.for_each_taken(|node| nodes.set.remove(&node))
        }));


        // === Move Nodes ===

        def mouse_tx_if_node_pressed = mouse.translation.gate(&touch.nodes.is_down);
        def _move_node_with_mouse    = mouse_tx_if_node_pressed.map2(&touch.nodes.down,|tx,node| {
            node.mod_position(|p| { p.x += tx.x; p.y += tx.y; })
        });

        def _move_selected_nodes = inputs.nodes.translate_selected.map(f!((nodes)(t) {
            nodes.selected.for_each(|node| {
                node.mod_position(|p| {
                    p.x += t.x;
                    p.y += t.y;
                })
            })
        }));


        // === Status ===

        def is_active_src = source::<bool>();
        def is_empty_src  = source::<bool>();
        def is_active = is_active_src.sampler();
        def is_empty  = is_empty_src.sampler();

        }

        is_active_src.emit(true);

        app.shortcuts.add (
            &[keyboard::Key::Character("n".into())],
            app::shortcut::Rule::new_(Self::LABEL, "add_node_at_cursor")
        );

        app.shortcuts.add (
            &[keyboard::Key::Backspace],
            app::shortcut::Rule::new_(Self::LABEL, "remove_selected_nodes")
        );

        let status = FrpStatus {is_active,is_empty};

        let frp = GraphEditorFrp {network,inputs,status};

        Self {logger,frp,nodes,display_object}
    }
}

impl display::Object for GraphEditor {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}
