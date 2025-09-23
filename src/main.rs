#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::wildcard_imports)]

mod widgets;
mod constants;
mod models;
mod controllers;
mod repository;
mod application;
mod ui;
mod ordered_map;
mod search_engine;
mod populator;
mod prelude;

fn main() -> anyhow::Result<()> {
    application::Application::new().run()
}
