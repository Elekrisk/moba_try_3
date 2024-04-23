use std::time::Duration;

use bevy::{prelude::*, utils::hashbrown::HashMap};
use common::{
    network::lobby::{LobbyId, LobbyInfo, Player as NetworkPlayer, PlayerId},
    Side,
};
use uuid::Uuid;

use crate::{
    nongame::network::{LeftLobby, PlayerJoinedLobby, PlayerLeftLobby, Request, UpdateLobbyInfo},
    ui::{button, stack, Animation, BuildContext, Widget, WidgetExt},
};

use super::{LobbyState, MenuHolder};

pub struct LobbyPlugin;

#[derive(Resource)]
pub struct CurrentLobby(pub LobbyId);

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(LobbyState::InLobby),
            (
                make_lobby_menu,
                enter_main_menu,
                |mut e: EventWriter<Request>, current_lobby: Res<CurrentLobby>| {
                    e.send(Request::GetLobbyInfo {
                        id: current_lobby.0,
                    });
                },
            ),
        )
        .add_systems(OnExit(LobbyState::InLobby), |mut commands: Commands| {
            commands.remove_resource::<CurrentLobby>();
        })
        .add_systems(
            Update,
            (new_lobby_info, player_joined, player_left, you_left)
                .run_if(in_state(LobbyState::InLobby)),
        );
    }
}

fn enter_main_menu(mut commands: Commands) {
    commands.insert_resource(State {
        info: LobbyInfo {
            id: LobbyId(Uuid::nil()),
            players: HashMap::new(),
            lobby_owner: PlayerId(Uuid::nil()),
        },
    });
}

#[derive(Resource)]
pub struct State {
    info: LobbyInfo,
}

#[derive(Component)]
pub struct MyPlayer {
    pub id: Uuid,
}

#[derive(Component)]
pub struct Player {
    pub id: Uuid,
}

#[derive(Component)]
struct PlayerList(Side);

fn make_lobby_menu(
    asset_server: Res<AssetServer>,
    q: Query<Entity, With<MenuHolder>>,
    mut commands: Commands,
) {
    let menu_holder = q.single();
    commands.entity(menu_holder).despawn_descendants();

    let mut root = stack(FlexDirection::Column);

    let lobby_title = "Lobby".to_string();

    let mut teams = stack(FlexDirection::Row);

    fn mk_team(team: Side) -> impl Widget {
        stack(FlexDirection::Column)
            .styled(|s| {
                s.flex_grow = 1.0;
            })
            .insert(PlayerList(team))
    }

    for side in Side::ALL {
        teams.add(mk_team(side));
    }

    root.add(lobby_title);
    root.add(teams.styled(|s| {
        s.width = Val::Percent(95.0);
    }));

    let root = root
        .styled(|s| {
            s.width = Val::Percent(100.0);
            s.height = Val::Percent(100.0);
            s.align_items = AlignItems::Center;
            s.row_gap = Val::Px(5.0);
        })
        .build(&mut BuildContext {
            asset_server: &asset_server,
            commands: &mut commands,
        });

    commands.entity(menu_holder).add_child(root);

    // let text_style = TextStyle {
    //     font: asset_server.load("fonts/Roboto-Light.ttf"),
    //     font_size: 16.0,
    //     color: Color::GOLD,
    // };

    // let lobby_title = commands
    //     .spawn(TextBundle {
    //         text: Text::from_section("Lobby", text_style.clone()),
    //         ..default()
    //     })
    //     .id();

    // let player_list = commands
    //     .spawn((NodeBundle { ..default() }, PlayerList))
    //     .id();

    // let root = commands
    //     .spawn(NodeBundle {
    //         style: Style {
    //             flex_direction: FlexDirection::Column,
    //             ..default()
    //         },
    //         ..default()
    //     })
    //     .push_children(&[lobby_title, player_list])
    //     .id();

    // commands.entity(menu_holder).add_child(root);
}

#[derive(Component)]
struct PlayerSlot(PlayerId);

