use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::ui::{BuildContext, Widget};

pub struct Label {
    text: String,
}

pub fn label(text: impl Into<String>) -> Label {
    Label { text: text.into() }
}

impl Widget for Label {
    fn build(self, cx: &mut BuildContext) -> bevy::prelude::Entity {
        cx.commands
            .spawn((
                TextBundle {
                    text: bevy::prelude::Text::from_section(
                        self.text,
                        TextStyle {
                            font: cx.asset_server.load("fonts/Roboto-Light.ttf"),
                            font_size: 16.0,
                            color: Color::GOLD,
                        },
                    ),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .id()
    }

    fn build_boxed(self: Box<Self>, cx: &mut BuildContext) -> bevy::prelude::Entity {
        self.build(cx)
    }
}
