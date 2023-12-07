use crate::game_logic::*;
use serde::{Deserialize, Serialize};
use std::cmp::max;

// only for tests
use crate::input_output;

#[derive(Clone, Serialize, Deserialize)]
pub struct ScoreCalculationSettings {
    pub allow_kokushi: bool,
    pub allow_chiitoitsu: bool,
}

fn set_max(element: &mut u8, value: u8) {
    *element = max(*element, value);
}

fn append_frequency_table(
    target_table: &mut TileFrequencyTable,
    addition_table: &TileFrequencyTable,
) {
    for i in 0..target_table.len() {
        if addition_table[i] > 0 {
            set_max(&mut target_table[i], 2);
        }
    }
}

pub struct ShantenCalculator {
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
        while i < self.hand_table.len() && self.hand_table[i] == 0 {
            i += 1;
        }

        if i >= self.hand_table.len() {
            // We've checked everything. See if this shanten is better than the current best.
            let current_shanten =
                (8 - (self.complete_sets * 2) - self.partial_sets - self.pair) as i8;
            if current_shanten < self.best_shanten {
                self.best_shanten = current_shanten;
                self.best_waits = EMPTY_FREQUENCY_TABLE;
                append_frequency_table(&mut self.best_waits, &self.waits_table);
                self.append_single_tiles_left();
            } else if current_shanten == self.best_shanten {
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
                self.hand_table[i] -= 1;
                self.hand_table[i + 1] -= 1;
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
                self.hand_table[i] += 1;
                self.hand_table[i + 1] += 1;
                self.partial_sets -= 1;
            }

            // Closed wait protorun
            if i < 30 && i % 10 <= 7 && self.hand_table[i + 2] != 0 {
                self.partial_sets += 1;
                self.hand_table[i] -= 1;
                self.hand_table[i + 2] -= 1;
                self.waits_table[i + 1] += 1;
                self.remove_potential_sets(i);
                self.waits_table[i + 1] -= 1;
                self.hand_table[i] += 1;
                self.hand_table[i + 2] += 1;
                self.partial_sets -= 1;
            }
        }

        // Check all alternative hand configurations
        self.remove_potential_sets(i + 1);
    }

    fn remove_completed_sets(&mut self, mut i: usize) {
        // Skip to the next tile that exists in the hand.
        while i < self.hand_table.len() && self.hand_table[i] == 0 {
            i += 1;
        }

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
            self.hand_table[i] -= 1;
            self.hand_table[i + 1] -= 1;
            self.hand_table[i + 2] -= 1;
            self.remove_completed_sets(i);
            self.hand_table[i] += 1;
            self.hand_table[i + 1] += 1;
            self.hand_table[i + 2] += 1;
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
            if self.hand_table[i] == 0 {
                continue;
            };

            unique_tile_count += 1;

            if self.hand_table[i] >= 2 {
                pair_count += 1;
            } else {
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
        } else if shanten == self.best_shanten {
            append_frequency_table(&mut self.best_waits, &self.waits_table);
        }
        self.waits_table = EMPTY_FREQUENCY_TABLE;
    }

    fn calculate_shanten_kokushi(&mut self) -> i8 {
        let mut unique_tile_count = 0;
        let mut have_pair = false;

        const TERMINALS_AND_HONORS_IDX: [usize; 13] =
            [0, 8, 10, 18, 20, 28, 30, 31, 32, 33, 34, 35, 36];

        for i in TERMINALS_AND_HONORS_IDX {
            if self.hand_table[i] != 0 {
                unique_tile_count += 1;

                if self.hand_table[i] >= 2 {
                    have_pair = true;
                } else {
                    self.waits_table[i] += 1;
                }
            } else {
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
        } else if shanten == self.best_shanten {
            append_frequency_table(&mut self.best_waits, &self.waits_table);
        }
        self.waits_table = EMPTY_FREQUENCY_TABLE;

        return shanten;
    }

    pub fn get_calculated_shanten(&self) -> i8 {
        return self.best_shanten;
    }

    pub fn get_best_waits(&self) -> &TileFrequencyTable {
        return &self.best_waits;
    }
}

