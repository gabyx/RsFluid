<img src="docs/logo-mod.png" style="margin-left: 20pt; width: 250px; margin-top:30pt" align="right">
<h1>RustoFluid</h1>

**Disclosure: The material in this repository has not been reviewed, endorsed, or approved of by the Rust Foundation. For more information on the Rust Foundation Trademark Policy 2023.**

A small demo project to learn Rust by implementing an Eulerian fluid simulation.
This project has been inspired by
[TenMinutesPhysics](https://matthias-research.github.io/pages/tenMinutePhysics/index.html).
The basics are easy to understand, but as always "the dog is buried in the
details" as we phrase it in German. Its probably a fact in the field of
scientific simulation that most boilerplate code is there to deal with the
beautiful (😨) boundaries of your simulation domain, data structures etc.

The simulation is implemented with the same procedure as as in
[fluid.sim](https://github.com/matthias-research/pages/blob/master/tenMinutePhysics/17-fluidSim.html)
with some additional dat`a structures to support the computation.

# Introduction

The CLI simulator is in [src/main.rs](src/main.rs) which sets up the scene with
a [grid](src/solver/grid.rs) and hands it over to the
[timestepper](src/solver/timestepper.rs) which is responsible to integrate the
grid. Disclaimer: This design is not yet perfect but it was more to play around
with the different concepts in Rust.

You can start the simulation with

```shell
cargo run --release --bin rustofluid -- -e 10.0 -t "$timestep" --incompress-iters 100 --dim "400,200"
```

To install `cargo` use
[this help here](https://doc.rust-lang.org/cargo/getting-started/installation.html).

To create the video with `30` frames use:

```shell
./create-video.sh 30
```

# Parallel Implementation

To implement the parallel version of
[`solve_incompressibility`](src/scene/grid.rs#L330) I needed to split
`grid.cells` successively into parts with an iterator chain until ending up with
an iterator which produces stencils in the form
[`PosStencilMut<Cell>`](src/scene/grid_stencil.rs#L44). This iterator can be
converted to a parallel iterator by replacing the iterator chain with the
parallel functions from `rayon`.

The following picture illustrates the topology:

 <img src="docs/simulation-grid.svg" alt="Simulation Grid" width="600px">

The parallel version with `./create-video.sh 30 --parallel` is currently much
slower than the serial one. The parallelization is probably to fine grained to
be efficient and the `Cell` driven layout is probably not that good for cache
friendliness.

# Videos

## Video Velocity

[![Liquid in Fluid: Velocity](docs/frame-1.png)](https://youtu.be/qZvKNIiBiw4)

## Video Liquid

[![Liquid in Fluid](docs/frame-3.png)](https://youtu.be/BxRfxUcNPv0)

## Video Pressure

[![Liquid in Fluid: Pressure](docs/frame-2.png)](https://youtu.be/44bBZcQKzLQ)
