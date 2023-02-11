extern crate rand;

use crate::rand::prelude::SliceRandom;
use rand::thread_rng;

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

#[derive(Clone)]
struct Hand {
    tiles: [Tile; 14],
}

struct GameState {
    hands: Vec<Hand>,
    dead_wall: [Tile; 14],
    dora_indicators: [Tile; 8], // 1-3 - dora indicators, 4-7 - uradora indicators
    live_wall: Vec<Tile>,
}

fn print_hand(hand: &Hand) {
    println!("{}", get_printable_tiles_set(&hand.tiles));
}

fn print_hand_sorted(hand: &Hand, count_in_last: bool) {
    println!("{}", get_printable_tiles_set(&get_sorted_tile_set(&hand.tiles, count_in_last)));
}

fn print_hand_unicode(hand: &Hand) {
    println!("{}", get_printable_tiles_set_unicode(&hand.tiles));
}

fn print_hand_sorted_unicode(hand: &Hand, count_in_last: bool) {
    println!("{}", get_printable_tiles_set_unicode(&get_sorted_tile_set(&hand.tiles, count_in_last)));
}

fn get_printable_tiles_set_unicode(tiles: &[Tile]) -> String {
    let mut result: String = "".to_string();

    for t in tiles {
        if t.value == 0 {
            break;
        }

        let shift = match t.suit {
            Suit::Man => 0,
            Suit::Pin => 9,
            Suit::Sou => 18,
            Suit::Special => 27,
        };

        result += TILE_UNICODE[(t.value + shift - 1) as usize];
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

fn get_printable_tiles_set(tiles: &[Tile]) -> String {
    let mut result: String = "".to_string();

    let mut last_suit = Suit::Special;
    for t in tiles {
        if t.value == 0 {
            break;
        }

        if t.suit != last_suit && result != "" {
            result += get_printable_suit(last_suit);
        }

        last_suit = t.suit;

        result += &t.value.to_string();
    }


    result += get_printable_suit(last_suit);

    return result;
}

fn get_sorted_tile_set(tiles: &[Tile], count_in_last: bool) -> Vec<Tile> {
    let mut sorted_tiles: Vec<Tile> = tiles.to_vec();

    if count_in_last {
        sorted_tiles.sort();
    }
    else {
        sorted_tiles[0..13].sort();
    }

    return sorted_tiles;
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
    // 13 tiles + 1 in hand
    // 14 tiles dead hand (1 dora indicator opened)

    let mut tiles = populate_full_set();
    tiles.shuffle(&mut thread_rng());

    let dead_wall: [Tile; 14] = tiles.split_off(tiles.len() - 14).try_into().unwrap();
     // 1-3 - dora indicators, 4-7 - uradora indicators
    let dora_indicators: [Tile; 8] = dead_wall[4..12].try_into().unwrap();
    print!("{}", get_printable_tiles_set(&dora_indicators));
    print!("{:?}", dora_indicators);

    let mut hands = Vec::with_capacity(player_count as usize);
    for _i in 0..player_count {
        hands.push(Hand{tiles: tiles.split_off(tiles.len() - 14).try_into().unwrap()});
    }

    return GameState{hands: hands, dead_wall: dead_wall, dora_indicators: dora_indicators, live_wall: tiles};
}

fn main() {
    let game = generate_normal_dealed_game(1);

    print_hand_sorted(&game.hands[0], true);
    print_hand_sorted_unicode(&game.hands[0], true);
}
