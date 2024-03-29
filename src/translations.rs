use crate::user_settings::*;
use std::collections::HashMap;

pub type Translations = HashMap<String, HashMap<&'static str, &'static str>>;

pub fn translate(
    key: &str,
    translations: &Translations,
    user_settings: &UserSettings,
) -> &'static str {
    return translations[&user_settings.display_settings.language_key][key];
}
