extern crate rand;

use crate::rand::prelude::SliceRandom;
use dashmap::DashMap;
use rand::thread_rng;
use std::cmp::max;
use std::env;
use std::fs;
use std::sync::Arc;
use teloxide::prelude::*;


const TILE_UNICODE: [&str; 37] = [
    "🀇", "🀈", "🀉", "🀊", "🀋", "🀌", "🀍", "🀎", "🀏", "err",
    "🀙", "🀚", "🀛", "🀜", "🀝", "🀞", "🀟", "🀠", "🀡", "err",
    "🀐", "🀑", "🀒", "🀓", "🀔", "🀕", "🀖", "🀗", "🀘", "err",
    "🀀", "🀁", "🀂", "🀃", "🀆", "🀅", "🀄"
];

#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
enum Suit {
    Man, // printed as "m"
    Pin, // printed as "p"
    Sou, // printed as "s"
    Special, // printed as "z", values: 0:Empty 1:East 2:South 3:West 4:North 5:White 6:Green 7:Red
}

#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
struct Tile {
    suit: Suit,
    value: u8,
}

const EMPTY_TILE : Tile = Tile{suit: Suit::Special, value: 0};

type HandTiles = [Tile; 14];
type DeadWall = [Tile; 14];

#[derive(Clone)]
struct Hand {
    tiles: HandTiles,
}

const EMPTY_HAND: Hand = Hand{tiles: [EMPTY_TILE; 14]};

// store tiles as cumulative frequency distribution (store count of every possible tile in a hand)
type TileFrequencyTable = [u8; 37];
const EMPTY_FREQUENCY_TABLE: TileFrequencyTable = [0; 37];

#[derive(Clone)]
struct GameState {
    hands: Vec<Hand>,
    discards: Vec<Vec<Tile>>,
    total_discards_table: TileFrequencyTable,
    _dead_wall: DeadWall,
    dora_indicators: [Tile; 8], // 1-3 - dora indicators, 4-7 - uradora indicators
    live_wall: Vec<Tile>,
}

#[derive(Debug)]
enum TileDisplayOption {
    Text,
    Unicode,
}

fn print_hand(hand: &Hand, tile_display: &TileDisplayOption) {
    println!("{}", get_printable_tiles_set(&hand.tiles, tile_display));
}

fn get_printable_tiles_set(tiles: &[Tile], tile_display: &TileDisplayOption) -> String {
    match tile_display {
        TileDisplayOption::Text => get_printable_tiles_set_text(&tiles),
        TileDisplayOption::Unicode => get_printable_tiles_set_unicode(&tiles),
    }
}

fn tile_to_string(tile: &Tile, tile_display: &TileDisplayOption) -> String {
    match tile_display {
        TileDisplayOption::Text => tile.value.to_string() + get_printable_suit(tile.suit),
        TileDisplayOption::Unicode => TILE_UNICODE[get_tile_index(&tile)].to_string(),
    }
}

fn get_tile_index(tile: &Tile) -> usize {
    let shift = match tile.suit {
        Suit::Man => 0,
        Suit::Pin => 10,
        Suit::Sou => 20,
        Suit::Special => 30,
    };

    return (tile.value + shift - 1) as usize;
}

fn get_tile_from_index(index: usize) -> Tile {
    return match index {
        i if i < (10 as usize) => Tile{suit: Suit::Man, value: (index + 1) as u8},
        i if i < (20 as usize) => Tile{suit: Suit::Pin, value: (index + 1 - 10) as u8},
        i if i < (30 as usize) => Tile{suit: Suit::Sou, value: (index + 1 - 20) as u8},
        _ => Tile{suit: Suit::Special, value: (index + 1 - 30) as u8},
    }
}

fn get_printable_tiles_set_unicode(tiles: &[Tile]) -> String {
    let mut result: String = "".to_string();

    for t in tiles {
        if t.value == 0 {
            break;
        }

        result += TILE_UNICODE[get_tile_index(t)];
    }

    return result;
}

fn get_printable_suit(suit: Suit) -> &'static str {
    match suit {
        Suit::Man => "m",
        Suit::Pin => "p",
        Suit::Sou => "s",
        Suit::Special => "z",
    }
}

fn get_suit_from_letter(suit_letter: char) -> Option<Suit> {
    match suit_letter {
        'm' => Some(Suit::Man),
        'p' => Some(Suit::Pin),
        's' => Some(Suit::Sou),
        'z' => Some(Suit::Special),
        _ => None,
    }
}

