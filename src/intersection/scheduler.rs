use crate::vehicle::route::{Direction, Route, BOX_MAX, BOX_MIN};
use crate::vehicle::{Vehicle, Velocity};

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
                    vehicles[i].velocity = Velocity::Medium;
                }
                newly_entering.push((vehicles[i].direction, vehicles[i].route));
            }
        }
    }

    fn in_stop_zone(v: &Vehicle) -> bool {
        let (x, y) = v.position();
        match v.direction {
            Direction::North => y > BOX_MIN - STOP_ZONE && y < BOX_MIN,
            Direction::South => y < BOX_MAX + STOP_ZONE && y > BOX_MAX,
            Direction::East  => x < BOX_MAX + STOP_ZONE && x > BOX_MAX,
            Direction::West  => x > BOX_MIN - STOP_ZONE && x < BOX_MIN,
        }
    }

    fn paths_conflict(d1: Direction, r1: Route, d2: Direction, r2: Route) -> bool {
        if d1 == d2 {
            return false;
        }
        if r1 == Route::Right && r2 == Route::Right {
            return false;
        }
        true
    }
}
