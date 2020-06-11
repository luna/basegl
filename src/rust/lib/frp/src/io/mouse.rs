//! Mouse FRP bindings.

use crate::prelude::*;

use crate as frp;
use nalgebra::Vector2;



// =============
// === Mouse ===
// =============

/// Mouse FRP bindings.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Mouse {
    pub network       : frp::Network,
    pub release       : frp::Source,
    pub press         : frp::Source,
    pub wheel         : frp::Source,
    pub leave         : frp::Source,
    pub is_down          : frp::Stream<bool>,
    pub is_up            : frp::Stream<bool>,
    pub position      : frp::Source<Vector2<f32>>,
    pub prev_position : frp::Stream<Vector2<f32>>,
    pub translation   : frp::Stream<Vector2<f32>>,
    pub distance      : frp::Stream<f32>,
    pub ever_moved    : frp::Stream<bool>,
}

impl Default for Mouse {
    fn default() -> Self {
        frp::new_network! { network
            release       <- source_();
            press         <- source_();
            wheel         <- source_();
            leave         <- source_();
            position      <- source();
            is_down          <- bool(&release,&press);
            is_up            <- is_down.map(|t|!t);
            prev_position <- position.previous();
            translation   <- position.map2(&prev_position,|t,s|t-s);
            distance      <- translation.map(|t:&Vector2<f32>|t.norm());
            ever_moved    <- position.constant(true);
        };
        Self {network,release,press,leave,wheel,is_down,is_up,position,prev_position,translation,distance
             ,ever_moved}
    }
}

impl Mouse {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }
}