fn get_printable_tiles_set_text(tiles: &[Tile]) -> String {
    let mut result: String = "".to_string();

    let mut last_suit = Suit::Special;
    for tile in tiles {
        if tile.value == 0 {
            break;
        }

        if tile.suit != last_suit && result != "" {
            result += get_printable_suit(last_suit);
        }

        last_suit = tile.suit;

        result += &tile.value.to_string();
    }

    result += get_printable_suit(last_suit);

    return result;
}

fn sort_hand(hand: &mut Hand) {
    if hand.tiles[13] == EMPTY_TILE {
        hand.tiles[0..13].sort();
    }
    else {
        hand.tiles.sort();
    }
}

fn populate_full_set() -> Vec<Tile> {
    let mut result = Vec::with_capacity((9*3 + 7) * 4);

    for i in 1..=9 {
        for _j in 0..4 {
            result.push(Tile{suit:Suit::Man, value:i});
        }
    }

    for i in 1..=9 {
        for _j in 0..4 {
            result.push(Tile{suit:Suit::Pin, value:i});
        }
    }

    for i in 1..=9 {
        for _j in 0..4 {
            result.push(Tile{suit:Suit::Sou, value:i});
        }
    }

    for i in 1..=7 {
        for _j in 0..4 {
            result.push(Tile{suit:Suit::Special, value:i});
        }
    }

    return result;
}

fn generate_normal_dealed_game(player_count: u32, deal_first_tile: bool) -> GameState {
    let mut tiles = populate_full_set();
    tiles.shuffle(&mut thread_rng());

    let dead_wall: [Tile; 14] = tiles.split_off(tiles.len() - 14).try_into().unwrap();
     // 1-3 - dora indicators, 4-7 - uradora indicators
    let dora_indicators: [Tile; 8] = dead_wall[4..12].try_into().unwrap();

    let mut hands = Vec::with_capacity(player_count as usize);
    let mut discards = Vec::with_capacity(player_count as usize);
    for i in 0..player_count {
        let new_tiles = [tiles.split_off(tiles.len() - 13), [EMPTY_TILE].to_vec()].concat();
        hands.push(Hand{tiles: new_tiles.try_into().unwrap()});
        sort_hand(&mut hands[i as usize]);
        discards.push(Vec::new());
    }

    let mut game_state = GameState{
        hands: hands,
        discards: discards,
        total_discards_table: EMPTY_FREQUENCY_TABLE,
        _dead_wall: dead_wall,
        dora_indicators: dora_indicators,
        live_wall: tiles
    };

    if deal_first_tile {
        draw_tile_to_hand(&mut game_state, 0);
    }

    return game_state;
}

fn make_hand_from_string(hand_string: &str) -> Hand {
    if hand_string.is_empty() {
        return EMPTY_HAND;
    }

    // we can't have a valid hand less than 13 tiles + suit letter
    if hand_string.len() < 14 {
        println!("The provided hand can't be parsed, make sure you copied the full string");
        return EMPTY_HAND;
    }

    let tiles_count = hand_string.chars().filter(|c| c.is_numeric()).count();
    if tiles_count < 13 || tiles_count > 14 {
        println!("The provided hand can't be parsed, possibly incorrect amount of tiles");
        return EMPTY_HAND;
    }

    let mut hand: Hand = EMPTY_HAND;
    let mut current_suit: Option<Suit> = None;
    let mut tile_position = tiles_count;
    for i in (0..hand_string.len()).rev() {
        let letter = hand_string.chars().nth(i).unwrap();
        let value = letter.to_string().parse().unwrap_or(0);
        if value > 0 {
            if current_suit.is_none() {
                println!("The provided hand can't be parsed, possibly incorrect suit letter");
                return EMPTY_HAND;
            }
            tile_position -= 1;
            hand.tiles[tile_position].suit = current_suit.unwrap();
            hand.tiles[tile_position].value = value;
        }
        else {
            current_suit = get_suit_from_letter(letter);
        }
    }
    return hand;
}

