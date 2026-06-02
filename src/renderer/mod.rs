pub mod animation;
pub mod assets;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::Window;

use crate::vehicle::route::{
    BOX_MAX_X, BOX_MAX_Y, BOX_MIN_X, BOX_MIN_Y, CENTER_X, CENTER_Y,
    LANE_INNER, LANE_MID, LANE_OUTER, ROAD_HALF, WINDOW_H, WINDOW_W,
    WINDOW_HF, WINDOW_WF,
};
use crate::vehicle::Vehicle;

pub use assets::{draw_arrow_icon, draw_scenery, draw_vehicle_sprite, Assets};

// ── Win95 / early-90s palette ────────────────────────────────────────────────
const COLOR_ROAD: Color = Color::RGB(128, 128, 128);
const COLOR_ROAD_HIGHLIGHT: Color = Color::RGB(192, 192, 192);
const COLOR_ROAD_SHADOW: Color = Color::RGB(64, 64, 64);
const COLOR_LANE_LINE: Color = Color::RGB(255, 255, 0);
const COLOR_LANE_LABEL: Color = Color::RGB(255, 255, 255);
const COLOR_LANE_DIVIDER: Color = Color::RGB(255, 255, 255);
const COLOR_LANE_DIVIDER_OPPOSITE: Color = Color::RGB(0, 0, 255);
const COLOR_TITLE_BAR: Color = Color::RGB(0, 0, 128);
const COLOR_TITLE_BAR_HIGHLIGHT: Color = Color::RGB(16, 132, 208);
const COLOR_TITLE_BAR_SHADOW: Color = Color::RGB(0, 0, 64);
const COLOR_STATUS_BAR: Color = Color::RGB(192, 192, 192);
const COLOR_STATUS_SHADOW: Color = Color::RGB(128, 128, 128);
const COLOR_STATUS_HIGHLIGHT: Color = Color::RGB(255, 255, 255);

const LANE_LABEL_OFFSET: f32 = 70.0;
const MARKER_ARROW_FROM_BOX: f32 = LANE_LABEL_OFFSET - 9.0;
const MARKER_LETTER_FROM_BOX: f32 = LANE_LABEL_OFFSET + 8.0;
const DASH_ON: i32 = 6;
const DASH_OFF: i32 = 6;
/// Gap between intersection edge and first lane-dash (keeps centre pavement clean).
const DIVIDER_BOX_GAP: i32 = 22;
const TITLE_BAR_H: u32 = 30;
const TITLE_ICON_W: i32 = 34;
const TITLE_ICON_H: i32 = 18;
const TITLE_ICON_PAD: i32 = 6;
const TITLE_TEXT_PAD_X: i32 = 12;
const STATUS_BAR_H: u32 = 30;
const STATUS_TEXT_PAD_X: i32 = 10;
const STATUS_DOT_SIZE: i32 = 10;
const STATUS_DOT_GAP: i32 = 8;
const COLOR_LED_ON: Color = Color::RGB(0, 168, 0);
const COLOR_LED_OFF: Color = Color::RGB(208, 0, 0);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SimState {
    Running,
    Paused,
    SlowMo,
}

pub struct LiveHudStats {
    pub active_vehicles: usize,
    pub close_calls: u32,
    pub avg_crossing_secs: f32,
    pub sim_state: SimState,
}

