use core::cmp::{max, min};
use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgba, SubImage};

use crate::game_logic::*;
use crate::ukeire_calculator::*;

pub type ImageBuf = ImageBuffer<Rgba<u8>, Vec<u8>>;

pub struct SizedImageData {
    tiles_atlas: DynamicImage,
    tile_width: u32,
    tile_height: u32,
    bg_color: Rgba<u8>,
}

pub struct ImageRenderData {
    pub sizes: [SizedImageData; 3],
}

pub fn load_sized_image_data(path: &str) -> SizedImageData {
    let atlas = ImageReader::open(path)
        .expect(&format!("file '{}' not found", path))
        .decode()
        .expect(&format!("file '{}' can't be decoded", path));
    let tile_width = atlas.width() / 10;
    let tile_height = atlas.height() / 4;

    SizedImageData {
        tiles_atlas: atlas,
        tile_width: tile_width,
        tile_height: tile_height,
        bg_color: Rgba([53, 101, 77, 255]),
    }
}

pub fn load_static_render_data() -> ImageRenderData {
    ImageRenderData {
        sizes: [
            load_sized_image_data("resources/tileset_atlas_small.png"),
            load_sized_image_data("resources/tileset_atlas_medium.png"),
            load_sized_image_data("resources/tileset_atlas_large.png"),
        ],
    }
}

fn get_tile_image<'a>(tile: &Tile, render_data: &'a SizedImageData) -> SubImage<&'a DynamicImage> {
    let index = get_tile_index(tile);
    let x = (index % 10) as u32;
    let y = (index / 10) as u32;
    render_data.tiles_atlas.view(
        x * render_data.tile_width,
        y * render_data.tile_height,
        render_data.tile_width,
        render_data.tile_height,
    )
}

fn get_back_side_image<'a>(render_data: &'a SizedImageData) -> SubImage<&'a DynamicImage> {
    render_data.tiles_atlas.view(
        9 * render_data.tile_width,
        3 * render_data.tile_height,
        render_data.tile_width,
        render_data.tile_height,
    )
}

fn render_hand_to_image(
    img: &mut (impl GenericImageView<Pixel = Rgba<u8>> + GenericImage),
    hand: &Hand,
    render_data: &SizedImageData,
    x: u32,
    y: u32,
    drawn_tile_gap: u32,
) {
    for i in 0..13 {
        let tile_sprite_view = get_tile_image(&hand.tiles[i], &render_data);
        img.copy_from(
            &tile_sprite_view.to_image(),
            x + render_data.tile_width * i as u32,
            y,
        )
        .unwrap();
    }

    if hand.tiles[13] != EMPTY_TILE {
        let tile_sprite_view = get_tile_image(&hand.tiles[13], &render_data);
        img.copy_from(
            &tile_sprite_view.to_image(),
            x + render_data.tile_width * 13 + drawn_tile_gap,
            y,
        )
        .unwrap();
    }
}

fn render_discards_to_image(
    img: &mut (impl GenericImageView<Pixel = Rgba<u8>> + GenericImage),
    tiles: &[Tile],
    render_data: &SizedImageData,
    x: u32,
    y: u32,
    width: u32,
) {
    let mut pos_x = 0;
    let mut pos_y = 0;
    for tile in tiles {
        let tile_sprite_view = get_tile_image(&tile, &render_data);
        img.copy_from(
            &tile_sprite_view.to_image(),
            x + pos_x * render_data.tile_width,
            y + pos_y * render_data.tile_height,
        )
        .unwrap();
        pos_x += 1;
        if pos_x >= width {
            pos_y += 1;
            pos_x = 0;
        }
    }
}

fn render_dora_indicators_to_image(
    img: &mut (impl GenericImageView<Pixel = Rgba<u8>> + GenericImage),
    dora_indicators: &[Tile],
    render_data: &SizedImageData,
    x: u32,
    y: u32,
) {
    for i in 0..7 {
        if i != 4 {
            let tile_sprite_view = get_back_side_image(&render_data);
            img.copy_from(
                &tile_sprite_view.to_image(),
                x + i * render_data.tile_width,
                y,
            )
            .unwrap();
        }
    }

    {
        let tile_sprite_view = get_tile_image(&dora_indicators[0], &render_data);
        img.copy_from(
            &tile_sprite_view.to_image(),
            x + 4 * render_data.tile_width,
            y,
        )
        .unwrap();
    }
}

