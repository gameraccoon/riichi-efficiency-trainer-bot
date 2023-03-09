extern crate rand;

use crate::rand::prelude::SliceRandom;
use dashmap::DashMap;
use rand::thread_rng;
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use std::cmp::max;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use teloxide::prelude::*;


const TILE_UNICODE: [&str; 37] = [
    "🀇", "🀈", "🀉", "🀊", "🀋", "🀌", "🀍", "🀎", "🀏", "err",
    "🀙", "🀚", "🀛", "🀜", "🀝", "🀞", "🀟", "🀠", "🀡", "err",
    "🀐", "🀑", "🀒", "🀓", "🀔", "🀕", "🀖", "🀗", "🀘", "err",
    "🀀", "🀁", "🀂", "🀃", "🀆", "🀅", "🀄"
];

const TILE_ENGLISH: [&str; 37] = [
    "one of man", "two of man", "three of man", "four of man", "five of man", "six of man", "seven of man", "eight of man", "nine of man", "err",
    "one of pin", "two of pin", "three of pin", "four of pin", "five of pin", "six of pin", "seven of pin", "eight of pin", "nine of pin", "err",
    "one of sou", "two of sou", "three of sou", "four of sou", "five of sou", "six of sou", "seven of sou", "eight of sou", "nine of sou", "err",
    "east wind", "south wind", "west wind", "north wind", "white dragon", "green dragon", "red dragon"
];

