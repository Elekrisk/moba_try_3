mod network;

use std::{
    marker::ConstParamTy,
    net::{SocketAddr, TcpStream},
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
};

use bevy::{app::AppExit, ecs::system::RunSystemOnce, prelude::*, utils::HashMap};
use common::network::TcpStreamExt;

use crate::ui::{
    view::{self, ButtonAction},
    UiRoot, UiRootComponent, View,
};

use self::network::{
    JoinedLobby, LeftLobby, ServerConnectionStatus, UpdateLobbyInfo, UpdateLobbyList,
};

pub struct NonGame {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
enum MenuId {
    ConnectToServer,
    ConnectingToServer,
    ConnectionToServerFailed,
    Main,
    LobbyList,
    InLobby,
}

#[derive(Resource, Clone, Default)]
struct MenuEntities {
    active_menu: Option<MenuId>,
    entities: HashMap<MenuId, Entity>,
}

impl MenuEntities {
    fn new() -> Self {
        Self {
            active_menu: None,
            entities: HashMap::new(),
        }
    }

    fn get(&self, id: MenuId) -> Entity {
        *self.entities.get(&id).unwrap()
    }

    fn set(&mut self, id: MenuId, entity: Entity) {
        self.entities.insert(id, entity);
    }
}

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
            .add_event::<JoinedLobby>()
            .add_event::<LeftLobby>()
            .add_event::<ConnectToServer>();

        app.init_non_send_resource::<EventChannel>()
            .init_resource::<RequestChannel>();

        app.add_systems(Update, event_channel_listener);

        app.add_systems(
            Update,
            (
                connect_to_server_system.run_if(in_state(NonGameState::ConnectToServer)),
                connected_to_server.run_if(in_state(NonGameState::ConnectingToServer)),
                joined_lobby.run_if(in_state(NonGameState::LobbyMenu)),
                left_lobby.run_if(in_state(NonGameState::InLobby)),
            ),
        );

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(UiRootComponent(Box::new(UiRoot::new(
                0,
                connect_to_server_menu,
            ))));
        });

        app.insert_state(NonGameState::ConnectToServer);

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera3dBundle { ..default() });
        });
    }
}

fn connect_to_server_system(
    mut events: EventReader<ConnectToServer>,
    mut next_state: ResMut<NextState<NonGameState>>,
    mut event_channel: NonSendMut<EventChannel>,
    mut request_channel: ResMut<RequestChannel>,
) {
    let events = events.read().collect::<Vec<_>>();
    let &ConnectToServer(addr) = match &events[..] {
        [] => return,
        [event] => *event,
        _ => panic!("Missed connection events"),
    };

    let (send_event, recv_event) = mpsc::channel();
    let (send_request, recv_request) = mpsc::channel();

    next_state.set(NonGameState::ConnectingToServer);

    event_channel.channel = Some(recv_event);
    request_channel.channel = Some(send_request);

    std::thread::spawn(move || network::connect_to_server(addr, send_event, recv_request));
}

fn connected_to_server(
    mut reader: EventReader<ServerConnectionStatus>,
    mut next_state: ResMut<NextState<NonGameState>>,
) {
    for event in reader.read() {
        match event {
            ServerConnectionStatus::Connected => next_state.set(NonGameState::MainMenu),
            ServerConnectionStatus::ConnectionFailed => {
                next_state.set(NonGameState::ConnectToServer)
            }
        }
    }
}

fn joined_lobby(
    mut reader: EventReader<JoinedLobby>,
    mut next_state: ResMut<NextState<NonGameState>>,
) {
    for event in reader.read() {
        next_state.set(NonGameState::InLobby);
    }
}

fn left_lobby(mut reader: EventReader<LeftLobby>, mut next_state: ResMut<NextState<NonGameState>>) {
    for event in reader.read() {
        next_state.set(NonGameState::LobbyMenu);
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, States)]
pub enum NonGameState {
    ConnectToServer,
    ConnectingToServer,
    ConnectionToServerFailed,
    MainMenu,
    LobbyMenu,
    Matchmaking,
    InLobby,
}

fn event_channel_listener(
    event_channel: NonSend<EventChannel>,
    mut connected_to_server: EventWriter<ServerConnectionStatus>,
    mut update_lobby_list: EventWriter<UpdateLobbyList>,
    mut update_lobby_info: EventWriter<UpdateLobbyInfo>,
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
        network::Event::JoinedLobby(event) => {
            joined_lobby.send(event);
        }
        network::Event::LeftLobby(event) => {
            left_lobby.send(event);
        }
    }
}

#[derive(Clone, Event)]
pub struct ConnectToServer(SocketAddr);

fn connect_to_server_menu(state: &mut i32) -> impl View<i32> {
    view::Stack::new(FlexDirection::Column)
        .with_child(
            view::Button::new("Increase")
                .with_action(ButtonAction::ModifyState(Box::new(|s| *s += 1))),
        )
        .with_child(view::Label::new(*state))
        .with_child(
            view::Button::new("Decrease")
                .with_action(ButtonAction::ModifyState(Box::new(|s| *s -= 1))),
        )
}
