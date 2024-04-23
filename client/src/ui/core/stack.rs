use bevy::prelude::*;
use bevy_mod_picking::picking_core::Pickable;

use crate::ui::{BuildContext, Widget};

pub struct Stack<'w> {
    widgets: Vec<Box<dyn Widget + 'w>>,
    dir: FlexDirection,
}

pub fn stack<'w>(dir: FlexDirection) -> Stack<'w> {
    Stack {
        widgets: vec![],
        dir,
    }
}

impl<'w> Stack<'w> {
    pub fn add(&mut self, child: impl Widget + 'w) {
        self.widgets.push(Box::new(child));
    }

    pub fn with(mut self, child: impl Widget + 'w) -> Self {
        self.add(child);
        self
    }
}

impl<'w> Widget for Stack<'w> {
    fn build(self, cx: &mut BuildContext) -> bevy::prelude::Entity {
        let children = self
            .widgets
            .into_iter()
            .map(|w| w.build_boxed(cx))
            .collect::<Vec<_>>();
        cx.commands
            .spawn((
                NodeBundle {
                    style: Style {
                        flex_direction: self.dir,
                        ..default()
                    },
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .push_children(&children)
            .id()
    }

    fn build_boxed(self: Box<Self>, cx: &mut BuildContext) -> bevy::prelude::Entity {
        self.build(cx)
    }
}
