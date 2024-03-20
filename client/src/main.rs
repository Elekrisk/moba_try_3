#![feature(trivial_bounds)]

use bevy::{app::App, DefaultPlugins};
use nongame::NonGame;

mod nongame;

fn main() {
    App::new().add_plugins(DefaultPlugins).add_plugins(NonGame {}).run();
}
