extern crate rand;

use crate::rand::prelude::SliceRandom;
use rand::thread_rng;


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

struct GameState {
    hands: Vec<Hand>,
    discards: Vec<Vec<Tile>>,
    dead_wall: DeadWall,
    dora_indicators: [Tile; 8], // 1-3 - dora indicators, 4-7 - uradora indicators
    live_wall: Vec<Tile>,
}

// store tiles as cumulative frequency distribution (store count of every possible tile in a hand)
type HandFrequencyTable = [u8; 37];
const EMPTY_FREQUENCY_TABLE: HandFrequencyTable = [0; 37];

fn print_hand(hand: &Hand) {
    println!("{}", get_printable_tiles_set(&hand.tiles));
}

fn print_hand_unicode(hand: &Hand) {
    println!("{}", get_printable_tiles_set_unicode(&hand.tiles));
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

fn get_printable_tiles_set(tiles: &[Tile]) -> String {
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

fn generate_normal_dealed_game(player_count: u32) -> GameState {
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

    return GameState{hands: hands, discards: discards, dead_wall: dead_wall, dora_indicators: dora_indicators, live_wall: tiles};
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

fn generate_dealed_game_with_hand(player_count: u32, hand_string: &str) -> GameState {
    println!("requested hand: {}", hand_string);

    let predefined_hand = make_hand_from_string(&hand_string);

    if predefined_hand.tiles[0] == EMPTY_TILE {
        return generate_normal_dealed_game(player_count);
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

    return GameState{hands: hands, discards: discards, dead_wall: dead_wall, dora_indicators: dora_indicators, live_wall: tiles};
}

fn make_frequency_table(tiles: &[Tile]) -> HandFrequencyTable {
    let mut result: HandFrequencyTable = EMPTY_FREQUENCY_TABLE;

    for tile in tiles {
        result[get_tile_index(tile)] += 1
    }

    return result;
}

fn convert_frequency_table_to_flat_vec(frequency_table: &HandFrequencyTable) -> Vec<Tile> {
    let mut result = Vec::new();

    for i in 0..frequency_table.len() {
        if frequency_table[i] > 0 {
            let tile = get_tile_from_index(i);
            result.push(tile);
        }
    }

    return result;
}

fn append_frequency_table(target_table: &mut HandFrequencyTable, addition_table: &HandFrequencyTable) {
    for i in 0..target_table.len() {
        target_table[i] += addition_table[i];
    }
}

pub struct ShantenCalculator {
    hand_table: HandFrequencyTable,
    waits_table: HandFrequencyTable,
    complete_sets: i8,
    pair: i8,
    partial_sets: i8,
    best_shanten: i8,
    best_waits: HandFrequencyTable,
}

impl ShantenCalculator {
    fn remove_potential_sets(&mut self, mut i: usize) {
        // Skip to the next tile that exists in the hand
        while i < self.hand_table.len() && self.hand_table[i] == 0 { i += 1; }

        if i >= self.hand_table.len() {
            // We've checked everything. See if this shanten is better than the current best.
            let current_shanten = (8 - (self.complete_sets * 2) - self.partial_sets - self.pair) as i8;
            if current_shanten < self.best_shanten {
                self.best_shanten = current_shanten;
                self.best_waits = self.waits_table.clone();
                // append single tiles that was left, as uncompleted pairs
                append_frequency_table(&mut self.best_waits, &self.hand_table);
            }
            else if current_shanten == self.best_shanten {
                append_frequency_table(&mut self.best_waits, &self.waits_table);
                // append single tiles that was left, as uncompleted pairs
                append_frequency_table(&mut self.best_waits, &self.hand_table);
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
            if i < 30 && i % 10 <= 8 && self.hand_table[i + 2] != 0 {
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
    assert!(tiles.len() == 13, "Shanten calculation alghorithm is expected to be called on 13 tiles");

    let mut calculator = ShantenCalculator{
        hand_table: make_frequency_table(&tiles),
        waits_table: EMPTY_FREQUENCY_TABLE,
        complete_sets: 0,
        pair: 0,
        partial_sets: 0,
        best_shanten: 8,
        best_waits: EMPTY_FREQUENCY_TABLE,
    };
    calculator.calculate_shanten();
    return calculator;
}

fn draw_tile_to_hand(game: &mut GameState, hand_index: usize) {
    game.hands[hand_index].tiles[13] = game.live_wall.split_off(game.live_wall.len() - 1)[0];
}

fn discard_tile(game: &mut GameState, hand_index: usize, tile_index: usize) {
    game.discards[hand_index].push(game.hands[hand_index].tiles[tile_index]);
    game.hands[hand_index].tiles[tile_index..14].rotate_left(1);
    game.hands[hand_index].tiles[13] = EMPTY_TILE;
    // we sort in the tile that was added the last
    sort_hand(&mut game.hands[hand_index]);
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

fn is_hand_complete(hand: &Hand) -> bool {
    assert!(hand.tiles[13] != EMPTY_TILE, "is_hand_complete should be called on a hand with 14 tiles");

    let shanten_calculator = calculate_shanten(&hand.tiles[0..13]);

    return shanten_calculator.get_calculated_shanten() == 0 && shanten_calculator.best_waits[get_tile_index(&hand.tiles[13])] > 0;
}

fn main() {
    let mut should_quit_game = false;
    let mut predefined_hand: String = "".to_string();

    while !should_quit_game {
        let mut should_restart_game = false;
        let mut game = generate_dealed_game_with_hand(1, &predefined_hand);

        while !should_restart_game && !should_quit_game {
            if game.hands[0].tiles[13] == EMPTY_TILE {
                let shanten_calculator = calculate_shanten(&game.hands[0].tiles[0..13]);
                let shanten = shanten_calculator.get_calculated_shanten();
                if shanten > 0 {
                    println!("Shanten: {}", shanten);
                    println!("Tiles that can improve hand: {}", get_printable_tiles_set(&convert_frequency_table_to_flat_vec(&shanten_calculator.best_waits)))
                }
                else {
                    println!("The hand is tenpai (ready) now");
                    println!("Waits: {}", get_printable_tiles_set(&convert_frequency_table_to_flat_vec(&shanten_calculator.best_waits)))
                }

                draw_tile_to_hand(&mut game, 0);
                println!("Drawn {}{}", game.hands[0].tiles[13].value.to_string(), get_printable_suit(game.hands[0].tiles[13].suit));
                println!("{} tiles left in the live wall", game.live_wall.len());
            }

            print_hand(&game.hands[0]);
            print_hand_unicode(&game.hands[0]);

            if is_hand_complete(&game.hands[0]) {
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
