use std::time::Instant;

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::pixels::Color;
use smart_road::audio::Audio;
use smart_road::intersection::Intersection;
use smart_road::renderer::{
    draw_hud, draw_lane_arrows, draw_road, draw_vehicles, Assets, LiveHudStats, SimState,
};
use smart_road::vehicle::route::{Direction, WINDOW_H, WINDOW_W};

const WINDOW_TITLE: &str = "Smart Road Intersection";

const RANDOM_SPAWN_INTERVAL: f32 = 0.8;
const SLOW_MO_SCALE: f32 = 0.25;

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

fn ensure_wsl_audio() {
    if std::env::var("PULSE_SERVER").is_err() {
        let wslg_pulse = "/mnt/wslg/PulseServer";
        if std::path::Path::new(wslg_pulse).exists() {
            // SAFETY: called once at startup before other threads exist.
            unsafe {
                std::env::set_var("PULSE_SERVER", format!("unix:{wslg_pulse}"));
            }
        }
    }
}

fn try_spawn_manual(
    intersection: &mut Intersection,
    audio: &Option<Audio>,
    direction: Direction,
) {
    if intersection.spawn_vehicle(direction) {
        if let Some(audio) = audio {
            audio.play_spawn();
        }
    }
}

fn main() {
    ensure_wsl_audio();
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
    let assets = Assets::load(&texture_creator);
    let font = ttf_context
        .load_font(find_font(), 22)
        .expect("Font not found — install fonts-dejavu-core or fonts-liberation");
    let lane_font = ttf_context
        .load_font(find_font(), 16)
        .expect("Font not found — install fonts-dejavu-core or fonts-liberation");
    let mut event_pump = sdl_context.event_pump().unwrap();
    let audio = Audio::try_init(&sdl_context);
    if audio.is_none() {
        eprintln!("Warning: audio unavailable — toggle sounds disabled");
    }

    let mut intersection = Intersection::new();
    let mut last_frame = Instant::now();
    let mut random_spawning = false;
    let mut random_spawn_timer = 0.0f32;
    let mut paused = false;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    repeat: false,
                    ..
                } => paused = !paused,
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => try_spawn_manual(&mut intersection, &audio, Direction::South),
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => try_spawn_manual(&mut intersection, &audio, Direction::North),
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => try_spawn_manual(&mut intersection, &audio, Direction::West),
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => try_spawn_manual(&mut intersection, &audio, Direction::East),
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    random_spawning = !random_spawning;
                    if let Some(audio) = &audio {
                        if random_spawning {
                            audio.play_random_on();
                        } else {
                            audio.play_random_off();
                        }
                    }
                }
                _ => {}
            }
        }

        let keys = event_pump.keyboard_state();
        let slow_mo = keys.is_scancode_pressed(Scancode::LShift)
            || keys.is_scancode_pressed(Scancode::RShift);
        let sim_state = if paused {
            SimState::Paused
        } else if slow_mo {
            SimState::SlowMo
        } else {
            SimState::Running
        };

        let now = Instant::now();
        let frame_dt = now.duration_since(last_frame).as_secs_f32();
        last_frame = now;

        let sim_dt = if paused {
            0.0
        } else if slow_mo {
            frame_dt * SLOW_MO_SCALE
        } else {
            frame_dt
        };

        if random_spawning && sim_dt > 0.0 {
            random_spawn_timer += sim_dt;
            while random_spawn_timer >= RANDOM_SPAWN_INTERVAL {
                intersection.spawn_random_vehicle();
                random_spawn_timer -= RANDOM_SPAWN_INTERVAL;
            }
        }

        intersection.update(sim_dt);

        let live = LiveHudStats {
            active_vehicles: intersection.vehicles.len(),
            vehicles_passed: intersection.stats.total_passed,
            close_calls: intersection.stats.close_calls,
            avg_crossing_secs: intersection.stats.average_crossing_time(),
            sim_state,
        };

        canvas.set_draw_color(Color::RGB(61, 139, 55));
        canvas.clear();
        draw_road(&mut canvas, &assets);
        draw_lane_arrows(&mut canvas, &lane_font, &texture_creator, &assets);
        draw_vehicles(&mut canvas, &assets, &intersection.vehicles);
        draw_hud(
            &mut canvas,
            &lane_font,
            &texture_creator,
            random_spawning,
            &live,
        );
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
        intersection.stats.draw(
            &mut canvas,
            &font,
            &lane_font,
            &texture_creator,
            intersection.total_time,
        );
        canvas.present();
    }
}
