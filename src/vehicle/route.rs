// Coordinate system: 800×800 window, top-left origin (y increases downward).
// Intersection box: [300, 300] → [500, 500].
// Three incoming lanes per road arm, each 40 px wide.
// Southbound uses x ∈ {300, 340, 380}; Northbound x ∈ {420, 460, 500}.
// Westbound  uses y ∈ {300, 340, 380}; Eastbound  y ∈ {420, 460, 500}.

pub const WINDOW_W: f32 = 800.0;
pub const WINDOW_H: f32 = 800.0;
pub const BOX_MIN: f32 = 300.0;
pub const BOX_MAX: f32 = 500.0;
pub const LANE_WIDTH: f32 = 40.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Route {
    Right,
    Straight,
    Left,
}

/// Which arm of the intersection the vehicle enters from.
/// `North` means the vehicle spawns at the top and travels south, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub waypoints: Vec<(f32, f32)>,
}

impl Path {
    pub fn new(direction: Direction, route: Route) -> Self {
        let waypoints = match (direction, route) {
            // From North → going South ──────────────────────────────────
            (Direction::North, Route::Right)    => vec![(300.0,   0.0), (300.0, 300.0), (  0.0, 300.0)],
            (Direction::North, Route::Straight) => vec![(340.0,   0.0), (340.0, 800.0)],
            (Direction::North, Route::Left)     => vec![(380.0,   0.0), (380.0, 300.0), (380.0, 420.0), (800.0, 420.0)],
            // From South → going North ──────────────────────────────────
            (Direction::South, Route::Right)    => vec![(500.0, 800.0), (500.0, 500.0), (800.0, 500.0)],
            (Direction::South, Route::Straight) => vec![(460.0, 800.0), (460.0,   0.0)],
            (Direction::South, Route::Left)     => vec![(420.0, 800.0), (420.0, 500.0), (420.0, 380.0), (  0.0, 380.0)],
            // From East → going West ────────────────────────────────────
            (Direction::East,  Route::Right)    => vec![(800.0, 300.0), (500.0, 300.0), (500.0,   0.0)],
            (Direction::East,  Route::Straight) => vec![(800.0, 340.0), (  0.0, 340.0)],
            (Direction::East,  Route::Left)     => vec![(800.0, 380.0), (500.0, 380.0), (380.0, 380.0), (380.0, 800.0)],
            // From West → going East ────────────────────────────────────
            (Direction::West,  Route::Right)    => vec![(  0.0, 500.0), (300.0, 500.0), (300.0, 800.0)],
            (Direction::West,  Route::Straight) => vec![(  0.0, 460.0), (800.0, 460.0)],
            (Direction::West,  Route::Left)     => vec![(  0.0, 420.0), (300.0, 420.0), (420.0, 420.0), (420.0,   0.0)],
        };
        Path { waypoints }
    }

    /// Total arc length of the path in pixels.
    pub fn total_length(&self) -> f32 {
        self.waypoints.windows(2).map(|w| seg_len(w[0], w[1])).sum()
    }

    /// World position at `t` pixels from the start of the path.
    pub fn position_at(&self, t: f32) -> (f32, f32) {
        let mut remaining = t.max(0.0);
        let n = self.waypoints.len();
        for i in 0..n - 1 {
            let a = self.waypoints[i];
            let b = self.waypoints[i + 1];
            let len = seg_len(a, b);
            if remaining <= len || i == n - 2 {
                let frac = if len > 0.0 { (remaining / len).min(1.0) } else { 1.0 };
                return lerp(a, b, frac);
            }
            remaining -= len;
        }
        *self.waypoints.last().unwrap()
    }

    /// Heading in radians at `t` pixels from the start.
    /// 0 = east (+x), π/2 = south (+y), π = west, −π/2 = north.
    pub fn heading_at(&self, t: f32) -> f32 {
        let mut remaining = t.max(0.0);
        let n = self.waypoints.len();
        for i in 0..n - 1 {
            let a = self.waypoints[i];
            let b = self.waypoints[i + 1];
            let len = seg_len(a, b);
            if remaining <= len || i == n - 2 {
                return (b.1 - a.1).atan2(b.0 - a.0);
            }
            remaining -= len;
        }
        0.0
    }
}

fn seg_len(a: (f32, f32), b: (f32, f32)) -> f32 {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    (dx * dx + dy * dy).sqrt()
}

fn lerp(a: (f32, f32), b: (f32, f32), t: f32) -> (f32, f32) {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
}
