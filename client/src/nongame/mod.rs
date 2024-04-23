mod connect_to_server;
mod connecting_to_server;
mod main_menu;
mod network;

use std::sync::mpsc::{Receiver, Sender, TryRecvError};

use bevy::{app::AppExit, prelude::*};

use crate::DEBUG;

use self::{
    connect_to_server::{ConnectToServer, InConnectToServerPlugin},
    connecting_to_server::InConnectingToServerPlugin,
    main_menu::MainMenuPlugin,
    network::{
        JoinedLobby, LeftLobby, PlayerJoinedLobby, PlayerLeftLobby, Request,
        ServerConnectionStatus, UpdateLobbyInfo, UpdateLobbyList,
    },
};

pub struct NonGame {}

#[derive(Resource, Default)]
struct RequestChannel {
    channel: Option<Sender<network::Request>>,
}

#[derive(Default)]
struct EventChannel {
    channel: Option<Receiver<network::Event>>,
}

impl Plugin for NonGame {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<network::Request>()
            .add_event::<ServerConnectionStatus>()
            .add_event::<UpdateLobbyList>()
            .add_event::<UpdateLobbyInfo>()
            .add_event::<PlayerJoinedLobby>()
            .add_event::<PlayerLeftLobby>()
            .add_event::<JoinedLobby>()
            .add_event::<LeftLobby>();

        app.init_non_send_resource::<EventChannel>()
            .init_resource::<RequestChannel>();

        app.add_systems(Update, (event_channel_listener, request_channel_listener));

        app.insert_state(ConnectingState::NotConnected);

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera3dBundle { ..default() });
        });

        app.add_plugins((
            InConnectToServerPlugin,
            InConnectingToServerPlugin,
            MainMenuPlugin,
        ));
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, States)]
pub enum ConnectingState {
    NotConnected,
    Connecting,
    ConnectionFailed,
    Connected,
}

#[allow(clippy::too_many_arguments)]
fn event_channel_listener(
    event_channel: NonSend<EventChannel>,
    mut connected_to_server: EventWriter<ServerConnectionStatus>,
    mut update_lobby_list: EventWriter<UpdateLobbyList>,
    mut update_lobby_info: EventWriter<UpdateLobbyInfo>,
    mut player_joined_lobby: EventWriter<PlayerJoinedLobby>,
    mut player_left_lobby: EventWriter<PlayerLeftLobby>,
    mut joined_lobby: EventWriter<JoinedLobby>,
    mut left_lobby: EventWriter<LeftLobby>,
) {
    let Some(channel) = event_channel.channel.as_ref() else {
        return;
    };

    match match channel.try_recv() {
        Ok(event) => event,
        Err(TryRecvError::Empty) => return,
        Err(TryRecvError::Disconnected) => return,
    } {
        network::Event::ServerConnectionStatus(event) => {
            connected_to_server.send(event);
        }
        network::Event::UpdateLobbyList(event) => {
            update_lobby_list.send(event);
        }
        network::Event::UpdateLobbyInfo(event) => {
            update_lobby_info.send(event);
        }
        network::Event::PlayerJoinedLobby(event) => {
            player_joined_lobby.send(event);
        }
        network::Event::PlayerLeftLobby(event) => {
            player_left_lobby.send(event);
        }
        network::Event::JoinedLobby(event) => {
            joined_lobby.send(event);
        }
        network::Event::LeftLobby(event) => {
            left_lobby.send(event);
        }
    }
}

fn request_channel_listener(
    mut events: EventReader<Request>,
    request_channel: Res<RequestChannel>,
) {
    for event in events.read() {
        request_channel
            .channel
            .as_ref()
            .unwrap()
            .send(event.clone())
            .unwrap();
    }
}

#[derive(Component)]
struct Menu;

fn destroy_menu(mut commands: Commands) {
    commands.add(|w: &mut World| {
        let Ok(e) = w.query_filtered::<Entity, With<Menu>>().get_single(w) else {
            return;
        };
        w.entity_mut(e).despawn_recursive();
    });
}

fn quit(mut e: EventWriter<AppExit>) {
    e.send(AppExit);
}
