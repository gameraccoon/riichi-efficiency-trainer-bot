use dashmap::DashMap;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use teloxide::prelude::*;

use crate::game_logic::*;
use crate::efficiency_calculator::*;
use crate::user_settings::*;
use crate::user_state::*;
use crate::input_output::*;
use crate::translations::*;

fn read_telegram_token() -> String {
    return fs::read_to_string("./telegramApiToken.txt")
        .expect("Can't read file \"telegramApiToken.txt\", please make sure the file exists and contains the bot API Token");
}

fn start_game(user_state: &mut UserState) -> String {
    let game_state = &user_state.game_state.as_ref().unwrap();
    user_state.current_score = 0;
    user_state.best_score = 0;
    return format!("Dealed new hand:\n{}\nDora indicator: {}", &get_printable_hand(&game_state.hands[0], user_state.settings.display_settings.tile_display), get_capitalized(&tile_to_string(&game_state.dora_indicators[0], user_state.settings.display_settings.terms_display)));
}

fn get_move_explanation_text(previous_move: &PreviousMoveData, user_settings: &UserSettings) -> String {
    assert!(previous_move.game_state.hands[previous_move.hand_index].tiles[13] != EMPTY_TILE, "Expected move state hand have 14 tiles before the discard");

    let mut visible_tiles = get_visible_tiles(&previous_move.game_state, previous_move.hand_index);
    let best_discards = calculate_best_discards_ukeire2(&previous_move.game_state.hands[previous_move.hand_index].tiles, previous_move.full_hand_shanten, &mut visible_tiles, &user_settings.score_settings);

    let mut result = String::new();
    for discard_info in best_discards {
        let tile_string = tile_to_string(&discard_info.tile, user_settings.display_settings.terms_display);
        result += &format!("{}: {} (score {})\n",
            get_capitalized(&tile_string),
            get_printable_tiles_set(&discard_info.tiles_improving_shanten, user_settings.display_settings.tile_display),
            discard_info.score,
        )
    }
    return result;
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
                    let predefined_hand = make_hand_from_string(&value);
                    user_state.game_state = generate_dealed_game_with_hand(1, predefined_hand, true);
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
            return [format!("Current hand:\n{}", &get_printable_hand(&game_state.hands[0], settings.display_settings.tile_display))].to_vec();
        },
        Some("/discards") => {
            if user_state.game_state.is_none() {
                return [NO_HAND_IN_PROGRESS_MESSAGE.to_string()].to_vec();
            }
            let game_state = &user_state.game_state.as_ref().unwrap();
            return [format!("Dora indicator: {}\nDiscards:\n{}", get_capitalized(&tile_to_string(&game_state.dora_indicators[0], settings.display_settings.terms_display)), &get_printable_tiles_set_main(&game_state.discards[0], settings.display_settings.tile_display))].to_vec();
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
            settings.display_settings.tile_display = TileDisplayOption::Unicode;
            user_state.settings_unsaved = true;
            return ["Set display style to unicode".to_string()].to_vec()
        },
        Some("/display_text") => {
            settings.display_settings.tile_display = TileDisplayOption::Text;
            user_state.settings_unsaved = true;
            return ["Set display style to text".to_string()].to_vec()
        },
        Some("/display_url") => {
            settings.display_settings.tile_display = TileDisplayOption::UrlImage;
            user_state.settings_unsaved = true;
            return ["Set display style to image url".to_string()].to_vec()
        },
        Some("/terms_eng") => {
            settings.display_settings.terms_display = TermsDisplayOption::EnglishTerms;
            settings.display_settings.language_key = "ene".to_string();
            user_state.settings_unsaved = true;
            return ["Set terminology to English".to_string()].to_vec()
        },
        Some("/terms_jap") => {
            settings.display_settings.terms_display = TermsDisplayOption::JapaneseTerms;
            settings.display_settings.language_key = "enj".to_string();
            user_state.settings_unsaved = true;
            return ["Set terminology to Japanese".to_string()].to_vec()
        },
        Some("/toggle_kokushi") => {
            settings.score_settings.allow_kokushi = !settings.score_settings.allow_kokushi;
            user_state.settings_unsaved = true;
            return [format!("Kokushi musou is now {}counted for shanten calculation",  if settings.score_settings.allow_kokushi {""} else {"not "})].to_vec()
        },
        Some("/toggle_chiitoi") => {
            settings.score_settings.allow_chiitoitsu = !settings.score_settings.allow_chiitoitsu;
            user_state.settings_unsaved = true;
            return [format!("Chiitoitsu is now {}counted for shanten calculation", if settings.score_settings.allow_chiitoitsu {""} else {"not "})].to_vec()
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

    let full_hand_shanten = calculate_shanten(&game_state.hands[0].tiles, &settings.score_settings).get_calculated_shanten();
    let best_discards = calculate_best_discards_ukeire2(&game_state.hands[0].tiles, full_hand_shanten, &mut get_visible_tiles(&game_state, 0), &settings.score_settings);

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

            let shanten_calculator = calculate_shanten(&game_state.hands[0].tiles[0..13], &settings.score_settings);
            let new_shanten = shanten_calculator.get_calculated_shanten();
            if new_shanten > 0 {
                answer += &format!("Discarded {} ({}/{})\n", tile_to_string(&tile, settings.display_settings.terms_display), current_discard_score, best_discard_scores.score);
                if has_potential_for_furiten(&shanten_calculator.get_best_waits(), &game_state.discards[0]) {
                    answer += "Possible furiten\n";
                }
            }
            else {
                answer += translate("tenpai_hand", &translations, &settings);
                answer += "\n";
                let wait_tiles = filter_tiles_finishing_hand(&game_state.hands[0].tiles[0..13], &convert_frequency_table_to_flat_vec(&shanten_calculator.get_best_waits()), &settings.score_settings);
                answer += &format!("Waits: {} ({} tiles)", get_printable_tiles_set(&wait_tiles, settings.display_settings.tile_display), find_potentially_available_tile_count(&get_visible_tiles(&game_state, 0), &wait_tiles));
                if has_furiten_waits(&wait_tiles, &game_state.discards[0]) {
                    answer += " furiten";
                }
                answer += "\n";
            }
        },
        None => { answer += "Could not find the given tile in the hand\n"; },
    }

    if game_state.hands[0].tiles[13] == EMPTY_TILE {
        let shanten_calculator = calculate_shanten(&game_state.hands[0].tiles[0..13], &settings.score_settings);
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
                        answer += &format!("Better discards: {}\n", get_printable_tiles_set(&best_discard_scores.tiles, settings.display_settings.tile_display));
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
        answer += &format!("Drawn {}\n", tile_to_string(&game_state.hands[0].tiles[13], settings.display_settings.terms_display));
        answer += &format!("{} tiles left in the live wall\n", game_state.live_wall.len());
    }

    answer += &get_printable_hand(&game_state.hands[0], settings.display_settings.tile_display);

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

pub async fn run_telegram_bot() {
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
