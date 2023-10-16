use dashmap::DashMap;
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;
use teloxide::prelude::*;

use crate::efficiency_calculator::*;
use crate::game_logic::*;
use crate::image_render::*;
use crate::input_output::*;
use crate::translations::*;
use crate::user_settings::*;
use crate::user_state::*;

fn read_telegram_token() -> String {
    return fs::read_to_string("./telegramApiToken.txt")
        .expect("Can't read file \"telegramApiToken.txt\", please make sure the file exists and contains the bot API Token");
}

fn start_game(user_state: &mut UserState, static_data: &StaticData) -> Response {
    let game_state = &user_state.game_state.as_ref().unwrap();
    user_state.current_score = 0;
    user_state.best_score = 0;
    user_state.efficiency_sum = 0.0;
    user_state.moves = 0;
    return single_image_response(
        render_game_state(
            &game_state,
            &static_data.render_data.sizes[user_state.settings.display_settings.render_size],
        ),
        "Dealt new hand".to_string(),
    );
}

fn get_move_explanation_text(
    previous_move: &PreviousMoveData,
    user_settings: &UserSettings,
) -> String {
    assert_ne!(
        previous_move.game_state.hands[previous_move.hand_index].tiles[13], EMPTY_TILE,
        "Expected move state hand have 14 tiles before the discard"
    );

    let mut visible_tiles = get_visible_tiles(&previous_move.game_state, previous_move.hand_index);
    let best_discards = calculate_best_discards_ukeire2(
        &previous_move.game_state.hands[previous_move.hand_index].tiles,
        previous_move.full_hand_shanten,
        &mut visible_tiles,
        &user_settings.score_settings,
    );

    if best_discards.is_empty() {
        return "No appropriate discards. This shouldn't happen. Please report this error to the developers".to_string();
    }

    let mut result = String::new();
    for discard_info in best_discards {
        let tile_string = tile_to_string(
            &discard_info.tile,
            user_settings.display_settings.terms_display,
        );
        result += &format!(
            "{}: {}\n",
            get_capitalized(&tile_string),
            discard_info.score,
        )
    }

    return result;
}

struct StaticData {
    translations: Translations,
    render_data: ImageRenderData,
}

#[derive(Clone)]
struct Response {
    text: String,
    image: Option<teloxide::types::InputFile>,
}

fn text_response(text: &str) -> Vec<Response> {
    [Response {
        text: text.to_string(),
        image: None,
    }]
    .to_vec()
}

fn text_response_str(text: String) -> Vec<Response> {
    [Response {
        text,
        image: None,
    }]
    .to_vec()
}

fn single_image_response(img: ImageBuf, text: String) -> Response {
    let mut buf: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageOutputFormat::Png)
        .expect("Failed to convert image to png");
    let photo = teloxide::types::InputFile::memory(buf.get_ref().to_vec());
    return Response {
        text,
        image: Some(photo),
    };
}

fn image_response(img: ImageBuf, text: String) -> Vec<Response> {
    return [single_image_response(img, text)].to_vec();
}

