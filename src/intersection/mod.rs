pub mod scheduler;

use crate::vehicle::route::Direction;
use crate::vehicle::{Vehicle, Velocity, SAFE_DISTANCE};
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

    pub fn spawn_vehicle(&mut self, direction: Direction) {
        use crate::vehicle::route::Route;
        // Placeholder: always spawn straight for now; Step 5 adds random routes + throttle.
        let route = Route::Straight;
        self.vehicles
            .push(Vehicle::new(self.next_id, direction, route));
        self.next_id += 1;
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

    pub fn apply_safety_distances(vehicles: &mut Vec<Vehicle>){
        let mut northbound = Vec::new();
        let mut southbound = Vec::new();
        let mut eastbound = Vec::new();
        let mut westbound = Vec::new();
        for (i,vehicle) in vehicles.iter().enumerate() {
            match vehicle.direction {
               Direction::North => {
                    northbound.push(i);
                }
                Direction::South => {
                    southbound.push(i);
                }
                Direction::East => {
                    eastbound.push(i);
                }
                Direction::West => {
                    westbound.push(i);
                }
            }
            
           
        }
        northbound.sort_by(|&a, &b| {
            vehicles[b].distance_travelled
                .partial_cmp(&vehicles[a].distance_travelled)
                .unwrap()
        });
        southbound.sort_by(|&a, &b| {
            vehicles[b].distance_travelled
                .partial_cmp(&vehicles[a].distance_travelled)
                .unwrap()
        });
        eastbound.sort_by(|&a, &b| {
            vehicles[b].distance_travelled
                .partial_cmp(&vehicles[a].distance_travelled)
                .unwrap()
        });
        westbound.sort_by(|&a, &b| {
            vehicles[b].distance_travelled
                .partial_cmp(&vehicles[a].distance_travelled)
                .unwrap()
        });
        for pair in northbound.windows(2) {
        let leader_idx   = pair[0];  // higher distance_travelled (sorted descending)
        let follower_idx = pair[1];

        let gap = vehicles[leader_idx].distance_travelled
                - vehicles[follower_idx].distance_travelled;

        if gap < SAFE_DISTANCE {
            vehicles[follower_idx].velocity = Velocity::Slow;
        } else {
            vehicles[follower_idx].velocity = Velocity::Medium;
        }
        }
        for pair in southbound.windows(2) {
            let leader_idx   = pair[0];  // higher distance_travelled (sorted descending)
            let follower_idx = pair[1];

            let gap = vehicles[leader_idx].distance_travelled
                    - vehicles[follower_idx].distance_travelled;

            if gap < SAFE_DISTANCE {
                vehicles[follower_idx].velocity = Velocity::Slow;
            } else {
                vehicles[follower_idx].velocity = Velocity::Medium;
            }
        }
        for pair in eastbound.windows(2) {
            let leader_idx   = pair[0];  // higher distance_travelled (sorted descending)
            let follower_idx = pair[1];

            let gap = vehicles[leader_idx].distance_travelled
                    - vehicles[follower_idx].distance_travelled;

            if gap < SAFE_DISTANCE {
                vehicles[follower_idx].velocity = Velocity::Slow;
            } else {
                vehicles[follower_idx].velocity = Velocity::Medium;
            }
        }
        for pair in westbound.windows(2) {
            let leader_idx   = pair[0];  // higher distance_travelled (sorted descending)
            let follower_idx = pair[1];

            let gap = vehicles[leader_idx].distance_travelled
                    - vehicles[follower_idx].distance_travelled;

            if gap < SAFE_DISTANCE {
                vehicles[follower_idx].velocity = Velocity::Slow;
            } else {
                vehicles[follower_idx].velocity = Velocity::Medium;
            }
        }
    }
    

}
