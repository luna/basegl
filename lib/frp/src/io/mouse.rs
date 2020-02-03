//! Mouse FRP bindings.

use crate::prelude::*;

use crate::nodes::*;
use crate::frp_def;



// ================
// === Position ===
// ================

/// A 2-dimensional position. Used for storing the mouse position on the screen.
#[derive(Clone,Copy,Debug,Default)]
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

impl std::ops::Sub<&Position> for &Position {
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
#[derive(Debug)]
pub struct Mouse {
    /// The mouse up event.
    pub up : Dynamic<()>,
    /// The mouse down event.
    pub down : Dynamic<()>,
    /// Mouse button press status.
    pub is_down : Dynamic<bool>,
    /// Current mouse position.
    pub position : Dynamic<Position>,
}

impl Default for Mouse {
    fn default() -> Self {
        frp_def! { mouse.up        = source() }
        frp_def! { mouse.down      = source() }
        frp_def! { mouse.position  = source() }
        frp_def! { mouse.down_bool = down.constant(true) }
        frp_def! { mouse.up_bool   = up.constant(false) }
        frp_def! { mouse.is_down   = down_bool.merge(&up_bool) }
        Self {up,down,is_down,position}
    }
}

impl Mouse {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }
}
