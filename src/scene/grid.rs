use crate::log::{debug, info, warn, Logger};
use crate::scene::timestepper::Integrate;
use crate::types::*;
use itertools::Itertools;
use rayon::prelude::*;
use std::any::Any;
use std::num::Wrapping;

#[derive(Clone, Debug, PartialEq)]
pub enum CellTypes {
    Solid,
    Fluid,
}

#[derive(Clone, Debug)]
pub struct Cell {
    // Velocity x,y:
    // - v_x is at the location (h/2, 0),
    // - v_y is at the location (0, h/2),
    pub velocity: FrontBackBuffer<Vector2>,

    pub pressure: Scalar,
    pub smoke: FrontBackBuffer<Scalar>,
    pub div: Scalar,

    pub mode: CellTypes,

    index: Index2,
}

#[derive(Clone, Debug)]
pub struct Stats {
    pub velocity: Vector2,
    pub velocity_norm: Scalar,
    pub pressure: Scalar,
    pub smoke: Scalar,
    pub div: Scalar,
}

impl Cell {
    pub fn new(index: Index2) -> Self {
        let default_vel = Vector2::from_element(0.0);
        let default_pressure = 0.0;
        let default_smoke = 0.0;

        return Cell {
            velocity: FrontBackBuffer {
                front: default_vel,
                back: default_vel,
            },
            div: 0.0,
            pressure: default_pressure,
            smoke: FrontBackBuffer {
                front: default_smoke,
                back: default_smoke,
            },
            mode: CellTypes::Fluid,
            index,
        };
    }

    pub fn index(&self) -> Index2 {
        return self.index;
    }
}

impl Stats {
    fn identity<const I: usize>() -> Stats {
        let init = if I == 0 { std::f64::MAX } else { std::f64::MIN };
        let init_vec2 = Vector2::from_element(init);

        return Stats {
            velocity: init_vec2,
            velocity_norm: init,
            pressure: init,
            smoke: init,
            div: init,
        };
    }

    pub fn from(cell: &Cell) -> Stats {
        return Stats {
            velocity: cell.velocity.back,
            velocity_norm: cell.velocity.back.norm(),
            pressure: cell.pressure,
            smoke: cell.smoke.back,
            div: cell.div,
        };
    }

    fn accumulate<const I: usize>(&self, stats: &Stats) -> Stats {
        const MIN_MAX: [fn(f64, f64) -> f64; 2] = [Scalar::min, Scalar::max];
        const MIN_MAX_V2: [fn(&Vector2, &Vector2) -> Vector2; 2] = [Vector2::inf, Vector2::sup];

        return Stats {
            velocity: MIN_MAX_V2[I](&self.velocity, &stats.velocity),
            velocity_norm: MIN_MAX[I](self.velocity_norm, stats.velocity_norm),
            pressure: MIN_MAX[I](self.pressure, stats.pressure),
            smoke: MIN_MAX[I](self.smoke, stats.smoke),
            div: MIN_MAX[I](self.div, stats.div),
        };
    }

    pub fn min_identity() -> Stats {
        return Self::identity::<0>();
    }
    pub fn max_identity() -> Stats {
        return Self::identity::<1>();
    }

    pub fn min(&self, stats: &Stats) -> Stats {
        return self.accumulate::<0>(&stats);
    }
    pub fn max(&self, stats: &Stats) -> Stats {
        return self.accumulate::<1>(&stats);
    }
}

pub struct Grid {
    pub cell_width: Scalar,
    pub dim: Index2,

    pub stats: [Stats; 2], //Min and max. accumulator statistics.

    cells: Vec<Cell>,

    extent: Vector2,

    // Grid offsets for each axis of the velocity in the cells..
    offsets: [Vector2; 2],
}

type GridIndex = Index2;

pub struct GridIndexIterator {
    curr: GridIndex,

    min: Index2,
    max: Index2,
}

impl GridIndexIterator {
    pub fn new(dim: Index2) -> GridIndexIterator {
        return GridIndexIterator {
            curr: idx!(0, 0),
            min: idx!(0, 0),
            max: dim,
        };
    }
}

impl Iterator for GridIndexIterator {
    type Item = GridIndex;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.curr; // Copy current.

        // Advance to next cell.
        let next = &mut self.curr;
        next.x += 1;
        if next.x >= self.max.x {
            next.y += 1;
            next.x = self.min.x;
        }

        if Grid::is_inside_range(self.min, self.max, curr) {
            return Some(curr);
        }

        return None;
    }
}

