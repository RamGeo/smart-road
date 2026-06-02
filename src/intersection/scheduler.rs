use std::collections::HashMap;
use std::sync::OnceLock;

use crate::vehicle::route::{Direction, Path, Route, BOX_MAX_X, BOX_MAX_Y, BOX_MIN_X, BOX_MIN_Y};
use crate::vehicle::{Vehicle, Velocity, VEHICLE_SIZE};

const STOP_ZONE: f32 = 60.0;

pub struct Scheduler;

impl Scheduler {
    pub fn new() -> Self {
        Scheduler
    }

    pub fn schedule(&self, vehicles: &mut [Vehicle], _dt: f32) {
        let in_box: Vec<usize> = vehicles
            .iter()
            .enumerate()
            .filter(|(_, v)| v.in_intersection())
            .map(|(i, _)| i)
            .collect();

        // Track which direction+route combos we have already let through THIS
        // frame, so we don't release two conflicting vehicles simultaneously.
        let mut newly_entering: Vec<(Direction, Route)> = Vec::new();

        for i in 0..vehicles.len() {
            if Self::in_stop_zone(&vehicles[i]) || vehicles[i].in_intersection() {
                vehicles[i].detected_by_scheduler = true;
            }

            if vehicles[i].in_intersection() {
                continue;
            }
            if !Self::in_stop_zone(&vehicles[i]) {
                continue;
            }

            let blocked = in_box.iter().any(|&j| {
                Self::paths_conflict(
                    vehicles[i].direction,
                    vehicles[i].route,
                    vehicles[j].direction,
                    vehicles[j].route,
                )
            }) || newly_entering.iter().any(|(d, r)| {
                Self::paths_conflict(vehicles[i].direction, vehicles[i].route, *d, *r)
            });

            if blocked {
                vehicles[i].velocity = Velocity::Stopped;
            } else {
                // Intersection is clear — release (or keep released).
                if vehicles[i].velocity == Velocity::Stopped {
                    vehicles[i].velocity = Velocity::Fast;
                }
                newly_entering.push((vehicles[i].direction, vehicles[i].route));
            }
        }
    }

    fn in_stop_zone(v: &Vehicle) -> bool {
        let (x, y) = v.position();
        match v.direction {
            Direction::North => y > BOX_MIN_Y - STOP_ZONE && y < BOX_MIN_Y,
            Direction::South => y < BOX_MAX_Y + STOP_ZONE && y > BOX_MAX_Y,
            Direction::East => x < BOX_MAX_X + STOP_ZONE && x > BOX_MAX_X,
            Direction::West => x > BOX_MIN_X - STOP_ZONE && x < BOX_MIN_X,
        }
    }

    pub(crate) fn paths_conflict(d1: Direction, r1: Route, d2: Direction, r2: Route) -> bool {
        if d1 == d2 && r1 == r2 {
            return false;
        }
        if d1 == d2 {
            return false;
        }
        *conflict_map()
            .get(&((d1, r1), (d2, r2)))
            .unwrap_or(&true)
    }
}

fn conflict_map() -> &'static HashMap<((Direction, Route), (Direction, Route)), bool> {
    static CONFLICT_MAP: OnceLock<HashMap<((Direction, Route), (Direction, Route)), bool>> =
        OnceLock::new();

    CONFLICT_MAP.get_or_init(|| {
        let mut map = HashMap::new();
        let movements = [
            (Direction::North, Route::Right),
            (Direction::North, Route::Straight),
            (Direction::North, Route::Left),
            (Direction::South, Route::Right),
            (Direction::South, Route::Straight),
            (Direction::South, Route::Left),
            (Direction::East, Route::Right),
            (Direction::East, Route::Straight),
            (Direction::East, Route::Left),
            (Direction::West, Route::Right),
            (Direction::West, Route::Straight),
            (Direction::West, Route::Left),
        ];

        for a in movements {
            for b in movements {
                map.insert((a, b), geometric_conflict(a, b));
            }
        }
        map
    })
}

fn geometric_conflict(a: (Direction, Route), b: (Direction, Route)) -> bool {
    if a.0 == b.0 {
        return false;
    }

    let path_a = Path::new(a.0, a.1);
    let path_b = Path::new(b.0, b.1);
    let points_a = sampled_points(&path_a, 16.0);
    let points_b = sampled_points(&path_b, 16.0);
    let threshold = (VEHICLE_SIZE - 2.0).max(10.0);

    points_a.iter().any(|&(ax, ay)| {
        points_b.iter().any(|&(bx, by)| {
            let dx = ax - bx;
            let dy = ay - by;
            (dx * dx + dy * dy).sqrt() < threshold
        })
    })
}

fn sampled_points(path: &Path, step: f32) -> Vec<(f32, f32)> {
    let mut points = Vec::new();
    let mut t = 0.0;
    let total = path.total_length();
    while t < total {
        points.push(path.position_at(t));
        t += step;
    }
    points.push(path.position_at(total));
    points
}
