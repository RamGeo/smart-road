use std::collections::HashSet;

use crate::vehicle::Vehicle;

#[derive(Debug, Default)]
pub struct SimulationStats {
    pub max_vehicles_in_intersection: usize,
    pub max_velocity: f32,
    pub min_velocity: Option<f32>,
    pub max_pass_time: Option<f32>,
    pub min_pass_time: Option<f32>,
    pub close_calls: usize,
    pub vehicles_completed: usize,
    seen_close_call_pairs: HashSet<(u32, u32)>,
}

impl SimulationStats {
    pub fn observe_speed(&mut self, speed: f32) {
        self.max_velocity = self.max_velocity.max(speed);
        self.min_velocity = Some(match self.min_velocity {
            Some(min) => min.min(speed),
            None => speed,
        });
    }

    pub fn observe_vehicles_in_intersection(&mut self, count: usize) {
        self.max_vehicles_in_intersection = self.max_vehicles_in_intersection.max(count);
    }

    pub fn record_close_call_pair(&mut self, id_a: u32, id_b: u32) {
        let pair = if id_a < id_b {
            (id_a, id_b)
        } else {
            (id_b, id_a)
        };
        if self.seen_close_call_pairs.insert(pair) {
            self.close_calls += 1;
        }
    }

    pub fn record_completed_vehicle(&mut self, v: &Vehicle) {
        self.vehicles_completed += 1;
        self.max_pass_time = Some(match self.max_pass_time {
            Some(max) => max.max(v.time_since_detected),
            None => v.time_since_detected,
        });
        self.min_pass_time = Some(match self.min_pass_time {
            Some(min) => min.min(v.time_since_detected),
            None => v.time_since_detected,
        });
    }

    pub fn report(&self) -> String {
        format!(
            "Simulation statistics\n\
             - Max vehicles in intersection: {}\n\
             - Max velocity reached: {:.1}\n\
             - Min velocity reached: {:.1}\n\
             - Max pass time: {:.2}s\n\
             - Min pass time: {:.2}s\n\
             - Close calls: {}\n\
             - Vehicles completed: {}",
            self.max_vehicles_in_intersection,
            self.max_velocity,
            self.min_velocity.unwrap_or(0.0),
            self.max_pass_time.unwrap_or(0.0),
            self.min_pass_time.unwrap_or(0.0),
            self.close_calls,
            self.vehicles_completed
        )
    }
}