impl Grid {
    pub fn new(mut dim: Index2, cell_width: Scalar) -> Self {
        dim.x += 2;
        dim.y += 2;

        let h_2 = cell_width as Scalar * 0.5;
        let extent = dim.cast::<Scalar>() * cell_width;

        return Grid {
            dim,
            cell_width,

            cells: GridIndexIterator::new(dim)
                .map(|it| Cell::new(it))
                .collect(),

            stats: [Stats::min_identity(), Stats::max_identity()],

            extent,
            // `x`-values lie at offset `(0, h/2)` and
            // `y`-values at `(h/2, 0)`.
            offsets: [vec2!(0.0, h_2), vec2!(h_2, 0.0)],
        };
    }

    pub fn iter_index(&self) -> GridIndexIterator {
        return GridIndexIterator::new(self.dim);
    }

    pub fn iter_index_inside(&self) -> GridIndexIterator {
        return GridIndexIterator {
            curr: idx!(1, 1),
            min: idx!(1, 1),
            max: self.dim - idx!(1, 1),
        };
    }

    pub fn clamp_to_range<T>(min: Vector2T<T>, max: Vector2T<T>, index: Vector2T<T>) -> Vector2T<T>
    where
        T: nalgebra::Scalar + PartialOrd + Copy,
    {
        return Vector2T::<T>::new(
            nalgebra::clamp(index.x, min.x, max.x),
            nalgebra::clamp(index.y, min.y, max.y),
        );
    }

    pub fn is_inside_range(min: Index2, max: Index2, index: Index2) -> bool {
        return index < max && index >= min;
    }

    pub fn is_inside_border(&self, index: Index2) -> bool {
        return Grid::is_inside_range(Index2::zeros() + idx!(1, 1), self.dim - idx!(1, 1), index);
    }

    pub fn get_neighbors_indices(index: Index2) -> [[Index2; 2]; 2] {
        let decrement = |x| (Wrapping(x) - Wrapping(1usize)).0;

        return [
            [
                // Negative neighbors.
                Index2::new(decrement(index.x), index.y),
                Index2::new(index.x, decrement(index.y)),
            ],
            [
                // Positive neighbors.
                Index2::new(index.x + 1, index.y),
                Index2::new(index.x, index.y + 1),
            ],
        ];
    }

    pub fn set_obstacle(&mut self, pos: Vector2, radius: f64, velocity: Option<Vector2>) {
        let vel = velocity.unwrap_or(Vector2::zeros());

        for idx in self.iter_index_inside() {
            let c = (idx.cast::<Scalar>() + vec2!(0.5, 0.5)) * self.cell_width;

            if (c - pos).norm_squared() <= radius * radius {
                let c = self.cell_mut(idx);
                c.mode = CellTypes::Solid;
                c.velocity.back = vel;
            } else {
                self.cell_mut(idx).mode = CellTypes::Fluid;
            }
        }
    }

    fn compute_stats(&mut self, log: &Logger) {
        // Parallelized accumulation of statistics.
        self.stats[0] = self
            .cells
            .par_iter()
            .map(|c| Stats::from(c))
            .reduce(|| Stats::identity::<0>(), |a, b| Stats::min(&a, &b));

        self.stats[1] = self
            .cells
            .par_iter()
            .map(|c| Stats::from(c))
            .reduce(|| Stats::identity::<1>(), |a, b| Stats::max(&a, &b));

        info!(
            log,
            "Divergence range: {:.4?}, {:.4?}", self.stats[0].div, self.stats[1].div
        );
        info!(
            log,
            "Pressure range: {:.4?}, {:.4?}", self.stats[0].pressure, self.stats[1].pressure
        );
        info!(
            log,
            "Velocity range: {:.4?}, {:.4?}",
            self.stats[0].velocity_norm,
            self.stats[1].velocity_norm
        );
    }
}

