extern crate rand;

use crate::rand::prelude::SliceRandom;
use rand::thread_rng;
use std::cmp;

const TILE_UNICODE: [&str; 34] = [
    "🀇", "🀈", "🀉", "🀊", "🀋", "🀌", "🀍", "🀎", "🀏",
    "🀙", "🀚", "🀛", "🀜", "🀝", "🀞", "🀟", "🀠", "🀡",
    "🀐", "🀑", "🀒", "🀓", "🀔", "🀕", "🀖", "🀗", "🀘",
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

struct GameState {
    hands: Vec<Hand>,
    discards: Vec<Vec<Tile>>,
    dead_wall: DeadWall,
    dora_indicators: [Tile; 8], // 1-3 - dora indicators, 4-7 - uradora indicators
    live_wall: Vec<Tile>,
}

// store tiles as cumulative frequency distribution (store count of every possible tile in a hand)
type HandFrequencyTable = [u8; 34];

fn print_hand(hand: &Hand) {
    println!("{}", get_printable_tiles_set(&hand.tiles));
}

fn print_hand_unicode(hand: &Hand) {
    println!("{}", get_printable_tiles_set_unicode(&hand.tiles));
}

fn get_tile_index(tile: &Tile) -> usize {
    let shift = match tile.suit {
        Suit::Man => 0,
        Suit::Pin => 9,
        Suit::Sou => 18,
        Suit::Special => 27,
    };

    return (tile.value + shift - 1) as usize;
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

fn make_frequency_table(tiles: &[Tile]) -> HandFrequencyTable {
    let mut result: HandFrequencyTable = [0; 34];

    for tile in tiles {
        result[get_tile_index(tile)] += 1
    }

    return result;
}

fn remove_potential_sets(hand_table: &mut HandFrequencyTable, best_shanten: &mut i32, complete_sets: &mut i32, partial_sets: &mut i32, pair: &mut i32, mut i: usize) {
    // Skip to the next tile that exists in the hand
    while i < hand_table.len() && hand_table[i] == 0 { i += 1; }

    if i >= hand_table.len() {
        // We've checked everything. See if this shanten is better than the current best.
        let current_shanten = 8 - (*complete_sets * 2) - *partial_sets - *pair;
        if current_shanten < *best_shanten {
            *best_shanten = current_shanten;
        }
        return;
    }

    // A standard hand will only ever have four groups plus a pair.
    if *complete_sets + *partial_sets < 4 {
        // Pair
        if hand_table[i] == 2 {
            *partial_sets += 1;
            hand_table[i] -= 2;
            remove_potential_sets(hand_table, best_shanten, complete_sets, partial_sets, pair, i);
            hand_table[i] += 2;
            *partial_sets -= 1;
        }

        // Edge or Side wait protorun
        if i < 30 && hand_table[i + 1] != 0 {
            *partial_sets += 1;
            hand_table[i] -= 1; hand_table[i + 1] -= 1;
            remove_potential_sets(hand_table, best_shanten, complete_sets, partial_sets, pair, i);
            hand_table[i] += 1; hand_table[i + 1] += 1;
            *partial_sets -= 1;
        }

        // Closed wait protorun
        if i < 30 && i % 10 <= 8 && hand_table[i + 2] != 0 {
            *partial_sets += 1;
            hand_table[i] -= 1; hand_table[i + 2] -= 1;
            remove_potential_sets(hand_table, best_shanten, complete_sets, partial_sets, pair, i);
            hand_table[i] += 1; hand_table[i + 2] += 1;
            *partial_sets -= 1;
        }
    }

    // Check all alternative hand configurations
    remove_potential_sets(hand_table, best_shanten, complete_sets, partial_sets, pair, i + 1);
}

fn remove_completed_sets(hand_table: &mut HandFrequencyTable, best_shanten: &mut i32, complete_sets: &mut i32, partial_sets: &mut i32, pair: &mut i32, mut i: usize) {
    // Skip to the next tile that exists in the hand.
    while i < hand_table.len() && hand_table[i] == 0 { i += 1; }

    if i >= hand_table.len() {
        // We've gone through the whole hand, now check for partial sets.
        remove_potential_sets(hand_table, best_shanten, complete_sets, partial_sets, pair, 1);
        return;
    }

    // Triplet
    if hand_table[i] >= 3 {
        *complete_sets += 1;
        hand_table[i] -= 3;
        remove_completed_sets(hand_table, best_shanten, complete_sets, partial_sets, pair, i);
        hand_table[i] += 3;
        *complete_sets -= 1;
    }

    // Sequence
    if i < 30 && hand_table[i + 1] != 0 && hand_table[i + 2] != 0 {
        *complete_sets += 1;
        hand_table[i] -= 1; hand_table[i + 1] -= 1; hand_table[i + 2] -= 1;
        remove_completed_sets(hand_table, best_shanten, complete_sets, partial_sets, pair, i);
        hand_table[i] += 1; hand_table[i + 1] += 1; hand_table[i + 2] += 1;
        *complete_sets -= 1;
    }

    // Check all alternative hand configurations
    remove_completed_sets(hand_table, best_shanten, complete_sets, partial_sets, pair, i + 1);
}

fn calculate_shanten_on_table(hand_table: &mut HandFrequencyTable) -> i32 {
    let mut complete_sets = 0;
    let mut pair = 0;
    let mut partial_sets = 0;
    let mut best_shanten = 8;

    for i in 0..hand_table.len() {
        if hand_table[i] >= 2 {
            pair += 1;
            hand_table[i] -= 2;
            remove_completed_sets(hand_table, &mut best_shanten, &mut complete_sets, &mut partial_sets, &mut pair, 1);
            hand_table[i] += 2;
            pair -= 1;
        }
    }

    remove_completed_sets(hand_table, &mut best_shanten, &mut complete_sets, &mut partial_sets, &mut pair, 1);

    return best_shanten;
}

fn calculate_shanten(tiles: &[Tile]) -> i32 {
    let mut hand_table: HandFrequencyTable = make_frequency_table(&tiles);
    return calculate_shanten_on_table(&mut hand_table);
}

fn calculate_hand_shanten(hand: &Hand) -> i32 {
    assert!(hand.tiles[13] == EMPTY_TILE, "Shanten calculation alghorithm is expected to be called on 13 tiles");
    return calculate_shanten(&hand.tiles[0..13]);
}

fn calculate_hand_shanten_14_tiles(hand: &Hand) -> i32 {
    assert!(hand.tiles[13] != EMPTY_TILE, "14 tiles shanten calculation alghorithm is expected to be called on 14 tiles");
    let mut minimal_shanten = 8;
    let mut hand_table: HandFrequencyTable = make_frequency_table(&hand.tiles);
    for tile in hand.tiles {
        let tile_index = get_tile_index(&tile);
        hand_table[tile_index] -= 1;
        minimal_shanten = cmp::min(minimal_shanten, calculate_shanten_on_table(&mut hand_table));
        hand_table[tile_index] += 1;
    }
    // one more for the discard that needs to be made
    return minimal_shanten + 1;
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

fn main() {
    let mut should_quit_game = false;

    while !should_quit_game {
        let mut should_restart_game = false;
        let mut game = generate_normal_dealed_game(1);

        while !should_restart_game && !should_quit_game {
            if game.hands[0].tiles[13] == EMPTY_TILE {
                println!("Shanten: {}", calculate_hand_shanten(&game.hands[0]));

                draw_tile_to_hand(&mut game, 0);
                println!("Drawn {}{}", game.hands[0].tiles[13].value.to_string(), get_printable_suit(game.hands[0].tiles[13].suit));
                println!("{} tiles left in the live wall", game.live_wall.len());
            }

            print_hand(&game.hands[0]);
            print_hand_unicode(&game.hands[0]);
            println!("Send tile to discard, e.g. \"1m\", \"n\" to start new hand, \"q\" to quit");

            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_input) => {},
                Err(_err) => {},
            }
            input = input.trim().strip_suffix("\r\n").or(input.strip_suffix("\n")).unwrap_or(&input).to_string();

            if input == "n" {
                should_restart_game = true;
                println!("Dealing a new hand");
            }
            else if input == "q" {
                should_quit_game = true;
                println!("Quitting");
            }
            else {
                let requested_tile = get_tile_from_input(&input);
                if requested_tile == EMPTY_TILE {
                    println!("Entered string doesn't seem to be a tile representation, tile should be a digit followed by 'm', 'p', 's', or 'z'");
                }
                match game.hands[0].tiles.iter().position(|&r| r == requested_tile) {
                    Some(tile_index_in_hand) => { discard_tile(&mut game, 0, tile_index_in_hand as usize); },
                    None => { println!("Could not find the given tile in the hand"); },
                }
            }
        }
    }
}
