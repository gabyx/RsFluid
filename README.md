<img src="docs/logo-mod.png" style="margin-left: 20pt; width: 250px; margin-top:30pt" align="right">
<h1>RustoFluid</h1>

A small demo project to learn Rust by implementing an Eulerian fluid simulation.
This project has been inspired by
[TenMinutesPhysics](https://matthias-research.github.io/pages/tenMinutePhysics/index.html).
The basics are easy to understand, but as always "the dog is buried in the
details" as we phrase it in German. Its probably a fact in the field of
scientific simulation that most boilerplate code is there to deal with the
beautiful (😨) boundaries of your simulation domain, data structures etc.

The simulation is implemented with the same logic as as in
[fluid.sim](https://github.com/matthias-research/pages/blob/master/tenMinutePhysics/17-fluidSim.html).

# Introduction

The CLI simulator is in [src/main.rs](src/main.rs) which sets up the scene with
a [grid](src/solver/grid.rs) and hands it over to the
[timestepper](src/solver/timestepper.rs) which is responsible to integrate the
grid. Disclaimer: This design is not yet perfect but it was more to playaround
the different concepts in Rust.

You can start the simulation with

```shell
cargo run --release --bin rustofluid -- -e 10.0 -t "$timestep" --incompress-iters 100 --dim "400,200"
```

To create the video with `30` frames use:

```shell
./create-video.sh 30
```

# Video

[![Liquid in Fluid](docs/frame-example.png)](https://www.youtube.com/watch?v=I1DTGRb3dvA)