pub trait CellGetter<'a, I> {
    type Item: 'a;

    type Output = &'a Self::Item;
    type OutputMut = &'a mut Self::Item;

    fn cell(&'a self, index: I) -> Self::Output;
    fn cell_mut(&'a mut self, index: I) -> Self::OutputMut;

    type OutputOpt = Option<&'a Self::Item>;
    type OutputMutOpt = Option<&'a mut Self::Item>;

    fn cell_opt(&'a self, index: Index2) -> Self::OutputOpt;
    fn cell_mut_opt(&'a mut self, index: Index2) -> Self::OutputMutOpt;
}

impl<'t> CellGetter<'t, Index2> for Grid {
    type Item = Cell;

    fn cell(&'t self, index: Index2) -> &Cell {
        return &self.cells[index.x + index.y * self.dim.x];
    }

    fn cell_mut(&'t mut self, index: Index2) -> &mut Cell {
        return &mut self.cells[index.x + index.y * self.dim.x];
    }

    fn cell_opt(&'t self, index: Index2) -> Option<&Cell> {
        return Grid::is_inside_range(Index2::zeros(), self.dim, index).then(|| self.cell(index));
    }

    fn cell_mut_opt(&'t mut self, index: Index2) -> Option<&mut Cell> {
        return Grid::is_inside_range(Index2::zeros(), self.dim, index)
            .then(|| self.cell_mut(index));
    }
}

impl Grid {
    pub fn modify_cells<F, const N: usize>(&mut self, indices: [usize; N], mut f: F) -> ()
    where
        F: FnMut([&mut Cell; N]),
    {
        let refs = self.cells.get_many_mut(indices).expect("Wrong indices.");
        f(refs);
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
}

impl Integrate for Grid {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn reset(&mut self, log: &Logger) {
        info!(log, "Reset stats.");
        self.stats = [Stats::min_identity(), Stats::max_identity()];
    }

    fn integrate(&mut self, log: &Logger, dt: Scalar, gravity: Vector2) {
        debug!(log, "Integrate grid.");

        for cell in self.cells.iter_mut() {
            cell.integrate(log, dt, gravity); // integrate
        }

        // Extrapolate to fluid cells on border.
        let ranges = [
            [idx!(0, 1), idx!(0, self.dim.y)],
            [idx!(self.dim.x - 1, self.dim.x), idx!(0, self.dim.y)],
            [idx!(0, self.dim.x), idx!(0, 1)],
            [idx!(0, self.dim.x), idx!(self.dim.y - 1, self.dim.y)],
        ];

        debug!(log, "Extrapolate border.");

        for range in ranges {
            let xr = range[0];
            let yr = range[1];

            for (i, j) in (xr[0]..xr[1]).cartesian_product(yr[0]..yr[1]) {
                let idx = idx!(i, j);

                if self.cell(idx).mode == CellTypes::Solid {
                    continue;
                }

                for dir in 0..2 {
                    let pos = idx.cast::<Scalar>() * self.cell_width + self.offsets[dir];

                    // Just sample on the inside grid by clamping.
                    self.cell_mut(idx).velocity.back[dir] = self.sample_field(
                        idx!(1, 1),
                        self.dim - idx!(1, 1),
                        pos,
                        Some(dir),
                        |cell: &Cell| cell.velocity.back[dir],
                    );
                }
            }
        }
    }

    fn solve_incompressibility(
        &mut self,
        log: &Logger,
        dt: Scalar,
        iterations: u64,
        density: Scalar,
    ) {
        // Set pressure field to zero.
        for idx in self.iter_index() {
            self.cell_mut(idx).pressure = 0.0;
        }

        let r = 1.9; // Overrelaxation factor.
        let cp = density * self.cell_width / dt;

        for _iter in 0..iterations {
            for idx in self.iter_index_inside() {
                if self.cell(idx).mode == CellTypes::Solid {
                    continue;
                }

                let s_factor = |index: Index2| {
                    return if self.cell(index).mode == CellTypes::Solid {
                        0.0
                    } else {
                        1.0
                    };
                };

                let nbs = Grid::get_neighbors_indices(idx);

                // Normalization values `s`
                // for negative/positive neighbors.
                // - 0: solid, 1: fluid.
                let mut nbs_s = [Vector2::zeros(), Vector2::zeros()];
                let mut s = 0.0;

                for neg_pos in 0..2 {
                    nbs_s[neg_pos] = vec2!(s_factor(nbs[neg_pos][0]), s_factor(nbs[neg_pos][1]));
                    s += nbs_s[neg_pos].sum();
                }

                if s == 0.0 {
                    warn!(log, "Fluid in-face count is 0.0 for {:?}", idx);
                    continue;
                }

                let get_vel = |index: Index2, dir: usize| {
                    return self.cell(index).velocity.back[dir];
                };

                let mut div: Scalar = 0.0; // Net outflow on this cell.
                let pos_idx = 1;
                let nbs_pos = &nbs[pos_idx];
                for dir in 0..2 {
                    div += get_vel(nbs_pos[dir], dir) - get_vel(idx, dir)
                }

                self.cell_mut(idx).div = div;

                // Normalize outflow to the cells we can control.
                let div_normed = div / s;
                self.cell_mut(idx).pressure -= cp * div_normed;

                // Add outflow-part to inflows to reach net 0-outflow.
                self.cell_mut(idx).velocity.back += r * nbs_s[0] * div_normed;

                // Subtract outflow-part to outflows to iteratively reach net 0-outflow (div(v) == 0).
                self.cell_mut(nbs[pos_idx][0]).velocity.back.x -=
                    r * (nbs_s[pos_idx].x) * div_normed;

                self.cell_mut(nbs[pos_idx][1]).velocity.back.y -= r * nbs_s[pos_idx].y * div_normed;
            }
        }

        self.compute_stats(&log);
    }

    fn advect(&mut self, log: &slog::Logger, dt: Scalar) {
        self.advect_velocity(log, dt);
        self.advect_smoke(log, dt);
    }
}

impl Grid {
    fn advect_velocity(&mut self, _log: &slog::Logger, dt: Scalar) {
        self.cells
            .par_iter_mut()
            .for_each(|c| c.velocity.front = c.velocity.back);

        for idx in self.iter_index_inside() {
            if self.cell(idx).mode == CellTypes::Solid {
                continue;
            }

            let nbs = Grid::get_neighbors_indices(idx);

            // Advect the two staggered grids (x and then y-direction).
            for dir in 0..2 {
                // Is the negative neighbor a solid cell, then do not advect this velocity.
                if self.cell(nbs[0][dir]).mode == CellTypes::Solid {
                    continue;
                }

                let mut pos = idx.cast::<Scalar>() * self.cell_width + self.offsets[dir];
                let mut vel: Vector2 = self.cell(idx).velocity.back;

                let sample = |pos: Vector2, dir: usize| {
                    return self.sample_field(
                        idx!(1, 1),
                        self.dim - idx!(1, 1),
                        pos,
                        Some(dir),
                        |cell: &Cell| cell.velocity.back[dir],
                    );
                };

                let other_dir = (dir + 1) % 2;
                vel[other_dir] = sample(pos, other_dir);

                // Get position of particle which reached this position.
                pos = pos - dt * vel;
                //debug!(log, "Idx: {}", idx);

                // Set the past velocity at this cell.
                self.cell_mut(idx).velocity.front[dir] = sample(pos, dir);
            }
        }

        self.cells.par_iter_mut().for_each(|c| c.velocity.swap());
    }
    fn advect_smoke(&mut self, _log: &slog::Logger, dt: Scalar) {
        self.cells
            .par_iter_mut()
            .for_each(|c| c.smoke.front = c.smoke.back);

        for idx in self.iter_index_inside() {
            if self.cell(idx).mode == CellTypes::Solid {
                continue;
            }

            let nbs = Grid::get_neighbors_indices(idx);
            let mut pos = (idx.cast::<Scalar>() + vec2!(0.5, 0.5)) * self.cell_width;

            let mut vel = Vector2::zeros();
            for dir in 0..2 {
                vel += vec2!(
                    self.cell(nbs[dir][0]).velocity.back.x,
                    self.cell(nbs[dir][1]).velocity.back.y
                ) * 0.5;
            }

            pos = pos - dt * vel;

            self.cell_mut(idx).smoke.front = self.sample_field(
                idx!(1, 1),
                self.dim - idx!(1, 1),
                pos,
                None,
                |cell: &Cell| cell.smoke.back,
            );
        }

        self.cells.par_iter_mut().for_each(|c| c.smoke.swap());
    }

    pub fn sample_field<F: Fn(&Cell) -> Scalar>(
        &self,
        min: Index2,
        max: Index2,
        mut pos: Vector2,
        dir: Option<usize>,
        get_val: F,
    ) -> Scalar {
        let h = self.cell_width;
        let h_inv = 1.0 / self.cell_width;

        // If `dir` is set, we need some offset.
        // For velocities as they are on a staggered grid.
        let offset = dir.map_or(Vector2::zeros(), |d| self.offsets[d]);
        pos = pos - offset; // Compute position on staggered grid.
        pos = Grid::clamp_to_range(Vector2::zeros(), self.extent, pos);

        // Compute index.
        let mut index = Index2::from_iterator((pos * h_inv).iter().map(|v| *v as usize));

        let clamp_index = |i| Grid::clamp_to_range(min, max - idx!(1, 1), i);

        index = clamp_index(index);
        let pos_cell = pos - index.cast::<Scalar>() * h;
        let alpha = Grid::clamp_to_range(vec2!(0.0, 0.0), vec2!(1.0, 1.0), pos_cell * h_inv);

        // debug!(log, "Sample at: {}", index);

        // Get all neighbor indices (column major).
        // [ (0,1), (1,1)
        //   (0,0), (1,0) ]
        let nbs = [
            clamp_index(index + Index2::new(0, 1)),
            index,
            clamp_index(index + Index2::new(1, 1)),
            clamp_index(index + Index2::new(1, 0)),
        ];

        // Get all values on the grid.
        let m = Matrix2::from_iterator(
            nbs.map(|i| {
                return get_val(self.cell(i));
            })
            .into_iter(), // Column major order.
        );

        let t1 = vec2!(1.0 - alpha.x, alpha.x);
        let t2 = vec2!(alpha.y, 1.0 - alpha.y);

        return t2.dot(&(m * t1));
    }
}
