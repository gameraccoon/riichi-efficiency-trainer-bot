use crate::game_logic::*;
use crate::ukeire_calculator::*;
use crate::user_settings::*;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