fn generate_dealed_game_with_hand(player_count: u32, hand_string: &str, deal_first_tile: bool) -> GameState {
    let predefined_hand = make_hand_from_string(&hand_string);

    if predefined_hand.tiles[0] == EMPTY_TILE {
        return generate_normal_dealed_game(player_count, deal_first_tile);
    }

    let mut tiles = populate_full_set();

    for tile in predefined_hand.tiles {
        if tile != EMPTY_TILE {
            let index = tiles.iter().position(|&t| t == tile).unwrap();
            tiles.remove(index);
        }
    }

    tiles.shuffle(&mut thread_rng());

    let dead_wall: [Tile; 14] = tiles.split_off(tiles.len() - 14).try_into().unwrap();
     // 1-3 - dora indicators, 4-7 - uradora indicators
    let dora_indicators: [Tile; 8] = dead_wall[4..12].try_into().unwrap();

    let mut hands = Vec::with_capacity(player_count as usize);
    let mut discards = Vec::with_capacity(player_count as usize);

    hands.push(predefined_hand);
    discards.push(Vec::new());

    for i in 1..player_count {
        let new_tiles = [tiles.split_off(tiles.len() - 13), [EMPTY_TILE].to_vec()].concat();
        hands.push(Hand{tiles: new_tiles.try_into().unwrap()});
        sort_hand(&mut hands[i as usize]);
        discards.push(Vec::new());
    }

    let mut game_state = GameState{
        hands: hands,
        discards: discards,
        total_discards_table: EMPTY_FREQUENCY_TABLE,
        _dead_wall: dead_wall,
        dora_indicators: dora_indicators,
        live_wall: tiles,
    };

    if game_state.hands[0].tiles.len() < 14 && deal_first_tile {
        draw_tile_to_hand(&mut game_state, 0);
    }

    return game_state;
}

fn make_frequency_table(tiles: &[Tile]) -> TileFrequencyTable {
    let mut result: TileFrequencyTable = EMPTY_FREQUENCY_TABLE;

    for tile in tiles {
        if *tile == EMPTY_TILE {
            panic!("Incorrect tile in shanten calculation: {}", get_printable_tiles_set_text(tiles));
        }
        result[get_tile_index(tile)] += 1
    }

    return result;
}

fn convert_frequency_table_to_flat_vec(frequency_table: &TileFrequencyTable) -> Vec<Tile> {
    let mut result = Vec::new();

    for i in 0..frequency_table.len() {
        if frequency_table[i] > 0 {
            let tile = get_tile_from_index(i);
            result.push(tile);
        }
    }

    return result;
}

fn set_max(element: &mut u8, value: u8) {
    *element = max(*element, value);
}

fn append_frequency_table(target_table: &mut TileFrequencyTable, addition_table: &TileFrequencyTable) {
    for i in 0..target_table.len() {
        if addition_table[i] > 0 {
            set_max(&mut target_table[i], 2);
        }
    }
}

struct ShantenCalculator {
    hand_table: TileFrequencyTable,
    waits_table: TileFrequencyTable,
    complete_sets: i8,
    pair: i8,
    partial_sets: i8,
    best_shanten: i8,
    best_waits: TileFrequencyTable,
}

const MAX_SHANTEN: i8 = 8;

impl ShantenCalculator {
    fn append_single_tiles_left(&mut self) {
        for i in 0..self.hand_table.len() {
            if self.hand_table[i] == 1 {
                // potential pair
                set_max(&mut self.best_waits[i], 1);
                // potential protoruns
                if i < 30 {
                    let pos_in_suit = i % 10;
                    if pos_in_suit >= 1 {
                        set_max(&mut self.best_waits[i - 1], 1);
                    }
                    if pos_in_suit >= 2 {
                        set_max(&mut self.best_waits[i - 2], 1);
                    }
                    if pos_in_suit <= 7 {
                        set_max(&mut self.best_waits[i + 1], 1);
                    }
                    if pos_in_suit <= 6 {
                        set_max(&mut self.best_waits[i + 2], 1);
                    }
                }
            }
        }
    }

    fn remove_potential_sets(&mut self, mut i: usize) {
        // Skip to the next tile that exists in the hand
        while i < self.hand_table.len() && self.hand_table[i] == 0 { i += 1; }

        if i >= self.hand_table.len() {
            // We've checked everything. See if this shanten is better than the current best.
            let current_shanten = (8 - (self.complete_sets * 2) - self.partial_sets - self.pair) as i8;
            if current_shanten < self.best_shanten {
                self.best_shanten = current_shanten;
                self.best_waits = self.waits_table.clone();
                self.append_single_tiles_left();
            }
            else if current_shanten == self.best_shanten {
                append_frequency_table(&mut self.best_waits, &self.waits_table);
                self.append_single_tiles_left();
            }
            return;
        }

        // A standard hand will only ever have four groups plus a pair.
        if self.complete_sets + self.partial_sets < 4 {
            // Pair
            if self.hand_table[i] == 2 {
                self.partial_sets += 1;
                self.hand_table[i] -= 2;
                self.waits_table[i] += 1;
                self.remove_potential_sets(i);
                self.waits_table[i] -= 1;
                self.hand_table[i] += 2;
                self.partial_sets -= 1;
            }

            // Edge or Side wait protorun
            if i < 30 && self.hand_table[i + 1] != 0 {
                let left_edge_wait = i % 10 == 0;
                let right_edge_wait = i % 10 == 7;

                self.partial_sets += 1;
                self.hand_table[i] -= 1; self.hand_table[i + 1] -= 1;
                if !left_edge_wait {
                    self.waits_table[i - 1] += 1;
                }
                if !right_edge_wait {
                    self.waits_table[i + 2] += 1;
                }
                self.remove_potential_sets(i);
                if !left_edge_wait {
                    self.waits_table[i - 1] -= 1;
                }
                if !right_edge_wait {
                    self.waits_table[i + 2] -= 1;
                }
                self.hand_table[i] += 1; self.hand_table[i + 1] += 1;
                self.partial_sets -= 1;
            }

            // Closed wait protorun
            if i < 30 && i % 10 <= 7 && self.hand_table[i + 2] != 0 {
                self.partial_sets += 1;
                self.hand_table[i] -= 1; self.hand_table[i + 2] -= 1;
                self.waits_table[i + 1] += 1;
                self.remove_potential_sets(i);
                self.waits_table[i + 1] -= 1;
                self.hand_table[i] += 1; self.hand_table[i + 2] += 1;
                self.partial_sets -= 1;
            }
        }

        // Check all alternative hand configurations
        self.remove_potential_sets(i + 1);
    }

