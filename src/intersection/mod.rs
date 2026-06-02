pub mod scheduler;

use crate::vehicle::route::Direction;
use crate::vehicle::Vehicle;
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
        self.scheduler.schedule(&mut self.vehicles, dt);
        for v in &mut self.vehicles {
            v.update(dt);
        }
        self.vehicles.retain(|v| !v.is_done());
    }
}
