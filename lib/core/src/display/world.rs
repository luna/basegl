#![allow(missing_docs)]

#[warn(missing_docs)]
pub mod event_loop;
#[warn(missing_docs)]
pub mod scene;
#[warn(missing_docs)]
pub mod workspace;

use crate::prelude::*;

pub use crate::data::container::*;
pub use crate::display::world::workspace::SymbolId;

use crate::closure;
use crate::control::callback::CallbackHandle;
use crate::data::dirty;
use crate::data::dirty::traits::*;
use crate::debug::stats::Stats;
use crate::promote_all;
use crate::promote_workspace_types;
use crate::promote;
use crate::system::web;
use crate::system::web::group;
use crate::system::web::Logger;
use crate::display::shape::text::font::Fonts;
use crate::debug::monitor;
use crate::debug::monitor::Monitor;
use crate::debug::monitor::Panel;
use crate::system::gpu::scope::uniform::UniformScope;
use crate::system::gpu::scope::uniform::Uniform;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, JsValue};

use web_sys::{Performance,KeyboardEvent};
use event_loop::EventLoop;
use eval_tt::*;



// =============
// === World ===
// =============

// === Definition ===

/// Shared reference to the `World` object.
#[derive(Clone,Debug)]
pub struct World {
    pub rc: Rc<RefCell<WorldData>>,
}

impl World {
    /// Create new shared reference.
    pub fn new(world: WorldData) -> Self {
        let rc = Rc::new(RefCell::new(world));
        Self {rc}
    }

    /// Cheap clone of the world reference.
    pub fn clone_ref(&self) -> Self {
        self.clone()
    }

    /// Dispose the world object, cancel all handlers and events.
    pub fn dispose(&self) {
        self.rc.borrow_mut().dispose()
    }

    /// Run the provided callback on every frame. Returns a `CallbackHandle`,
    /// which when dropped will cancel the callback. If you want the function
    /// to run forever, you can use the `forget` method in the handle.
    pub fn on_frame<F:FnMut(&World)+'static>
    (&self, mut callback:F) -> CallbackHandle {
        let this = self.clone_ref();
        let func = move || callback(&this);
        self.rc.borrow_mut().event_loop.add_callback(func)
    }

    pub fn mod_stats<F:FnOnce(&Stats)>(&self, f:F) {
        f(&self.rc.borrow().stats);
    }
}

impl<T> Add<T> for World where WorldData: Add<T> {
    type Result = AddResult<WorldData,T>;
    fn add(&mut self, t:T) -> Self::Result {
        self.rc.borrow_mut().add(t)
    }
}

impl Deref for World {
    type Target = Rc<RefCell<WorldData>>;

    fn deref(&self) -> &Self::Target {
        &self.rc
    }
}



// ====================
// === StatsMonitor ===
// ====================

#[derive(Clone,Debug)]
pub struct StatsMonitor {
    rc: Rc<RefCell<StatsMonitorData>>
}

impl StatsMonitor {
    pub fn new(stats:&Stats) -> Self {
        let rc = Rc::new(RefCell::new(StatsMonitorData::new(stats)));
        Self {rc}
    }

    pub fn begin(&self) {
        self.rc.borrow_mut().begin()
    }

    pub fn end(&self) {
        self.rc.borrow_mut().end()
    }
}


#[derive(Debug)]
pub struct StatsMonitorData {
    stats   : Stats,
    monitor : Monitor,
    panels  : Vec<Panel>
}

impl StatsMonitorData {
    fn new(stats:&Stats) -> Self {
        let stats       = stats.clone_ref();
        let mut monitor = Monitor::new();
        let panels = vec![
            monitor.add( monitor::FrameTime          :: new()       ),
            monitor.add( monitor::Fps                :: new()       ),
            monitor.add( monitor::WasmMemory         :: new()       ),
            monitor.add( monitor::GpuMemoryUsage     :: new(&stats) ),
            monitor.add( monitor::DrawCallCount      :: new(&stats) ),
            monitor.add( monitor::DataUploadCount    :: new(&stats) ),
            monitor.add( monitor::DataUploadSize     :: new(&stats) ),
            monitor.add( monitor::BufferCount        :: new(&stats) ),
            monitor.add( monitor::SymbolCount        :: new(&stats) ),
            monitor.add( monitor::ShaderCount        :: new(&stats) ),
            monitor.add( monitor::ShaderCompileCount :: new(&stats) ),
            monitor.add( monitor::SpriteSystemCount  :: new(&stats) ),
            monitor.add( monitor::SpriteCount        :: new(&stats) ),
        ];
        Self {stats,monitor,panels}
    }

    fn begin(&mut self) {
        for panel in &self.panels {
            panel.begin();
        }
    }

    fn end(&mut self) {
        for panel in &self.panels {
            panel.end();
        }
        self.monitor.draw();
        self.stats.reset();
    }
}



