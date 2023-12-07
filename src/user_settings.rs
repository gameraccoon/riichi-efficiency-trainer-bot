use crate::input_output::*;
use crate::ukeire_calculator::ScoreCalculationSettings;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub display_settings: DisplaySettings,
    pub score_settings: ScoreCalculationSettings,
}

pub fn get_default_settings() -> UserSettings {
    UserSettings {
        display_settings: DisplaySettings {
            terms_display: TermsDisplayOption::EnglishTerms,
            language_key: "ene".to_string(),
            render_size: 1,
        },
        score_settings: ScoreCalculationSettings {
            allow_kokushi: true,
            allow_chiitoitsu: true,
        },
    }
}
