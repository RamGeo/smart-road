use std::time::Instant;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use smart_road::intersection::Intersection;
use smart_road::renderer::{draw_road, draw_vehicles};
use smart_road::vehicle::route::Direction;

pub const WINDOW_W: u32 = 800;
pub const WINDOW_H: u32 = 800;
pub const WINDOW_TITLE: &str = "Road Intersection";

// Minimum seconds between random spawns when R is held.
const RANDOM_SPAWN_INTERVAL: f32 = 0.8;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(WINDOW_TITLE, WINDOW_W, WINDOW_H)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut intersection = Intersection::new();
    let mut last_frame = Instant::now();
    let mut random_spawning = false;
    let mut last_random_spawn = Instant::now();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,

                // Arrow Up   = vehicles from South heading North
                // Arrow Down = vehicles from North heading South
                // Arrow Right = vehicles from West heading East
                // Arrow Left  = vehicles from East heading West
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    intersection.spawn_vehicle(Direction::South);
                }
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    intersection.spawn_vehicle(Direction::North);
                }
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    intersection.spawn_vehicle(Direction::West);
                }
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    intersection.spawn_vehicle(Direction::East);
                }
                Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                    random_spawning = !random_spawning;
                }
                _ => {}
            }
        }

        // Continuous random spawning when R is toggled on.
        if random_spawning
            && last_random_spawn.elapsed().as_secs_f32() >= RANDOM_SPAWN_INTERVAL
        {
            intersection.spawn_random_vehicle();
            last_random_spawn = Instant::now();
        }

        let now = Instant::now();
        let dt = now.duration_since(last_frame).as_secs_f32();
        last_frame = now;
        intersection.update(dt);

        canvas.clear();
        draw_road(&mut canvas);
        draw_vehicles(&mut canvas, &intersection.vehicles);
        canvas.present();
    }
}
