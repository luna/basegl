use crate::prelude::*;

use crate::data::function::callback::*;
use crate::data::opt_vec::OptVec;
use crate::dirty;
use crate::dirty::traits::*;
use crate::display::symbol::scope;
use crate::system::web::Logger;
use crate::system::web::group;
use crate::system::web::fmt;
use std::slice::SliceIndex;
use crate::closure;
use paste;
use num_enum::IntoPrimitive;
use crate::{promote, promote_all, promote_scope_types};
use eval_tt::*;

// ================
// === Geometry ===
// ================

// === Definition ===

#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Geometry<OnDirty> {
    #[shrinkwrap(main_field)]
    pub scopes       : Scopes      <OnDirty>,
    pub scopes_dirty : ScopesDirty <OnDirty>,
    pub logger       : Logger,
}

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Scopes<OnDirty> {
    pub point     : AttributeScope <OnDirty>,
    pub vertex    : AttributeScope <OnDirty>,
    pub primitive : AttributeScope <OnDirty>,
    pub instance  : AttributeScope <OnDirty>,
    pub object    : UniformScope   <OnDirty>,
    pub global    : GlobalScope    <OnDirty>,
}

#[derive(Copy,Clone,Debug,IntoPrimitive)]
#[repr(u8)]
pub enum ScopesDirtyStatus {
    point,
    vertex,
    primitive,
    instance,
    object,
    global,
}

impl From<ScopesDirtyStatus> for usize {
    fn from(t: ScopesDirtyStatus) -> Self {
        Into::<u8>::into(t).into()
    }
}

// === Types ===

pub type ScopesDirty    <Callback> = dirty::SharedEnum<u8,ScopesDirtyStatus, Callback>;
pub type AttributeScope <Callback> = scope::Scope<ScopeOnChange<Callback>>;
pub type UniformScope   <Callback> = scope::Scope<ScopeOnChange<Callback>>; // FIXME
pub type GlobalScope    <Callback> = scope::Scope<ScopeOnChange<Callback>>; // FIXME

promote_scope_types!{ [ScopeOnChange] scope }
#[macro_export]
macro_rules! promote_geometry_types { ($($args:tt)*) => {
    crate::promote_scope_types! { $($args)* }
    promote! { $($args)*
        [Geometry,Scopes,AttributeScope,UniformScope,GlobalScope]
    }
};}

// === Callbacks ===

closure! {
fn scope_on_change<C:Callback0>(dirty:ScopesDirty<C>, item:ScopesDirtyStatus) ->
    ScopeOnChange { || dirty.set_with((item,)) }
}

// === Implementation ===

impl<OnDirty: Callback0> Geometry<OnDirty> {
    pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
        let scopes_logger = logger.sub("scopes_dirty");
        let scopes_dirty  = ScopesDirty::new(scopes_logger,on_dirty);
        let scopes        = group!(logger, "Initializing.", {
            macro_rules! new_scope { ($cls:ident { $($name:ident),* } ) => {$(
                let sub_logger = logger.sub(stringify!($name));
                let status_mod = ScopesDirtyStatus::$name;
                let scs_dirty  = scopes_dirty.clone();
                let callback   = scope_on_change(scs_dirty, status_mod);
                let $name      = $cls::new(sub_logger, callback);
            )*}}

            new_scope!(AttributeScope { point, vertex, primitive, instance });
            new_scope!(AttributeScope { object });
            new_scope!(AttributeScope { global });

            Scopes { point, vertex, primitive, instance, object, global }
        });
        Self { scopes, scopes_dirty, logger }
    }

    pub fn update(&mut self) {
        group!(self.logger, "Updating.", {
            if self.scopes_dirty.check() {
                if self.scopes_dirty.check_for(&(ScopesDirtyStatus::point,)) {
                    self.scopes.point.update()
                }
                if self.scopes_dirty.check_for(&(ScopesDirtyStatus::vertex,)) {
                    self.scopes.vertex.update()
                }
                if self.scopes_dirty.check_for(&(ScopesDirtyStatus::primitive,)) {
                    self.scopes.primitive.update()
                }
                if self.scopes_dirty.check_for(&(ScopesDirtyStatus::instance,)) {
                    self.scopes.instance.update()
                }
                if self.scopes_dirty.check_for(&(ScopesDirtyStatus::object,)) {
                    self.scopes.object.update()
                }
                if self.scopes_dirty.check_for(&(ScopesDirtyStatus::global,)) {
                    self.scopes.global.update()
                }
                self.scopes_dirty.unset()
            }
        })
    }

}



