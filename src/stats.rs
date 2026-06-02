use std::collections::HashSet;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::Window;

use crate::vehicle::route::{WINDOW_H, WINDOW_W};
use crate::vehicle::Vehicle;

pub struct Stats {
    pub total_passed: u32,
    pub max_simultaneous: usize,
    pub max_speed: f32,
    pub min_speed: f32,
    pub max_crossing_time: f32,
    pub min_crossing_time: f32,
    pub close_calls: u32,
    seen_close_call_pairs: HashSet<(u32, u32)>,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            total_passed: 0,
            max_simultaneous: 0,
            max_speed: 0.0,
            min_speed: f32::MAX,
            max_crossing_time: 0.0,
            min_crossing_time: f32::MAX,
            close_calls: 0,
            seen_close_call_pairs: HashSet::new(),
        }
    }

    pub fn record_vehicle_exit(&mut self, v: &Vehicle) {
        self.total_passed += 1;
        if v.max_speed_reached > self.max_speed {
            self.max_speed = v.max_speed_reached;
        }
        if v.min_speed_reached < f32::MAX && v.min_speed_reached < self.min_speed {
            self.min_speed = v.min_speed_reached;
        }
        // time_since_detected: elapsed from scheduler first seeing the vehicle → exit
        if v.time_since_detected > self.max_crossing_time {
            self.max_crossing_time = v.time_since_detected;
        }
        if v.time_since_detected > 0.0 && v.time_since_detected < self.min_crossing_time {
            self.min_crossing_time = v.time_since_detected;
        }
    }

    pub fn record_close_call_pair(&mut self, id_a: u32, id_b: u32) {
        let pair = (id_a.min(id_b), id_a.max(id_b));
        if self.seen_close_call_pairs.insert(pair) {
            self.close_calls += 1;
        }
    }

    pub fn draw<T>(&self, canvas: &mut Canvas<Window>, font: &Font, tc: &TextureCreator<T>) {
        const PANEL_W: u32 = 520;
        const PANEL_H: u32 = 580;
        let panel_x = (WINDOW_W - PANEL_W) / 2;
        let panel_y = (WINDOW_H - PANEL_H) / 2;
        let text_x = panel_x + 40;

        canvas.set_draw_color(Color::RGB(20, 20, 40));
        canvas
            .fill_rect(Rect::new(panel_x as i32, panel_y as i32, PANEL_W, PANEL_H))
            .ok();
        canvas.set_draw_color(Color::RGB(100, 100, 200));
        canvas
            .draw_rect(Rect::new(panel_x as i32, panel_y as i32, PANEL_W, PANEL_H))
            .ok();
        canvas
            .draw_rect(Rect::new(
                panel_x as i32 + 1,
                panel_y as i32 + 1,
                PANEL_W - 2,
                PANEL_H - 2,
            ))
            .ok();

        let min_speed = if self.min_speed >= f32::MAX {
            0.0
        } else {
            self.min_speed
        };
        let min_time = if self.min_crossing_time >= f32::MAX {
            0.0
        } else {
            self.min_crossing_time
        };

        let lines: Vec<(String, Color)> = vec![
            ("Session Statistics".into(), Color::RGB(255, 220, 80)),
            (String::new(), Color::WHITE),
            (
                format!("Vehicles passed:      {}", self.total_passed),
                Color::WHITE,
            ),
            (
                format!("Max simultaneous:     {}", self.max_simultaneous),
                Color::WHITE,
            ),
            (
                format!("Max speed:            {:.0} px/s", self.max_speed),
                Color::WHITE,
            ),
            (
                format!("Min speed (moving):   {:.0} px/s", min_speed),
                Color::WHITE,
            ),
            (
                format!("Max crossing time:    {:.2} s", self.max_crossing_time),
                Color::WHITE,
            ),
            (
                format!("Min crossing time:    {:.2} s", min_time),
                Color::WHITE,
            ),
            (
                format!("Close calls:          {}", self.close_calls),
                if self.close_calls > 0 {
                    Color::RGB(255, 100, 100)
                } else {
                    Color::RGB(100, 255, 100)
                },
            ),
            (String::new(), Color::WHITE),
            ("Press Esc to exit".into(), Color::RGB(160, 160, 160)),
        ];

        let mut y = panel_y as i32 + 45;
        for (text, color) in &lines {
            if text.is_empty() {
                y += 18;
                continue;
            }
            if let Ok(surface) = font.render(text).blended(*color) {
                if let Ok(texture) = tc.create_texture_from_surface(&surface) {
                    let q = texture.query();
                    canvas
                        .copy(
                            &texture,
                            None,
                            Rect::new(text_x as i32, y, q.width, q.height),
                        )
                        .ok();
                }
            }
            y += 44;
        }
    }
}
