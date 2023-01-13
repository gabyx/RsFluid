use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, PlotConfiguration, AxisScale};
use rayon::prelude::*;

#[derive(Clone, Debug)]
struct C {
    i: usize,
}

#[derive(Clone, Debug)]
struct G {
    cells: Vec<C>,
}

fn run_grid_single(n: usize, grid: &mut G) {
    let l = grid.cells.len() - 1;

    for _ in 0..n {
        for i in 0..grid.cells.len() {
            grid.cells[i].i += i + grid.cells[(i + 1).min(l)].i
        }
    }
}

fn run_grid_parallel(n: usize, grid: &mut G) {
    let mut black: Vec<&mut C> = vec![];
    let mut white: Vec<&mut C> = vec![];

    grid.cells.iter_mut().enumerate().for_each(|(i, c)| {
        if i % 2 == 0 {
            black.push(c);
        } else {
            white.push(c)
        }
    });

    for _ in 0..n {
        let mut l = white.len() - 1;
        black
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, c)| c.i += i + white[i.min(l)].i);

        l = black.len() - 1;
        white.par_iter_mut().enumerate().for_each(|(i, c)| {
            c.i += i + black[i.min(l)].i;
        });
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut grid = G {
        cells: vec![C { i: 3 }; 100 * 100],
    };

    let mut group = c.benchmark_group("Grid Single vs. Parallel");
    for i in [10, 40, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("Single", i), i, |b, i| {
            b.iter(|| run_grid_parallel(*i, &mut grid))
        });

        group.bench_with_input(BenchmarkId::new("Parallel", i), i, |b, i| {
            b.iter(|| run_grid_single(*i, &mut grid))
        });
    }

    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Linear);
    group.plot_config(plot_config);

    group.measurement_time(Duration::from_secs(20));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
