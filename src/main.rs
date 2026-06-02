use std::time::Instant;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use smart_road::intersection::Intersection;
use smart_road::renderer::{draw_intersection_box, draw_lane_arrows, draw_road, draw_vehicles};
use smart_road::vehicle::route::{Direction, WINDOW_H, WINDOW_W};

const WINDOW_TITLE: &str = "Road Intersection";

const RANDOM_SPAWN_INTERVAL: f32 = 0.8;

fn find_font() -> &'static str {
    const CANDIDATES: &[&str] = &[
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/truetype/ubuntu/Ubuntu-R.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
    ];
    for &p in CANDIDATES {
        if std::path::Path::new(p).exists() {
            return p;
        }
    }
    CANDIDATES[0]
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init()
        .expect("SDL2 TTF init failed — install libsdl2-ttf-dev");
    let window = video_subsystem
        .window(WINDOW_TITLE, WINDOW_W, WINDOW_H)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let font = ttf_context
        .load_font(find_font(), 22)
        .expect("Font not found — install fonts-dejavu-core or fonts-liberation");
    let lane_font = ttf_context
        .load_font(find_font(), 16)
        .expect("Font not found — install fonts-dejavu-core or fonts-liberation");
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
        draw_lane_arrows(&mut canvas, &lane_font, &texture_creator);
        draw_intersection_box(&mut canvas);
        draw_vehicles(&mut canvas, &intersection.vehicles);
        canvas.present();
    }

    // Stats screen — stays open until Esc or window close.
    'stats: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'stats,
                _ => {}
            }
        }
        canvas.set_draw_color(Color::RGB(10, 10, 10));
        canvas.clear();
        intersection.stats.draw(&mut canvas, &font, &texture_creator);
        canvas.present();
    }
}