    fn remove_completed_sets(&mut self, mut i: usize) {
        // Skip to the next tile that exists in the hand.
        while i < self.hand_table.len() && self.hand_table[i] == 0 { i += 1; }

        if i >= self.hand_table.len() {
            // We've gone through the whole hand, now check for partial sets.
            self.remove_potential_sets(0);
            return;
        }

        // Triplet
        if self.hand_table[i] >= 3 {
            self.complete_sets += 1;
            self.hand_table[i] -= 3;
            self.remove_completed_sets(i);
            self.hand_table[i] += 3;
            self.complete_sets -= 1;
        }

        // Sequence
        if i < 30 && self.hand_table[i + 1] != 0 && self.hand_table[i + 2] != 0 {
            self.complete_sets += 1;
            self.hand_table[i] -= 1; self.hand_table[i + 1] -= 1; self.hand_table[i + 2] -= 1;
            self.remove_completed_sets(i);
            self.hand_table[i] += 1; self.hand_table[i + 1] += 1; self.hand_table[i + 2] += 1;
            self.complete_sets -= 1;
        }

        // Check all alternative hand configurations
        self.remove_completed_sets(i + 1);
    }

    fn calculate_shanten(&mut self) {
        for i in 0..self.hand_table.len() {
            if self.hand_table[i] >= 2 {
                self.pair += 1;
                self.hand_table[i] -= 2;
                self.waits_table[i] += 1;
                self.remove_completed_sets(0);
                self.waits_table[i] -= 1;
                self.hand_table[i] += 2;
                self.pair -= 1;
            }
        }

        self.remove_completed_sets(0);
    }

    pub fn get_calculated_shanten(&self) -> i8 {
        return self.best_shanten;
    }
}

fn calculate_shanten(tiles: &[Tile]) -> ShantenCalculator {
    let mut calculator = ShantenCalculator{
        hand_table: make_frequency_table(&tiles),
        waits_table: EMPTY_FREQUENCY_TABLE,
        complete_sets: 0,
        pair: 0,
        partial_sets: 0,
        best_shanten: MAX_SHANTEN,
        best_waits: EMPTY_FREQUENCY_TABLE,
    };
    calculator.calculate_shanten();
    return calculator;
}

fn draw_tile_to_hand(game: &mut GameState, hand_index: usize) {
    game.hands[hand_index].tiles[13] = game.live_wall.split_off(game.live_wall.len() - 1)[0];
}

fn discard_tile(game: &mut GameState, hand_index: usize, tile_index: usize) -> Tile {
    let discarded_tile = game.hands[hand_index].tiles[tile_index];
    game.hands[hand_index].tiles[tile_index..14].rotate_left(1);
    game.hands[hand_index].tiles[13] = EMPTY_TILE;
    // we sort in the tile that was added the last
    sort_hand(&mut game.hands[hand_index]);

    game.total_discards_table[get_tile_index(&discarded_tile)] += 1;
    game.discards[hand_index].push(discarded_tile);
    return discarded_tile;
}

fn get_tile_from_input(input: &str) -> Tile {
    if input.len() != 2 {
        return EMPTY_TILE;
    }

    let suit = get_suit_from_letter(input.chars().nth(1).unwrap());
    if suit.is_none() {
        return EMPTY_TILE;
    }

    let value: u8 = input[0..1].parse().unwrap_or(255);
    if value == 255 {
        return EMPTY_TILE;
    }

    return Tile{suit:suit.unwrap(), value:value};
}

