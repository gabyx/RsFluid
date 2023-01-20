use crate::types::{Scalar, Vector2};
use slog::{info, Logger};
use std::any::Any;

pub trait Integrate {
    fn reset(&mut self, _log: &Logger) {}
    fn integrate(&mut self, _log: &Logger, _dt: Scalar, _gravity: Vector2) {}
    fn solve_incompressibility(
        &mut self,
        _log: &Logger,
        _dt: Scalar,
        _iterations: u64,
        _density: Scalar,
        _parallel: ExecutionMode,
    ) {
    }

    fn advect(&mut self, _log: &Logger, _dt: Scalar) {}

    // For downcasting.
    // This can be solved differently and nicer.
    // The timestepper should no own the objects.
    // This however requires shared pointers and `RefCell`, e.g.
    // another design.
    fn as_any(&self) -> &dyn Any;

    // For downcasting.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Manipulator {
    fn manipulate(
        &self,
        log: &Logger,
        t: Scalar,
        dt: Scalar,
        objects: &mut Vec<Box<dyn Integrate>>,
    );
}

pub struct TimeStepper<'a> {
    gravity: Vector2,
    density: Scalar,
    incompress_iters: u64,

    t: Scalar,
    execution_mode: ExecutionMode,

    pub objects: Vec<Box<dyn Integrate>>,
    pub manipulators: Vec<Box<dyn Manipulator>>,

    log: &'a Logger,
}

#[derive(Copy, Clone)]
pub enum ExecutionMode {
    Single,
    Parallel,
    ParallelUnsafe,
}

impl<'a> TimeStepper<'a> {
    pub fn new(
        log: &'a Logger,
        density: Scalar,
        gravity: Vector2,
        incompress_iters: u64,
        execution_mode: ExecutionMode,
        objects: Vec<Box<dyn Integrate>>,
        manipulators: Vec<Box<dyn Manipulator>>,
    ) -> Self {
        return TimeStepper {
            log,
            gravity,
            density,
            incompress_iters,
            execution_mode: execution_mode,
            objects,
            manipulators,
            t: 0.0,
        };
    }

    pub fn compute_step(&mut self, dt: Scalar) {
        if dt <= 0.0 {
            panic!("Timestep is invalid.")
        }

        info!(self.log, "Time at: '{:0.3}'.", self.t);
        self.manipulate(self.t, dt);

        self.reset();
        self.integrate(dt);
        self.solve_incompressibility(dt);
        self.advect(dt);

        self.t = self.t + dt;
    }

    fn reset(&mut self) {
        for obj in self.objects.iter_mut() {
            obj.reset(self.log);
        }
    }

    fn manipulate(&mut self, t: Scalar, dt: Scalar) {
        for manip in self.manipulators.iter_mut() {
            manip.manipulate(self.log, t, dt, &mut self.objects);
        }
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
            obj.solve_incompressibility(
                self.log,
                dt,
                self.incompress_iters,
                self.density,
                self.execution_mode,
            );
        }
    }

    fn advect(&mut self, dt: Scalar) {
        info!(self.log, "Advect at t: '{:0.3}'.", self.t,);

        for obj in self.objects.iter_mut() {
            obj.advect(self.log, dt);
        }
    }
}
