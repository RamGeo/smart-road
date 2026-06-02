pub mod scheduler;

use std::collections::HashMap;

use rand::Rng;

use crate::vehicle::route::{Direction, Path, Route};
use crate::vehicle::{Vehicle, Velocity, SAFE_DISTANCE, VEHICLE_SIZE};
use scheduler::Scheduler;

pub struct Intersection {
    pub vehicles: Vec<Vehicle>,
    scheduler: Scheduler,
    next_id: u32,
    pub total_time: f32,
}

impl Intersection {
    pub fn new() -> Self {
        Intersection {
            vehicles: Vec::new(),
            scheduler: Scheduler::new(),
            next_id: 0,
            total_time: 0.0,
        }
    }

    /// Spawn a vehicle from a specific direction with a random route.
    /// Silently skipped if the spawn point for the chosen route is occupied.
    pub fn spawn_vehicle(&mut self, direction: Direction) {
        let route = Self::random_route();
        self.try_spawn(direction, route);
    }

    /// Spawn a vehicle with a fully random direction and route.
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
        for v in &mut self.vehicles {
            v.update(dt);
        }
        self.vehicles.retain(|v| !v.is_done());
    }

    // ── private helpers ───────────────────────────────────────────────────────

    fn random_route() -> Route {
        match rand::thread_rng().gen_range(0u8..3) {
            0 => Route::Right,
            1 => Route::Straight,
            _ => Route::Left,
        }
    }

    /// Spawn only if no existing vehicle is within 2×SAFE_DISTANCE of the
    /// new vehicle's start position (prevents stacking on key spam).
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

    /// Groups vehicles by their exact lane (Direction + Route) and enforces
    /// following distance only within the same lane.
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

                // Hard clamp: never let two vehicles on the same path overlap,
                // regardless of what the velocity logic does.
                if gap < VEHICLE_SIZE {
                    vehicles[follower].distance_travelled =
                        vehicles[leader].distance_travelled - VEHICLE_SIZE;
                }

                // Always update the follower's velocity based on the gap.
                // The scheduler runs after this and will re-apply Stopped
                // if the intersection is still blocked — so it gets the final say.
                let leader_stopped = vehicles[leader].velocity == Velocity::Stopped;
                if gap < SAFE_DISTANCE {
                    if leader_stopped {
                        vehicles[follower].velocity = Velocity::Stopped;
                    } else {
                        vehicles[follower].velocity = Velocity::Slow;
                    }
                } else {
                    // Gap is safe — release the follower; scheduler overrides if needed.
                    vehicles[follower].velocity = Velocity::Medium;
                }
            }
        }
    }
}
