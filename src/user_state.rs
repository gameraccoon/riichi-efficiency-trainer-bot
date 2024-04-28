use crate::game_logic::*;
use crate::json_file_updater::*;
use crate::ukeire_calculator::*;
use crate::user_settings::*;
use crate::user_state_updaters;
use dashmap::DashMap;
use std::path::Path;
use teloxide::types::ChatId;

use crate::user_state_updaters::update_user_states_to_the_latest_version;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize)]
pub struct UserStates {
    // json format version, needed to update if the format changes
    // should not be changed after deserialized
    version: String,

    pub states: DashMap<ChatId, UserState>,
}

#[derive(Clone)]
pub struct UserState {
    pub game_state: Option<GameState>,
    pub current_score: u32,
    pub best_score: u32,
    pub efficiency_sum: f32,
    pub moves: u32,
    pub previous_move: Option<PreviousMoveData>,
    pub settings: UserSettings,
    pub settings_unsaved: bool,
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

pub fn get_default_user_state() -> UserState {
    UserState {
        game_state: None,
        current_score: 0,
        best_score: 0,
        efficiency_sum: 0.0,
        moves: 0,
        previous_move: None,
        settings: get_default_settings(),
        settings_unsaved: false,
    }
}

pub fn get_default_user_states() -> UserStates {
    UserStates {
        version: user_state_updaters::LATEST_SAVE_VERSION.to_string(),
        states: DashMap::new(),
    }
}

pub fn read_user_states_from_file(user_states_file_path: &Path) -> UserStates {
    // create default file that will be used if the file doesn't exist
    let default_states = get_default_user_states();
    // if the file doesn't exist, create it
    if !user_states_file_path.exists() {
        let data = serde_json::to_string(&default_states);
        let data = match data {
            Ok(data) => data,
            Err(err) => {
                panic!("Failed to serialize the default UserStates: {}", err);
            }
        };
        let result = std::fs::write(&user_states_file_path, data);
        if let Err(err) = result {
            panic!("Failed to write the default UserStates: {}", err);
        }
    }

    // read the user states file from the disk
    let data = std::fs::read_to_string(&user_states_file_path);
    let data = match data {
        Ok(data) => data,
        Err(err) => {
            panic!("Failed to read the user states file: {}", err);
        }
    };
    let user_states_json = serde_json::from_str(&data);
    let mut user_states_json = match user_states_json {
        Ok(user_states_json) => user_states_json,
        Err(err) => {
            panic!("Failed to parse the user states file: {}", err);
        }
    };

    let update_result = update_user_states_to_the_latest_version(&mut user_states_json);
    if let UpdateResult::Error(error) = update_result {
        match error {
            JsonFileUpdaterError::UnknownVersion {
                version,
                latest_version,
            } => {
                panic!(
                    "Unknown version of the user states file: {}. The latest version is {}",
                    version, latest_version
                );
            }
        };
    }

    let user_states = serde_json::from_value(user_states_json);
    let user_states: UserStates = match user_states {
        Ok(user_states) => user_states,
        Err(err) => {
            if update_result == UpdateResult::Updated {
                panic!("Failed to deserialize the updated user states: {}", err);
            }
            panic!("Failed to deserialize user states file: {}", err);
        }
    };

    if update_result == UpdateResult::Updated {
        println!("The user states file has been updated to the latest version");
        let data = serde_json::to_string(&user_states);
        let data = match data {
            Ok(data) => data,
            Err(err) => {
                panic!("Failed to serialize the updated user states: {}", err);
            }
        };
        let result = std::fs::write(&user_states_file_path, data);
        if let Err(err) = result {
            panic!("Failed to write the updated user states: {}", err);
        }
    }

    return user_states;
}

pub fn save_user_states_to_file(user_states: &UserStates, user_states_file_path: &Path) {
    let data = serde_json::to_string(&user_states);
    let data = match data {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Failed to serialize the user states: {}", err);
            return;
        }
    };
    let result = std::fs::write(&user_states_file_path, data);
    if let Err(err) = result {
        eprintln!("Failed to write the user states: {}", err);
    }
}

pub fn save_single_user_state(file_path: &Path, chat_id: ChatId, user_state: &UserState) {
    // this is quite terrible, but we need to do that in order to not lock states of other users
    // this should ideally be replaced by  sqlite database
    let user_states = read_user_states_from_file(file_path);
    user_states.states.insert(chat_id, user_state.clone());
    save_user_states_to_file(&user_states, file_path);
}
