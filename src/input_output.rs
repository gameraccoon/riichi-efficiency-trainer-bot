use std::collections::HashMap;
use crate::game_logic::*;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    pub terms_display: TermsDisplayOption,
    pub language_key: String,
    pub render_size: usize,
}

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

pub fn get_printable_suit(suit: Suit, terms_display: TermsDisplayOption) -> &'static str {
    match suit {
        Suit::Man => match terms_display {
            TermsDisplayOption::EnglishTerms => "man",
            TermsDisplayOption::JapaneseTerms => "wan",
        }
        Suit::Pin => "pin",
        Suit::Sou => "sou",
        Suit::Special => "",
    }
}

pub fn get_suit_from_letter(suit_letter: char) -> Option<Suit> {
    match suit_letter {
        'm' => Some(Suit::Man),
        'p' => Some(Suit::Pin),
        's' => Some(Suit::Sou),
        'z' => Some(Suit::Special),
        _ => None,
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum TermsDisplayOption {
    EnglishTerms,
    JapaneseTerms,
}

pub fn get_capitalized(string: &str) -> String {
    string.chars().nth(0).unwrap().to_uppercase().to_string() + &string[1..]
}

pub fn get_printable_tiles_set_text(tiles: &[Tile], terms_display: TermsDisplayOption) -> String {
    let mut result: String = "".to_string();

    if tiles.is_empty() {
        return "".to_string();
    }

    let mut last_suit = Suit::Special;
    for tile in tiles {
        if tile.value == 0 {
            break;
        }

        if result != "" {
            if tile.suit != last_suit {
                if last_suit != Suit::Special {
                    result += " ";
                    result += get_printable_suit(last_suit, terms_display);
                    result += ", ";
                }
            }
            else {
                result += ", ";
            }
        }

        last_suit = tile.suit;

        if tile.suit != Suit::Special {
            result += &tile.value.to_string();
        }
        else {
            result += tile_to_string(tile, terms_display);
        }
    }

    if last_suit != Suit::Special {
        result += " ";
        result += get_printable_suit(last_suit, terms_display);
    }

    return result;
}

pub fn tile_to_string(tile: &Tile, terms_display: TermsDisplayOption) -> &'static str {
    match terms_display {
        TermsDisplayOption::EnglishTerms => TILE_ENGLISH[get_tile_index(&tile)],
        TermsDisplayOption::JapaneseTerms => TILE_JAPANESE[get_tile_index(&tile)],
    }
}

pub fn get_tile_from_input(input: &str) -> Tile {
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

pub fn make_hand_from_string(hand_string: &str) -> Hand {
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
