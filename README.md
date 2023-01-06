<img src="docs/logo-mod.png" style="margin-left: 20pt; width: 250px; margin-top:30pt" align="right">
<h1>RustoFluid</h1>

A small demo project to learn Rust by implementing a simple fluid simulation.
This project has been inspired by
[TenMinutesPhysics](https://matthias-research.github.io/pages/tenMinutePhysics/index.html).

The simulation is implemented in the same style as in
[fluid.sim](https://github.com/matthias-research/pages/blob/master/tenMinutePhysics/17-fluidSim.html).


![[Video.mp4](docs/video.mp4)](docs/staggered-grid.png)

# Introduction

The CLI simulator is in [src/main.rs](src/main.rs) which sets up the scene with
a [grid](src/solver/grid.rs) and hands it over to the
[timestepper](src/solver/timestepper.rs) which is responsible to integrate the
grid.

You can start the simulator with

```shell
cargo run --release --bin rustofluid -- -e 10.0 -t "$timestep" --incompress-iters 100 --dim "400,200"
```

To create the video with `30` frames use:

```shell
./create-video.sh 30
```