pub fn calculate_shanten(tiles: &[Tile], settings: &ScoreCalculationSettings) -> ShantenCalculator {
    let mut calculator = ShantenCalculator {
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

pub fn get_visible_tiles(game: &GameState, visible_hand_index: usize) -> TileFrequencyTable {
    let mut result = game.total_discards_table.clone();

    for tile in game.hands[visible_hand_index].tiles {
        result[get_tile_index(&tile)] += 1;
    }

    for i in 0..game.opened_dora_indicators {
        result[get_tile_index(&game.dora_indicators[i as usize])] += 1;
    }

    return result;
}

pub fn find_potentially_available_tile_count(
    visible_tiles: &TileFrequencyTable,
    tiles: &[Tile],
) -> u8 {
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

pub fn filter_tiles_improving_shanten(
    hand_tiles: &[Tile],
    tiles: &[Tile],
    current_shanten: i8,
    settings: &ScoreCalculationSettings,
) -> Vec<Tile> {
    assert!(
        hand_tiles.len() == 13,
        "filter_tiles_improving_shanten is expected to be called on 13 tiles"
    );

    let mut extended_hand = [hand_tiles.to_vec(), [EMPTY_TILE].to_vec()].concat();

    let mut result = Vec::new();

    for i in 0..tiles.len() {
        extended_hand[13] = tiles[i];

        let calculator = calculate_shanten(&extended_hand, &settings);

        if calculator.get_calculated_shanten() < current_shanten {
            result.push(tiles[i]);
        }
    }

    return result;
}

pub fn filter_tiles_finishing_hand(
    hand_tiles: &[Tile],
    tiles: &[Tile],
    settings: &ScoreCalculationSettings,
) -> Vec<Tile> {
    assert!(
        hand_tiles.len() == 13,
        "filter_tiles_improving_shanten is expected to be called on 13 tiles"
    );

    let mut extended_hand = [hand_tiles.to_vec(), [EMPTY_TILE].to_vec()].concat();

    let mut result = Vec::new();

    for i in 0..tiles.len() {
        extended_hand[13] = tiles[i];

        let calculator = calculate_shanten(&extended_hand, &settings);

        if calculator.get_calculated_shanten() < 0 {
            result.push(tiles[i]);
        }
    }

    return result;
}

pub struct WeightedDiscard {
    pub tile: Tile,
    pub tiles_improving_shanten: Vec<Tile>,
    pub score: u32,
}

fn sort_weighted_discards(weighted_discards: &mut [WeightedDiscard]) {
    weighted_discards.sort_by(|a: &WeightedDiscard, b: &WeightedDiscard| {
        if a.score > b.score {
            std::cmp::Ordering::Less
        } else if a.score < b.score {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    });
}

pub fn calculate_best_discards_ukeire1(
    hand_tiles: &[Tile],
    minimal_shanten: i8,
    visible_tiles: &TileFrequencyTable,
    settings: &ScoreCalculationSettings,
) -> Vec<WeightedDiscard> {
    assert!(
        hand_tiles[13] != EMPTY_TILE,
        "calculate_best_discards_ukeire1 expected hand with 14 tiles"
    );

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

        let calculator = calculate_shanten(&reduced_tiles, &settings);

        if calculator.get_calculated_shanten() == minimal_shanten {
            let tiles_improving_shanten = filter_tiles_improving_shanten(
                &reduced_tiles,
                &convert_frequency_table_to_flat_vec(&calculator.best_waits),
                minimal_shanten,
                &settings,
            );
            let available_tiles =
                find_potentially_available_tile_count(&visible_tiles, &tiles_improving_shanten);
            if available_tiles > 0 {
                possible_discards.push(WeightedDiscard {
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

pub fn calculate_best_discards_ukeire2(
    hand_tiles: &[Tile],
    minimal_shanten: i8,
    visible_tiles: &mut TileFrequencyTable,
    settings: &ScoreCalculationSettings,
) -> Vec<WeightedDiscard> {
    assert!(
        hand_tiles[13] != EMPTY_TILE,
        "calculate_best_discards_ukeire2 expected hand with 14 tiles"
    );

    if minimal_shanten <= 0 {
        return calculate_best_discards_ukeire1(
            &hand_tiles,
            minimal_shanten,
            &visible_tiles,
            &settings,
        );
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

        let calculator = calculate_shanten(&reduced_tiles, &settings);

        if calculator.get_calculated_shanten() == minimal_shanten {
            let tiles_improving_shanten = filter_tiles_improving_shanten(
                &reduced_tiles,
                &convert_frequency_table_to_flat_vec(&calculator.best_waits),
                minimal_shanten,
                &settings,
            );
            let mut score: u32 = 0;
            for tile in &tiles_improving_shanten {
                let tile_index = get_tile_index(tile);
                let available_tiles = 4 - visible_tiles[tile_index];
                if available_tiles == 0 {
                    continue;
                }
                reduced_tiles.push(*tile);
                visible_tiles[tile_index] += 1;
                let weighted_discards = calculate_best_discards_ukeire1(
                    &reduced_tiles,
                    minimal_shanten - 1,
                    visible_tiles,
                    settings,
                );
                if !weighted_discards.is_empty() {
                    score += weighted_discards[0].score * (available_tiles as u32);
                }
                visible_tiles[tile_index] -= 1;
                reduced_tiles.pop();
            }

            possible_discards.push(WeightedDiscard {
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
pub struct DiscardScores {
    pub tiles: Vec<Tile>,
    pub score: u32,
}

pub fn get_best_discard_scores(best_discards: &Vec<WeightedDiscard>) -> DiscardScores {
    if !best_discards.is_empty() {
        let mut result = DiscardScores {
            tiles: Vec::new(),
            score: best_discards[0].score,
        };
        for tile_info in best_discards {
            if tile_info.score < result.score {
                break;
            }

            result.tiles.push(tile_info.tile);
        }
        return result;
    }

    return DiscardScores {
        tiles: Vec::new(),
        score: 0,
    };
}

pub fn get_discard_score(best_discards: &Vec<WeightedDiscard>, tile: &Tile) -> u32 {
    if !best_discards.is_empty() {
        for tile_info in best_discards {
            if tile_info.tile == *tile {
                return tile_info.score;
            }
        }
    }

    return 0;
}

pub fn has_furiten_waits(waits: &Vec<Tile>, discards: &Vec<Tile>) -> bool {
    let discards_table = make_frequency_table(discards);
    for tile in waits {
        if discards_table[get_tile_index(&tile)] > 0 {
            return true;
        }
    }

    return false;
}

pub fn has_potential_for_furiten(waits_table: &TileFrequencyTable, discards: &Vec<Tile>) -> bool {
    for tile in discards {
        if waits_table[get_tile_index(&tile)] > 1 {
            return true;
        }
    }

    return false;
}

fn get_tile_from_index(index: usize) -> Tile {
    return match index {
        i if i < (10 as usize) => Tile {
            suit: Suit::Man,
            value: (index + 1) as u8,
        },
        i if i < (20 as usize) => Tile {
            suit: Suit::Pin,
            value: (index + 1 - 10) as u8,
        },
        i if i < (30 as usize) => Tile {
            suit: Suit::Sou,
            value: (index + 1 - 20) as u8,
        },
        _ => Tile {
            suit: Suit::Special,
            value: (index + 1 - 30) as u8,
        },
    };
}

pub fn convert_frequency_table_to_flat_vec(frequency_table: &TileFrequencyTable) -> Vec<Tile> {
    let mut result = Vec::new();

    for i in 0..frequency_table.len() {
        if frequency_table[i] > 0 {
            let tile = get_tile_from_index(i);
            result.push(tile);
        }
    }

    return result;
}

pub struct PreviousMoveData {
    pub game_state: GameState,
    pub hand_index: usize,
    pub full_hand_shanten: i8,
    pub discarded_tile: Tile,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculator_defaults() {
        let calculator = ShantenCalculator {
            hand_table: EMPTY_FREQUENCY_TABLE,
            waits_table: EMPTY_FREQUENCY_TABLE,
            complete_sets: 0,
            pair: 0,
            partial_sets: 0,
            best_shanten: MAX_SHANTEN,
            best_waits: EMPTY_FREQUENCY_TABLE,
        };

        assert_eq!(calculator.hand_table, EMPTY_FREQUENCY_TABLE);
        assert_eq!(calculator.waits_table, EMPTY_FREQUENCY_TABLE);
        assert_eq!(calculator.complete_sets, 0);
        assert_eq!(calculator.pair, 0);
        assert_eq!(calculator.partial_sets, 0);
        assert_eq!(calculator.best_shanten, MAX_SHANTEN);
        assert_eq!(calculator.best_waits, EMPTY_FREQUENCY_TABLE);
    }

    #[test]
    fn test_example_hand_one_shanten() {
        let hand = input_output::make_hand_from_string("123456789m1334p");
        let calculator = calculate_shanten(
            &hand.tiles[0..13],
            &ScoreCalculationSettings {
                allow_kokushi: true,
                allow_chiitoitsu: true,
            },
        );
        assert_eq!(calculator.get_calculated_shanten(), 1);
        assert_eq!(
            convert_frequency_table_to_flat_vec(&calculator.best_waits),
            input_output::make_tile_sequence_from_string("123456p")
        );
    }

    #[test]
    fn test_example_hand_two_shanten() {
        let hand = input_output::make_hand_from_string("122456789m1369p");
        let calculator = calculate_shanten(
            &hand.tiles[0..13],
            &ScoreCalculationSettings {
                allow_kokushi: true,
                allow_chiitoitsu: true,
            },
        );
        assert_eq!(calculator.get_calculated_shanten(), 2);
        assert_eq!(
            convert_frequency_table_to_flat_vec(&calculator.best_waits),
            input_output::make_tile_sequence_from_string("1234m2456789p")
        );
    }
}
