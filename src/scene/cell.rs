use crate::types::*;
use crate::log::Logger;
use crate::scene::timestepper::Integrate;
use std::any::Any;

#[derive(Clone, Debug, PartialEq)]
pub enum CellTypes {
    Solid,
    Fluid,
}

#[derive(Clone, Debug)]
pub struct Cell {
    /// The index of the cell.
    index: Index2,

    /// The mode of the Cell, fluid or solid.
    pub mode: CellTypes,

    /// Velocity x,y:
    /// - v_x is at the location (h/2, 0),
    /// - v_y is at the location (0, h/2),
    pub velocity: FrontBackBuffer<Vector2>,

    /// The pressure value.
    pub pressure: Scalar,

    /// The advected smoke value in `[0,1]`.
    pub smoke: FrontBackBuffer<Scalar>,

    /// The divergence in the cell.
    /// Corresponds to the net-outflow.
    pub div: Scalar,

    // Fields for parallel computation (only).
    //  ================================================================
    /// Divergence normed (only for parallel computation).
    pub div_normed: Scalar,

    /// Divergence ratio for velocity correction (only for parallel computation).
    /// For fluid cells: `1.0 / (Sum(fluid neighbors))` =
    ///                  `1.0 / s_nbs.sum()`
    pub s_tot_inv: Scalar,

    /// Flag denoting if neighbor is a fluid cell:
    /// `[neg-direction, pos-direction]`  (only for parallel computation).
    pub s_nbs: [Vector2; 2],
    // ==================================================================
}

impl Cell {
    pub fn new(index: Index2) -> Self {
        let default_vel = Vector2::from_element(0.0);
        let default_pressure = 0.0;
        let default_smoke = 0.0;

        return Cell {
            index,
            mode: CellTypes::Fluid,
            velocity: FrontBackBuffer {
                front: default_vel,
                back: default_vel,
            },
            pressure: default_pressure,
            smoke: FrontBackBuffer {
                front: default_smoke,
                back: default_smoke,
            },
            div: 0.0,

            div_normed: 0.0,
            s_tot_inv: 0.0,
            s_nbs: [Vector2::zeros(), Vector2::zeros()],
        };
    }

    pub fn index(&self) -> Index2 {
        return self.index;
    }
}

impl Integrate for Cell {
    fn integrate(&mut self, _log: &Logger, dt: Scalar, gravity: Vector2) {
        self.velocity.back = match self.mode {
            CellTypes::Solid => self.velocity.back,
            CellTypes::Fluid => self.velocity.back + dt * gravity,
        };
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
