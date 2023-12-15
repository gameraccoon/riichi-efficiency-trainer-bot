use serde::{Deserialize, Serialize};

use crate::game_logic::GameSettings;
use crate::input_output::*;
use crate::ukeire_calculator::ScoreCalculationSettings;

#[derive(Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub display_settings: DisplaySettings,
    pub score_settings: ScoreCalculationSettings,
    pub game_settings: GameSettings,
}

pub fn get_default_settings() -> UserSettings {
    UserSettings {
        display_settings: DisplaySettings {
            terms_display: TermsDisplayOption::EnglishTerms,
            language_key: "ene".to_string(),
        },
        score_settings: ScoreCalculationSettings {
            allow_kokushi: true,
            allow_chiitoitsu: true,
        },
        game_settings: GameSettings {
            deal_first_tile: true,
            include_honors: true,
        },
    }
}
