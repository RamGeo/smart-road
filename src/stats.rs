use std::collections::HashSet;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::Window;

use crate::vehicle::route::{WINDOW_H, WINDOW_W};
use crate::vehicle::Vehicle;

// Win95 dialog colours
const COLOR_DESKTOP: Color = Color::RGB(61, 139, 55);
const COLOR_FACE: Color = Color::RGB(192, 192, 192);
const COLOR_SHADOW: Color = Color::RGB(128, 128, 128);
const COLOR_DARK_SHADOW: Color = Color::RGB(64, 64, 64);
const COLOR_HIGHLIGHT: Color = Color::RGB(255, 255, 255);
const COLOR_TITLE: Color = Color::RGB(0, 0, 128);

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
        canvas.set_draw_color(COLOR_DESKTOP);
        canvas.clear();

        const PANEL_W: u32 = 540;
        const PANEL_H: u32 = 420;
        const TITLE_H: u32 = 22;
        let panel_x = (WINDOW_W - PANEL_W) / 2;
        let panel_y = (WINDOW_H - PANEL_H) / 2;

        draw_raised_box(canvas, panel_x, panel_y, PANEL_W, PANEL_H);

        // Title bar
        canvas.set_draw_color(COLOR_TITLE);
        canvas
            .fill_rect(Rect::new(
                panel_x as i32 + 3,
                panel_y as i32 + 3,
                PANEL_W - 6,
                TITLE_H,
            ))
            .ok();
        draw_text(
            canvas,
            font,
            tc,
            "Session Statistics",
            panel_x as i32 + 10,
            panel_y as i32 + 5,
            Color::RGB(255, 255, 255),
        );

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

        let close_color = if self.close_calls > 0 {
            Color::RGB(128, 0, 0)
        } else {
            Color::RGB(0, 128, 0)
        };

        let lines: Vec<(String, Color)> = vec![
            (
                format!("Vehicles passed:      {}", self.total_passed),
                Color::RGB(0, 0, 0),
            ),
            (
                format!("Max simultaneous:     {}", self.max_simultaneous),
                Color::RGB(0, 0, 0),
            ),
            (
                format!("Max speed:            {:.0} px/s", self.max_speed),
                Color::RGB(0, 0, 0),
            ),
            (
                format!("Min speed (moving):   {:.0} px/s", min_speed),
                Color::RGB(0, 0, 0),
            ),
            (
                format!("Max crossing time:    {:.2} s", self.max_crossing_time),
                Color::RGB(0, 0, 0),
            ),
            (
                format!("Min crossing time:    {:.2} s", min_time),
                Color::RGB(0, 0, 0),
            ),
            (
                format!("Close calls:          {}", self.close_calls),
                close_color,
            ),
            (String::new(), Color::BLACK),
            (
                "Press Esc to exit".into(),
                Color::RGB(64, 64, 64),
            ),
        ];

        let text_x = panel_x as i32 + 24;
        let mut y = panel_y as i32 + TITLE_H as i32 + 28;
        for (text, color) in &lines {
            if text.is_empty() {
                y += 12;
                continue;
            }
            draw_text(canvas, font, tc, text, text_x, y, *color);
            y += 36;
        }
    }
}

fn draw_raised_box(canvas: &mut Canvas<Window>, x: u32, y: u32, w: u32, h: u32) {
    canvas.set_draw_color(COLOR_FACE);
    canvas
        .fill_rect(Rect::new(x as i32, y as i32, w, h))
        .ok();
    // outer shadow
    canvas.set_draw_color(COLOR_DARK_SHADOW);
    canvas
        .draw_line(
            (x as i32 + w as i32 - 1, y as i32),
            (x as i32 + w as i32 - 1, y as i32 + h as i32 - 1),
        )
        .ok();
    canvas
        .draw_line(
            (x as i32, y as i32 + h as i32 - 1),
            (x as i32 + w as i32 - 1, y as i32 + h as i32 - 1),
        )
        .ok();
    // outer highlight
    canvas.set_draw_color(COLOR_HIGHLIGHT);
    canvas
        .draw_line((x as i32, y as i32), (x as i32 + w as i32 - 1, y as i32))
        .ok();
    canvas
        .draw_line((x as i32, y as i32), (x as i32, y as i32 + h as i32 - 1))
        .ok();
    // inner shadow
    canvas.set_draw_color(COLOR_SHADOW);
    canvas
        .draw_line(
            (x as i32 + 1, y as i32 + h as i32 - 2),
            (x as i32 + w as i32 - 2, y as i32 + h as i32 - 2),
        )
        .ok();
    canvas
        .draw_line(
            (x as i32 + w as i32 - 2, y as i32 + 1),
            (x as i32 + w as i32 - 2, y as i32 + h as i32 - 2),
        )
        .ok();
}

fn draw_text<T>(
    canvas: &mut Canvas<Window>,
    font: &Font,
    tc: &TextureCreator<T>,
    text: &str,
    x: i32,
    y: i32,
    color: Color,
) {
    if let Ok(surface) = font.render(text).blended(color) {
        if let Ok(texture) = tc.create_texture_from_surface(&surface) {
            let q = texture.query();
            canvas
                .copy(&texture, None, Rect::new(x, y, q.width, q.height))
                .ok();
        }
    }
}
