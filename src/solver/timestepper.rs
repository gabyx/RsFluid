use slog::{info, Logger};

use crate::types::{Scalar, Vector2};

pub trait Integrate {
    fn integrate(&mut self, _dt: Scalar, _gravity: Vector2) {}
    fn solve_incompressibility(&mut self, _dt: Scalar, _iterations: u64) {}
}

pub struct TimeStepper<'a> {
    gravity: Vector2,
    incompress_iters: u64,

    t: Scalar,
    objects: Vec<Box<dyn Integrate>>,

    log: &'a Logger,
}

impl<'a> TimeStepper<'a> {
    pub fn new(
        log: &'a Logger,
        gravity: Vector2,
        incompress_iters: u64,
        objects: Vec<Box<dyn Integrate>>,
    ) -> Self {
        return TimeStepper {
            log,
            gravity,
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
            obj.integrate(dt, self.gravity);
        }

        self.t = self.t + dt;
    }

    fn solve_incompressibility(&mut self, dt: Scalar) {
        info!(self.log, "Solve incompressibility at t: '{:0.3}'.", self.t,);

        for obj in self.objects.iter_mut() {
            obj.solve_incompressibility(dt, self.incompress_iters);
        }
    }
}