fn get_discards_reducing_shanten(tiles: &[Tile], current_shanten: i8) -> Vec<Tile> {
    assert!(tiles.len() == 14, "get_discards_reducing_shanten is expected to be called on 14 tiles");

    let mut result = Vec::with_capacity(tiles.len());
    let mut previous_tile = EMPTY_TILE;

    for i in 0..14 {
        if tiles[i] == previous_tile {
            continue;
        }
        let mut reduced_tiles = tiles.to_vec();
        reduced_tiles.remove(i);

        let calculator = calculate_shanten(&reduced_tiles);

        if calculator.get_calculated_shanten() < current_shanten {
            result.push(tiles[i]);
        }
        previous_tile = tiles[i];
    }

    return result;
}

fn find_potentially_available_tile_count(game: &GameState, visible_hand_index: usize, tiles: &[Tile]) -> u8 {
    let mut result = 0;
    let mut previous_tile = EMPTY_TILE;
    for tile in tiles {
        if *tile == previous_tile {
            continue;
        }

        result += 4
                -(game.total_discards_table[get_tile_index(tile)] as u8)
                -(game.hands[visible_hand_index].tiles.iter().filter(|&t| *t == *tile).count() as u8)
                -(if game.dora_indicators[0] == *tile {1} else {0});
        previous_tile = *tile;
    }

    return result;
}

fn filter_tiles_improving_shanten(hand_tiles: &[Tile], tiles: &[Tile], current_shanten: i8) -> Vec<Tile> {
    assert!(hand_tiles.len() == 13, "filter_tiles_improving_shanten is expected to be called on 13 tiles");

    let mut extended_hand = [hand_tiles.to_vec(), [EMPTY_TILE].to_vec()].concat();

    let mut result = Vec::new();

    for i in 0..tiles.len() {
        extended_hand[13] = tiles[i];

        let calculator = calculate_shanten(&extended_hand);

        if calculator.get_calculated_shanten() < current_shanten {
            result.push(tiles[i]);
        }
    }

    return result;
}

fn filter_tiles_finishing_hand(hand_tiles: &[Tile], tiles: &[Tile]) -> Vec<Tile> {
    assert!(hand_tiles.len() == 13, "filter_tiles_improving_shanten is expected to be called on 13 tiles");

    let mut extended_hand = [hand_tiles.to_vec(), [EMPTY_TILE].to_vec()].concat();

    let mut result = Vec::new();

    for i in 0..tiles.len() {
        extended_hand[13] = tiles[i];

        let calculator = calculate_shanten(&extended_hand);

        if calculator.get_calculated_shanten() < 0 {
            result.push(tiles[i]);
        }
    }

    return result;
}

struct WeightedDiscard {
    tile: Tile,
    tiles_improving_shanten: Vec<Tile>,
    improvement_tile_count: u8,
}

fn calculate_best_discards(game: &GameState, hand_index: usize, minimal_shanten: i8) -> Vec<WeightedDiscard> {
    assert!(game.hands[hand_index].tiles[13] != EMPTY_TILE, "calculate_best_discards expected hand with 14 tiles");

    let mut possible_discards = Vec::with_capacity(14);
    let mut previous_tile = EMPTY_TILE;

    for i in 0..14 {
        if game.hands[hand_index].tiles[i] == previous_tile {
            continue;
        }
        let mut reduced_tiles = game.hands[hand_index].tiles.to_vec();
        reduced_tiles.remove(i);

        let calculator = calculate_shanten(&reduced_tiles);

        if calculator.get_calculated_shanten() == minimal_shanten {
            let tiles_improving_shanten = filter_tiles_improving_shanten(&reduced_tiles, &convert_frequency_table_to_flat_vec(&calculator.best_waits), minimal_shanten);
            let available_tiles = find_potentially_available_tile_count(&game, 0, &tiles_improving_shanten);
            possible_discards.push(WeightedDiscard{
                tile: game.hands[hand_index].tiles[i],
                tiles_improving_shanten: tiles_improving_shanten,
                improvement_tile_count: available_tiles,
            });
        }

        previous_tile = game.hands[hand_index].tiles[i];
    }

    possible_discards.sort_by(|a: &WeightedDiscard, b: &WeightedDiscard| {
        if a.improvement_tile_count > b.improvement_tile_count
        {
            std::cmp::Ordering::Less
        }
        else if a.improvement_tile_count < b.improvement_tile_count
        {
            std::cmp::Ordering::Greater
        }
        else {
            std::cmp::Ordering::Equal
        }
    });

    return possible_discards;
}