const TILE_JAPANESE: [&str; 37] = [
    "ii wan", "ryan wan", "san wan", "suu wan", "uu wan", "rou wan", "chii wan", "paa wan", "kyuu wan", "err",
    "ii pin", "ryan pin", "san pin", "suu pin", "uu pin", "rou pin", "chii pin", "paa pin", "kyuu pin", "err",
    "ii sou", "ryan sou", "san sou", "suu sou", "uu sou", "rou sou", "chii sou", "paa sou", "kyuu sou", "err",
    "ton", "nan", "shaa", "pei", "haku", "hatsu", "chun"
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
    opened_dora_indicators: u8,
    live_wall: Vec<Tile>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
enum TileDisplayOption {
    Text,
    Unicode,
    UrlImage,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
enum TermsDisplayOption {
    EnglishTerms,
    JapaneseTerms,
}

struct PreviousMoveData {
    game_state: GameState,
    hand_index: usize,
    full_hand_shanten: i8,
    discarded_tile: Tile,
}

#[derive(Clone, Serialize, Deserialize)]
struct UserSettings {
    tile_display: TileDisplayOption,
    terms_display: TermsDisplayOption,
    language_key: String,
    allow_kokushi: bool,
    allow_chiitoitsu: bool,
}

type Translations = HashMap<String, HashMap<&'static str, &'static str>>;

fn translate(key: &str, translations: &Translations, user_settings: &UserSettings) -> &'static str {
    return translations[&user_settings.language_key][key];
}

fn print_hand(hand: &Hand, tile_display: TileDisplayOption) {
    println!("{}", get_printable_tiles_set(&hand.tiles, tile_display));
}

fn get_printable_hand(hand: &Hand, tile_display: TileDisplayOption) -> String {
    get_printable_tiles_set_main(&hand.tiles, tile_display)
}

fn get_printable_tiles_set_main(tiles: &[Tile], tile_display: TileDisplayOption) -> String {
    match tile_display {
        TileDisplayOption::Text => get_printable_tiles_set_text(&tiles),
        TileDisplayOption::Unicode => get_printable_tiles_set_unicode(&tiles),
        TileDisplayOption::UrlImage => get_printable_tiles_set_url_image(&tiles),
    }
}

fn get_printable_tiles_set(tiles: &[Tile], tile_display: TileDisplayOption) -> String {
    match tile_display {
        TileDisplayOption::Text => get_printable_tiles_set_text(&tiles),
        TileDisplayOption::Unicode => get_printable_tiles_set_unicode(&tiles),
        TileDisplayOption::UrlImage => get_printable_tiles_set_text(&tiles),
    }
}

fn get_printable_tiles_set_text(tiles: &[Tile]) -> String {
    let mut result: String = "".to_string();

    if tiles.is_empty() {
        return "".to_string();
    }

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

fn get_printable_tiles_set_url_image(tiles: &[Tile]) -> String {
    if tiles.is_empty() {
        return "".to_string();
    }

    return "https://api.tempai.net/image/".to_string() + &get_printable_tiles_set_text(&tiles) + ".png";
}

fn tile_to_string(tile: &Tile, terms_display: TermsDisplayOption) -> &'static str {
    match terms_display {
        TermsDisplayOption::EnglishTerms => TILE_ENGLISH[get_tile_index(&tile)],
        TermsDisplayOption::JapaneseTerms => TILE_JAPANESE[get_tile_index(&tile)],
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
        opened_dora_indicators: 1,
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
        return EMPTY_HAND;
    }

    let tiles_count = hand_string.chars().filter(|c| c.is_numeric()).count();
    if tiles_count < 13 || tiles_count > 14 {
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

fn generate_dealed_game_with_hand(player_count: u32, hand_string: &str, deal_first_tile: bool) -> Option<GameState> {
    let predefined_hand = make_hand_from_string(&hand_string);

    if predefined_hand.tiles[0] == EMPTY_TILE {
        return None;
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
        opened_dora_indicators: 1,
        live_wall: tiles,
    };

    if game_state.hands[0].tiles[13] == EMPTY_TILE && deal_first_tile {
        draw_tile_to_hand(&mut game_state, 0);
    }

    return Some(game_state);
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
                self.best_waits = EMPTY_FREQUENCY_TABLE;
                append_frequency_table(&mut self.best_waits, &self.waits_table);
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

    fn calculate_shanten_standard(&mut self) {
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

    fn calculate_shanten_chiitoitsu(&mut self) {
        let mut pair_count = 0;
        let mut unique_tile_count = 0;

        for i in 0..self.hand_table.len() {
            if self.hand_table[i] == 0 { continue };

            unique_tile_count += 1;

            if self.hand_table[i] >= 2 {
                pair_count += 1;
            }
            else {
                self.waits_table[i] += 2;
            }
        }

        let mut shanten = 6 - pair_count;

        if unique_tile_count < 7 {
            shanten += 7 - unique_tile_count;
        }

        if shanten < self.best_shanten {
            self.best_shanten = shanten;
            self.best_waits = EMPTY_FREQUENCY_TABLE;
            append_frequency_table(&mut self.best_waits, &self.waits_table);
        }
        else if shanten == self.best_shanten {
            append_frequency_table(&mut self.best_waits, &self.waits_table);
        }
        self.waits_table = EMPTY_FREQUENCY_TABLE;
    }

    fn calculate_shanten_kokushi(&mut self) -> i8 {
        let mut unique_tile_count = 0;
        let mut have_pair = false;

        const TERMINALS_AND_HONORS_IDX: [usize; 13] = [0, 8, 10, 18, 20, 28, 30, 31, 32, 33, 34, 35, 36];

        for i in TERMINALS_AND_HONORS_IDX {
            if self.hand_table[i] != 0 {
                unique_tile_count += 1;

                if self.hand_table[i] >= 2 {
                    have_pair = true;
                }
                else {
                    self.waits_table[i] += 1;
                }
            }
            else {
                self.waits_table[i] += 2;
            }
        }

        let shanten = 13 - unique_tile_count - (have_pair as i8);

        if shanten < self.best_shanten {
            self.best_shanten = shanten;
            self.best_waits = EMPTY_FREQUENCY_TABLE;

            if have_pair {
                // eliminate improvements to make a pair
                for i in TERMINALS_AND_HONORS_IDX {
                    if self.waits_table[i] == 1 {
                        self.waits_table[i] = 0;
                    }
                }
            }

            append_frequency_table(&mut self.best_waits, &self.waits_table);
        }
        else if shanten == self.best_shanten {
            append_frequency_table(&mut self.best_waits, &self.waits_table);
        }
        self.waits_table = EMPTY_FREQUENCY_TABLE;

        return shanten;
    }

    pub fn get_calculated_shanten(&self) -> i8 {
        return self.best_shanten;
    }
}

fn calculate_shanten(tiles: &[Tile], settings: &UserSettings) -> ShantenCalculator {
    let mut calculator = ShantenCalculator{
        hand_table: make_frequency_table(&tiles),
        waits_table: EMPTY_FREQUENCY_TABLE,
        complete_sets: 0,
        pair: 0,
        partial_sets: 0,
        best_shanten: MAX_SHANTEN,
        best_waits: EMPTY_FREQUENCY_TABLE,
    };

    if settings.allow_chiitoitsu {
        calculator.calculate_shanten_chiitoitsu();

        if calculator.get_calculated_shanten() < 0 {
            return calculator;
        }
    }

    if settings.allow_kokushi {
        let shanten_kokushi = calculator.calculate_shanten_kokushi();

        // if a hand has a kokushi shanten of 3 or less, it cannot possibly be closer to a standard hand
        if shanten_kokushi < 3 {
            return calculator;
        }
    }

    calculator.calculate_shanten_standard();

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
    // this is very stupid but it seems Rust doesn't yet support initializing constants like this
    let tile_names: HashMap<&str, Tile> = HashMap::from([
        ("east", Tile{suit: Suit::Special, value: 1}),
        ("south", Tile{suit: Suit::Special, value: 2}),
        ("west", Tile{suit: Suit::Special, value: 3}),
        ("north", Tile{suit: Suit::Special, value: 4}),
        ("white", Tile{suit: Suit::Special, value: 5}),
        ("green", Tile{suit: Suit::Special, value: 6}),
        ("red", Tile{suit: Suit::Special, value: 7}),
        ("ton", Tile{suit: Suit::Special, value: 1}),
        ("nan", Tile{suit: Suit::Special, value: 2}),
        ("shaa", Tile{suit: Suit::Special, value: 3}),
        ("pei", Tile{suit: Suit::Special, value: 4}),
        ("haku", Tile{suit: Suit::Special, value: 5}),
        ("hatsu", Tile{suit: Suit::Special, value: 6}),
        ("chun", Tile{suit: Suit::Special, value: 7}),
    ]);

    let suit_names: HashMap<&str, Suit> = HashMap::from([
        ("man", Suit::Man),
        ("wan", Suit::Man),
        ("pin", Suit::Pin),
        ("sou", Suit::Sou),
    ]);

    let values_names: HashMap<&str, u8> = HashMap::from([
        ("1", 1),
        ("2", 2),
        ("3", 3),
        ("4", 4),
        ("5", 5),
        ("6", 6),
        ("7", 7),
        ("8", 8),
        ("9", 9),
        ("one", 1),
        ("two", 2),
        ("three", 3),
        ("four", 4),
        ("five", 5),
        ("six", 6),
        ("seven", 7),
        ("eight", 8),
        ("nine", 9),
        ("ii", 1),
        ("ryan", 2),
        ("san", 3),
        ("suu", 4),
        ("uu", 5),
        ("rou", 6),
        ("chii", 7),
        ("paa", 8),
        ("kyuu", 9),
        ("ichi", 1),
        ("ni", 2),
        ("yon", 4),
        ("go", 5),
        ("roku", 6),
        ("nana", 7),
        ("hachi", 8),
        ("kyu", 9),
    ]);

    if input.len() == 2 {
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
    else if input.contains(" ") {
        let mut parts = Vec::with_capacity(2);
        let mut count = 0;
        for part in input.split_whitespace() {
            if count >= 2 {
                return EMPTY_TILE;
            }
            count += 1;

            parts.push(part.clone());
        }
        if parts.len() != 2 {
            return EMPTY_TILE;
        }

        let value: u8;
        match values_names.get(parts[0]) {
            Some(found_value) => value = *found_value,
            None => return EMPTY_TILE,
        }

        let suit: Suit;
        match suit_names.get(parts[1]) {
            Some(found_suit) => suit = *found_suit,
            None => return EMPTY_TILE,
        }

        return Tile{suit: suit, value: value};
    }

    return match tile_names.get(input) {
        Some(found_tile) => *found_tile,
        None => EMPTY_TILE,
    }
}

fn get_discards_reducing_shanten(tiles: &[Tile], current_shanten: i8, user_settings: &UserSettings) -> Vec<Tile> {
    assert!(tiles.len() == 14, "get_discards_reducing_shanten is expected to be called on 14 tiles");

    let mut result = Vec::with_capacity(tiles.len());
    let mut previous_tile = EMPTY_TILE;

    for i in 0..14 {
        if tiles[i] == previous_tile {
            continue;
        }
        let mut reduced_tiles = tiles.to_vec();
        reduced_tiles.remove(i);

        let calculator = calculate_shanten(&reduced_tiles, &user_settings);

        if calculator.get_calculated_shanten() < current_shanten {
            result.push(tiles[i]);
        }
        previous_tile = tiles[i];
    }

    return result;
}

fn get_visible_tiles(game: &GameState, visible_hand_index: usize) -> TileFrequencyTable {
    let mut result = game.total_discards_table.clone();

    for tile in game.hands[visible_hand_index].tiles {
        result[get_tile_index(&tile)] += 1;
    }

    for i in 0..game.opened_dora_indicators {
        result[get_tile_index(&game.dora_indicators[i as usize])] += 1;
    }

    return result;
}

fn find_potentially_available_tile_count(visible_tiles: &TileFrequencyTable, tiles: &[Tile]) -> u8 {
    let mut result = 0;
    let mut previous_tile = EMPTY_TILE;
    for tile in tiles {
        if *tile == previous_tile {
            continue;
        }

        result += 4 - visible_tiles[get_tile_index(tile)];
        previous_tile = *tile;
    }

    return result;
}

fn filter_tiles_improving_shanten(hand_tiles: &[Tile], tiles: &[Tile], current_shanten: i8, user_settings: &UserSettings) -> Vec<Tile> {
    assert!(hand_tiles.len() == 13, "filter_tiles_improving_shanten is expected to be called on 13 tiles");

    let mut extended_hand = [hand_tiles.to_vec(), [EMPTY_TILE].to_vec()].concat();

    let mut result = Vec::new();

    for i in 0..tiles.len() {
        extended_hand[13] = tiles[i];

        let calculator = calculate_shanten(&extended_hand, &user_settings);

        if calculator.get_calculated_shanten() < current_shanten {
            result.push(tiles[i]);
        }
    }

    return result;
}

fn filter_tiles_finishing_hand(hand_tiles: &[Tile], tiles: &[Tile], user_settings: &UserSettings) -> Vec<Tile> {
    assert!(hand_tiles.len() == 13, "filter_tiles_improving_shanten is expected to be called on 13 tiles");

    let mut extended_hand = [hand_tiles.to_vec(), [EMPTY_TILE].to_vec()].concat();

    let mut result = Vec::new();

    for i in 0..tiles.len() {
        extended_hand[13] = tiles[i];

        let calculator = calculate_shanten(&extended_hand, &user_settings);

        if calculator.get_calculated_shanten() < 0 {
            result.push(tiles[i]);
        }
    }

    return result;
}

struct WeightedDiscard {
    tile: Tile,
    tiles_improving_shanten: Vec<Tile>,
    score: u32,
}

fn sort_weighted_discards(weighted_discards: &mut [WeightedDiscard]) {
    weighted_discards.sort_by(|a: &WeightedDiscard, b: &WeightedDiscard| {
        if a.score > b.score
        {
            std::cmp::Ordering::Less
        }
        else if a.score < b.score
        {
            std::cmp::Ordering::Greater
        }
        else {
            std::cmp::Ordering::Equal
        }
    });
}

fn calculate_best_discards_ukeire1(hand_tiles: &[Tile], minimal_shanten: i8, visible_tiles: &TileFrequencyTable, user_settings: &UserSettings) -> Vec<WeightedDiscard> {
    assert!(hand_tiles[13] != EMPTY_TILE, "calculate_best_discards_ukeire1 expected hand with 14 tiles");

    let mut possible_discards = Vec::with_capacity(14);
    let mut previous_tile = EMPTY_TILE;
    let mut full_hand = hand_tiles.to_vec();
    full_hand.sort();
    let full_hand = full_hand;

    for i in 0..full_hand.len() {
        if full_hand[i] == previous_tile {
            continue;
        }
        let mut reduced_tiles = full_hand.clone();
        reduced_tiles.remove(i);

        let calculator = calculate_shanten(&reduced_tiles, &user_settings);

        if calculator.get_calculated_shanten() == minimal_shanten {
            let tiles_improving_shanten = filter_tiles_improving_shanten(&reduced_tiles, &convert_frequency_table_to_flat_vec(&calculator.best_waits), minimal_shanten, &user_settings);
            let available_tiles = find_potentially_available_tile_count(&visible_tiles, &tiles_improving_shanten);
            if available_tiles > 0 {
                possible_discards.push(WeightedDiscard{
                    tile: full_hand[i],
                    tiles_improving_shanten: tiles_improving_shanten,
                    score: available_tiles as u32,
                });
            }
        }

        previous_tile = full_hand[i];
    }

    sort_weighted_discards(&mut possible_discards);

    return possible_discards;
}

fn calculate_best_discards_ukeire2(hand_tiles: &[Tile], minimal_shanten: i8, visible_tiles: &mut TileFrequencyTable, user_settings: &UserSettings) -> Vec<WeightedDiscard> {
    assert!(hand_tiles[13] != EMPTY_TILE, "calculate_best_discards_ukeire2 expected hand with 14 tiles");

    if minimal_shanten <= 0 {
        return calculate_best_discards_ukeire1(&hand_tiles, minimal_shanten, &visible_tiles, &user_settings);
    }

    let mut possible_discards = Vec::with_capacity(14);
    let mut previous_tile = EMPTY_TILE;
    let mut full_hand = hand_tiles.to_vec();
    full_hand.sort();
    let full_hand = full_hand;

    for i in 0..full_hand.len() {
        if full_hand[i] == previous_tile {
            continue;
        }
        let mut reduced_tiles = full_hand.clone();
        reduced_tiles.remove(i);

        let calculator = calculate_shanten(&reduced_tiles, &user_settings);

        if calculator.get_calculated_shanten() == minimal_shanten {
            let tiles_improving_shanten = filter_tiles_improving_shanten(&reduced_tiles, &convert_frequency_table_to_flat_vec(&calculator.best_waits), minimal_shanten, &user_settings);
            let mut score: u32 = 0;
            for tile in &tiles_improving_shanten {
                let tile_index = get_tile_index(tile);
                let available_tiles = 4 - visible_tiles[tile_index];
                if available_tiles == 0 {
                    continue;
                }
                reduced_tiles.push(*tile);
                visible_tiles[tile_index] += 1;
                let weighted_discards = calculate_best_discards_ukeire1(&reduced_tiles, minimal_shanten - 1, visible_tiles, user_settings);
                if !weighted_discards.is_empty() {
                    score += weighted_discards[0].score * (available_tiles as u32);
                }
                visible_tiles[tile_index] -= 1;
                reduced_tiles.pop();
            }

            possible_discards.push(WeightedDiscard{
                tile: full_hand[i],
                tiles_improving_shanten: tiles_improving_shanten,
                score: score,
            });
        }

        previous_tile = full_hand[i];
    }

    sort_weighted_discards(&mut possible_discards);

    return possible_discards;
}

#[derive(Clone)]
struct DiscardScores {
    tiles: Vec<Tile>,
    score: u32,
}

fn get_best_discard_scores(best_discards: &Vec<WeightedDiscard>) -> DiscardScores {
    if !best_discards.is_empty() {
        let mut result = DiscardScores{ tiles: Vec::new(), score: best_discards[0].score };
        for tile_info in best_discards {
            if tile_info.score < result.score {
                break;
            }

            result.tiles.push(tile_info.tile);
        }
        return result;
    }

    return DiscardScores{ tiles: Vec::new(), score: 0 };
}

fn get_discard_score(best_discards: &Vec<WeightedDiscard>, tile: &Tile) -> u32 {
    if !best_discards.is_empty() {
        for tile_info in best_discards {
            if tile_info.tile == *tile {
                return tile_info.score;
            }
        }
    }

    return 0;
}

fn get_default_settings() -> UserSettings {
    UserSettings{
        tile_display: TileDisplayOption::UrlImage,
        terms_display: TermsDisplayOption::EnglishTerms,
        language_key: "ene".to_string(),
        allow_kokushi: true,
        allow_chiitoitsu: true,
    }
}

fn get_move_explanation_text(previous_move: &PreviousMoveData, user_settings: &UserSettings) -> String {
    assert!(previous_move.game_state.hands[previous_move.hand_index].tiles[13] != EMPTY_TILE, "Expected move state hand have 14 tiles before the discard");

    let mut visible_tiles = get_visible_tiles(&previous_move.game_state, previous_move.hand_index);
    let best_discards = calculate_best_discards_ukeire2(&previous_move.game_state.hands[previous_move.hand_index].tiles, previous_move.full_hand_shanten, &mut visible_tiles, &user_settings);

    let mut result = String::new();
    for discard_info in best_discards {
        let tile_string = tile_to_string(&discard_info.tile, user_settings.terms_display);
        result += &format!("{}{}: {} (score {})\n",
            tile_string.chars().nth(0).unwrap().to_uppercase(), &tile_string[1..],
            get_printable_tiles_set(&discard_info.tiles_improving_shanten, user_settings.tile_display),
            discard_info.score,
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
    let user_settings = get_default_settings();

    while !should_quit_game {
        let mut should_restart_game = false;
        let game = generate_dealed_game_with_hand(1, &predefined_hand, false);
        let mut game = game.unwrap_or(generate_normal_dealed_game(1, false));

        println!("Dealed hand: {}", get_printable_hand(&game.hands[0], TileDisplayOption::Text));

        while !should_restart_game && !should_quit_game && !game.live_wall.is_empty() {
            let full_hand_shanten;
            if game.hands[0].tiles[13] == EMPTY_TILE {
                let shanten_calculator = calculate_shanten(&game.hands[0].tiles[0..13], &user_settings);
                let shanten = shanten_calculator.get_calculated_shanten();
                if shanten > 0 {
                    println!("Shanten: {}", shanten);
                    let tiles_improving_shanten = filter_tiles_improving_shanten(&game.hands[0].tiles[0..13], &convert_frequency_table_to_flat_vec(&shanten_calculator.best_waits), shanten, &user_settings);
                    println!("Tiles that can improve shanten: {}, total {} tiles", get_printable_tiles_set(&tiles_improving_shanten, TileDisplayOption::Text), find_potentially_available_tile_count(&get_visible_tiles(&game, 0), &tiles_improving_shanten));
                }
                else {
                    println!("The hand is ready now");
                    println!("Waits: {}", get_printable_tiles_set(&filter_tiles_finishing_hand(&game.hands[0].tiles[0..13], &convert_frequency_table_to_flat_vec(&shanten_calculator.best_waits), &user_settings), TileDisplayOption::Text));
                }

                draw_tile_to_hand(&mut game, 0);
                println!("Drawn {}", tile_to_string(&game.hands[0].tiles[13], TermsDisplayOption::EnglishTerms));
                println!("{} tiles left in the live wall", game.live_wall.len());
                full_hand_shanten = calculate_shanten(&game.hands[0].tiles, &user_settings).get_calculated_shanten();
                if full_hand_shanten < shanten {
                    println!("Discards that reduce shanten: {}", get_printable_tiles_set(&get_discards_reducing_shanten(&game.hands[0].tiles, shanten, &user_settings), TileDisplayOption::Text));
                }
            }
            else {
                full_hand_shanten = calculate_shanten(&game.hands[0].tiles, &user_settings).get_calculated_shanten();
            }

            print_hand(&game.hands[0], TileDisplayOption::Text);
            print_hand(&game.hands[0], TileDisplayOption::Unicode);

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
                let requested_tile = get_tile_from_input(&input.to_lowercase());
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
    current_score: u32,
    best_score: u32,
    previous_move: Option<PreviousMoveData>,
    settings: UserSettings,
    settings_unsaved: bool,
}

impl Serialize for UserState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.settings.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for UserState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut user_state = get_default_user_state();
        user_state.settings = UserSettings::deserialize(deserializer)?;
        return Ok(user_state);
    }
}

fn get_default_user_state() -> UserState {
    UserState{
        game_state: None,
        current_score: 0,
        best_score: 0,
        previous_move: None,
        settings: get_default_settings(),
        settings_unsaved: false,
    }
}

fn start_game(user_state: &mut UserState) -> String {
    let game_state = &user_state.game_state.as_ref().unwrap();
    user_state.current_score = 0;
    user_state.best_score = 0;
    return format!("Dealed new hand:\n{}\nDora indicator: {}", &get_printable_hand(&game_state.hands[0], user_state.settings.tile_display), tile_to_string(&game_state.dora_indicators[0], user_state.settings.terms_display));
}

fn process_user_message(user_state: &mut UserState, message: &Message, translations: &Translations) -> Vec<String> {
    if message.text().is_none() {
        return ["No message received".to_string()].to_vec();
    }

    const NO_HAND_IN_PROGRESS_MESSAGE: &str = "No hand is in progress, send /start to start a new hand";
    const SETTINGS_TEXT: &str = "Choose tile display type:
/display_text - use text representation of tiles
/display_unicode - use unicode characters of mahjong tiles
/display_url - send url to image of the hand

Choose terminology:
/terms_eng - English terminology
/terms_jap - Japanese terminology

Choose rules:
/toggle_chiitoi - turn on/off counting for Chiitoitsu
/toggle_kokushi - turn on/off counting for Kokushi musou";

    let message_text = &message.text().unwrap();
    let mut answer: String = String::new();
    let mut settings = &mut user_state.settings;
    let mut message_split = message_text.split_whitespace();

    match message_split.next() {
        Some("/start") => {
            match message_split.next() {
                Some(value) => {
                    user_state.game_state = generate_dealed_game_with_hand(1, value, true);
                    if user_state.game_state.is_none() {
                        return ["Given string doesn't represent a valid hand".to_string()].to_vec();
                    }
                },
                None => {
                    user_state.game_state = Some(generate_normal_dealed_game(1, true));
                },
            }
            return [start_game(user_state)].to_vec();
        },
        Some("/hand") => {
            if user_state.game_state.is_none() {
                return [NO_HAND_IN_PROGRESS_MESSAGE.to_string()].to_vec();
            }
            let game_state = &user_state.game_state.as_ref().unwrap();
            return [format!("Current hand:\n{}", &get_printable_hand(&game_state.hands[0], settings.tile_display))].to_vec();
        },
        Some("/discards") => {
            if user_state.game_state.is_none() {
                return [NO_HAND_IN_PROGRESS_MESSAGE.to_string()].to_vec();
            }
            let game_state = &user_state.game_state.as_ref().unwrap();
            return [format!("Dora indicator: {}\nDiscards:\n{}", tile_to_string(&game_state.dora_indicators[0], settings.terms_display), &get_printable_tiles_set_main(&game_state.discards[0], settings.tile_display))].to_vec();
        },
        Some("/explain") => {
            return match &user_state.previous_move {
                Some(previous_move) => [get_move_explanation_text(&previous_move, &settings)].to_vec(),
                None => ["No moves are recorded to explain".to_string()].to_vec(),
            }
        },
        Some("/settings") => {
            return [SETTINGS_TEXT.to_string()].to_vec()
        },
        Some("/display_unicode") => {
            settings.tile_display = TileDisplayOption::Unicode;
            user_state.settings_unsaved = true;
            return ["Set display style to unicode".to_string()].to_vec()
        },
        Some("/display_text") => {
            settings.tile_display = TileDisplayOption::Text;
            user_state.settings_unsaved = true;
            return ["Set display style to text".to_string()].to_vec()
        },
        Some("/display_url") => {
            settings.tile_display = TileDisplayOption::UrlImage;
            user_state.settings_unsaved = true;
            return ["Set display style to image url".to_string()].to_vec()
        },
        Some("/terms_eng") => {
            settings.terms_display = TermsDisplayOption::EnglishTerms;
            settings.language_key = "ene".to_string();
            user_state.settings_unsaved = true;
            return ["Set terminology to English".to_string()].to_vec()
        },
        Some("/terms_jap") => {
            settings.terms_display = TermsDisplayOption::JapaneseTerms;
            settings.language_key = "enj".to_string();
            user_state.settings_unsaved = true;
            return ["Set terminology to Japanese".to_string()].to_vec()
        },
        Some("/toggle_kokushi") => {
            settings.allow_kokushi = !settings.allow_kokushi;
            user_state.settings_unsaved = true;
            return [format!("Kokushi musou is now {}counted for shanten calculation",  if settings.allow_kokushi {""} else {"not "})].to_vec()
        },
        Some("/toggle_chiitoi") => {
            settings.allow_chiitoitsu = !settings.allow_chiitoitsu;
            user_state.settings_unsaved = true;
            return [format!("Chiitoitsu is now {}counted for shanten calculation", if settings.allow_chiitoitsu {""} else {"not "})].to_vec()
        },
        Some(_) => {},
        None => {},
    }

    if user_state.game_state.is_none() {
        return [NO_HAND_IN_PROGRESS_MESSAGE.to_string()].to_vec();
    }

    let mut game_state = user_state.game_state.as_mut().unwrap();

    let requested_tile = get_tile_from_input(&message_text.to_lowercase());
    if requested_tile == EMPTY_TILE {
        return ["Entered string doesn't seem to be a tile representation, tile should be a digit followed by 'm', 'p', 's', or 'z' or a tile name (e.g. all \"7z\", \"red\", and \"chun\" are acceptable inputs for the red dragon tile)".to_string()].to_vec();
    }

    let full_hand_shanten = calculate_shanten(&game_state.hands[0].tiles, &settings).get_calculated_shanten();
    let best_discards = calculate_best_discards_ukeire2(&game_state.hands[0].tiles, full_hand_shanten, &mut get_visible_tiles(&game_state, 0), &settings);

    let best_discard_scores = get_best_discard_scores(&best_discards);
    let mut discarded_tile = None;

    match game_state.hands[0].tiles.iter().position(|&r| r == requested_tile) {
        Some(tile_index_in_hand) => {
            user_state.previous_move = Some(PreviousMoveData{ game_state: (*game_state).clone(), hand_index: 0, full_hand_shanten: full_hand_shanten, discarded_tile: EMPTY_TILE });
            let tile = discard_tile(&mut game_state, 0, tile_index_in_hand as usize);
            discarded_tile = Some(tile);
            let current_discard_score = get_discard_score(&best_discards, &tile);

            user_state.best_score += best_discard_scores.score;
            user_state.current_score += current_discard_score;
            user_state.previous_move.as_mut().unwrap().discarded_tile = tile;

            let shanten_calculator = calculate_shanten(&game_state.hands[0].tiles[0..13], &settings);
            let new_shanten = shanten_calculator.get_calculated_shanten();
            if new_shanten > 0 {
                answer += &format!("Discarded {} ({}/{})\n", tile_to_string(&tile, settings.terms_display), current_discard_score, best_discard_scores.score);
                if has_potential_for_furiten(&shanten_calculator.best_waits, &game_state.discards[0]) {
                    answer += "Possible furiten\n";
                }
            }
            else {
                answer += translate("tenpai_hand", &translations, &settings);
                answer += "\n";
                let wait_tiles = filter_tiles_finishing_hand(&game_state.hands[0].tiles[0..13], &convert_frequency_table_to_flat_vec(&shanten_calculator.best_waits), &settings);
                answer += &format!("Waits: {} ({} tiles)", get_printable_tiles_set(&wait_tiles, settings.tile_display), find_potentially_available_tile_count(&get_visible_tiles(&game_state, 0), &wait_tiles));
                if has_furiten_waits(&wait_tiles, &game_state.discards[0]) {
                    answer += " furiten";
                }
                answer += "\n";
            }
        },
        None => { answer += "Could not find the given tile in the hand\n"; },
    }

    if game_state.hands[0].tiles[13] == EMPTY_TILE {
        let shanten_calculator = calculate_shanten(&game_state.hands[0].tiles[0..13], &settings);
        let shanten = shanten_calculator.get_calculated_shanten();
        match discarded_tile {
            Some(tile) => {
                if shanten > full_hand_shanten {
                    answer += "Went back in shanten\n";
                } else {
                    if best_discard_scores.tiles.contains(&tile) {
                        answer += "Best discard\n";
                    }
                    else {
                        answer += &format!("Better discards: {}\n", get_printable_tiles_set(&best_discard_scores.tiles, settings.tile_display));
                    }
                }

                if shanten <= 0 {
                    if user_state.best_score > 0 {
                        answer += &format!("Score: {}/{} or {}%", user_state.current_score, user_state.best_score, (100.0 * (user_state.current_score as f32) / (user_state.best_score as f32)).floor());
                    }
                    else {
                        answer += &format!("Some error occured, best possible score was zero, current score: {}", user_state.current_score);
                    }
                    user_state.game_state = Some(generate_normal_dealed_game(1, true));
                    return [answer, start_game(user_state)].to_vec();
                }
            },
            None => panic!("We got 13 tiles but nothing discarded, that is broken"),
        }

        draw_tile_to_hand(&mut game_state, 0);
        answer += &format!("Drawn {}\n", tile_to_string(&game_state.hands[0].tiles[13], settings.terms_display));
        answer += &format!("{} tiles left in the live wall\n", game_state.live_wall.len());
    }

    answer += &get_printable_hand(&game_state.hands[0], settings.tile_display);

    return [answer].to_vec();
}

fn load_translations() -> Translations {
    let mut translations = HashMap::new();

    {
        translations.insert("ene".to_string(), HashMap::from([
            ("tenpai_hand", "The hand is ready now"),
        ]));
    }

    {
        translations.insert("enj".to_string(), HashMap::from([
            ("tenpai_hand", "Tenpai"),
        ]));
    }

    return translations;
}

fn load_user_states(file_path: &str) -> DashMap<ChatId, UserState> {
    if Path::new(file_path).exists() {
        let data = fs::read_to_string(file_path).expect("Can't open file");
        return serde_json::from_str(&data).expect("Can't parse user states file");
    }
    else {
        return DashMap::new();
    }
}

fn save_user_state(file_path: &str, chat_id: ChatId, user_state: &UserState) {
    fs::create_dir_all(std::path::Path::new(file_path).parent().unwrap()).expect("The directory can't be created");
    let mut hash_map: HashMap<ChatId, UserSettings>;
    if Path::new(file_path).exists() {
        let data = fs::read_to_string(file_path).expect("Can't open file");
        hash_map = serde_json::from_str(&data).expect("Can't parse user states file");
    }
    else {
        hash_map = HashMap::new();
    }

    hash_map.insert(chat_id, user_state.settings.clone());

    let data = serde_json::to_string(&hash_map).expect("Can't serialize user data");
    fs::write(file_path, data).expect("Can't write to file");
}

async fn run_telegram_bot() {
    pretty_env_logger::init();
    log::info!("Starting the bot");

    let token = read_telegram_token();

    let bot = Bot::new(token);

    let translations = load_translations();

    type UserStates = DashMap<ChatId, UserState>;
    type SharedUserStates = Arc<UserStates>;
    type SharedTranslations = Arc<Translations>;

    let user_states = SharedUserStates::new(load_user_states("./data/user_states.json"));
    let shared_translations = SharedTranslations::new(translations);

    let handler = Update::filter_message().endpoint(
        |bot: Bot, user_states: SharedUserStates, translations: SharedTranslations, message: Message| async move {
            let user_state: &mut UserState = &mut user_states.entry(message.chat.id).or_insert_with(||get_default_user_state());

            let responses = process_user_message(user_state, &message, &translations);
            if user_state.settings_unsaved {
                save_user_state("./data/user_states.json", message.chat.id, &user_state);
                user_state.settings_unsaved = false;
            }
            for response in responses {
                bot.send_message(message.chat.id, response).await?;
            }
            respond(())
        },
    );

    Dispatcher::builder(bot, handler)
        // Pass the shared state to the handler as a dependency.
        .dependencies(dptree::deps![user_states.clone(), shared_translations.clone()])
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
