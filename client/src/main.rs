#![feature(trivial_bounds)]
#![feature(adt_const_params)]
#![feature(let_chains)]
#![feature(decl_macro)]
#![feature(more_qualified_paths)]

use bevy::{app::App, DefaultPlugins};
use bevy_framepace::FramepacePlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use nongame::NonGame;
use ui::UiPlugin;
use ui2::BindingPlugin;

mod nongame;
mod ui;
mod ui2;

fn main() {
    App::new().add_plugins((DefaultPlugins, FramepacePlugin, DefaultPickingPlugins, UiPlugin, BindingPlugin, NonGame {})).run();
}