pub fn draw_road(canvas: &mut Canvas<Window>, assets: &Assets) {
    canvas
        .copy(&assets.grass, None, Rect::new(0, 0, WINDOW_W, WINDOW_H))
        .unwrap();

    draw_scenery(canvas, assets);

    let cx = CENTER_X as i32;
    let cy = CENTER_Y as i32;
    let road_half = ROAD_HALF as i32;

    canvas.set_draw_color(COLOR_ROAD);
    canvas
        .fill_rect(Rect::new(cx - road_half, 0, (road_half * 2) as u32, WINDOW_H))
        .unwrap();
    canvas
        .fill_rect(Rect::new(0, cy - road_half, WINDOW_W, (road_half * 2) as u32))
        .unwrap();

    // 3D bevel on road edges
    canvas.set_draw_color(COLOR_ROAD_HIGHLIGHT);
    canvas
        .fill_rect(Rect::new(cx - road_half, 0, 2, WINDOW_H))
        .ok();
    canvas
        .fill_rect(Rect::new(0, cy - road_half, WINDOW_W, 2))
        .ok();
    canvas.set_draw_color(COLOR_ROAD_SHADOW);
    canvas
        .fill_rect(Rect::new(cx + road_half - 2, 0, 2, WINDOW_H))
        .ok();
    canvas
        .fill_rect(Rect::new(0, cy + road_half - 2, WINDOW_W, 2))
        .ok();

    // centre lines (stop before intersection so the cross stays clean grey)
    canvas.set_draw_color(COLOR_LANE_LINE);
    let gap = DIVIDER_BOX_GAP;
    let box_min_y = BOX_MIN_Y as i32 - gap;
    let box_max_y = BOX_MAX_Y as i32 + gap;
    let box_min_x = BOX_MIN_X as i32 - gap;
    let box_max_x = BOX_MAX_X as i32 + gap;
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

pub fn draw_lane_arrows<T>(
    canvas: &mut Canvas<Window>,
    font: &Font,
    texture_creator: &TextureCreator<T>,
    assets: &Assets,
) {
    draw_opposite_lane_dividers(canvas);

    draw_lane_approach(
        canvas,
        font,
        texture_creator,
        assets,
        LaneLayout::NorthSouth {
            label_y: BOX_MIN_Y - LANE_LABEL_OFFSET,
            divider_y_box: BOX_MIN_Y,
            divider_y_outer: 0.0,
            arrow_toward_increasing_y: true,
        },
        [CENTER_X - LANE_OUTER, CENTER_X - LANE_MID, CENTER_X - LANE_INNER],
        ("←", "↓", "→"),
    );
    draw_lane_approach(
        canvas,
        font,
        texture_creator,
        assets,
        LaneLayout::NorthSouth {
            label_y: BOX_MAX_Y + LANE_LABEL_OFFSET,
            divider_y_box: BOX_MAX_Y,
            divider_y_outer: WINDOW_HF,
            arrow_toward_increasing_y: false,
        },
        [CENTER_X + LANE_INNER, CENTER_X + LANE_MID, CENTER_X + LANE_OUTER],
        ("←", "↑", "→"),
    );
    draw_lane_approach(
        canvas,
        font,
        texture_creator,
        assets,
        LaneLayout::EastWest {
            arrow_x: BOX_MAX_X + MARKER_ARROW_FROM_BOX,
            letter_x: BOX_MAX_X + MARKER_LETTER_FROM_BOX,
            divider_x0: BOX_MAX_X,
            divider_x1: WINDOW_WF,
        },
        [CENTER_Y - LANE_OUTER, CENTER_Y - LANE_MID, CENTER_Y - LANE_INNER],
        ("↑", "←", "↓"),
    );
    draw_lane_approach(
        canvas,
        font,
        texture_creator,
        assets,
        LaneLayout::EastWest {
            arrow_x: BOX_MIN_X - MARKER_ARROW_FROM_BOX,
            letter_x: BOX_MIN_X - MARKER_LETTER_FROM_BOX,
            divider_x0: 0.0,
            divider_x1: BOX_MIN_X,
        },
        [CENTER_Y + LANE_INNER, CENTER_Y + LANE_MID, CENTER_Y + LANE_OUTER],
        ("↑", "→", "↓"),
    );
}

pub fn draw_hud<T>(
    canvas: &mut Canvas<Window>,
    status_font: &Font,
    texture_creator: &TextureCreator<T>,
    random_spawning: bool,
    live: &LiveHudStats,
) {
    // Title bar — Win95-style active window caption
    canvas.set_draw_color(COLOR_TITLE_BAR);
    canvas
        .fill_rect(Rect::new(0, 0, WINDOW_W, TITLE_BAR_H))
        .unwrap();
    canvas.set_draw_color(COLOR_TITLE_BAR_HIGHLIGHT);
    canvas.fill_rect(Rect::new(0, 0, WINDOW_W, 2)).ok();
    canvas.set_draw_color(COLOR_TITLE_BAR_SHADOW);
    canvas
        .fill_rect(Rect::new(0, TITLE_BAR_H as i32 - 2, WINDOW_W, 2))
        .ok();

    let icon_x = TITLE_ICON_PAD;
    let icon_y = (TITLE_BAR_H as i32 - TITLE_ICON_H) / 2;
    draw_bevel_box(
        canvas,
        icon_x,
        icon_y,
        TITLE_ICON_W,
        TITLE_ICON_H,
        Color::RGB(192, 192, 192),
    );
    draw_text_centered(
        canvas,
        status_font,
        texture_creator,
        "SRI",
        icon_x + TITLE_ICON_W / 2,
        icon_y + TITLE_ICON_H / 2,
        Color::RGB(0, 0, 128),
    );

    let title_text = "Smart Road Intersection";
    let title_x = icon_x + TITLE_ICON_W + TITLE_TEXT_PAD_X;
    let title_y = (TITLE_BAR_H as i32 - status_font.height() as i32) / 2;
    draw_text(
        canvas,
        status_font,
        texture_creator,
        title_text,
        title_x,
        title_y,
        Color::RGB(255, 255, 255),
    );

    if live.sim_state != SimState::Running {
        let (state_label, state_color) = match live.sim_state {
            SimState::Paused => ("[ PAUSED ]", Color::RGB(255, 220, 100)),
            SimState::SlowMo => ("[ SLOW-MO ]", Color::RGB(180, 255, 180)),
            SimState::Running => unreachable!(),
        };
        draw_text_right(
            canvas,
            status_font,
            texture_creator,
            state_label,
            WINDOW_W as i32 - TITLE_TEXT_PAD_X,
            title_y,
            state_color,
        );
    }

    // Status bar with 3D bevel
    let bar_y = WINDOW_H as i32 - STATUS_BAR_H as i32;
    canvas.set_draw_color(COLOR_STATUS_BAR);
    canvas
        .fill_rect(Rect::new(0, bar_y, WINDOW_W, STATUS_BAR_H))
        .unwrap();
    canvas.set_draw_color(COLOR_STATUS_HIGHLIGHT);
    canvas
        .fill_rect(Rect::new(0, bar_y, WINDOW_W, 2))
        .ok();
    canvas.set_draw_color(COLOR_STATUS_SHADOW);
    canvas
        .fill_rect(Rect::new(0, bar_y + STATUS_BAR_H as i32 - 2, WINDOW_W, 2))
        .ok();

    let text_y = bar_y + (STATUS_BAR_H as i32 - status_font.height() as i32) / 2;
    let text_center_y = bar_y + STATUS_BAR_H as i32 / 2;

    let dot_x = STATUS_TEXT_PAD_X;
    let dot_y = bar_y + (STATUS_BAR_H as i32 - STATUS_DOT_SIZE) / 2;
    draw_led_dot(canvas, dot_x, dot_y, STATUS_DOT_SIZE, random_spawning);

    let spawn_label = if random_spawning {
        "Random spawn: ON  (R to toggle)"
    } else {
        "Random spawn: OFF (R to toggle)"
    };
    let label_x = dot_x + STATUS_DOT_SIZE + STATUS_DOT_GAP;
    draw_text(
        canvas,
        status_font,
        texture_creator,
        spawn_label,
        label_x,
        text_y,
        Color::RGB(0, 0, 0),
    );

    let close_color = if live.close_calls > 0 {
        Color::RGB(128, 0, 0)
    } else {
        Color::RGB(0, 0, 0)
    };
    let live_line = format!(
        "Active: {}   Close: {}   Avg cross: {:.1}s",
        live.active_vehicles,
        live.close_calls,
        live.avg_crossing_secs,
    );
    draw_text_centered(
        canvas,
        status_font,
        texture_creator,
        &live_line,
        WINDOW_W as i32 / 2,
        text_center_y,
        close_color,
    );

    draw_text_right(
        canvas,
        status_font,
        texture_creator,
        "Space: pause  Shift: slow  |  Esc: stats",
        WINDOW_W as i32 - STATUS_TEXT_PAD_X,
        text_y,
        Color::RGB(0, 0, 0),
    );
}

enum LaneLayout {
    NorthSouth {
        label_y: f32,
        divider_y_box: f32,
        divider_y_outer: f32,
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
    assets: &Assets,
    layout: LaneLayout,
    lane_centers: [f32; 3],
    arrows: (&str, &str, &str),
) {
    let suffixes = ["r", "s", "l"];

    match layout {
        LaneLayout::NorthSouth {
            label_y,
            divider_y_box,
            divider_y_outer,
            arrow_toward_increasing_y,
        } => {
            draw_ns_lane_dividers(
                canvas,
                lane_centers,
                divider_y_box,
                divider_y_outer,
                COLOR_LANE_DIVIDER,
            );
            let y = label_y as i32;
            for i in 0..3 {
                draw_lane_marker(
                    canvas,
                    font,
                    texture_creator,
                    assets,
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
            draw_ew_lane_dividers(
                canvas,
                lane_centers,
                divider_x0,
                divider_x1,
                COLOR_LANE_DIVIDER,
            );

            let arrow_xi = arrow_x as i32;
            let letter_xi = letter_x as i32;
            for i in 0..3 {
                draw_lane_marker_inline(
                    canvas,
                    font,
                    texture_creator,
                    assets,
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

fn arrow_to_degrees(arrow: &str) -> f64 {
    match arrow {
        "↑" => 0.0,
        "→" => 90.0,
        "↓" => 180.0,
        "←" => 270.0,
        _ => 0.0,
    }
}

fn draw_lane_marker_inline<T>(
    canvas: &mut Canvas<Window>,
    font: &Font,
    texture_creator: &TextureCreator<T>,
    assets: &Assets,
    arrow: &str,
    suffix: &str,
    arrow_x: i32,
    letter_x: i32,
    center_y: i32,
) {
    draw_arrow_icon(
        canvas,
        &assets.arrow,
        arrow_x,
        center_y,
        arrow_to_degrees(arrow),
        assets.arrow_size,
    );
    draw_text_centered(
        canvas,
        font,
        texture_creator,
        suffix,
        letter_x,
        center_y,
        COLOR_LANE_LABEL,
    );
}

fn draw_lane_marker<T>(
    canvas: &mut Canvas<Window>,
    font: &Font,
    texture_creator: &TextureCreator<T>,
    assets: &Assets,
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

    draw_arrow_icon(
        canvas,
        &assets.arrow,
        center_x,
        arrow_y,
        arrow_to_degrees(arrow),
        assets.arrow_size,
    );
    draw_text_centered(
        canvas,
        font,
        texture_creator,
        suffix,
        center_x,
        letter_y,
        COLOR_LANE_LABEL,
    );
}

fn draw_opposite_lane_dividers(canvas: &mut Canvas<Window>) {
    draw_ns_lane_dividers(
        canvas,
        [
            CENTER_X + LANE_INNER,
            CENTER_X + LANE_MID,
            CENTER_X + LANE_OUTER,
        ],
        BOX_MIN_Y,
        0.0,
        COLOR_LANE_DIVIDER_OPPOSITE,
    );
    draw_ns_lane_dividers(
        canvas,
        [
            CENTER_X - LANE_INNER,
            CENTER_X - LANE_MID,
            CENTER_X - LANE_OUTER,
        ],
        BOX_MAX_Y,
        WINDOW_HF,
        COLOR_LANE_DIVIDER_OPPOSITE,
    );
    draw_ew_lane_dividers(
        canvas,
        [
            CENTER_Y + LANE_INNER,
            CENTER_Y + LANE_MID,
            CENTER_Y + LANE_OUTER,
        ],
        BOX_MAX_X,
        WINDOW_WF,
        COLOR_LANE_DIVIDER_OPPOSITE,
    );
    draw_ew_lane_dividers(
        canvas,
        [
            CENTER_Y - LANE_INNER,
            CENTER_Y - LANE_MID,
            CENTER_Y - LANE_OUTER,
        ],
        0.0,
        BOX_MIN_X,
        COLOR_LANE_DIVIDER_OPPOSITE,
    );
}

fn draw_ns_lane_dividers(
    canvas: &mut Canvas<Window>,
    lane_centers: [f32; 3],
    y_box: f32,
    y_outer: f32,
    color: Color,
) {
    let div_a = ((lane_centers[0] + lane_centers[1]) / 2.0) as i32;
    let div_b = ((lane_centers[1] + lane_centers[2]) / 2.0) as i32;
    let y_box = y_box as i32;
    let y_outer = y_outer as i32;
    draw_dotted_vertical_from_box(canvas, div_a, y_box, y_outer, color);
    draw_dotted_vertical_from_box(canvas, div_b, y_box, y_outer, color);
}

fn draw_dotted_vertical_from_box(
    canvas: &mut Canvas<Window>,
    x: i32,
    y_box: i32,
    y_outer: i32,
    color: Color,
) {
    canvas.set_draw_color(color);
    if y_box <= y_outer {
        let mut y = y_box + DIVIDER_BOX_GAP;
        if y >= y_outer {
            return;
        }
        while y < y_outer {
            let remaining = y_outer - y;
            let h = if remaining <= DASH_ON {
                remaining
            } else {
                DASH_ON
            };
            canvas.fill_rect(Rect::new(x, y, 2, h as u32)).ok();
            if remaining <= DASH_ON {
                break;
            }
            y += DASH_ON + DASH_OFF;
        }
    } else {
        let mut y = y_box - DIVIDER_BOX_GAP - 1;
        if y < y_outer {
            return;
        }
        while y >= y_outer {
            let remaining = y - y_outer + 1;
            let h = if remaining <= DASH_ON {
                remaining
            } else {
                DASH_ON
            };
            canvas
                .fill_rect(Rect::new(x, y - h + 1, 2, h as u32))
                .ok();
            if remaining <= DASH_ON {
                break;
            }
            y -= DASH_ON + DASH_OFF;
        }
    }
}

fn draw_ew_lane_dividers(
    canvas: &mut Canvas<Window>,
    lane_centers: [f32; 3],
    x_start: f32,
    x_end: f32,
    color: Color,
) {
    let div_a = ((lane_centers[0] + lane_centers[1]) / 2.0) as i32;
    let div_b = ((lane_centers[1] + lane_centers[2]) / 2.0) as i32;
    draw_dotted_horizontal(canvas, div_a, x_start as i32, x_end as i32, color);
    draw_dotted_horizontal(canvas, div_b, x_start as i32, x_end as i32, color);
}

fn draw_dotted_horizontal(
    canvas: &mut Canvas<Window>,
    y: i32,
    x_start: i32,
    x_end: i32,
    color: Color,
) {
    canvas.set_draw_color(color);
    let box_min_x = BOX_MIN_X as i32;
    let box_max_x = BOX_MAX_X as i32;
    let (mut x_start, mut x_end) = if x_start <= x_end {
        (x_start, x_end)
    } else {
        (x_end, x_start)
    };
    if x_end <= box_min_x + 2 {
        x_end -= DIVIDER_BOX_GAP;
    }
    if x_start >= box_max_x - 2 {
        x_start += DIVIDER_BOX_GAP;
    }
    if x_start >= x_end {
        return;
    }
    let mut x = x_start;
    while x < x_end {
        let remaining = x_end - x;
        let w = if remaining <= DASH_ON {
            remaining
        } else {
            DASH_ON
        };
        canvas.fill_rect(Rect::new(x, y, w as u32, 2)).ok();
        if remaining <= DASH_ON {
            break;
        }
        x += DASH_ON + DASH_OFF;
    }
}

fn draw_bevel_box(
    canvas: &mut Canvas<Window>,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    fill: Color,
) {
    canvas.set_draw_color(fill);
    canvas.fill_rect(Rect::new(x, y, w as u32, h as u32)).ok();
    canvas.set_draw_color(COLOR_STATUS_HIGHLIGHT);
    canvas.fill_rect(Rect::new(x, y, w as u32, 1)).ok();
    canvas.fill_rect(Rect::new(x, y, 1, h as u32)).ok();
    canvas.set_draw_color(COLOR_STATUS_SHADOW);
    canvas
        .fill_rect(Rect::new(x + w - 1, y, 1, h as u32))
        .ok();
    canvas.fill_rect(Rect::new(x, y + h - 1, w as u32, 1)).ok();
}

fn draw_led_dot(canvas: &mut Canvas<Window>, x: i32, y: i32, size: i32, on: bool) {
    let fill = if on { COLOR_LED_ON } else { COLOR_LED_OFF };
    canvas.set_draw_color(fill);
    canvas.fill_rect(Rect::new(x, y, size as u32, size as u32)).ok();
    canvas.set_draw_color(COLOR_STATUS_HIGHLIGHT);
    canvas.fill_rect(Rect::new(x, y, size as u32, 1)).ok();
    canvas.fill_rect(Rect::new(x, y, 1, size as u32)).ok();
    canvas.set_draw_color(COLOR_STATUS_SHADOW);
    canvas
        .fill_rect(Rect::new(x + size - 1, y, 1, size as u32))
        .ok();
    canvas
        .fill_rect(Rect::new(x, y + size - 1, size as u32, 1))
        .ok();
}

fn draw_text_right<T>(
    canvas: &mut Canvas<Window>,
    font: &Font,
    texture_creator: &TextureCreator<T>,
    text: &str,
    right_x: i32,
    y: i32,
    color: Color,
) {
    if let Ok(surface) = font.render(text).blended(color) {
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            let q = texture.query();
            let x = right_x - q.width as i32;
            canvas
                .copy(&texture, None, Rect::new(x, y, q.width, q.height))
                .ok();
        }
    }
}

fn draw_text<T>(
    canvas: &mut Canvas<Window>,
    font: &Font,
    texture_creator: &TextureCreator<T>,
    text: &str,
    x: i32,
    y: i32,
    color: Color,
) {
    if let Ok(surface) = font.render(text).blended(color) {
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            let q = texture.query();
            canvas
                .copy(&texture, None, Rect::new(x, y, q.width, q.height))
                .ok();
        }
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

pub fn draw_vehicles(canvas: &mut Canvas<Window>, assets: &Assets, vehicles: &[Vehicle]) {
    for v in vehicles {
        let texture = assets.car_for_direction(v.direction);
        let (x, y) = v.position();
        let heading = v.heading();
        draw_vehicle_sprite(
            canvas,
            texture,
            x,
            y,
            heading,
            assets.car_w,
            assets.car_h,
        );
    }
}
