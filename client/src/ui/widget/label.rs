use bevy::{
    hierarchy::DespawnRecursiveExt, render::color::Color, text::{Text, TextStyle}, ui::node_bundles::TextBundle, utils::default
};

use crate::ui::Widget;

#[derive(Debug)]
pub struct Label {
    pub label: String,
}

impl Widget for Label {
    fn build(&mut self, commands: &mut bevy::prelude::Commands) -> bevy::prelude::Entity {
        commands
            .spawn(TextBundle {
                text: Text::from_section(
                    self.label.clone(),
                    TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                ..default()
            })
            .id()
    }

    fn rebuild(&mut self, entity: bevy::prelude::Entity, commands: &mut bevy::prelude::Commands) {
        let label = self.label.clone();
        commands
            .entity(entity)
            .add(move |mut e: bevy::prelude::EntityWorldMut<'_>| {
                e.get_mut::<Text>().unwrap().sections[0].value = label
            });
    }

    fn delete(&mut self, entity: bevy::prelude::Entity, commands: &mut bevy::prelude::Commands) {
        commands.entity(entity).despawn_recursive();
    }
}
