use crate::rand::prelude::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum Suit {
    Man,     // printed as "m"
    Pin,     // printed as "p"
    Sou,     // printed as "s"
    Special, // printed as "z", values: 0:Empty 1:East 2:South 3:West 4:North 5:White 6:Green 7:Red
}

#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Tile {
    pub suit: Suit,
    pub value: u8,
}

pub const EMPTY_TILE: Tile = Tile {
    suit: Suit::Special,
    value: 0,
};

pub type HandTiles = [Tile; 14];
pub type DeadWall = [Tile; 14];

#[derive(Clone)]
pub struct Hand {
    pub tiles: HandTiles,
}

pub const EMPTY_HAND: Hand = Hand {
    tiles: [EMPTY_TILE; 14],
};

#[derive(Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub deal_first_tile: bool,
    pub include_honors: bool,
}

// store tiles as cumulative frequency distribution (store count of every possible tile in a hand)
pub type TileFrequencyTable = [u8; 37];
pub const EMPTY_FREQUENCY_TABLE: TileFrequencyTable = [0; 37];

#[derive(Clone)]
pub struct GameState {
    pub hands: Vec<Hand>,
    pub discards: Vec<Vec<Tile>>,
    pub total_discards_table: TileFrequencyTable,
    pub _dead_wall: DeadWall,
    pub dora_indicators: [Tile; 8], // 1-3 - dora indicators, 4-7 - uradora indicators
    pub opened_dora_indicators: u8,
    pub live_wall: Vec<Tile>,
}

pub fn get_tile_index(tile: &Tile) -> usize {
    let shift = match tile.suit {
        Suit::Man => 0,
        Suit::Pin => 10,
        Suit::Sou => 20,
        Suit::Special => 30,
    };

    return (tile.value + shift - 1) as usize;
}

fn sort_hand(hand: &mut Hand) {
    if hand.tiles[13] == EMPTY_TILE {
        hand.tiles[0..13].sort();
    } else {
        hand.tiles.sort();
    }
}

fn populate_full_set(game_settings: &GameSettings) -> Vec<Tile> {
    let mut result = Vec::with_capacity((9 * 3 + 7) * 4);

    for i in 1..=9 {
        for _j in 0..4 {
            result.push(Tile {
                suit: Suit::Man,
                value: i,
            });
        }
    }

    for i in 1..=9 {
        for _j in 0..4 {
            result.push(Tile {
                suit: Suit::Pin,
                value: i,
            });
        }
    }

    for i in 1..=9 {
        for _j in 0..4 {
            result.push(Tile {
                suit: Suit::Sou,
                value: i,
            });
        }
    }

    if game_settings.include_honors {
        for i in 1..=7 {
            for _j in 0..4 {
                result.push(Tile {
                    suit: Suit::Special,
                    value: i,
                });
            }
        }
    }

    return result;
}

pub fn generate_normal_dealt_game(
    player_count: u32,
    game_settings: &GameSettings,
) -> Result<GameState, String> {
    let mut tiles = populate_full_set(game_settings);
    tiles.shuffle(&mut thread_rng());

    let dead_wall: [Tile; 14] = tiles
        .split_off(tiles.len() - 14)
        .try_into()
        .map_err(|_| "Incorrect number of tiles")?;
    // 1-3 - dora indicators, 4-7 - uradora indicators
    let dora_indicators: [Tile; 8] = dead_wall[4..12]
        .try_into()
        .map_err(|_| "Incorrect number of tiles")?;

    let mut hands = Vec::with_capacity(player_count as usize);
    let mut discards = Vec::with_capacity(player_count as usize);
    for i in 0..player_count {
        let new_tiles = [tiles.split_off(tiles.len() - 13), [EMPTY_TILE].to_vec()].concat();
        hands.push(Hand {
            tiles: new_tiles
                .try_into()
                .map_err(|_| "Incorrect number of tiles")?,
        });
        sort_hand(&mut hands[i as usize]);
        discards.push(Vec::new());
    }

    let mut game_state = GameState {
        hands: hands,
        discards: discards,
        total_discards_table: EMPTY_FREQUENCY_TABLE,
        _dead_wall: dead_wall,
        dora_indicators: dora_indicators,
        opened_dora_indicators: 1,
        live_wall: tiles,
    };

    if game_settings.deal_first_tile {
        draw_tile_to_hand(&mut game_state, 0);
    }

    return Ok(game_state);
}

