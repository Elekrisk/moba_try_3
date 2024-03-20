mod menu;

use bevy::{
    app::{AppExit, Plugin, Startup}, core_pipeline::core_3d::Camera3dBundle, ecs::{system::Commands, world::World}, prelude::default
};

use self::menu::{Menu, MenuButton};

pub struct NonGame {}

impl Plugin for NonGame {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, move |mut commands: Commands| {
            commands.spawn(Camera3dBundle {
                ..default()
            });

            let main_menu = Menu {
                title: "Main Menu".to_string(),
                options: vec![Box::new(MenuButton {
                    label: "Quit".to_string(),
                    on_click: Box::new(|world: &mut World| {
                        world.send_event(AppExit);
                    })
                })],
            };
            main_menu.build(&mut commands);
        });
    }
}
