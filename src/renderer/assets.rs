use std::f32::consts::PI;
use std::path::Path;

use resvg::usvg;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::Window;
use tiny_skia::Pixmap;

use crate::vehicle::route::{CENTER_X, CENTER_Y, ROAD_HALF, WINDOW_H, WINDOW_W};

const CAR_RENDER_W: u32 = 28;
const CAR_RENDER_H: u32 = 36;
const ARROW_RENDER_SIZE: u32 = 14;
const GRASS_TILE_SIZE: u32 = 32;

fn assets_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

fn asset_path(relative: &str) -> std::path::PathBuf {
    assets_root().join(relative)
}

fn rasterize_svg(path: &Path, width: u32, height: u32) -> Vec<u8> {
    let svg_data = std::fs::read(path).unwrap_or_else(|e| {
        panic!("failed to read SVG {}: {e}", path.display())
    });
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(&svg_data, &opt).unwrap_or_else(|e| {
        panic!("failed to parse SVG {}: {e}", path.display())
    });

    let svg_size = tree.size();
    let scale_x = width as f32 / svg_size.width();
    let scale_y = height as f32 / svg_size.height();

    let mut pixmap = Pixmap::new(width, height).expect("pixmap allocation failed");
    pixmap.fill(tiny_skia::Color::TRANSPARENT);

    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale_x, scale_y),
        &mut pixmap.as_mut(),
    );

    pixmap.data().to_vec()
}

fn rgba_to_surface(rgba: Vec<u8>, width: u32, height: u32) -> Surface<'static> {
    let mut surface = Surface::new(width, height, PixelFormatEnum::RGBA32)
        .expect("surface allocation failed");
    let pitch = surface.pitch() as usize;
    surface.as_mut().with_lock_mut(|dest| {
        let w = width as usize;
        let h = height as usize;
        for y in 0..h {
            for x in 0..w {
                let src_i = (y * w + x) * 4;
                let dst_i = y * pitch + x * 4;
                dest[dst_i..dst_i + 4].copy_from_slice(&rgba[src_i..src_i + 4]);
            }
        }
    });
    surface
}

fn load_svg_texture<'tex>(
    texture_creator: &'tex TextureCreator<sdl2::video::WindowContext>,
    path: &Path,
    width: u32,
    height: u32,
) -> Texture<'tex> {
    let rgba = rasterize_svg(path, width, height);
    let surface = rgba_to_surface(rgba, width, height);
    texture_creator
        .create_texture_from_surface(&surface)
        .unwrap_or_else(|e| panic!("texture from {}: {e}", path.display()))
}

fn tile_grass_texture<'tex>(
    texture_creator: &'tex TextureCreator<sdl2::video::WindowContext>,
    tile_paths: [&Path; 2],
) -> Texture<'tex> {
    let tiles: [Surface<'static>; 2] = tile_paths.map(|path| {
        let rgba = rasterize_svg(path, GRASS_TILE_SIZE, GRASS_TILE_SIZE);
        rgba_to_surface(rgba, GRASS_TILE_SIZE, GRASS_TILE_SIZE)
    });

    let mut full = Surface::new(WINDOW_W, WINDOW_H, PixelFormatEnum::RGBA32)
        .expect("grass surface allocation failed");

    let step = GRASS_TILE_SIZE as usize;
    for ty in (0..WINDOW_H as usize).step_by(step) {
        for tx in (0..WINDOW_W as usize).step_by(step) {
            let variant = ((tx / step) + (ty / step)) % 2;
            tiles[variant]
                .blit(
                    None,
                    full.as_mut(),
                    Rect::new(tx as i32, ty as i32, 0, 0),
                )
                .ok();
        }
    }

    texture_creator
        .create_texture_from_surface(&full)
        .expect("grass texture creation failed")
}

pub struct Assets<'tex> {
    pub grass: Texture<'tex>,
    pub cars: [Texture<'tex>; 4],
    pub arrow: Texture<'tex>,
    pub building_apartment: Texture<'tex>,
    pub building_office: Texture<'tex>,
    pub building_shop: Texture<'tex>,
    pub park: Texture<'tex>,
    pub tree: Texture<'tex>,
    pub car_w: u32,
    pub car_h: u32,
    pub arrow_size: u32,
    pub building_apartment_w: u32,
    pub building_apartment_h: u32,
    pub building_office_w: u32,
    pub building_office_h: u32,
    pub building_shop_w: u32,
    pub building_shop_h: u32,
    pub park_w: u32,
    pub park_h: u32,
    pub tree_w: u32,
    pub tree_h: u32,
}