fn process_user_message(
    user_state: &mut UserState,
    message: &Message,
    static_data: &StaticData,
) -> Vec<Response> {
    if message.text().is_none() {
        return text_response("No message received");
    }

    const NO_HAND_IN_PROGRESS_MESSAGE: &str =
        "No hand is in progress, send /start to start a new hand";
    const SETTINGS_TEXT: &str = "
Choose terminology:
/terms_eng - English terminology
/terms_jap - Japanese terminology

Choose rules:
/toggle_chiitoi - turn on/off counting for Chiitoitsu
/toggle_kokushi - turn on/off counting for Kokushi musou

Choose render size (smaller = faster):
/render_small - small size
/render_medium - medium size
/render_large - large size";

    let message_text = &message.text().unwrap();
    let mut answer: String = String::new();
    let mut settings = &mut user_state.settings;
    let mut message_split = message_text.split_whitespace();

    match message_split.next() {
        Some("/start") => {
            match message_split.next() {
                Some(hand_string) => {
                    let (hand_string, discards_msg) = if hand_string.contains("-") {
                        let mut split = hand_string.split("-");
                        (split.next().unwrap_or_default(), split.next())
                    } else {
                        (hand_string, message_split.next())
                    };

                    let predefined_hand = make_hand_from_string(&hand_string);
                    let discards = match discards_msg {
                        Some(discards_string) => make_discards_from_string(&discards_string),
                        None => Vec::new(),
                    };
                    user_state.game_state = generate_dealt_game_with_hand_and_discards(1, predefined_hand, discards, true);
                    if user_state.game_state.is_none() {
                        return text_response("Given string doesn't represent a valid hand");
                    }
                }
                None => {
                    user_state.game_state = Some(generate_normal_dealt_game(1, true));
                }
            }
            return [start_game(user_state, &static_data)].to_vec();
        }
        Some("/table") => {
            if user_state.game_state.is_none() {
                return text_response(NO_HAND_IN_PROGRESS_MESSAGE);
            }
            let game_state = &user_state.game_state.as_ref().unwrap();
            return image_response(
                render_game_state(
                    &game_state,
                    &static_data.render_data.sizes[settings.display_settings.render_size],
                ),
                format!("Tiles left: {}", game_state.live_wall.len()),
            );
        }
        Some("/explain") => {
            return match &user_state.previous_move {
                Some(previous_move) => image_response(
                    render_move_explanation(
                        &previous_move,
                        &settings.score_settings,
                        &static_data.render_data.sizes[settings.display_settings.render_size],
                    ),
                    get_move_explanation_text(&previous_move, &settings),
                ),
                None => text_response("No moves are recorded to explain"),
            }
        }
        Some("/settings") => return text_response(SETTINGS_TEXT),
        Some("/terms_eng") => {
            settings.display_settings.terms_display = TermsDisplayOption::EnglishTerms;
            settings.display_settings.language_key = "ene".to_string();
            user_state.settings_unsaved = true;
            return text_response("Set terminology to English");
        }
        Some("/terms_jap") => {
            settings.display_settings.terms_display = TermsDisplayOption::JapaneseTerms;
            settings.display_settings.language_key = "enj".to_string();
            user_state.settings_unsaved = true;
            return text_response("Set terminology to Japanese");
        }
        Some("/toggle_kokushi") => {
            settings.score_settings.allow_kokushi = !settings.score_settings.allow_kokushi;
            user_state.settings_unsaved = true;
            return text_response_str(format!(
                "Kokushi musou is now {}counted for shanten calculation",
                if settings.score_settings.allow_kokushi {
                    ""
                } else {
                    "not "
                }
            ));
        }
        Some("/toggle_chiitoi") => {
            settings.score_settings.allow_chiitoitsu = !settings.score_settings.allow_chiitoitsu;
            user_state.settings_unsaved = true;
            return text_response_str(format!(
                "Chiitoitsu is now {}counted for shanten calculation",
                if settings.score_settings.allow_chiitoitsu {
                    ""
                } else {
                    "not "
                }
            ));
        }
        Some("/render_small") => {
            settings.display_settings.render_size = 0;
            user_state.settings_unsaved = true;
            return text_response("Set render size to small");
        }
        Some("/render_medium") => {
            settings.display_settings.render_size = 1;
            user_state.settings_unsaved = true;
            return text_response("Set render size to medium");
        }
        Some("/render_large") => {
            settings.display_settings.render_size = 2;
            user_state.settings_unsaved = true;
            return text_response("Set render size to large");
        }
        Some(_) => {}
        None => {}
    }

    if user_state.game_state.is_none() {
        return text_response(NO_HAND_IN_PROGRESS_MESSAGE);
    }

    let mut game_state = user_state.game_state.as_mut().unwrap();

    let requested_tile = get_tile_from_input(&message_text.to_lowercase());
    if requested_tile == EMPTY_TILE {
        return text_response("Entered string doesn't seem to be a tile representation, tile should be a digit followed by 'm', 'p', 's', or 'z' or a tile name (e.g. all \"7z\", \"red\", and \"chun\" are acceptable inputs for the red dragon tile)");
    }

    let full_hand_shanten = calculate_shanten(&game_state.hands[0].tiles, &settings.score_settings)
        .get_calculated_shanten();
    let best_discards = calculate_best_discards_ukeire2(
        &game_state.hands[0].tiles,
        full_hand_shanten,
        &mut get_visible_tiles(&game_state, 0),
        &settings.score_settings,
    );

    let best_discard_scores = get_best_discard_scores(&best_discards);
    let mut discarded_tile = None;

    match game_state.hands[0]
        .tiles
        .iter()
        .position(|&r| r == requested_tile)
    {
        Some(tile_index_in_hand) => {
            user_state.previous_move = Some(PreviousMoveData {
                game_state: (*game_state).clone(),
                hand_index: 0,
                full_hand_shanten,
                discarded_tile: EMPTY_TILE,
            });
            let tile = discard_tile(&mut game_state, 0, tile_index_in_hand as usize);
            discarded_tile = Some(tile);
            let current_discard_score = get_discard_score(&best_discards, &tile);

            user_state.best_score += best_discard_scores.score;
            user_state.current_score += current_discard_score;
            user_state.efficiency_sum +=
                current_discard_score as f32 / best_discard_scores.score as f32;
            user_state.moves += 1;
            user_state.previous_move.as_mut().unwrap().discarded_tile = tile;

            let shanten_calculator =
                calculate_shanten(&game_state.hands[0].tiles[0..13], &settings.score_settings);
            let new_shanten = shanten_calculator.get_calculated_shanten();
            if new_shanten > 0 {
                answer += &format!(
                    "Discarded {} ({}/{})\n",
                    tile_to_string(&tile, settings.display_settings.terms_display),
                    current_discard_score,
                    best_discard_scores.score
                );
                if has_potential_for_furiten(
                    &shanten_calculator.get_best_waits(),
                    &game_state.discards[0],
                ) {
                    answer += "Possible furiten\n";
                }
            } else {
                answer += translate("tenpai_hand", &static_data.translations, &settings);
                answer += "\n";
                let wait_tiles = filter_tiles_finishing_hand(
                    &game_state.hands[0].tiles[0..13],
                    &convert_frequency_table_to_flat_vec(&shanten_calculator.get_best_waits()),
                    &settings.score_settings,
                );
                answer += &format!(
                    "Waits: {} ({} tiles)",
                    get_printable_tiles_set_text(
                        &wait_tiles,
                        settings.display_settings.terms_display
                    ),
                    find_potentially_available_tile_count(
                        &get_visible_tiles(&game_state, 0),
                        &wait_tiles
                    )
                );
                if has_furiten_waits(&wait_tiles, &game_state.discards[0]) {
                    answer += " furiten";
                }
                answer += "\n";
            }
        }
        None => {
            answer += "Could not find the given tile in the hand\n";
        }
    }

    if game_state.hands[0].tiles[13] == EMPTY_TILE {
        let shanten_calculator =
            calculate_shanten(&game_state.hands[0].tiles[0..13], &settings.score_settings);
        let shanten = shanten_calculator.get_calculated_shanten();
        match discarded_tile {
            Some(tile) => {
                if shanten > full_hand_shanten {
                    answer += "Went back in shanten\n";
                } else {
                    if best_discard_scores.tiles.contains(&tile) {
                        answer += "Best discard\n";
                    } else {
                        answer += &format!(
                            "Better discards: {}\n",
                            get_capitalized(&get_printable_tiles_set_text(
                                &best_discard_scores.tiles,
                                settings.display_settings.terms_display
                            ))
                        );
                    }
                }

                if shanten <= 0 {
                    if user_state.best_score > 0 {
                        answer += &format!(
                            "Score: {}/{}\nAverage efficiency {}% for {} turns",
                            user_state.current_score,
                            user_state.best_score,
                            (100.0 * (user_state.efficiency_sum / user_state.moves as f32)).floor(),
                            user_state.moves
                        );
                    } else {
                        answer += &format!(
                            "Some error occurred, best possible score was zero, current score: {}",
                            user_state.current_score
                        );
                    }
                    user_state.game_state = None;
                    answer += "\nSend /start to start new game";
                    return text_response_str(answer);
                }
            }
            None => panic!("We got 13 tiles but nothing discarded, that is broken"),
        }

        if game_state.live_wall.is_empty() {
            user_state.game_state = None;
            answer += "\nEnd of life wall, no more tiles left\nSend /start to start new game";
            return text_response_str(answer);
        }

        draw_tile_to_hand(&mut game_state, 0);
        answer += &format!(
            "Drew {}\n{} tiles left in the live wall\n",
            tile_to_string(
                &game_state.hands[0].tiles[13],
                settings.display_settings.terms_display
            ),
            game_state.live_wall.len()
        );
    }

    return image_response(
        render_game_state(
            &game_state,
            &static_data.render_data.sizes[settings.display_settings.render_size],
        ),
        answer,
    );
}

