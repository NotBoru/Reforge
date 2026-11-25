#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod core;
mod i18n;
mod ui;

use app::App;
use iced::window;
use iced::Size;
use ui::theme::{
    FONT_REGULAR, INTER_BOLD_BYTES, INTER_MEDIUM_BYTES, INTER_REGULAR_BYTES, INTER_SEMIBOLD_BYTES,
};

const ICON_64_RGBA: &[u8] = include_bytes!("../assets/reforge_64.rgba");

fn main() -> iced::Result {
    let icon = window::icon::from_rgba(ICON_64_RGBA.to_vec(), 64, 64).ok();

    iced::application(App::boot, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .font(INTER_REGULAR_BYTES)
        .font(INTER_MEDIUM_BYTES)
        .font(INTER_SEMIBOLD_BYTES)
        .font(INTER_BOLD_BYTES)
        .default_font(FONT_REGULAR)
        .antialiasing(true)
        .window(window::Settings {
            size: Size::new(1180.0, 740.0),
            min_size: Some(Size::new(920.0, 600.0)),
            position: window::Position::Centered,
            icon,
            ..Default::default()
        })
        .run()
}
