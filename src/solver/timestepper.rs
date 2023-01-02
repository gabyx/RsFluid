use crate::types::{Scalar, Vector2};
use slog::{info, Logger};
use std::any::Any;

pub trait Integrate {

    fn integrate(&mut self, _log: &Logger, _dt: Scalar, _gravity: Vector2) {}
    fn solve_incompressibility(
        &mut self,
        _log: &Logger,
        _dt: Scalar,
        _iterations: u64,
        _density: Scalar,
    ) {
    }

    // For downcasting.
    fn as_any(&self) -> &dyn Any;
}

pub struct TimeStepper<'a> {
    gravity: Vector2,
    density: Scalar,
    incompress_iters: u64,

    t: Scalar,
    pub objects: Vec<Box<dyn Integrate>>,

    log: &'a Logger,
}

impl<'a> TimeStepper<'a> {
    pub fn new(
        log: &'a Logger,
        density: Scalar,
        gravity: Vector2,
        incompress_iters: u64,
        objects: Vec<Box<dyn Integrate>>,
    ) -> Self {
        return TimeStepper {
            log,
            gravity,
            density,
            incompress_iters,
            objects,
            t: 0.0,
        };
    }

    pub fn compute_step(&mut self, dt: Scalar) {
        info!(self.log, "Time at: '{:0.3}'.", self.t,);

        self.integrate(dt);
        self.solve_incompressibility(dt);

        self.t = self.t + dt;
    }

    fn integrate(&mut self, dt: Scalar) {
        info!(
            self.log,
            "Integrate from t: '{:0.3}' -> '{:0.3}'.",
            self.t,
            self.t + dt
        );

        for obj in self.objects.iter_mut() {
            obj.integrate(self.log, dt, self.gravity);
        }
    }

    fn solve_incompressibility(&mut self, dt: Scalar) {
        info!(self.log, "Solve incompressibility at t: '{:0.3}'.", self.t,);

        for obj in self.objects.iter_mut() {
            obj.solve_incompressibility(self.log, dt, self.incompress_iters, self.density);
        }
    }
}
