use std::ops::{Index, IndexMut};

use crate::solver::timestepper::Integrate;
use crate::types::*;

#[derive(Clone, Debug)]
pub enum CellTypes {
    Solid,
    Fluid,
}

#[derive(Clone, Debug, Copy)]
pub struct GlobalIndex(usize, usize);

#[derive(Clone, Debug)]
pub struct Cell {

    // Velocity x,y:
    // - v_x is at the location (h/2, 0),
    // - v_y is at the location (0, h/2),
    pub velocity: FrontBackBuffer<Vector2>,

    pub pressure: Scalar,
    pub smoke: FrontBackBuffer<Scalar>,

    pub mode: CellTypes,

    index: GlobalIndex,
}

impl Cell {
    pub fn new(index: GlobalIndex) -> Self {
        let default_vel = Vector2::from_element(0.0);
        let default_pressure = 0.0;
        let default_smoke = 0.0;

        return Cell {
            velocity: FrontBackBuffer {
                front: default_vel,
                back: default_vel,
            },
            pressure: default_pressure,
            smoke: FrontBackBuffer {
                front: default_smoke,
                back: default_smoke,
            },
            mode: CellTypes::Fluid,
            index,
        };
    }

    pub fn index(&self) -> GlobalIndex {
        return self.index;
    }
}

pub struct Grid {
    pub cell_width: Scalar,
    pub dim_x: usize,
    pub dim_y: usize,

    cells: Vec<Cell>,
}

impl Grid {
    pub fn new(mut dim_x: usize, mut dim_y: usize, cell_width: Scalar) -> Self {
        dim_x += 2;
        dim_y += 2;
        let n = dim_x * dim_y;

        let mut cells = vec![Cell::new(GlobalIndex(0, 0)); n];

        for i in 0..dim_x {
            for j in 0..dim_y {
                cells[i + j * dim_x] = Cell::new(GlobalIndex(i, j));
            }
        }

        return Grid {
            cell_width,
            dim_x,
            dim_y,
            cells,
        };
    }
}

impl Index<GlobalIndex> for Grid {
    type Output = Cell;
    fn index<'a>(&'a self, index: GlobalIndex) -> &'a Cell {
        return &self.cells[index.0 + index.1 * self.dim_x];
    }
}

impl IndexMut<GlobalIndex> for Grid {
    fn index_mut<'a>(&'a mut self, index: GlobalIndex) -> &'a mut Cell {
        return &mut self.cells[index.0 + index.1 * self.dim_x];
    }
}

type InsideIndex = (usize, usize);

impl Index<InsideIndex> for Grid {
    type Output = Cell;
    fn index<'a>(&'a self, index: InsideIndex) -> &'a Cell {
        return &self[GlobalIndex(index.0 + 1, index.1 + 1)];
    }
}

impl IndexMut<InsideIndex> for Grid {
    fn index_mut<'a>(&'a mut self, index: InsideIndex) -> &'a mut Cell {
        return &mut self[GlobalIndex(index.0 + 1, index.1 + 1)];
    }
}

impl Integrate for Cell {
    fn integrate(&mut self, dt: Scalar, gravity: Vector2) {
        self.velocity.front = self.velocity.back + dt * gravity;
    }
}

impl Integrate for Grid {
    fn integrate(&mut self, dt: Scalar, gravity: Vector2) {
        for cell in &mut self.cells {
            cell.integrate(dt, gravity); // integrate
        }
    }
}
