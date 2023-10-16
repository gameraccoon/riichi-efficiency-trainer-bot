mod efficiency_calculator;
mod game_logic;
mod image_render;
mod input_output;
mod telegram_bot;
mod translations;
mod user_settings;
mod user_state;

extern crate rand;

use crate::telegram_bot::run_telegram_bot;

#[tokio::main]
async fn main() {
    run_telegram_bot().await;
}
