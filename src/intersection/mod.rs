pub mod scheduler;

use std::collections::HashMap;

use rand::Rng;

use crate::stats::Stats;
use crate::vehicle::route::{Direction, Path, Route};
use crate::vehicle::{Vehicle, Velocity, SAFE_DISTANCE, VEHICLE_SIZE};
use scheduler::Scheduler;

pub struct Intersection {
    pub vehicles: Vec<Vehicle>,
    scheduler: Scheduler,
    pub stats: Stats,
    next_id: u32,
    pub total_time: f32,
}

impl Intersection {
    pub fn new() -> Self {
        Intersection {
            vehicles: Vec::new(),
            scheduler: Scheduler::new(),
            stats: Stats::new(),
            next_id: 0,
            total_time: 0.0,
        }
    }

    pub fn spawn_vehicle(&mut self, direction: Direction) {
        let route = Self::random_route();
        self.try_spawn(direction, route);
    }

    pub fn spawn_random_vehicle(&mut self) {
        let direction = match rand::thread_rng().gen_range(0u8..4) {
            0 => Direction::North,
            1 => Direction::South,
            2 => Direction::East,
            _ => Direction::West,
        };
        let route = Self::random_route();
        self.try_spawn(direction, route);
    }

    pub fn update(&mut self, dt: f32) {
        self.total_time += dt;
        Self::apply_safety_distances(&mut self.vehicles);
        self.scheduler.schedule(&mut self.vehicles, dt);
        self.record_close_calls();

        let in_box = self.vehicles.iter().filter(|v| v.in_intersection()).count();
        if in_box > self.stats.max_simultaneous {
            self.stats.max_simultaneous = in_box;
        }

        for v in &mut self.vehicles {
            v.update(dt);
        }

        for v in self.vehicles.iter().filter(|v| v.is_done()) {
            self.stats.record_vehicle_exit(v);
        }
        self.vehicles.retain(|v| !v.is_done());
    }

    // ── private helpers ───────────────────────────────────────────────────────

    fn record_close_calls(&mut self) {
        let len = self.vehicles.len();
        let mut to_record: Vec<(u32, u32)> = Vec::new();
        for i in 0..len {
            for j in (i + 1)..len {
                // Only check vehicles that the scheduler has already detected.
                if !self.vehicles[i].detected_by_scheduler
                    || !self.vehicles[j].detected_by_scheduler
                {
                    continue;
                }
                // Only conflicting paths can produce a true close call.
                if !Scheduler::paths_conflict(
                    self.vehicles[i].direction,
                    self.vehicles[i].route,
                    self.vehicles[j].direction,
                    self.vehicles[j].route,
                ) {
                    continue;
                }
                let (ax, ay) = self.vehicles[i].position();
                let (bx, by) = self.vehicles[j].position();
                let dist = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt();
                if dist < SAFE_DISTANCE {
                    to_record.push((self.vehicles[i].id, self.vehicles[j].id));
                }
            }
        }
        for (id_a, id_b) in to_record {
            self.stats.record_close_call_pair(id_a, id_b);
        }
    }

    fn random_route() -> Route {
        match rand::thread_rng().gen_range(0u8..3) {
            0 => Route::Right,
            1 => Route::Straight,
            _ => Route::Left,
        }
    }

    fn try_spawn(&mut self, direction: Direction, route: Route) {
        let spawn_pos = Path::new(direction, route).waypoints[0];
        let occupied = self.vehicles.iter().any(|v| {
            let (vx, vy) = v.position();
            let dx = vx - spawn_pos.0;
            let dy = vy - spawn_pos.1;
            (dx * dx + dy * dy).sqrt() < SAFE_DISTANCE * 2.0
        });
        if !occupied {
            self.vehicles
                .push(Vehicle::new(self.next_id, direction, route));
            self.next_id += 1;
        }
    }

    fn apply_safety_distances(vehicles: &mut Vec<Vehicle>) {
        let mut lanes: HashMap<(Direction, Route), Vec<usize>> = HashMap::new();

        for (i, v) in vehicles.iter().enumerate() {
            lanes.entry((v.direction, v.route)).or_default().push(i);
        }

        for indices in lanes.values_mut() {
            indices.sort_by(|&a, &b| {
                vehicles[b]
                    .distance_travelled
                    .partial_cmp(&vehicles[a].distance_travelled)
                    .unwrap()
            });

            for pair in indices.windows(2) {
                let leader = pair[0];
                let follower = pair[1];
                let gap = vehicles[leader].distance_travelled
                    - vehicles[follower].distance_travelled;

                if gap < VEHICLE_SIZE {
                    vehicles[follower].distance_travelled =
                        vehicles[leader].distance_travelled - VEHICLE_SIZE;
                }

                let leader_stopped = vehicles[leader].velocity == Velocity::Stopped;
                if gap < SAFE_DISTANCE {
                    if leader_stopped {
                        vehicles[follower].velocity = Velocity::Stopped;
                    } else {
                        vehicles[follower].velocity = Velocity::Slow;
                    }
                } else {
                    if vehicles[follower].velocity != Velocity::Fast {
                        vehicles[follower].velocity = Velocity::Medium;
                    }
                }
            }
        }
    }
}
