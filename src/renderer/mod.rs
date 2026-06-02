pub mod animation;
pub mod assets;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::Window;

use crate::vehicle::route::{
    Direction, BOX_HALF, BOX_MAX_X, BOX_MAX_Y, BOX_MIN_X, BOX_MIN_Y, CENTER_X, CENTER_Y,
    LANE_INNER, LANE_MID, LANE_OUTER, ROAD_HALF, WINDOW_H, WINDOW_W, WINDOW_HF, WINDOW_WF,
};
use crate::vehicle::Vehicle;

// ── colours ──────────────────────────────────────────────────────────────────
const COLOR_GRASS:        Color = Color::RGB( 34, 139,  34);
const COLOR_ROAD:         Color = Color::RGB( 50,  50,  50);
const COLOR_INTERSECTION: Color = Color::RGB( 70,  70,  70);
const COLOR_LANE_LINE:    Color = Color::RGB(200, 200,  50);
const COLOR_LANE_ARROW:   Color = Color::RGB(180, 180, 180);
const COLOR_LANE_DIVIDER: Color = Color::RGB(160, 160, 160);

/// Centre of the label row/column on each approach (same spacing on all four arms).
const LANE_LABEL_OFFSET: f32 = 70.0;
/// Distance from the intersection box edge to the arrow / letter (matches N/S offsets 9 and 8).
const MARKER_ARROW_FROM_BOX: f32 = LANE_LABEL_OFFSET - 9.0;
const MARKER_LETTER_FROM_BOX: f32 = LANE_LABEL_OFFSET + 8.0;
const DASH_ON: i32 = 6;
const DASH_OFF: i32 = 6;

// ── vehicle colours by incoming direction ────────────────────────────────────
const COLOR_NORTH: Color = Color::RGB(220,  80,  80); // red
const COLOR_SOUTH: Color = Color::RGB( 80,  80, 220); // blue
const COLOR_EAST:  Color = Color::RGB( 80, 200,  80); // green
const COLOR_WEST:  Color = Color::RGB(220, 220,  80); // yellow

pub fn draw_road(canvas: &mut Canvas<Window>) {
    let cx = CENTER_X as i32;
    let cy = CENTER_Y as i32;
    let road_half = ROAD_HALF as i32;
    let box_half = BOX_HALF as i32;

    // grass background
    canvas.set_draw_color(COLOR_GRASS);
    canvas.fill_rect(Rect::new(0, 0, WINDOW_W, WINDOW_H)).unwrap();

    // vertical road (N-S)
    canvas.set_draw_color(COLOR_ROAD);
    canvas
        .fill_rect(Rect::new(cx - road_half, 0, (road_half * 2) as u32, WINDOW_H))
        .unwrap();

    // horizontal road (E-W)
    canvas
        .fill_rect(Rect::new(0, cy - road_half, WINDOW_W, (road_half * 2) as u32))
        .unwrap();

    // intersection box slightly lighter
    canvas.set_draw_color(COLOR_INTERSECTION);
    canvas
        .fill_rect(Rect::new(
            cx - box_half,
            cy - box_half,
            (box_half * 2) as u32,
            (box_half * 2) as u32,
        ))
        .unwrap();

    // solid centre lines — flush with the intersection box, none inside it
    canvas.set_draw_color(COLOR_LANE_LINE);
    let box_min_y = BOX_MIN_Y as i32;
    let box_max_y = BOX_MAX_Y as i32;
    let box_min_x = BOX_MIN_X as i32;
    let box_max_x = BOX_MAX_X as i32;
    let win_h = WINDOW_H as i32;
    let win_w = WINDOW_W as i32;

    if box_min_y > 0 {
        canvas
            .fill_rect(Rect::new(cx - 1, 0, 2, box_min_y as u32))
            .unwrap();
    }
    if box_max_y < win_h {
        canvas
            .fill_rect(Rect::new(cx - 1, box_max_y, 2, (win_h - box_max_y) as u32))
            .unwrap();
    }
    if box_min_x > 0 {
        canvas
            .fill_rect(Rect::new(0, cy - 1, box_min_x as u32, 2))
            .unwrap();
    }
    if box_max_x < win_w {
        canvas
            .fill_rect(Rect::new(box_max_x, cy - 1, (win_w - box_max_x) as u32, 2))
            .unwrap();
    }
}

