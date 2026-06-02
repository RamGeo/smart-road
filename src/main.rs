use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use smart_road::intersection::Intersection;
use smart_road::renderer::{draw_road, draw_vehicles};
use smart_road::vehicle::route::Direction;

pub const WINDOW_W: u32 = 800;
pub const WINDOW_H: u32 = 800;
pub const WINDOW_TITLE: &str = "Road Intersection";

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

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    intersection.spawn_vehicle(Direction::North);
                }
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    intersection.spawn_vehicle(Direction::South);
                }
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    intersection.spawn_vehicle(Direction::West);
                }
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    intersection.spawn_vehicle(Direction::East);
                }
                _ => {}
            }
        }

        let dt = 1.0 / 60.0;
        intersection.update(dt);

        canvas.clear();
        draw_road(&mut canvas);
        draw_vehicles(&mut canvas, &intersection.vehicles);
        canvas.present();
    }
}
