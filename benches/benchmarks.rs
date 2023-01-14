use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration,
};
use itertools::Itertools;
use rayon::prelude::*;
use std::{marker::PhantomData, time::Duration};

type Index2 = nalgebra::Vector2<usize>;

#[macro_export]
macro_rules! idx {
    ($x:expr, $($y:expr),+ ) => {
        Index2::new($x, $($y),+)
    };
}

#[derive(Clone, Debug)]
pub struct C {
    pub i: usize,
    pub index: Index2,
}

#[derive(Clone, Debug)]
pub struct G {
    pub cells: Vec<C>,
    pub dim: Index2,
}

pub trait GetIndex {
    fn index(&self) -> Index2;
}

impl GetIndex for C {
    fn index(&self) -> Index2 {
        return self.index;
    }
}

pub struct PosStencilMut<'a, T: 'a>
where
    T: GetIndex,
{
    cells: *mut T,
    dim: Index2,
    phantom: PhantomData<&'a mut T>,
}

impl<'a, T: GetIndex> PosStencilMut<'a, T> {
    fn cell(&self) -> &T {
        return unsafe { &*self.cells };
    }

    fn cell_mut(&mut self) -> &mut T {
        return unsafe { &mut *self.cells };
    }

    fn neighbor(&mut self, dir: usize) -> Option<&mut T> {
        assert!(dir <= 1 && self.dim.x >= 1 && self.dim.y >= 1);

        if self.cell().index()[dir] >= self.dim[dir] - 1 {
            return None;
        }
        let offset = if dir == 0 { 1 } else { self.dim[0] };
        return Some(unsafe { &mut *self.cells.add(offset) });
    }
}

pub fn pos_stencils_mut<T: GetIndex>(
    data: &mut Vec<T>,
    dim: Index2,
    min: Option<Index2>,
    max: Option<Index2>,
) -> impl Iterator<Item = PosStencilMut<'_, T>> {
    assert!(dim > idx!(0, 0));

    let min = min.unwrap_or(Index2::zeros());
    let max = max.unwrap_or(dim - idx!(1, 1));

    assert!(min >= Index2::zeros() && max < dim);

    return (min[0]..max[0])
        .step_by(2)
        .cartesian_product((min[1]..max[1]).step_by(2))
        .map(move |(i, j)| PosStencilMut {
            cells: &mut data[i + j * dim[0]],
            dim,
            phantom: PhantomData,
        });
}

// Safety: `PosStencilMut` can only be created by `pos_stencils_mut`
// which guarantees non-aliased mutable references.
// Therefore it can safely be transferred to another thread.
unsafe impl<'a, T: GetIndex> Send for PosStencilMut<'a, T> {}

// For PosStencilMut to be Sync we have to enforce that you can't write to something stored
// in a &PosStencilMut while that same something could be read or written to from another &PosStencilMut.
// Since you need an &mut PosStencilMut to write to the pointer, and the borrow checker enforces that
// mutable references must be exclusive, there are no soundness issues making PosStencilMut sync either.
unsafe impl<'a, T: GetIndex> Sync for PosStencilMut<'a, T> {}

fn run_grid_parallel(n: usize, grid: &mut G) {
    let mut stencils: Vec<_> = pos_stencils_mut(&mut grid.cells, grid.dim, None, None).collect();

    for _ in 0..n {
        stencils.par_iter_mut().for_each(|stencil| {
            stencil.cell_mut().i += match stencil.neighbor(0) {
                Some(c) => c.i,
                None => 0,
            } + match stencil.neighbor(1) {
                Some(c) => c.i,
                None => 0,
            }
        })
    }
}

fn run_grid_single(n: usize, grid: &mut G) {
    let mut stencils: Vec<_> = pos_stencils_mut(&mut grid.cells, grid.dim, None, None).collect();

    for _ in 0..n {
        stencils.iter_mut().for_each(|stencil| {
            stencil.cell_mut().i += match stencil.neighbor(0) {
                Some(c) => c.i,
                None => 0,
            } + match stencil.neighbor(1) {
                Some(c) => c.i,
                None => 0,
            }
        })
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let dim = idx!(100, 100);

    let cell_generator = (0..dim[0]).cartesian_product(0..dim[1]).map(|(i, j)| C {
        i: i + j,
        index: idx!(i, j),
    });

    let mut grid = G {
        cells: Vec::from_iter(cell_generator),
        dim,
    };

    let mut group = c.benchmark_group("Grid Single vs. Parallel");
    for i in [10, 40, 100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("Single", i), i, |b, i| {
            b.iter(|| run_grid_parallel(*i, &mut grid))
        });

        group.bench_with_input(BenchmarkId::new("Parallel", i), i, |b, i| {
            b.iter(|| run_grid_single(*i, &mut grid))
        });
    }

    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    group.plot_config(plot_config);

    group.measurement_time(Duration::from_secs(20));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
