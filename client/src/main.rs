#![feature(trivial_bounds)]
#![feature(adt_const_params)]
#![feature(let_chains)]
#![feature(decl_macro)]
#![feature(more_qualified_paths)]
#![feature(trait_alias)]
#![feature(div_duration)]


use bevy::{app::App, DefaultPlugins};
use bevy_framepace::FramepacePlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use nongame::NonGame;

const DEBUG: bool = true;

mod nongame;
pub mod ui;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // FramepacePlugin,
            DefaultPickingPlugins,
            ui::UiPlugin,
            NonGame {},
        ))
        .run();
}
