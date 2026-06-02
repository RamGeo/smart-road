pub mod route;

use route::{BOX_MAX_X, BOX_MAX_Y, BOX_MIN_X, BOX_MIN_Y, Direction, Path, Route};

pub const SLOW_SPEED: f32 = 60.0;
pub const MEDIUM_SPEED: f32 = 120.0;
pub const FAST_SPEED: f32 = 200.0;
pub const SAFE_DISTANCE: f32 = 40.0;
pub const VEHICLE_SIZE: f32 = 22.0; // slightly larger than the rendered 20×20 square

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Velocity {
    Stopped,
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
    pub time_since_detected: f32,
    pub detected_by_scheduler: bool,
    pub close_call: bool,
    pub max_speed_reached: f32,
    pub min_speed_reached: f32,
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
            time_since_detected: 0.0,
            detected_by_scheduler: false,
            close_call: false,
            max_speed_reached: 0.0,
            min_speed_reached: f32::MAX,
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
            Velocity::Stopped => 0.0,
            Velocity::Slow    => SLOW_SPEED,
            Velocity::Medium  => MEDIUM_SPEED,
            Velocity::Fast    => FAST_SPEED,
        }
    }

    pub fn is_done(&self) -> bool {
        self.distance_travelled >= self.path.total_length()
    }

    pub fn in_intersection(&self) -> bool {
        let (x, y) = self.position();
        x >= BOX_MIN_X && x <= BOX_MAX_X && y >= BOX_MIN_Y && y <= BOX_MAX_Y
    }

    pub fn update(&mut self, dt: f32) {
        let spd = self.speed();
        if spd > self.max_speed_reached {
            self.max_speed_reached = spd;
        }
        if spd > 0.0 && spd < self.min_speed_reached {
            self.min_speed_reached = spd;
        }
        self.distance_travelled += spd * dt;
        if self.detected_by_scheduler {
            self.time_since_detected += dt;
        }
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

