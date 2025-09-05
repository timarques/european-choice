mod widgets;
mod constants;
mod models;
mod repository;
mod application;
mod prelude;

fn main() -> anyhow::Result<()> {
    application::Application::new().activate()
}
