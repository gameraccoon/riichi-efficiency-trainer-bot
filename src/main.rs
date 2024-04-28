mod game_logic;
mod image_render;
mod input_output;
mod json_file_updater;
mod telegram_bot;
mod translations;
mod ukeire_calculator;
mod user_settings;
mod user_state;
mod user_state_updaters;

extern crate rand;

#[tokio::main]
async fn main() {
    telegram_bot::run_telegram_bot().await;
}
