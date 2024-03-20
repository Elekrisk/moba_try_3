use bevy::{app::{Plugin, Update}, asset::AssetServer, ecs::{component::Component, entity::Entity, query::With, system::{Command, Commands, Query}, world::World}, hierarchy::{BuildChildren, ChildBuilder}, render::color::Color, text::{Text, TextStyle}, ui::{node_bundles::{ButtonBundle, NodeBundle, TextBundle}, widget::Button, AlignItems, BackgroundColor, Interaction, JustifyContent, Style, Val}, utils::default};


pub struct MenuPlugin {
    
}

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, MenuButton::system);
    }
}

pub struct Menu {
    pub title: String,
    pub options: Vec<Box<dyn MenuOption>>
}

impl Menu {
    pub fn build(self, commands: &mut Commands) {
        let mut root_node = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: BackgroundColor(Color::BLACK),
            ..default()
        });
        for option in self.options {
            root_node.with_children(|parent| option.build(parent));
        }
    }
}

pub trait MenuOption {
    fn build(self, parent: &mut ChildBuilder);
}

pub struct MenuButton {
    pub label: String,
    pub on_click: Box<dyn (Fn() -> Box<dyn Fn(&mut World) + Sync + Send>) + Sync + Send>
}

#[derive(Component)]
struct ButtonAction(Box<dyn (Fn() -> Box<dyn Fn(&mut World) + Sync + Send>) + Sync + Send>);

impl MenuButton {
    fn system(query: Query<(&Interaction, &ButtonAction), With<Button>>, mut commands: Commands) {
        for (interaction, action) in &query {
            if *interaction == Interaction::Pressed {
                commands.add((action.0)());
            }
        }
    }
}

impl MenuOption for MenuButton {
    fn build(self, parent: &mut ChildBuilder) {
        let button = parent.spawn((ButtonBundle {
            background_color: BackgroundColor(Color::GRAY),
            ..default()
        }, ButtonAction(self.on_click))).with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(&self.label, TextStyle {
                    font_size: 32.0,
                    color: Color::WHITE,
                    ..default()
                }),
                ..default()
            });
        });
    }
}
