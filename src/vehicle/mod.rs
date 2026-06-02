pub mod route;

use route::{BOX_MAX, BOX_MIN, Direction, Path, Route};

pub const SLOW_SPEED: f32 = 60.0;
pub const MEDIUM_SPEED: f32 = 120.0;
pub const FAST_SPEED: f32 = 200.0;
pub const SAFE_DISTANCE: f32 = 40.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Velocity {
    Slow,
    Medium,
    Fast,
}

pub struct Vehicle {
    pub id: u32,
    pub direction: Direction,
    pub route: Route,
    pub path: Path,
    pub distance_travelled: f32,
    pub velocity: Velocity,
    pub time_in_intersection: f32,
    pub entry_time: f32,
    pub close_call: bool,
}

impl Vehicle {
    pub fn new(id: u32, direction: Direction, route: Route) -> Self {
        Vehicle {
            id,
            direction,
            route,
            path: Path::new(direction, route),
            distance_travelled: 0.0,
            velocity: Velocity::Medium,
            time_in_intersection: 0.0,
            entry_time: 0.0,
            close_call: false,
        }
    }

    pub fn position(&self) -> (f32, f32) {
        self.path.position_at(self.distance_travelled)
    }

    pub fn heading(&self) -> f32 {
        self.path.heading_at(self.distance_travelled)
    }

    pub fn speed(&self) -> f32 {
        match self.velocity {
            Velocity::Slow => SLOW_SPEED,
            Velocity::Medium => MEDIUM_SPEED,
            Velocity::Fast => FAST_SPEED,
        }
    }

    pub fn is_done(&self) -> bool {
        self.distance_travelled >= self.path.total_length()
    }

    pub fn in_intersection(&self) -> bool {
        let (x, y) = self.position();
        x >= BOX_MIN && x <= BOX_MAX && y >= BOX_MIN && y <= BOX_MAX
    }

    pub fn update(&mut self, dt: f32) {
        self.distance_travelled += self.speed() * dt;
        if self.in_intersection() {
            self.time_in_intersection += dt;
        }
    }

    /// Euclidean distance between this vehicle's position and another's.
    pub fn distance_to(&self, other: &Vehicle) -> f32 {
        let (ax, ay) = self.position();
        let (bx, by) = other.position();
        let dx = bx - ax;
        let dy = by - ay;
        (dx * dx + dy * dy).sqrt()
    }
}