impl<'tex> Assets<'tex> {
    pub fn load(texture_creator: &'tex TextureCreator<sdl2::video::WindowContext>) -> Self {
        let root = assets_root();
        assert!(
            root.is_dir(),
            "assets directory missing at {}",
            root.display()
        );

        let grass = tile_grass_texture(
            texture_creator,
            [
                &asset_path("ui/grass-tile-a.svg"),
                &asset_path("ui/grass-tile-b.svg"),
            ],
        );

        let car_paths = [
            "sprites/car-red.svg",
            "sprites/car-blue.svg",
            "sprites/car-green.svg",
            "sprites/car-yellow.svg",
        ];
        let cars = car_paths.map(|p| {
            load_svg_texture(
                texture_creator,
                &asset_path(p),
                CAR_RENDER_W,
                CAR_RENDER_H,
            )
        });

        let arrow = load_svg_texture(
            texture_creator,
            &asset_path("icons/arrow.svg"),
            ARROW_RENDER_SIZE,
            ARROW_RENDER_SIZE,
        );

        const APT_W: u32 = 54;
        const APT_H: u32 = 96;
        const OFF_W: u32 = 80;
        const OFF_H: u32 = 93;
        const SHOP_W: u32 = 72;
        const SHOP_H: u32 = 65;
        const PARK_W: u32 = 180;
        const PARK_H: u32 = 158;
        const TREE_W: u32 = 28;
        const TREE_H: u32 = 36;

        let building_apartment = load_svg_texture(
            texture_creator,
            &asset_path("scenery/building-apartment.svg"),
            APT_W,
            APT_H,
        );
        let building_office = load_svg_texture(
            texture_creator,
            &asset_path("scenery/building-office.svg"),
            OFF_W,
            OFF_H,
        );
        let building_shop = load_svg_texture(
            texture_creator,
            &asset_path("scenery/building-shop.svg"),
            SHOP_W,
            SHOP_H,
        );
        let park = load_svg_texture(
            texture_creator,
            &asset_path("scenery/park.svg"),
            PARK_W,
            PARK_H,
        );
        let tree = load_svg_texture(
            texture_creator,
            &asset_path("scenery/tree.svg"),
            TREE_W,
            TREE_H,
        );

        Assets {
            grass,
            cars,
            arrow,
            building_apartment,
            building_office,
            building_shop,
            park,
            tree,
            car_w: CAR_RENDER_W,
            car_h: CAR_RENDER_H,
            arrow_size: ARROW_RENDER_SIZE,
            building_apartment_w: APT_W,
            building_apartment_h: APT_H,
            building_office_w: OFF_W,
            building_office_h: OFF_H,
            building_shop_w: SHOP_W,
            building_shop_h: SHOP_H,
            park_w: PARK_W,
            park_h: PARK_H,
            tree_w: TREE_W,
            tree_h: TREE_H,
        }
    }

    pub fn car_for_direction(&self, direction: crate::vehicle::route::Direction) -> &Texture<'tex> {
        use crate::vehicle::route::Direction;
        match direction {
            Direction::North => &self.cars[0],
            Direction::South => &self.cars[1],
            Direction::East => &self.cars[2],
            Direction::West => &self.cars[3],
        }
    }
}

/// Draw a vehicle sprite rotated to match its path heading.
/// Heading: 0 = east, π/2 = south. Sprite faces north at 0° rotation.
pub fn draw_vehicle_sprite(
    canvas: &mut Canvas<sdl2::video::Window>,
    texture: &Texture,
    x: f32,
    y: f32,
    heading: f32,
    car_w: u32,
    car_h: u32,
) {
    let angle_deg = f64::from((heading + PI / 2.0) * 180.0 / PI);
    let cx = x as i32;
    let cy = y as i32;
    let src = Rect::new(0, 0, car_w, car_h);
    let dst = Rect::new(cx - car_w as i32 / 2, cy - car_h as i32 / 2, car_w, car_h);

    canvas
        .copy_ex(texture, src, dst, angle_deg, None, false, false)
        .ok();
}

