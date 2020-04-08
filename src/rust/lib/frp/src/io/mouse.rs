//! Mouse FRP bindings.

use crate::prelude::*;

use crate as frp;



// ================
// === Position ===
// ================

/// A 2-dimensional position. Used for storing the mouse position on the screen.
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
#[allow(missing_docs)]
pub struct Position {
    pub x:i32,
    pub y:i32,
}

impl Position {
    /// Constructor.
    pub fn new(x:i32, y:i32) -> Self {
        Self {x,y}
    }
}

impl Sub<&Position> for &Position {
    type Output = Position;
    fn sub(self, rhs: &Position) -> Self::Output {
        let x = self.x - rhs.x;
        let y = self.y - rhs.y;
        Position {x,y}
    }
}



// =============
// === Mouse ===
// =============

/// Mouse FRP bindings.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Mouse {
    pub network  : frp::Network,
    pub on_up    : frp::Source,
    pub on_down  : frp::Source,
    pub on_wheel : frp::Source,
    pub on_leave : frp::Source,
    pub is_down  : frp::Stream<bool>,
    pub position : frp::Source<Position>,
}

impl Default for Mouse {
    fn default() -> Self {
        frp::new_network! { mouse
            def on_up     = source_();
            def on_down   = source_();
            def on_wheel  = source_();
            def on_leave  = source_();
            def position  = source();
            def down_bool = on_down.constant(true);
            def up_bool   = on_up.constant(false);
            def is_down   = down_bool.merge(&up_bool);
        };
        let network = mouse;
        Self {network,on_up,on_down,on_leave,on_wheel,is_down,position}
    }
}

impl Mouse {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }
}
