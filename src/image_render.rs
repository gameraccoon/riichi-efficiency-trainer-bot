use core::cmp::{min, max};
use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgba, SubImage};
use crate::game_logic::*;

pub type ImageBuf = ImageBuffer<Rgba<u8>, Vec<u8>>;

pub struct SizedImageData {
    tiles_atlas: DynamicImage,
    image_buffer: ImageBuf,
    tile_width: u32,
    tile_height: u32,
    drawn_tile_gap: u32,
    top_offset: u32,
}

pub struct ImageRenderData {
    pub sizes: [SizedImageData; 3],
}

pub fn load_sized_image_data(path: &str) -> SizedImageData {
    let atlas = ImageReader::open(path).expect(&format!("file '{}' not found", path)).decode().expect(&format!("file '{}' can't be decoded", path));
    let tile_width = atlas.width() / 10;
    let tile_height = atlas.height() / 4;

    let drawn_tile_gap = tile_width / 4;
    let top_offset = tile_height / 4;
    let total_width = tile_width * 14 + drawn_tile_gap;
    let total_height = tile_height * 10 + top_offset;

    let bg_color: Rgba<u8> = Rgba([53, 101, 77, 255]);
    
    SizedImageData{
        tiles_atlas: atlas,
        image_buffer: ImageBuffer::from_pixel(total_width, total_height, bg_color),
        tile_width: tile_width,
        tile_height: tile_height,
        drawn_tile_gap: drawn_tile_gap,
        top_offset: top_offset,
    }
}

pub fn load_static_render_data() -> ImageRenderData {
    ImageRenderData{
        sizes: [
            load_sized_image_data("resources/tileset_atlas_small.png"),
            load_sized_image_data("resources/tileset_atlas_medium.png"),
            load_sized_image_data("resources/tileset_atlas_large.png")
        ]
    }
}

fn get_tile_image<'a>(tile: &Tile, render_data: &'a SizedImageData) -> SubImage<&'a DynamicImage> {
    let index = get_tile_index(tile);
    let x = (index % 10) as u32;
    let y = (index / 10) as u32; 
    render_data.tiles_atlas.view(x * render_data.tile_width, y * render_data.tile_height, render_data.tile_width, render_data.tile_height)
}

fn get_back_side_image<'a>(render_data: &'a SizedImageData) -> SubImage<&'a DynamicImage> {
    render_data.tiles_atlas.view(9 * render_data.tile_width, 3 * render_data.tile_height, render_data.tile_width, render_data.tile_height)
}

fn render_hand_to_image(img: &mut (impl GenericImageView<Pixel = Rgba<u8>> + GenericImage), hand: &Hand, render_data: &SizedImageData, x: u32, y: u32, drawn_tile_gap: u32) {
    for i in 0..13 {
        let tile_sprite_view = get_tile_image(&hand.tiles[i], &render_data);
        img.copy_from(&tile_sprite_view.to_image(), x + render_data.tile_width * i as u32, y).unwrap();
    }

    if hand.tiles[13] != EMPTY_TILE {
        let tile_sprite_view = get_tile_image(&hand.tiles[13], &render_data);
        img.copy_from(&tile_sprite_view.to_image(), x + render_data.tile_width * 13 + drawn_tile_gap, y).unwrap();
    }
}

fn render_discards_to_image(img: &mut (impl GenericImageView<Pixel = Rgba<u8>> + GenericImage), tiles: &[Tile], render_data: &SizedImageData, x: u32, y: u32, width: u32) {
    let mut pos_x = 0;
    let mut pos_y = 0;
    for tile in tiles {
        let tile_sprite_view = get_tile_image(&tile, &render_data);
        img.copy_from(&tile_sprite_view.to_image(), x + pos_x * render_data.tile_width, y + pos_y * render_data.tile_height).unwrap();
        pos_x += 1;
        if pos_x >= width {
            pos_y += 1;
            pos_x = 0;
        }
    }
}

fn render_dora_indicators_to_image(img: &mut (impl GenericImageView<Pixel = Rgba<u8>> + GenericImage), dora_indicators: &[Tile], render_data: &SizedImageData, x: u32, y: u32) {
    for i in 0..7 {
        if i != 4 {
            let tile_sprite_view = get_back_side_image(&render_data);
            img.copy_from(&tile_sprite_view.to_image(), x + i * render_data.tile_width, y).unwrap();
        }
    }

    {
        let tile_sprite_view = get_tile_image(&dora_indicators[0], &render_data);
        img.copy_from(&tile_sprite_view.to_image(), x + 4 * render_data.tile_width, y).unwrap();
    }
}

pub fn render_game_state(game: &GameState, render_data: &SizedImageData) -> ImageBuf {
    let mut img = render_data.image_buffer.clone();
    let middle_x = (render_data.tile_width * 14 + render_data.drawn_tile_gap) / 2;

    render_hand_to_image(&mut img, &game.hands[0], &render_data, 0, render_data.top_offset + render_data.tile_height * 9, render_data.drawn_tile_gap);

    let discards: &Vec<Tile> = &game.discards[0];
    if !discards.is_empty() {
        let discards_width = min(max(6, 1 + (discards.len() - 1) / 6), 14) as u32;
        let mut discards_top_shift = (7 - (discards.len() - 1) / discards_width as usize) as u32;
        // after this size tiles won't fit normally anymore, reduce the gaps to fit more
        if discards.len() > 14*6 {
            discards_top_shift = 1;
        }
        render_discards_to_image(&mut img, &discards, &render_data, middle_x - render_data.tile_width * discards_width / 2, render_data.top_offset + discards_top_shift * render_data.tile_height, discards_width);
    }

    render_dora_indicators_to_image(&mut img, &game.dora_indicators, &render_data, middle_x - render_data.tile_width * 7 / 2, render_data.top_offset);

    return img;
}
