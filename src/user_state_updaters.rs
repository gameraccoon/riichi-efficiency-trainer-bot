// Copyright (C) Pavel Grebnev 2023-2024
// Distributed under the MIT License (license terms are at http://opensource.org/licenses/MIT).

use crate::json_file_updater::{JsonFileUpdater, UpdateResult};
use serde_json::Value as JsonValue;

static VERSION_FIELD_NAME: &str = "version";
pub static LATEST_SAVE_VERSION: &str = "0.2.0";

pub fn update_user_states_to_the_latest_version(user_states_json: &mut JsonValue) -> UpdateResult {
    let version = user_states_json[VERSION_FIELD_NAME].as_str();
    if let Some(version) = version {
        if version == LATEST_SAVE_VERSION {
            return UpdateResult::NoUpdateNeeded;
        }
    }

    let json_file_updater = register_json_updaters();
    return json_file_updater.update_json(user_states_json);
}

fn register_json_updaters() -> JsonFileUpdater {
    let mut json_file_updater = JsonFileUpdater::new(VERSION_FIELD_NAME);

    json_file_updater.add_update_function("0.2.0", v0_2_0_added_states_field);
    // add update functions above this line
    // don't forget to update LATEST_SAVE_VERSION at the beginning of the file

    json_file_updater
}

fn v0_2_0_added_states_field(user_states_json: &mut JsonValue) {
    // move everything fromthe root to the states field
    let states = user_states_json.take();
    user_states_json["states"] = states;
}
