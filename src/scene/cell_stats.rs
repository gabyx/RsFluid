use crate::scene::cell::*;
use crate::types::*;

#[derive(Clone, Debug)]
pub struct Stats {
    pub velocity: Vector2,
    pub velocity_norm: Scalar,
    pub pressure: Scalar,
    pub smoke: Scalar,
    pub div: Scalar,
}

impl Stats {
    pub fn identity<const I: usize>() -> Stats {
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

    pub fn accumulate<const I: usize>(&self, stats: &Stats) -> Stats {
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