/// Lane arrows (r / s / l) and dotted dividers on each approach.
pub fn draw_lane_arrows<T>(
    canvas: &mut Canvas<Window>,
    font: &Font,
    texture_creator: &TextureCreator<T>,
) {
    // North — arrow closer to intersection than letter (south of label row)
    draw_lane_approach(
        canvas,
        font,
        texture_creator,
        LaneLayout::NorthSouth {
            label_y: BOX_MIN_Y - LANE_LABEL_OFFSET,
            divider_y0: 0.0,
            divider_y1: BOX_MIN_Y,
            arrow_toward_increasing_y: true,
        },
        [CENTER_X - LANE_OUTER, CENTER_X - LANE_MID, CENTER_X - LANE_INNER],
        ("←", "↓", "→"),
    );
    // South (reference layout)
    draw_lane_approach(
        canvas,
        font,
        texture_creator,
        LaneLayout::NorthSouth {
            label_y: BOX_MAX_Y + LANE_LABEL_OFFSET,
            divider_y0: BOX_MAX_Y,
            divider_y1: WINDOW_HF,
            arrow_toward_increasing_y: false,
        },
        [CENTER_X + LANE_INNER, CENTER_X + LANE_MID, CENTER_X + LANE_OUTER],
        ("→", "↑", "←"),
    );
    // East — arrow then letter in a row (arrow nearer the intersection)
    draw_lane_approach(
        canvas,
        font,
        texture_creator,
        LaneLayout::EastWest {
            arrow_x: BOX_MAX_X + MARKER_ARROW_FROM_BOX,
            letter_x: BOX_MAX_X + MARKER_LETTER_FROM_BOX,
            divider_x0: BOX_MAX_X,
            divider_x1: WINDOW_WF,
        },
        [CENTER_Y - LANE_OUTER, CENTER_Y - LANE_MID, CENTER_Y - LANE_INNER],
        ("↑", "←", "↓"),
    );
    // West — arrow then letter (arrow nearer the intersection)
    draw_lane_approach(
        canvas,
        font,
        texture_creator,
        LaneLayout::EastWest {
            arrow_x: BOX_MIN_X - MARKER_ARROW_FROM_BOX,
            letter_x: BOX_MIN_X - MARKER_LETTER_FROM_BOX,
            divider_x0: 0.0,
            divider_x1: BOX_MIN_X,
        },
        [CENTER_Y + LANE_INNER, CENTER_Y + LANE_MID, CENTER_Y + LANE_OUTER],
        ("↓", "→", "↑"),
    );
}

enum LaneLayout {
    NorthSouth {
        label_y: f32,
        divider_y0: f32,
        divider_y1: f32,
        /// When true, the intersection lies at greater y than the label row (north approach).
        arrow_toward_increasing_y: bool,
    },
    EastWest {
        arrow_x: f32,
        letter_x: f32,
        divider_x0: f32,
        divider_x1: f32,
    },
}

fn draw_lane_approach<T>(
    canvas: &mut Canvas<Window>,
    font: &Font,
    texture_creator: &TextureCreator<T>,
    layout: LaneLayout,
    lane_centers: [f32; 3],
    arrows: (&str, &str, &str),
) {
    let div_a = ((lane_centers[0] + lane_centers[1]) / 2.0) as i32;
    let div_b = ((lane_centers[1] + lane_centers[2]) / 2.0) as i32;
    let suffixes = ["r", "s", "l"];

    match layout {
        LaneLayout::NorthSouth {
            label_y,
            divider_y0,
            divider_y1,
            arrow_toward_increasing_y,
        } => {
            let y0 = divider_y0 as i32;
            let y1 = divider_y1 as i32;
            draw_dotted_vertical(canvas, div_a, y0, y1);
            draw_dotted_vertical(canvas, div_b, y0, y1);
            let y = label_y as i32;
            for i in 0..3 {
                draw_lane_marker(
                    canvas,
                    font,
                    texture_creator,
                    [arrows.0, arrows.1, arrows.2][i],
                    suffixes[i],
                    lane_centers[i] as i32,
                    y,
                    arrow_toward_increasing_y,
                );
            }
        }
        LaneLayout::EastWest {
            arrow_x,
            letter_x,
            divider_x0,
            divider_x1,
        } => {
            let x0 = divider_x0 as i32;
            let x1 = divider_x1 as i32;
            draw_dotted_horizontal(canvas, div_a, x0, x1);
            draw_dotted_horizontal(canvas, div_b, x0, x1);

            let arrow_xi = arrow_x as i32;
            let letter_xi = letter_x as i32;
            for i in 0..3 {
                draw_lane_marker_inline(
                    canvas,
                    font,
                    texture_creator,
                    [arrows.0, arrows.1, arrows.2][i],
                    suffixes[i],
                    arrow_xi,
                    letter_xi,
                    lane_centers[i] as i32,
                );
            }
        }
    }
}