fn load_translations() -> Translations {
    let mut translations = HashMap::new();

    {
        translations.insert(
            "ene".to_string(),
            HashMap::from([("tenpai_hand", "The hand is ready now")]),
        );
    }

    {
        translations.insert(
            "enj".to_string(),
            HashMap::from([("tenpai_hand", "Tenpai")]),
        );
    }

    return translations;
}

fn load_user_states(file_path: &str) -> DashMap<ChatId, UserState> {
    return if Path::new(file_path).exists() {
        let data = fs::read_to_string(file_path).expect("Can't open file");
        serde_json::from_str(&data).expect("Can't parse user states file")
    } else {
        DashMap::new()
    };
}

fn save_user_state(file_path: &str, chat_id: ChatId, user_state: &UserState) {
    fs::create_dir_all(Path::new(file_path).parent().unwrap())
        .expect("The directory can't be created");
    let mut hash_map: HashMap<ChatId, UserSettings>;
    if Path::new(file_path).exists() {
        let data = fs::read_to_string(file_path).expect("Can't open file");
        hash_map = serde_json::from_str(&data).expect("Can't parse user states file");
    } else {
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

    type UserStates = DashMap<ChatId, UserState>;
    type SharedUserStates = Arc<UserStates>;
    type SharedStaticData = Arc<StaticData>;

    let user_states = SharedUserStates::new(load_user_states("./data/user_states.json"));
    let static_data = SharedStaticData::new(StaticData {
        translations: load_translations(),
        render_data: load_static_render_data(),
    });

    let handler = Update::filter_message().endpoint(
        |bot: Bot,
         user_states: SharedUserStates,
         static_data: SharedStaticData,
         message: Message| async move {
            let user_state: &mut UserState = &mut user_states
                .entry(message.chat.id)
                .or_insert_with(|| get_default_user_state());

            let responses = process_user_message(user_state, &message, &static_data);
            if user_state.settings_unsaved {
                save_user_state("./data/user_states.json", message.chat.id, &user_state);
                user_state.settings_unsaved = false;
            }
            for response in responses {
                if response.image.is_none() {
                    bot.send_message(message.chat.id, response.text).await?;
                } else {
                    let text = response.text;
                    let mut send_photo = bot.send_photo(message.chat.id, response.image.unwrap());
                    if !text.is_empty() {
                        send_photo.caption = Some(text);
                    }
                    send_photo.send().await.expect("Failed to send request");
                }
            }
            respond(())
        },
    );

    Dispatcher::builder(bot, handler)
        // Pass the shared state to the handler as a dependency.
        .dependencies(dptree::deps![user_states.clone(), static_data.clone()])
        .build()
        .dispatch()
        .await;
}
