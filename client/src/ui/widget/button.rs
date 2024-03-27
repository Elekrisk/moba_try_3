use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::ui::{view::ButtonEvent, Message, ViewId, ViewIdPath, Widget};

#[derive(Debug)]
pub struct Button {
    pub id_path: ViewIdPath,
    pub label: String,
}

impl Button {
    pub fn new(id_path: ViewIdPath, label: String) -> Self {
        Self { id_path, label }
    }
}

impl Widget for Button {
    fn build(&mut self, commands: &mut bevy::prelude::Commands) -> bevy::prelude::Entity {
        let id_path = self.id_path.clone();

        commands
            .spawn((
                ButtonBundle { ..default() },
                On::<Pointer<Click>>::run(move |ui_events: EventWriter<'_, Message>| {
                    button_clicked(id_path.clone(), ui_events)
                }),
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section(
                        &self.label,
                        TextStyle {
                            font_size: 16.0,
                            color: Color::BLACK,
                            ..default()
                        },
                    ),
                    ..default()
                });
            })
            .id()
    }

    fn rebuild(&mut self, entity: bevy::prelude::Entity, commands: &mut bevy::prelude::Commands) {
        println!("Rebuilding button...");
        let label = self.label.clone();
        commands
            .entity(entity)
            .add(move |mut e: EntityWorldMut<'_>| {
                let child = e.get::<Children>().unwrap()[0];
                e.world_scope(|w| w.get_mut::<Text>(child).unwrap().sections[0].value = label);
            });
    }

    fn delete(&mut self, entity: bevy::prelude::Entity, commands: &mut bevy::prelude::Commands) {
        commands.entity(entity).despawn_recursive();
    }
}

fn button_clicked(id_path: ViewIdPath, mut ui_events: EventWriter<Message>) {
    ui_events.send(Message::new(id_path, ButtonEvent::Click));
}