pub fn render_game_state(game: &GameState, render_data: &ImageRenderData) -> ImageBuf {
    let total_width_tiles = 14;
    let total_height_tiles = 10;

    // choose middle size for game state as it seem to fit the best with the dimensions chosen above
    let render_data = &render_data.sizes[1];

    let drawn_tile_gap = render_data.tile_width / 4;
    let top_offset = render_data.tile_height / 4;
    let total_width = render_data.tile_width * total_width_tiles + drawn_tile_gap;
    let total_height = render_data.tile_height * total_height_tiles + top_offset;

    let mut img = ImageBuffer::from_pixel(total_width, total_height, render_data.bg_color);
    let middle_x = (render_data.tile_width * total_width_tiles + drawn_tile_gap) / 2;

    render_hand_to_image(
        &mut img,
        &game.hands[0],
        &render_data,
        0,
        top_offset + render_data.tile_height * 9,
        drawn_tile_gap,
    );

    let discards: &Vec<Tile> = &game.discards[0];
    if !discards.is_empty() {
        let discards_width = min(max(6, 1 + (discards.len() - 1) / 6), 14) as u32;
        let mut discards_top_shift = (7 - (discards.len() - 1) / discards_width as usize) as u32;
        // after this size tiles won't fit normally anymore, reduce the gaps to fit more
        if discards.len() > 14 * 6 {
            discards_top_shift = 1;
        }
        render_discards_to_image(
            &mut img,
            &discards,
            &render_data,
            middle_x - render_data.tile_width * discards_width / 2,
            top_offset + discards_top_shift * render_data.tile_height,
            discards_width,
        );
    }

    render_dora_indicators_to_image(
        &mut img,
        &game.dora_indicators,
        &render_data,
        middle_x - render_data.tile_width * 7 / 2,
        top_offset,
    );

    return img;
}

fn render_explanation_line_to_image(
    img: &mut (impl GenericImageView<Pixel = Rgba<u8>> + GenericImage),
    discard: &Tile,
    improvements: &[Tile],
    total_improvements: &[Tile],
    render_data: &SizedImageData,
    x: u32,
    y: u32,
    gap_after_discard: u32,
) {
    {
        let tile_sprite_view = get_tile_image(&discard, &render_data);
        img.copy_from(&tile_sprite_view.to_image(), x, y).unwrap();
    }

    let mut local_i = 0;
    for i in 0..total_improvements.len() {
        if total_improvements[i] == improvements[local_i] {
            let tile_sprite_view = get_tile_image(&improvements[local_i], &render_data);
            img.copy_from(
                &tile_sprite_view.to_image(),
                x + gap_after_discard + (i as u32 + 1) * render_data.tile_width,
                y,
            )
            .unwrap();
            local_i += 1;
            if local_i == improvements.len() {
                break;
            }
        }
    }
}

pub fn render_move_explanation(
    previous_move: &PreviousMoveData,
    score_settings: &ScoreCalculationSettings,
    render_data: &ImageRenderData,
) -> ImageBuf {
    assert!(
        previous_move.game_state.hands[previous_move.hand_index].tiles[13] != EMPTY_TILE,
        "Expected move state hand have 14 tiles before the discard"
    );

    let mut visible_tiles = get_visible_tiles(&previous_move.game_state, previous_move.hand_index);
    let best_discards = calculate_best_discards_ukeire2(
        &previous_move.game_state.hands[previous_move.hand_index].tiles,
        previous_move.full_hand_shanten,
        &mut visible_tiles,
        &score_settings,
    );

    let mut total_improvements: Vec<Tile> = Vec::new();
    for discard_info in &best_discards {
        for improvement in &discard_info.tiles_improving_shanten {
            // todo: linear search should probably be more efficient here
            match total_improvements.binary_search(&improvement) {
                Ok(_pos) => {}
                Err(pos) => total_improvements.insert(pos, *improvement),
            }
        }
    }

    let total_width_tiles = total_improvements.len() as u32 + 1;
    let total_height_tiles = best_discards.len() as u32;

    let min_approximation = f32::min(
        (total_width_tiles * render_data.sizes[0].tile_width) as f32,
        (total_height_tiles * render_data.sizes[0].tile_height) as f32,
    );
    let max_approximation = f32::max(
        (total_width_tiles * render_data.sizes[0].tile_width) as f32,
        (total_height_tiles * render_data.sizes[0].tile_height) as f32,
    );

    // calculate the best fitting tile size based on amount of tiles
    // the more tiles the worse the quality will be
    // the values are chosen based on manual testing
    let size_idx = if min_approximation < 150.0 && max_approximation < 250.0 {
        2
    } else if min_approximation < 300.0 && max_approximation < 500.0 {
        1
    } else {
        0
    };
    let render_data = &render_data.sizes[size_idx];

    let horizontal_gap = render_data.tile_width / 4;
    let vertical_gap = render_data.tile_height / 4;

    let mut img = ImageBuffer::from_pixel(
        horizontal_gap * 3 + total_width_tiles * render_data.tile_width,
        vertical_gap + total_height_tiles * (render_data.tile_height + vertical_gap),
        render_data.bg_color,
    );

    let mut pos_y = vertical_gap;
    for discard_info in best_discards {
        render_explanation_line_to_image(
            &mut img,
            &discard_info.tile,
            &discard_info.tiles_improving_shanten,
            &total_improvements,
            &render_data,
            horizontal_gap,
            pos_y,
            horizontal_gap,
        );
        pos_y += render_data.tile_height + vertical_gap;
    }

    return img;
}