#[derive(Clone)]
struct DiscardScores {
    tiles: Vec<Tile>,
    improvement_tile_count: u8,
}

fn get_best_discard_scores(game: &GameState, hand_index: usize, minimal_shanten: i8) -> DiscardScores {
    let best_discards = calculate_best_discards(&game, hand_index, minimal_shanten);
    if !best_discards.is_empty() {
        let mut result = DiscardScores{ tiles: Vec::new(), improvement_tile_count: best_discards[0].improvement_tile_count };
        for tile_info in best_discards {
            if tile_info.improvement_tile_count < result.improvement_tile_count {
                break;
            }

            result.tiles.push(tile_info.tile);
        }
        return result;
    }

    return DiscardScores{ tiles: Vec::new(), improvement_tile_count: 0 };
}

struct PreviousMoveData {
    game_state: GameState,
    hand_index: usize,
    full_hand_shanten: i8,
    discarded_tile: Tile,
}

struct UserSettings {
    tile_display: TileDisplayOption,
}

fn get_default_settings() -> UserSettings {
    UserSettings{
        tile_display: TileDisplayOption::Text,
    }
}

fn get_move_explanation_text(previous_move: &PreviousMoveData, user_settings: &UserSettings) -> String {
    assert!(previous_move.game_state.hands[previous_move.hand_index].tiles[13] != EMPTY_TILE, "Expected move state hand have 14 tiles before the discard");

    let best_discards = calculate_best_discards(&previous_move.game_state, previous_move.hand_index, previous_move.full_hand_shanten);

    let mut result = String::new();
    for discard_info in best_discards {
        result += &format!("{}: {} ({} left)\n",
            tile_to_string(&discard_info.tile, &user_settings.tile_display),
            get_printable_tiles_set(&discard_info.tiles_improving_shanten, &user_settings.tile_display),
            discard_info.improvement_tile_count,
        )
    }
    return result;
}

fn has_furiten_waits(waits: &Vec<Tile>, discards: &Vec<Tile>) -> bool {
    let discards_table = make_frequency_table(discards);
    for tile in waits {
        if discards_table[get_tile_index(&tile)] > 0 {
            return true;
        }
    }

    return false;
}

fn has_potential_for_furiten(waits_table: &TileFrequencyTable, discards: &Vec<Tile>) -> bool {
    for tile in discards {
        if waits_table[get_tile_index(&tile)] > 1 {
            return true;
        }
    }

    return false;
}

fn play_in_console() {
    let mut should_quit_game = false;
    let mut predefined_hand: String = "".to_string();

    while !should_quit_game {
        let mut should_restart_game = false;
        let mut game = generate_dealed_game_with_hand(1, &predefined_hand, false);

        println!("Dealed hand: {}", get_printable_tiles_set(&game.hands[0].tiles, &TileDisplayOption::Text));

        while !should_restart_game && !should_quit_game && !game.live_wall.is_empty() {
            let full_hand_shanten;
            if game.hands[0].tiles[13] == EMPTY_TILE {
                let shanten_calculator = calculate_shanten(&game.hands[0].tiles[0..13]);
                let shanten = shanten_calculator.get_calculated_shanten();
                if shanten > 0 {
                    println!("Shanten: {}", shanten);
                    let tiles_improving_shanten = filter_tiles_improving_shanten(&game.hands[0].tiles[0..13], &convert_frequency_table_to_flat_vec(&shanten_calculator.best_waits), shanten);
                    println!("Tiles that can improve shanten: {}, total {} tiles", get_printable_tiles_set(&tiles_improving_shanten, &TileDisplayOption::Text), find_potentially_available_tile_count(&game, 0, &tiles_improving_shanten));
                }
                else {
                    println!("The hand is tenpai (ready) now");
                    println!("Waits: {}", get_printable_tiles_set(&filter_tiles_finishing_hand(&game.hands[0].tiles[0..13], &convert_frequency_table_to_flat_vec(&shanten_calculator.best_waits)), &TileDisplayOption::Text));
                }

                draw_tile_to_hand(&mut game, 0);
                println!("Drawn {}", tile_to_string(&game.hands[0].tiles[13], &TileDisplayOption::Text));
                println!("{} tiles left in the live wall", game.live_wall.len());
                full_hand_shanten = calculate_shanten(&game.hands[0].tiles).get_calculated_shanten();
                if full_hand_shanten < shanten {
                    println!("Discards that reduce shanten: {}", get_printable_tiles_set(&get_discards_reducing_shanten(&game.hands[0].tiles, shanten), &TileDisplayOption::Text));
                }
            }
            else {
                full_hand_shanten = calculate_shanten(&game.hands[0].tiles).get_calculated_shanten();
            }

            print_hand(&game.hands[0], &TileDisplayOption::Text);
            print_hand(&game.hands[0], &TileDisplayOption::Unicode);

            if full_hand_shanten < 0 {
                println!("The hand is complete! Send \"n\" to start new hand, \"q\" to quit, or you can continue discarding tiles");
            }
            else {
                println!("Send tile to discard, e.g. \"1m\", \"n\" to start new hand, \"q\" to quit");
            }

            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_input) => {},
                Err(_err) => {},
            }
            input = input.trim().strip_suffix("\r\n").or(input.strip_suffix("\n")).unwrap_or(&input).to_string();

            if input.starts_with("n") {
                should_restart_game = true;
                predefined_hand = "".to_string();
                println!("Dealing a new hand");

                if input.len() > 2 {
                    predefined_hand = input[2..].to_string();
                }
            }
            else if input == "q" {
                should_quit_game = true;
                println!("Quitting");
            }
            else {
                let requested_tile = get_tile_from_input(&input);
                if requested_tile == EMPTY_TILE {
                    println!("Entered string doesn't seem to be a tile representation, tile should be a digit followed by 'm', 'p', 's', or 'z', e.g. \"3s\"");
                }
                match game.hands[0].tiles.iter().position(|&r| r == requested_tile) {
                    Some(tile_index_in_hand) => { discard_tile(&mut game, 0, tile_index_in_hand as usize); },
                    None => { println!("Could not find the given tile in the hand"); },
                }
            }
        }
    }
}

