pub mod animation;
pub mod assets;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::vehicle::route::Direction;
use crate::vehicle::Vehicle;

// ── colours ──────────────────────────────────────────────────────────────────
const COLOR_GRASS:        Color = Color::RGB( 34, 139,  34);
const COLOR_ROAD:         Color = Color::RGB( 50,  50,  50);
const COLOR_INTERSECTION: Color = Color::RGB( 70,  70,  70);
const COLOR_LANE_LINE:    Color = Color::RGB(200, 200,  50);

// ── vehicle colours by incoming direction ────────────────────────────────────
const COLOR_NORTH: Color = Color::RGB(220,  80,  80); // red
const COLOR_SOUTH: Color = Color::RGB( 80,  80, 220); // blue
const COLOR_EAST:  Color = Color::RGB( 80, 200,  80); // green
const COLOR_WEST:  Color = Color::RGB(220, 220,  80); // yellow

pub fn draw_road(canvas: &mut Canvas<Window>) {
    // grass background
    canvas.set_draw_color(COLOR_GRASS);
    canvas.fill_rect(Rect::new(0, 0, 800, 800)).unwrap();

    // vertical road (N-S): lane centres at x=300..500, extend 20 px each side
    canvas.set_draw_color(COLOR_ROAD);
    canvas.fill_rect(Rect::new(280, 0, 240, 800)).unwrap();

    // horizontal road (E-W): lane centres at y=300..500, extend 20 px each side
    canvas.fill_rect(Rect::new(0, 280, 800, 240)).unwrap();

    // intersection box slightly lighter
    canvas.set_draw_color(COLOR_INTERSECTION);
    canvas.fill_rect(Rect::new(280, 280, 240, 240)).unwrap();

    // dashed centre line — vertical road (x = 400)
    canvas.set_draw_color(COLOR_LANE_LINE);
    let mut y = 0;
    while y < 800 {
        canvas.fill_rect(Rect::new(399, y, 2, 15)).unwrap();
        y += 30;
    }

    // dashed centre line — horizontal road (y = 400)
    let mut x = 0;
    while x < 800 {
        canvas.fill_rect(Rect::new(x, 399, 15, 2)).unwrap();
        x += 30;
    }
}

pub fn draw_vehicles(canvas: &mut Canvas<Window>, vehicles: &[Vehicle]) {
    for v in vehicles {
        let color = match v.direction {
            Direction::North => COLOR_NORTH,
            Direction::South => COLOR_SOUTH,
            Direction::East  => COLOR_EAST,
            Direction::West  => COLOR_WEST,
        };
        canvas.set_draw_color(color);
        let (x, y) = v.position();
        // 20×20 square centred on the vehicle's position
        canvas.fill_rect(Rect::new(x as i32 - 10, y as i32 - 10, 20, 20)).unwrap();
    }
}
