//! This module re-exports a lot of useful stuff. It is not meant to be used
//! by libraries, but it is definitely usefull for bigger projects. It also
//! defines several aliases and utils which may find their place in new
//! libraries in the future.

#![warn(unsafe_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![feature(specialization)]
#![feature(trait_alias)]

pub mod collections;
pub mod macros;
pub mod option;
pub mod phantom;
pub mod reference;
pub mod std_reexports;
pub mod string;
pub mod tp;
pub mod wrapper;

pub use collections::*;
pub use macros::*;
pub use option::*;
pub use phantom::*;
pub use reference::*;
pub use std_reexports::*;
pub use string::*;
pub use tp::*;
pub use wrapper::*;

pub use boolinator::Boolinator;
pub use derivative::Derivative;
pub use derive_more::*;
pub use enclose::enclose;
pub use failure::Fail;
pub use ifmt::*;
pub use itertools::Itertools;
pub use lazy_static::lazy_static;
pub use num::Num;
pub use paste;
pub use shrinkwraprs::Shrinkwrap;
pub use weak_table::traits::WeakElement;
pub use weak_table::WeakValueHashMap;
pub use weak_table;

use std::cell::UnsafeCell;


// ================
// === CloneRef ===
// ================

/// Like `Clone` but should be implemented only for cheap reference-based clones. Using `clone_ref`
/// instead of `clone` makes the code more clear and makes it easier to predict its performance.
pub trait CloneRef: Sized + Clone {
    fn clone_ref(&self) -> Self {
        self.clone()
    }
}

impl CloneRef for () {
    fn clone_ref(&self) -> Self {}
}

impl<T:?Sized> CloneRef for Rc<T> {
    fn clone_ref(&self) -> Self {
        self.clone()
    }
}

/// Provides method `to`, which is just like `into` but allows fo superfish syntax.
pub trait ToImpl: Sized {
    fn to<P>(self) -> P where Self:Into<P> {
        self.into()
    }
}
impl<T> ToImpl for T {}


// TODO
// This impl should be hidden behind a flag. Not everybody using prelude want to import nalgebra.
impl <T,R,C,S> TypeDisplay for nalgebra::Matrix<T,R,C,S>
where T:nalgebra::Scalar, R:nalgebra::DimName, C:nalgebra::DimName {
    fn type_display() -> String {
        let cols = <C as nalgebra::DimName>::dim();
        let rows = <R as nalgebra::DimName>::dim();
        let item = type_name::<T>();
        match cols {
            1 => format!("Vector{}<{}>"    , rows, item),
            _ => format!("Matrix{}x{}<{}>" , rows, cols, item)
        }
    }
}

#[macro_export]
macro_rules! clone_boxed {
    ( $name:ident ) => { paste::item! {
        #[allow(missing_docs)]
        pub trait [<CloneBoxedFor $name>] {
            fn clone_boxed(&self) -> Box<dyn $name>;
        }

        impl<T:Clone+$name+'static> [<CloneBoxedFor $name>] for T {
            fn clone_boxed(&self) -> Box<dyn $name> {
                Box::new(self.clone())
            }
        }

        impl Clone for Box<dyn $name> {
            fn clone(&self) -> Self {
                self.clone_boxed()
            }
        }
    }}
}

/// Alias for `for<'t> &'t Self : Into<T>`.
pub trait RefInto<T> = where for<'t> &'t Self : Into<T>;



// =================
// === CloneCell ===
// =================

#[derive(Debug)]
pub struct CloneCell<T> {
    data : UnsafeCell<T>
}

impl<T> CloneCell<T> {
    pub fn new(elem:T) -> CloneCell<T> {
        CloneCell { data:UnsafeCell::new(elem) }
    }

    pub fn get(&self) -> T where T:Clone {
        unsafe {(*self.data.get()).clone()}
    }

    pub fn set(&self, elem:T) {
        unsafe { *self.data.get() = elem; }
    }

    pub fn take(&self) -> T where T:Default {
        let ptr:&mut T = unsafe { &mut *self.data.get() };
        std::mem::take(ptr)
    }
}

impl<T:Clone> Clone for CloneCell<T> {
    fn clone(&self) -> Self {
        Self::new(self.get())
    }
}

impl<T:Default> Default for CloneCell<T> {
    fn default() -> Self {
        Self::new(default())
    }
}



// ================================
// === RefCell<Option<T>> Utils ===
// ================================

pub trait RefcellOptionOps<T> {
    fn clear(&self);
    fn set(&self, val:T);
    fn set_if_none(&self, val:T);
}

impl<T> RefcellOptionOps<T> for RefCell<Option<T>> {
    fn clear(&self) {
        *self.borrow_mut() = None;
    }

    fn set(&self, val:T) {
        *self.borrow_mut() = Some(val);
    }

    fn set_if_none(&self, val:T) {
        let mut ptr = self.borrow_mut();
        if ptr.is_some() { panic!("The value was already set.") }
        *ptr = Some(val)
    }
}