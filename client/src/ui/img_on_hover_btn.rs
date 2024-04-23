use bevy::prelude::*;

use super::{button, IntoSystemAny, Widget};

use crate::ui::WidgetExt as _;

pub struct ImgOnHoverBtnPlugin;

impl Plugin for ImgOnHoverBtnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, img_on_hover_btn_system);
    }
}

#[derive(Component)]
struct ImgOnHoverBtnComponent;

pub fn img_on_hover_btn<W: Widget, Marker>(
    inner: W,
    action: impl IntoSystemAny<Marker> + 'static,
) -> impl Widget {
    let inner = inner.custom(|e, cx| {
        cx.commands
            .spawn(ImageBundle {
                image: cx.asset_server.load("ui/blur.png").into(),
                ..default()
            })
            .add_child(e)
            .id()
    });
    button(inner, action).custom(|e, cx| {
        cx.commands.entity(e).insert(ImgOnHoverBtnComponent);
        e
    })
}

fn img_on_hover_btn_system(
    q: Query<(&Children, &Interaction), (Changed<Interaction>, With<ImgOnHoverBtnComponent>)>,
    mut cq: Query<&mut BackgroundColor>,
) {
    for (children, interaction) in &q {
        let child = children[0];
        let mut bg = cq.get_mut(child).unwrap();
        bg.0 = match interaction {
            Interaction::Pressed => Color::rgb(0.6, 0.6, 0.6),
            Interaction::Hovered => Color::rgb(0.8, 0.8, 0.8),
            Interaction::None => Color::NONE,
        }
    }
}