// =================
// === WorldData ===
// =================

// === Definition ===

/// World is the top-level structure managing several instances of `Workspace`.
/// It is responsible for updating the system on every animation frame.
#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct WorldData {
    pub workspace       : Workspace,
    pub workspace_dirty : WorkspaceDirty,
    pub logger          : Logger,
    pub event_loop      : EventLoop,
    pub variables       : UniformScope,
    pub performance     : Performance,
    pub start_time      : f32,
    pub time            : Uniform<f32>,
    pub display_mode    : Uniform<i32>,
    pub fonts           : Fonts,
    pub update_handle   : Option<CallbackHandle>,
    pub stats           : Stats,
    pub stats_monitor   : StatsMonitor,
}


// === Types ===

pub type WorkspaceID    = usize;
pub type WorkspaceDirty = dirty::SharedBool;
promote_workspace_types!{ [[WorkspaceOnChange]] workspace }


// === Callbacks ===

closure! {
fn workspace_on_change(dirty:WorkspaceDirty) -> WorkspaceOnChange {
    || dirty.set()
}}


// === Implementation ===

impl WorldData {
    /// Create and initialize new world instance.
    #[allow(clippy::new_ret_no_self)]
    pub fn new<Dom:Str>(dom:Dom) -> World {
        println!("NOTICE! When profiling in Chrome check 'Disable JavaScript Samples' under the \
                  gear icon in the 'Performance' tab. It can drastically slow the rendering.");
        let world     = World::new(Self::new_uninitialized(dom));
        let world_ref = world.clone_ref();
        with(world.borrow_mut(), |mut data| {
            let update          = move || world_ref.borrow_mut().run();
            let update_handle   = data.event_loop.add_callback(update);
            data.update_handle  = Some(update_handle);
        });

        // -----------------------------------------------------------------------------------------
        // FIXME: Hacky way of switching display_mode. To be fixed and refactored out.
        let world_copy = world.clone();
        let c: Closure<dyn Fn(JsValue)> = Closure::wrap(Box::new(move |val| {
            let val = val.unchecked_into::<KeyboardEvent>();
            let key = val.key();
            if      key == "0" { world_copy.borrow_mut().display_mode.set(0) }
            else if key == "1" { world_copy.borrow_mut().display_mode.set(1) }
        }));
        (&web::document().unwrap() as &web_sys::EventTarget).add_event_listener_with_callback("keydown",c.as_ref().unchecked_ref());
        c.forget();
        // -----------------------------------------------------------------------------------------

        world
    }

    /// Create new uninitialized world instance. You should rather not need to
    /// call this function directly.
    fn new_uninitialized<Dom:Str>(dom:Dom) -> Self {
        let stats                  = default();
        let logger                 = Logger::new("world");
        let workspace_logger       = logger.sub("workspace");
        let workspace_dirty_logger = logger.sub("workspace_dirty");
        let workspace_dirty        = WorkspaceDirty::new(workspace_dirty_logger,());
        let on_change              = workspace_on_change(workspace_dirty.clone_ref());
        let variables              = UniformScope::new(logger.sub("global_variables"));
        let time                   = variables.add_or_panic("time",0.0);
        let display_mode           = variables.add_or_panic("display_mode",0);
        let workspace              = Workspace::new(dom,&variables,workspace_logger,&stats,on_change).unwrap(); // fixme unwrap
        let fonts                  = Fonts::new();
        let event_loop             = EventLoop::new();
        let update_handle          = default();
        let stats_monitor          = StatsMonitor::new(&stats);
        let performance            = web::get_performance().unwrap();
        let start_time             = performance.now() as f32;
        let stats_monitor_cp_1     = stats_monitor.clone();
        let stats_monitor_cp_2     = stats_monitor.clone();
        event_loop.set_on_loop_started  (move || { stats_monitor_cp_1.begin(); });
        event_loop.set_on_loop_finished (move || { stats_monitor_cp_2.end();   });
        Self {workspace,workspace_dirty,logger,event_loop,variables,performance,start_time,time,display_mode,fonts,update_handle,stats,stats_monitor}
    }

    pub fn run(&mut self) {
        let relative_time = self.performance.now() as f32 - self.start_time;
        self.time.set(relative_time);
        self.update();
    }

    /// Check dirty flags and update the state accordingly.
    pub fn update(&mut self) {
        //        if self.workspace_dirty.check_all() {
        group!(self.logger, "Updating.", {
        // FIXME render only needed workspaces.
        self.workspace_dirty.unset_all();
        let fonts = &mut self.fonts;
        self.workspace.update(fonts);
            });
//        }
    }

    /// Dispose the world object, cancel all handlers and events.
    pub fn dispose(&mut self) {
        self.update_handle = None;
    }
}

impl Drop for WorldData {
    fn drop(&mut self) {
        self.logger.info("Dropping.");
    }
}


