use bevy::prelude::*;
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::On,
};

use crate::nongame::network::{JoinedLobby, Request, UpdateLobbyList};

use super::{lobby::CurrentLobby, LobbyState, MenuHolder};

pub struct LobbyListPlugin;

impl Plugin for LobbyListPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(LobbyState::NotInLobby),
            (make_lobby_list_menu, |mut e: EventWriter<Request>| {
                e.send(Request::GetLobbyList);
            }),
        )
        .add_systems(
            Update,
            (update_lobby_list, lobby_joined).run_if(in_state(LobbyState::NotInLobby)),
        );
    }
}

#[derive(Component)]
pub struct LobbyList;

fn make_lobby_list_menu(
    asset_server: Res<AssetServer>,
    q: Query<Entity, With<MenuHolder>>,
    mut commands: Commands,
) {
    let menu_holder = q.single();
    commands.entity(menu_holder).despawn_descendants();

    let font = asset_server.load("fonts/Roboto-Light.ttf");

    let text_style = TextStyle {
        font,
        font_size: 16.0,
        color: Color::GOLD,
    };

    let button_img = asset_server.load("ui/button.png");

    let create_lobby_button = commands
        .spawn((
            ButtonBundle {
                style: Style {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                    ..default()
                },
                image: button_img.clone().into(),
                ..default()
            },
            ImageScaleMode::Sliced(TextureSlicer {
                border: BorderRect::square(16.0),
                ..default()
            }),
            On::<Pointer<Click>>::run(|mut e: EventWriter<Request>| {
                println!("CREATE");
                e.send(Request::CreateLobby);
            }),
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section("Create Lobby", text_style.clone()),
                ..default()
            });
        })
        .id();

    let refresh_lobbies_button = commands
        .spawn((
            ButtonBundle {
                style: Style {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                    ..default()
                },
                image: button_img.into(),
                ..default()
            },
            ImageScaleMode::Sliced(TextureSlicer {
                border: BorderRect::square(16.0),
                ..default()
            }),
            On::<Pointer<Click>>::run(|mut e: EventWriter<Request>| {
                e.send(Request::GetLobbyList);
            }),
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section("Refresh Lobbies", text_style.clone()),
                ..default()
            });
        })
        .id();

    let buttonbar = commands
        .spawn(NodeBundle {
            style: Style { ..default() },
            ..default()
        })
        .push_children(&[create_lobby_button, refresh_lobbies_button])
        .id();

    let lobby_name_header = commands
        .spawn(TextBundle {
            text: Text::from_section("Lobby Name", text_style.clone()),
            style: Style {
                flex_grow: 1.0,
                ..default()
            },
            ..default()
        })
        .id();

    let lobby_player_count_header = commands
        .spawn(TextBundle {
            text: Text::from_section("Player Count", text_style.clone()),
            ..default()
        })
        .id();

    let lobby_join_button_header = commands.spawn(NodeBundle { ..default() }).id();

    let lobby_list_header = commands
        .spawn(NodeBundle {
            style: Style { ..default() },
            ..default()
        })
        .push_children(&[
            lobby_name_header,
            lobby_player_count_header,
            lobby_join_button_header,
        ])
        .id();

    let lobby_list = commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    ..default()
                },
                ..default()
            },
            LobbyList,
        ))
        .id();

    let root = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                ..default()
            },
            ..default()
        })
        .push_children(&[buttonbar, lobby_list_header, lobby_list])
        .id();

    commands.entity(menu_holder).add_child(root);
}

fn update_lobby_list(
    mut events: EventReader<UpdateLobbyList>,
    query: Query<Entity, With<LobbyList>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let font = asset_server.load("fonts/Roboto-Light.ttf");

    let text_style = TextStyle {
        font,
        font_size: 16.0,
        color: Color::GOLD,
    };

    let button_img = asset_server.load("ui/button.png");

    for event in events.read() {
        let Ok(e) = query.get_single() else { continue };

        commands.entity(e).despawn_descendants();

        for lobby in &event.lobbies {
            let name = commands
                .spawn(TextBundle {
                    text: Text::from_section(format!("Lobby {}", lobby.id), text_style.clone()),
                    style: Style {
                        flex_grow: 1.0,
                        ..default()
                    },
                    ..default()
                })
                .id();

            let players = commands
                .spawn(TextBundle {
                    text: Text::from_section(format!("{}", lobby.players), text_style.clone()),
                    ..default()
                })
                .id();

            let lobby_id = lobby.id;
            let join = commands
                .spawn((
                    ButtonBundle {
                        style: Style {
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                            ..default()
                        },
                        image: button_img.clone().into(),
                        ..default()
                    },
                    ImageScaleMode::Sliced(TextureSlicer {
                        border: BorderRect::square(16.0),
                        ..default()
                    }),
                    On::<Pointer<Click>>::run(move |mut e: EventWriter<Request>| {
                        e.send(Request::JoinLobby { id: lobby_id });
                    }),
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section("Join", text_style.clone()),
                        ..default()
                    });
                })
                .id();

            let entry = commands
                .spawn(NodeBundle { ..default() })
                .push_children(&[name, players, join])
                .id();

            commands.entity(e).push_children(&[entry]);
        }
    }
}

fn lobby_joined(
    mut e: EventReader<JoinedLobby>,
    mut next_state: ResMut<NextState<LobbyState>>,
    mut commands: Commands,
) {
    for e in e.read() {
        next_state.set(LobbyState::InLobby);
        commands.insert_resource(CurrentLobby(e.lobby_id));
    }
}
