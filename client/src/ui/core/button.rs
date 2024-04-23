use bevy::prelude::*;
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::On,
};

use crate::ui::Widget;

pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, button_highlight);
    }
}

pub trait IntoSystemAny<Marker> {
    fn into_handler(self: Box<Self>) -> On<Pointer<Click>>;
}

impl<Marker, T: IntoSystem<(), (), Marker>> IntoSystemAny<Marker> for T {
    fn into_handler(self: Box<Self>) -> On<Pointer<Click>> {
        On::run(*self)
    }
}

pub struct Button<W, Marker> {
    inner: W,
    action: Box<dyn IntoSystemAny<Marker>>,
}

pub fn button<W, Marker>(
    inner: W,
    action: impl IntoSystemAny<Marker> + 'static,
) -> Button<W, Marker> {
    Button {
        inner,
        action: Box::new(action),
    }
}

impl<W: Widget, Marker> Widget for Button<W, Marker> {
    fn build(self, cx: &mut crate::ui::BuildContext) -> bevy::prelude::Entity {
        let child = self.inner.build(cx);

        cx.commands
            .spawn((ButtonBundle { ..default() }, self.action.into_handler()))
            .add_child(child)
            .id()
    }

    fn build_boxed(self: Box<Self>, cx: &mut crate::ui::BuildContext) -> bevy::prelude::Entity {
        self.build(cx)
    }
}

fn button_highlight(mut q: Query<(&mut BackgroundColor, &Interaction), Changed<Interaction>>) {
    for (mut color, interaction) in &mut q {
        color.0 = match interaction {
            Interaction::Pressed => Color::rgb(0.7, 0.7, 0.7),
            Interaction::Hovered => Color::rgb(0.9, 0.9, 0.9),
            Interaction::None => Color::WHITE,
        }
    }
}
