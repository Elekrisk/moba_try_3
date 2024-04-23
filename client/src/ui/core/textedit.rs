use std::time::Duration;

use ab_glyph::{Font as _, ScaleFont};
use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
    ui::RelativeCursorPosition,
};
use bevy_mod_picking::prelude::*;

use crate::ui::{BuildContext, Focused, Widget};

pub struct TextEditPlugin;

impl Plugin for TextEditPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                textedit_char_input,
                textedit_keyboard_input,
                update_textedit,
                handle_cursor_get_focus_vis,
                handle_cursor_lose_focus_vis,
                handle_cursor_blink,
                handle_cursor_visibility,
            )
                .chain(),
        );
    }
}

pub struct TextEdit {
    text: String,
}

pub fn textedit(text: impl Into<String>) -> TextEdit {
    TextEdit { text: text.into() }
}

impl Widget for TextEdit {
    fn build(self, cx: &mut BuildContext) -> Entity {
        let mut text_entity = Entity::PLACEHOLDER;
        let mut cursor_entity = Entity::PLACEHOLDER;
        let text = self.text.clone();

        cx.commands
            .spawn((
                NodeBundle { ..default() },
                RelativeCursorPosition::default(),
                On::<Pointer<Click>>::run(handle_click),
            ))
            .with_children(|parent| {
                text_entity = parent
                    .spawn((
                        TextBundle {
                            text: Text::from_section(
                                text,
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
                    .id();
                cursor_entity = parent
                    .spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Px(1.0),
                                height: Val::Px(16.0),
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                ..default()
                            },
                            background_color: Color::GOLD.into(),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .id();
            })
            .insert((
                TextEditComponent {
                    text: self.text,
                    cursor: 0,
                    text_entity,
                    cursor_entity,
                },
                TextEditBlink {
                    remaining: Duration::ZERO,
                },
                TextEditCursorVis { visible: false },
            ))
            .id()
    }

    fn build_boxed(self: Box<Self>, cx: &mut BuildContext) -> Entity {
        self.build(cx)
    }
}

#[derive(Component)]
pub struct TextEditComponent {
    pub text: String,
    cursor: usize,
    text_entity: Entity,
    cursor_entity: Entity,
}

#[derive(Component)]
pub struct TextEditBlink {
    remaining: Duration,
}

impl TextEditBlink {
    fn reset(&mut self) {
        self.remaining = Duration::from_secs_f32(0.5);
    }
}

#[derive(Component)]
pub struct TextEditCursorVis {
    visible: bool,
}

fn handle_click(
    listener: Listener<Pointer<Click>>,
    mut q: Query<(
        &mut TextEditComponent,
        &mut TextEditBlink,
        &mut TextEditCursorVis,
        &Node,
        &RelativeCursorPosition,
    )>,
    tq: Query<&Text>,
    fonts: Res<Assets<Font>>,
) {
    let e = listener.listener();
    let Ok((mut textedit, mut blink, mut vis, node, rel_pos)) = q.get_mut(e) else {
        return;
    };
    vis.visible = true;
    blink.reset();

    let text = tq.get(textedit.text_entity).unwrap();

    let clicked_x = node.size().x * rel_pos.normalized.unwrap().x;
    let textstyle = &text.sections[0].style;
    let font = fonts
        .get(&textstyle.font)
        .unwrap()
        .font
        .as_scaled(textstyle.font_size);

    let mut x = 0.0;
    let mut pos = 0;
    for c in textedit.text.chars() {
        let glyph = font.glyph_id(c);
        let new_x = x + font.h_advance(glyph);

        if clicked_x < new_x {
            let diff = new_x - x;
            if clicked_x < x + diff / 2.0 {
                break;
            }
            pos += c.len_utf8();
            break;
        }

        pos += c.len_utf8();
        x = new_x;
    }

    textedit.cursor = pos;
}

fn handle_cursor_get_focus_vis(
    mut query: Query<(&mut TextEditCursorVis, &mut TextEditBlink), Added<Focused>>,
) {
    for (mut vis, mut blink) in &mut query {
        vis.visible = true;
        blink.reset();
    }
}

fn handle_cursor_lose_focus_vis(
    mut query: Query<(&mut TextEditCursorVis, &mut TextEditBlink)>,
    mut removed: RemovedComponents<Focused>,
) {
    for e in removed.read() {
        let Ok((mut vis, mut blink)) = query.get_mut(e) else {
            continue;
        };

        vis.visible = false;
        blink.reset();
    }
}

fn handle_cursor_visibility(
    query: Query<(&TextEditComponent, &TextEditCursorVis), Changed<TextEditCursorVis>>,
    mut vis: Query<&mut Visibility>,
) {
    for (textedit, cursor_vis) in &query {
        println!("VIS CHANGED: {}", cursor_vis.visible);
        *vis.get_mut(textedit.cursor_entity).unwrap() = if cursor_vis.visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn handle_cursor_blink(
    mut query: Query<(&mut TextEditBlink, &mut TextEditCursorVis), With<Focused>>,
    time: Res<Time>,
) {
    for (mut blink, mut vis) in &mut query {
        blink.remaining = blink.remaining.saturating_sub(time.delta());

        if blink.remaining.is_zero() {
            vis.visible = !vis.visible;
            blink.reset();
        }
    }
}

fn textedit_char_input(
    mut events: EventReader<ReceivedCharacter>,
    mut query: Query<&mut TextEditComponent, With<Focused>>,
) {
    let Ok(mut textedit) = query.get_single_mut() else {
        return;
    };
    for event in events.read() {
        if event.char.chars().any(|c| c.is_control()) {
            continue;
        }

        let idx = textedit.cursor;
        textedit.text.insert_str(idx, &event.char);
        textedit.cursor += event.char.len();
    }
}

fn textedit_keyboard_input(
    mut events: EventReader<KeyboardInput>,
    mut query: Query<
        (
            Entity,
            &mut TextEditComponent,
            &mut TextEditBlink,
            &mut TextEditCursorVis,
        ),
        With<Focused>,
    >,
    mut commands: Commands,
) {
    let Ok((e, mut textedit, mut blink, mut vis)) = query.get_single_mut() else {
        return;
    };
    for event in events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        blink.reset();
        vis.visible = true;

        match event.logical_key {
            Key::Escape => {
                commands.entity(e).remove::<Focused>();
            }
            Key::Enter => {}
            Key::ArrowLeft if textedit.cursor > 0 => {
                for _ in 0..4 {
                    textedit.cursor -= 1;
                    if textedit.text.is_char_boundary(textedit.cursor) {
                        break;
                    }
                }
            }
            Key::ArrowRight if textedit.cursor < textedit.text.len() => {
                for _ in 0..4 {
                    textedit.cursor += 1;
                    if textedit.text.is_char_boundary(textedit.cursor) {
                        break;
                    }
                }
            }
            Key::Home if textedit.cursor > 0 => {
                textedit.cursor = 0;
            }
            Key::End if textedit.cursor < textedit.text.len() => {
                textedit.cursor = textedit.text.len();
            }
            Key::Backspace if textedit.cursor > 0 => {
                for _ in 0..4 {
                    textedit.cursor -= 1;
                    if textedit.text.is_char_boundary(textedit.cursor) {
                        break;
                    }
                }

                let idx = textedit.cursor;
                textedit.text.remove(idx);
            }
            Key::Delete if textedit.cursor < textedit.text.len() => {
                let idx = textedit.cursor;
                textedit.text.remove(idx);
            }
            _ => {}
        }
    }
}

fn update_textedit(
    q: Query<&TextEditComponent, Changed<TextEditComponent>>,
    mut tq: Query<&mut Text>,
    mut cq: Query<&mut Style>,
    fonts: Res<Assets<Font>>,
) {
    for textedit in &q {
        let mut text = tq.get_mut(textedit.text_entity).unwrap();
        let mut cursor = cq.get_mut(textedit.cursor_entity).unwrap();

        let section = &mut text.sections[0];
        section.value = textedit.text.clone();
        let Some(font) = fonts.get(&section.style.font) else {
            println!("NOT YET LOADED");
            continue;
        };
        let scale = section.style.font_size;
        let scaled_font = font.font.as_scaled(scale);

        let mut x = 0.0;

        for c in text.sections[0].value[..textedit.cursor].chars() {
            let glyph = scaled_font.glyph_id(c);
            x += scaled_font.h_advance(glyph);
        }

        cursor.left = Val::Px(x);
    }
}