fn make_player_slot(
    player: &NetworkPlayer,
    fade_side: f32,
    asset_server: &AssetServer,
    commands: &mut Commands,
) -> Entity {
    stack(FlexDirection::Row)
        .with(player.username.as_str())
        .with(button("Kick", |mut e: EventWriter<Request>| {
            e.send(Request::LeaveLobby);
        }))
        .styled(move |s| {
            s.justify_content = JustifyContent::SpaceBetween;
            s.width = Val::Percent(100.0);
            s.padding = UiRect::axes(Val::Px(5.0), Val::Px(5.0));
            s.left = Val::Percent(100.0 * fade_side);
        })
        .insert((
            PlayerSlot(player.id),
            BackgroundColor(Color::BLUE),
            Animation::new(
                move |style: &mut Style, t| {
                    let start_left = 100.0 * fade_side;
                    let end_left = 0.0;
                    style.left = Val::Percent(start_left.lerp(end_left, t));
                },
                crate::ui::Easing::Custom(Box::new(|t| 1.0 - (t - 1.0) * (t - 1.0))),
                Duration::from_secs_f32(0.5),
            ),
        ))
        .build(&mut BuildContext {
            asset_server,
            commands,
        })

    // let root = commands
    //     .spawn((
    //         NodeBundle {
    //             background_color: Color::FUCHSIA.into(),
    //             ..default()
    //         },
    //         PlayerSlot(player.id),
    //     ))
    //     .with_children(|parent| {
    //         parent.spawn(TextBundle {
    //             text: Text::from_section(
    //                 &player.username,
    //                 TextStyle {
    //                     font_size: 16.0,
    //                     color: Color::GOLD,
    //                     ..default()
    //                 },
    //             ),
    //             ..default()
    //         });
    //     })
    //     .id();

    // root
}

fn new_lobby_info(
    mut e: EventReader<UpdateLobbyInfo>,
    mut state: ResMut<State>,
    q: Query<(Entity, &PlayerList)>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for e in e.read() {
        state.info = e.lobby_info.clone();

        for (e, PlayerList(side)) in &q {
            commands.entity(e).despawn_descendants();

            let mut slots = vec![];

            for player in state.info.players.get(side).unwrap_or(&vec![]) {
                let fade_side = match side {
                    Side::Red => -1.0,
                    Side::Blue => 1.0,
                };

                slots.push(make_player_slot(
                    player,
                    fade_side,
                    &asset_server,
                    &mut commands,
                ));
            }

            commands.entity(e).push_children(&slots);
        }
    }
}

fn player_joined(
    mut e: EventReader<PlayerJoinedLobby>,
    mut state: ResMut<State>,
    q: Query<(Entity, &PlayerList)>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for ev in e.read() {
        state
            .info
            .players
            .entry(ev.side)
            .or_default()
            .push(ev.player.clone());

        let e = q
            .iter()
            .find_map(|(e, PlayerList(s))| (*s == ev.side).then_some(e))
            .unwrap();

        let fade_side = match ev.side {
            Side::Red => -1.0,
            Side::Blue => 1.0,
        };

        let slot = make_player_slot(&ev.player, fade_side, &asset_server, &mut commands);
        commands.entity(e).add_child(slot);
    }
}

fn player_left(
    mut e: EventReader<PlayerLeftLobby>,
    mut state: ResMut<State>,
    q: Query<(Entity, &PlayerSlot)>,
    mut commands: Commands,
) {
    for ev in e.read() {
        let mut side = Side::Red;
        for (s, players) in state.info.players.iter_mut() {
            let Some(pos) = players.iter().position(|p| p.id == ev.player.id) else {
                continue;
            };
            side = *s;
            players.remove(pos);
        }

        let e = q
            .iter()
            .find_map(|(e, &PlayerSlot(id))| (id == ev.player.id).then_some(e))
            .unwrap();

        let fade_side = match side {
            Side::Red => -1.0,
            Side::Blue => 1.0,
        };

        commands.entity(e).add(move |mut e: EntityWorldMut<'_>| {
            e.insert(Animation::new(
                move |comp: &mut Style, t| {
                    let min = 0.0;
                    let max = 100.0 * fade_side;
                    comp.left = Val::Percent(min.lerp(max, t));
                },
                crate::ui::Easing::Custom(Box::new(|t| t * t)),
                Duration::from_secs_f32(0.5),
            ).on_finish(|e: In<Entity>, world: &mut World| {
                world.entity_mut(*e).despawn_recursive();
            }));
        });
    }
}

fn you_left(mut e: EventReader<LeftLobby>, mut next_state: ResMut<NextState<LobbyState>>) {
    for _ in e.read() {
        next_state.set(LobbyState::NotInLobby);
    }
}
