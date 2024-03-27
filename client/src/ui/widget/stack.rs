use bevy::prelude::*;

use crate::ui::{Pod, Widget};

#[derive(Debug)]
pub struct Stack {
    pub direction: FlexDirection,
    pub children: Vec<Pod>,
}

impl Stack {}

impl Widget for Stack {
    fn build(&mut self, commands: &mut bevy::prelude::Commands) -> bevy::prelude::Entity {
        let children = self
            .children
            .iter_mut()
            .map(|c| {
                let e = c.widget.build(commands);
                c.entity = Some(e);
                e
            })
            .collect::<Vec<_>>();

        let mut node = commands.spawn(NodeBundle {
            style: Style {
                flex_direction: self.direction,
                ..default()
            },
            ..default()
        });

        node.push_children(&children);

        node.id()
    }

    fn rebuild(&mut self, entity: bevy::prelude::Entity, commands: &mut bevy::prelude::Commands) {
        println!("Rebuilding stack...");

        let direction = self.direction;
        commands
            .entity(entity)
            .add(move |mut e: EntityWorldMut<'_>| {
                e.get_mut::<Style>().unwrap().flex_direction = direction;
            });

        for child in &mut self.children {
            child.widget.rebuild(child.entity.unwrap(), commands);
        }
    }

    fn delete(&mut self, entity: bevy::prelude::Entity, commands: &mut bevy::prelude::Commands) {
        commands.entity(entity).despawn_recursive();
    }
}