/// Draw a lane arrow icon rotated to point in the given direction (screen coords).
pub fn draw_arrow_icon(
    canvas: &mut Canvas<sdl2::video::Window>,
    texture: &Texture,
    center_x: i32,
    center_y: i32,
    angle_deg: f64,
    size: u32,
) {
    let dst = Rect::new(
        center_x - size as i32 / 2,
        center_y - size as i32 / 2,
        size,
        size,
    );
    canvas
        .copy_ex(
            texture,
            None,
            dst,
            angle_deg,
            None,
            false,
            false,
        )
        .ok();
}

fn blit_sprite(
    canvas: &mut Canvas<Window>,
    texture: &Texture,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
) {
    canvas.copy(texture, None, Rect::new(x, y, w, h))        .ok();
}

/// Buildings, parks, and trees in the four corner blocks around the crossroads.
pub fn draw_scenery(canvas: &mut Canvas<Window>, assets: &Assets) {
    // Northwest: apartment + park + trees
    blit_sprite(
        canvas,
        &assets.building_apartment,
        20,
        36,
        assets.building_apartment_w,
        assets.building_apartment_h,
    );
    blit_sprite(
        canvas,
        &assets.park,
        200,
        40,
        assets.park_w,
        assets.park_h,
    );
    for (x, y) in [(90, 200), (160, 170), (380, 60), (420, 190)] {
        blit_sprite(canvas, &assets.tree, x, y, assets.tree_w, assets.tree_h);
    }

    // Northeast: office, shop, park
    blit_sprite(
        canvas,
        &assets.building_office,
        820,
        36,
        assets.building_office_w,
        assets.building_office_h,
    );
    blit_sprite(
        canvas,
        &assets.building_shop,
        740,
        160,
        assets.building_shop_w,
        assets.building_shop_h,
    );
    blit_sprite(
        canvas,
        &assets.park,
        980,
        48,
        assets.park_w,
        assets.park_h,
    );
    for (x, y) in [(760, 80), (940, 210), (1100, 100)] {
        blit_sprite(canvas, &assets.tree, x, y, assets.tree_w, assets.tree_h);
    }

    // Southwest: apartment left, park centred-right, shop on the far side
    blit_sprite(
        canvas,
        &assets.building_apartment,
        16,
        560,
        assets.building_apartment_w,
        assets.building_apartment_h,
    );
    blit_sprite(
        canvas,
        &assets.park,
        160,
        536,
        assets.park_w,
        assets.park_h,
    );
    blit_sprite(
        canvas,
        &assets.building_shop,
        352,
        560,
        assets.building_shop_w,
        assets.building_shop_h,
    );

    // Southeast: shop near the road, park right-aligned, trees in open grass
    let road_right = (CENTER_X + ROAD_HALF) as i32;
    let road_bottom = (CENTER_Y + ROAD_HALF) as i32;
    let pad = 16;
    let status_bar_h = 30;
    let content_bottom = WINDOW_H as i32 - status_bar_h - pad;

    let se_park_x = WINDOW_W as i32 - pad - assets.park_w as i32;
    let se_park_y = content_bottom - assets.park_h as i32;
    let se_shop_x = road_right + pad;
    let se_shop_y = content_bottom - assets.building_shop_h as i32;

    blit_sprite(
        canvas,
        &assets.building_shop,
        se_shop_x,
        se_shop_y,
        assets.building_shop_w,
        assets.building_shop_h,
    );
    blit_sprite(
        canvas,
        &assets.park,
        se_park_x,
        se_park_y,
        assets.park_w,
        assets.park_h,
    );

    let shop_right = se_shop_x + assets.building_shop_w as i32;
    let gap_mid_x = shop_right + (se_park_x - shop_right) / 2;
    for (x, y) in [
        (gap_mid_x - 20, se_park_y + 24),
        (gap_mid_x + 36, se_park_y + 72),
        (se_park_x - 44, se_park_y + 118),
        (WINDOW_W as i32 - pad - assets.tree_w as i32, content_bottom - assets.tree_h as i32),
        (gap_mid_x + 8, road_bottom + 36),
    ] {
        blit_sprite(canvas, &assets.tree, x, y, assets.tree_w, assets.tree_h);
    }
}