pub fn generate_dealt_game_with_hand_and_discards(
    player_count: u32,
    predefined_hand: Hand,
    predefined_discards: Vec<Tile>,
    game_settings: &GameSettings,
) -> Result<GameState, String> {
    if predefined_hand.tiles[0] == EMPTY_TILE {
        return Err("Incorrect hand".to_string());
    }

    let game_settings = GameSettings {
        deal_first_tile: game_settings.deal_first_tile,
        include_honors: game_settings.include_honors
            || predefined_hand
                .tiles
                .iter()
                .position(|&t| t.suit == Suit::Special)
                .is_some()
            || predefined_discards
                .iter()
                .position(|&t| t.suit == Suit::Special)
                .is_some(),
    };

    let mut tiles = populate_full_set(&game_settings);

    for tile in predefined_hand.tiles {
        if tile != EMPTY_TILE {
            let index_result = tiles.iter().position(|&t| t == tile);

            if let Some(index) = index_result {
                tiles.remove(index);
            };
        }
    }

    tiles.shuffle(&mut thread_rng());

    let mut dead_wall: [Tile; 14] = tiles
        .split_off(tiles.len() - 14)
        .try_into()
        .map_err(|_| "Incorrect number of tiles left to form dead wall")?;

    if predefined_discards.len() > 0 && predefined_discards[0] != EMPTY_TILE {
        dead_wall[4] = predefined_discards[0];
    }

    // 1-3 - dora indicators, 4-7 - uradora indicators
    let dora_indicators: [Tile; 8] = dead_wall[4..12]
        .try_into()
        .map_err(|_| "Incorrect number of tiles in dead wall")?;

    let mut hands = Vec::with_capacity(player_count as usize);
    let mut discards = Vec::with_capacity(player_count as usize);

    hands.push(predefined_hand);
    discards.push(Vec::new());

    for i in 1..player_count {
        let new_tiles = [tiles.split_off(tiles.len() - 13), [EMPTY_TILE].to_vec()].concat();
        hands.push(Hand {
            tiles: new_tiles
                .try_into()
                .map_err(|_| "Incorrect number of tiles left to form player hand")?,
        });
        sort_hand(&mut hands[i as usize]);
        discards.push(Vec::new());
    }

    {
        let mut player_index = 0;
        for i in 1..predefined_discards.len() {
            let tile = predefined_discards[i];
            if tile != EMPTY_TILE {
                // find first tile in tiles
                let index_result = tiles.iter().position(|&t| t == tile);
                if let Some(index) = index_result {
                    tiles.remove(index);
                    discards[player_index].push(tile);
                    player_index = (player_index + 1) % player_count as usize;
                }
            }
        }
    }

    let mut game_state = GameState {
        hands: hands,
        discards: discards,
        total_discards_table: EMPTY_FREQUENCY_TABLE,
        _dead_wall: dead_wall,
        dora_indicators: dora_indicators,
        opened_dora_indicators: 1,
        live_wall: tiles,
    };

    if game_state.hands[0].tiles[13] == EMPTY_TILE && game_settings.deal_first_tile {
        draw_tile_to_hand(&mut game_state, 0);
    }

    return Ok(game_state);
}

pub fn make_frequency_table(tiles: &[Tile]) -> TileFrequencyTable {
    let mut result: TileFrequencyTable = EMPTY_FREQUENCY_TABLE;

    for tile in tiles {
        if *tile == EMPTY_TILE {
            panic!("Incorrect tile in shanten calculation {:?}", tiles);
        }
        result[get_tile_index(tile)] += 1
    }

    return result;
}

pub fn draw_tile_to_hand(game: &mut GameState, hand_index: usize) {
    game.hands[hand_index].tiles[13] = game.live_wall.split_off(game.live_wall.len() - 1)[0];
}

pub fn discard_tile(game: &mut GameState, hand_index: usize, tile_index: usize) -> Tile {
    let discarded_tile = game.hands[hand_index].tiles[tile_index];
    game.hands[hand_index].tiles[tile_index..14].rotate_left(1);
    game.hands[hand_index].tiles[13] = EMPTY_TILE;
    // we sort in the tile that was added the last
    sort_hand(&mut game.hands[hand_index]);

    game.total_discards_table[get_tile_index(&discarded_tile)] += 1;
    game.discards[hand_index].push(discarded_tile);
    return discarded_tile;
}
