// Coordinate system: WINDOW_W×WINDOW_H window, top-left origin (y increases downward).
// Intersection is centered at (CENTER_X, CENTER_Y).
// Three incoming lanes per road arm, each 40 px wide.

pub const WINDOW_W: u32 = 1200;
pub const WINDOW_H: u32 = 800;
pub const WINDOW_WF: f32 = WINDOW_W as f32;
pub const WINDOW_HF: f32 = WINDOW_H as f32;

pub const CENTER_X: f32 = (WINDOW_W / 2) as f32;
pub const CENTER_Y: f32 = (WINDOW_H / 2) as f32;

pub const BOX_HALF: f32 = 100.0;
pub const BOX_MIN_X: f32 = CENTER_X - BOX_HALF;
pub const BOX_MAX_X: f32 = CENTER_X + BOX_HALF;
pub const BOX_MIN_Y: f32 = CENTER_Y - BOX_HALF;
pub const BOX_MAX_Y: f32 = CENTER_Y + BOX_HALF;

/// Road extends 20 px beyond the intersection box on each side.
pub const ROAD_HALF: f32 = BOX_HALF + 20.0;
pub const LANE_WIDTH: f32 = 40.0;

/// Lane centre offset from intersection centre (outer / middle / inner).
pub const LANE_OUTER: f32 = 100.0;
pub const LANE_MID: f32 = 60.0;
pub const LANE_INNER: f32 = 20.0;

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
        let sb = |lane: u8| -> f32 {
            CENTER_X
                - match lane {
                    0 => LANE_OUTER,
                    1 => LANE_MID,
                    _ => LANE_INNER,
                }
        };
        let nb = |lane: u8| -> f32 {
            CENTER_X
                + match lane {
                    0 => LANE_INNER,
                    1 => LANE_MID,
                    _ => LANE_OUTER,
                }
        };
        let wb = |lane: u8| -> f32 {
            CENTER_Y
                - match lane {
                    0 => LANE_OUTER,
                    1 => LANE_MID,
                    _ => LANE_INNER,
                }
        };
        let eb = |lane: u8| -> f32 {
            CENTER_Y
                + match lane {
                    0 => LANE_INNER,
                    1 => LANE_MID,
                    _ => LANE_OUTER,
                }
        };

        let waypoints = match (direction, route) {
            // From North → going South ──────────────────────────────────
            (Direction::North, Route::Right) => {
                vec![(sb(0), 0.0), (sb(0), BOX_MIN_Y), (0.0, BOX_MIN_Y)]
            }
            (Direction::North, Route::Straight) => vec![(sb(1), 0.0), (sb(1), WINDOW_HF)],
            (Direction::North, Route::Left) => vec![
                (sb(2), 0.0),
                (sb(2), BOX_MIN_Y),
                (sb(2), eb(0)),
                (WINDOW_WF, eb(0)),
            ],
            // From South → going North ──────────────────────────────────
            (Direction::South, Route::Right) => {
                vec![(nb(2), WINDOW_HF), (nb(2), BOX_MAX_Y), (WINDOW_WF, BOX_MAX_Y)]
            }
            (Direction::South, Route::Straight) => vec![(nb(1), WINDOW_HF), (nb(1), 0.0)],
            (Direction::South, Route::Left) => vec![
                (nb(0), WINDOW_HF),
                (nb(0), BOX_MAX_Y),
                (nb(0), wb(2)),
                (0.0, wb(2)),
            ],
            // From East → going West ────────────────────────────────────
            (Direction::East, Route::Right) => {
                vec![(WINDOW_WF, wb(0)), (BOX_MAX_X, wb(0)), (BOX_MAX_X, 0.0)]
            }
            (Direction::East, Route::Straight) => vec![(WINDOW_WF, wb(1)), (0.0, wb(1))],
            (Direction::East, Route::Left) => vec![
                (WINDOW_WF, wb(2)),
                (BOX_MAX_X, wb(2)),
                (sb(2), wb(2)),
                (sb(2), WINDOW_HF),
            ],
            // From West → going East ────────────────────────────────────
            (Direction::West, Route::Right) => {
                vec![(0.0, eb(2)), (BOX_MIN_X, eb(2)), (BOX_MIN_X, WINDOW_HF)]
            }
            (Direction::West, Route::Straight) => vec![(0.0, eb(1)), (WINDOW_WF, eb(1))],
            (Direction::West, Route::Left) => vec![
                (0.0, eb(0)),
                (BOX_MIN_X, eb(0)),
                (nb(0), eb(0)),
                (nb(0), 0.0),
            ],
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
                let frac = if len > 0.0 {
                    (remaining / len).min(1.0)
                } else {
                    1.0
                };
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
