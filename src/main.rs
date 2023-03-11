mod game_logic;
mod efficiency_calculator;
mod user_settings;
mod user_state;
mod input_output;
mod translations;
mod telegram_bot;

extern crate rand;

use crate::telegram_bot::run_telegram_bot;

#[tokio::main]
async fn main() {
    run_telegram_bot().await;
}