fn read_telegram_token() -> String {
    return fs::read_to_string("./telegramApiToken.txt")
        .expect("Can't read file \"telegramApiToken.txt\", please make sure the file exists and contains the bot API Token");
}

struct UserState {
    game_state: Option<GameState>,
    previous_move: Option<PreviousMoveData>,
    settings: UserSettings,
}

fn process_user_message(user_state: &mut UserState, message: &Message) -> Vec<String> {
    if message.text().is_none() {
        return ["No message received".to_string()].to_vec();
    }

    let message_text = &message.text().unwrap();
    let mut answer: String = String::new();
    let mut settings = &mut user_state.settings;

    match *message_text {
        "/start" => {
            user_state.game_state = Some(generate_normal_dealed_game(1, true));
            let game_state = &user_state.game_state.as_ref().unwrap();
            return ["Dealed new hand:\n".to_string() + &get_printable_tiles_set(&game_state.hands[0].tiles, &settings.tile_display)].to_vec();
        },
        "/hand" => {
            if user_state.game_state.is_none() {
                return ["No hand is in progress, send /start to start a new hand".to_string()].to_vec();
            }
            let game_state = &user_state.game_state.as_ref().unwrap();
            return ["Current hand:\n".to_string() + &get_printable_tiles_set(&game_state.hands[0].tiles, &settings.tile_display)].to_vec();
        },
        "/explain" => {
            return match &user_state.previous_move {
                Some(previous_move) => [get_move_explanation_text(&previous_move, &settings)].to_vec(),
                None => ["No moves are recorded to explain".to_string()].to_vec(),
            }
        },
        "/display_unicode" => {
            settings.tile_display = TileDisplayOption::Unicode;
            return ["Set display style to unicode".to_string()].to_vec()
        },
        "/display_text" => {
            settings.tile_display = TileDisplayOption::Text;
            return ["Set display style to text".to_string()].to_vec()
        },
        _ => {},
    }

    if user_state.game_state.is_none() {
        return ["No hand is in progress, send /start to start a new hand".to_string()].to_vec();
    }

    let mut game_state = user_state.game_state.as_mut().unwrap();

    let requested_tile = get_tile_from_input(message_text);
    if requested_tile == EMPTY_TILE {
        return ["Entered string doesn't seem to be a tile representation, tile should be a digit followed by 'm', 'p', 's', or 'z', e.g. \"3s\"".to_string()].to_vec();
    }

    let full_hand_shanten = calculate_shanten(&game_state.hands[0].tiles).get_calculated_shanten();
    let best_discards = get_best_discard_scores(&game_state, 0, full_hand_shanten);
    let mut discarded_tile = None;

    match game_state.hands[0].tiles.iter().position(|&r| r == requested_tile) {
        Some(tile_index_in_hand) => {
            user_state.previous_move = Some(PreviousMoveData{ game_state: (*game_state).clone(), hand_index: 0, full_hand_shanten: full_hand_shanten, discarded_tile: EMPTY_TILE });
            let tile = discard_tile(&mut game_state, 0, tile_index_in_hand as usize);
            discarded_tile = Some(tile);
            user_state.previous_move.as_mut().unwrap().discarded_tile = tile;
            let shanten_calculator = calculate_shanten(&game_state.hands[0].tiles[0..13]);
            let new_shanten = shanten_calculator.get_calculated_shanten();
            if new_shanten > 0 {
                let possible_waits = convert_frequency_table_to_flat_vec(&shanten_calculator.best_waits);
                let tiles_improving_shanten = filter_tiles_improving_shanten(&game_state.hands[0].tiles[0..13], &possible_waits, new_shanten);
                answer += &format!("Discarded tile {} ({} tiles)\n", tile_to_string(&tile, &settings.tile_display), find_potentially_available_tile_count(&game_state, 0, &tiles_improving_shanten));
                if has_potential_for_furiten(&shanten_calculator.best_waits, &game_state.discards[0]) {
                    answer += "Possible furiten\n";
                }
            }
            else {
                answer += "The hand is ready now\n";
                let wait_tiles = filter_tiles_finishing_hand(&game_state.hands[0].tiles[0..13], &convert_frequency_table_to_flat_vec(&shanten_calculator.best_waits));
                answer += &format!("Waits: {} ({} tiles)", get_printable_tiles_set(&wait_tiles, &settings.tile_display), find_potentially_available_tile_count(&game_state, 0, &wait_tiles));
                if has_furiten_waits(&wait_tiles, &game_state.discards[0]) {
                    answer += " furiten";
                }
                answer += "\n";
            }
        },
        None => { answer += "Could not find the given tile in the hand\n"; },
    }

    if game_state.hands[0].tiles[13] == EMPTY_TILE {
        let shanten_calculator = calculate_shanten(&game_state.hands[0].tiles[0..13]);
        let shanten = shanten_calculator.get_calculated_shanten();
        match discarded_tile {
            Some(tile) => {
                if shanten > 0 {
                    if shanten > full_hand_shanten {
                        answer += "Went back in shanten\n";
                    } else {
                        if best_discards.tiles.contains(&tile) {
                            answer += "Best discard\n"
                        }
                        else {
                            answer += &format!("Better discards: {} ({} tiles)\n", get_printable_tiles_set(&best_discards.tiles, &settings.tile_display), best_discards.improvement_tile_count);
                        }
                    }
                }
                else {
                    if best_discards.tiles.contains(&tile) {
                        answer += "Best discard\n"
                    }
                    else {
                        answer += &format!("Better discards: {} ({} tiles)\n", get_printable_tiles_set(&best_discards.tiles, &settings.tile_display), best_discards.improvement_tile_count);
                    }

                    user_state.game_state = Some(generate_normal_dealed_game(1, true));
                    let game_state = user_state.game_state.as_ref().unwrap();
                    return [answer, "New hand:\n".to_string() + &get_printable_tiles_set(&game_state.hands[0].tiles, &settings.tile_display)].to_vec();
                }
            },
            None => panic!("We got 13 tiles but nothing discarded, that is broken"),
        }

        draw_tile_to_hand(&mut game_state, 0);
        answer += &format!("Drawn {}\n", tile_to_string(&game_state.hands[0].tiles[13], &settings.tile_display));
        answer += &format!("{} tiles left in the live wall\n", game_state.live_wall.len());
    }

    answer += &get_printable_tiles_set(&game_state.hands[0].tiles, &settings.tile_display);

    return [answer].to_vec();
}

async fn run_telegram_bot() {
    pretty_env_logger::init();
    log::info!("Starting the bot");

    let token = read_telegram_token();

    let bot = Bot::new(token);

    type UserStates = DashMap<ChatId, UserState>;
    type SharedUserStates = Arc<UserStates>;

    let user_states = SharedUserStates::new(UserStates::new());

    let handler = Update::filter_message().endpoint(
        |bot: Bot, user_states: SharedUserStates, message: Message| async move {
            let user_state: &mut UserState = &mut user_states.entry(message.chat.id).or_insert_with(||UserState{game_state: None, previous_move: None, settings: get_default_settings()});
            let responses = process_user_message(user_state, &message);
            for response in responses {
                bot.send_message(message.chat.id, response).await?;
            }
            respond(())
        },
    );

    Dispatcher::builder(bot, handler)
        // Pass the shared state to the handler as a dependency.
        .dependencies(dptree::deps![user_states.clone()])
        .build()
        .dispatch()
        .await;
}

#[tokio::main]
async fn main() {
    let args = env::args();
    if args.len() > 1 {
        play_in_console();
    }
    else {
        run_telegram_bot().await;
    }
}
