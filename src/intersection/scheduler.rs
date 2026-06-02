// Step 4 placeholder — FIFO reservation table goes here.
use crate::vehicle::Vehicle;

pub struct Scheduler;

impl Scheduler {
    pub fn new() -> Self {
        Scheduler
    }

    /// Called each frame; adjusts vehicle velocities to avoid conflicts.
    pub fn schedule(&self, _vehicles: &mut [Vehicle], _dt: f32) {}
}