/// East/West: arrow toward the intersection, letter further along the approach (same line).
fn draw_lane_marker_inline<T>(
    canvas: &mut Canvas<Window>,
    font: &Font,
    texture_creator: &TextureCreator<T>,
    arrow: &str,
    suffix: &str,
    arrow_x: i32,
    letter_x: i32,
    center_y: i32,
) {
    draw_text_centered(
        canvas,
        font,
        texture_creator,
        arrow,
        arrow_x,
        center_y,
        COLOR_LANE_ARROW,
    );
    draw_text_centered(
        canvas,
        font,
        texture_creator,
        suffix,
        letter_x,
        center_y,
        COLOR_LANE_ARROW,
    );
}

/// North/South: arrow between the label row and the intersection, letter farther out.
fn draw_lane_marker<T>(
    canvas: &mut Canvas<Window>,
    font: &Font,
    texture_creator: &TextureCreator<T>,
    arrow: &str,
    suffix: &str,
    center_x: i32,
    center_y: i32,
    arrow_toward_increasing_y: bool,
) {
    const ARROW_OFFSET: i32 = 9;
    const LETTER_OFFSET: i32 = 8;

    let (arrow_y, letter_y) = if arrow_toward_increasing_y {
        (center_y + ARROW_OFFSET, center_y - LETTER_OFFSET)
    } else {
        (center_y - ARROW_OFFSET, center_y + LETTER_OFFSET)
    };

    draw_text_centered(
        canvas,
        font,
        texture_creator,
        arrow,
        center_x,
        arrow_y,
        COLOR_LANE_ARROW,
    );
    draw_text_centered(
        canvas,
        font,
        texture_creator,
        suffix,
        center_x,
        letter_y,
        COLOR_LANE_ARROW,
    );
}

fn draw_dotted_vertical(canvas: &mut Canvas<Window>, x: i32, y_start: i32, y_end: i32) {
    canvas.set_draw_color(COLOR_LANE_DIVIDER);
    let (y_start, y_end) = if y_start <= y_end {
        (y_start, y_end)
    } else {
        (y_end, y_start)
    };
    let mut y = y_start;
    while y < y_end {
        let h = DASH_ON.min(y_end - y);
        canvas.fill_rect(Rect::new(x, y, 2, h as u32)).ok();
        y += DASH_ON + DASH_OFF;
    }
}

fn draw_dotted_horizontal(canvas: &mut Canvas<Window>, y: i32, x_start: i32, x_end: i32) {
    canvas.set_draw_color(COLOR_LANE_DIVIDER);
    let (x_start, x_end) = if x_start <= x_end {
        (x_start, x_end)
    } else {
        (x_end, x_start)
    };
    let mut x = x_start;
    while x < x_end {
        let w = DASH_ON.min(x_end - x);
        canvas.fill_rect(Rect::new(x, y, w as u32, 2)).ok();
        x += DASH_ON + DASH_OFF;
    }
}

fn draw_text_centered<T>(
    canvas: &mut Canvas<Window>,
    font: &Font,
    texture_creator: &TextureCreator<T>,
    text: &str,
    center_x: i32,
    center_y: i32,
    color: Color,
) {
    if let Ok(surface) = font.render(text).blended(color) {
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            let q = texture.query();
            let x = center_x - q.width as i32 / 2;
            let y = center_y - q.height as i32 / 2;
            canvas
                .copy(&texture, None, Rect::new(x, y, q.width, q.height))
                .ok();
        }
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
