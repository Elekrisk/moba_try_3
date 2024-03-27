#![feature(trivial_bounds)]
mod components;

use std::{
    collections::HashMap,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    time::Duration,
};

use bevy::{
    app::ScheduleRunnerPlugin,
    prelude::{Event as BevyEvent, *},
};
use common::{
    network::{
        lobby::{
            LobbyClientMessage, LobbyClientNewConnectionMessage, LobbyInfo, LobbyServerMessage,
            PlayerWithSide, ShortLobbyInfo,
        },
        TcpStreamExt,
    },
    Side,
};
use components::{InLobby, Lobby, Player, PlayerRef};
use uuid::Uuid;

enum Event {
    NewConnection(NewConnection),
    ConnectionBroken(ConnectionBroken),
    LobbyListRequested(LobbyListRequested),
    LobbyCreationRequested(LobbyCreationRequested),
    LobbyJoiningRequested(LobbyJoiningRequested),
    LobbyLeavingRequested(LobbyLeavingRequested),
    LobbyInfoRequested(LobbyInfoRequested),
}

struct EventChannel {
    channel: Receiver<Event>,
}

fn main() {
    let (send_event, recv_event) = mpsc::channel();

    std::thread::spawn(|| listen_for_incoming_requests(send_event));
    App::new()
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0,
            ))),
        )
        .add_event::<NewConnection>()
        .add_event::<ConnectionBroken>()
        .add_event::<LobbyListRequested>()
        .add_event::<LobbyCreationRequested>()
        .add_event::<LobbyLeavingRequested>()
        .add_event::<LobbyJoiningRequested>()
        .add_event::<LobbyInfoRequested>()
        .insert_non_send_resource(EventChannel {
            channel: recv_event,
        })
        .init_resource::<ClientMapping>()
        .add_systems(
            Update,
            (
                event_listener_system,
                new_connection_system,
                connection_broken_system,
                lobby_list_requested_system,
                lobby_creation_requested_system,
                lobby_joining_requested,
                lobby_leaving_requested_system,
                lobby_info_requested_system,
            ),
        )
        .run();
}

fn event_listener_system(
    channel: NonSend<EventChannel>,
    mut new_connection: EventWriter<NewConnection>,
    mut connection_broken: EventWriter<ConnectionBroken>,
    mut lobby_list_requested: EventWriter<LobbyListRequested>,
    mut lobby_creation_requested: EventWriter<LobbyCreationRequested>,
    mut lobby_leaving_requested: EventWriter<LobbyLeavingRequested>,
    mut lobby_joining_requested: EventWriter<LobbyJoiningRequested>,
    mut lobby_info_requested: EventWriter<LobbyInfoRequested>,
) {
    match channel.channel.try_recv() {
        Ok(msg) => match msg {
            Event::NewConnection(event) => {
                new_connection.send(event);
            }
            Event::ConnectionBroken(event) => {
                connection_broken.send(event);
            }
            Event::LobbyListRequested(event) => {
                lobby_list_requested.send(event);
            }
            Event::LobbyCreationRequested(event) => {
                lobby_creation_requested.send(event);
            }
            Event::LobbyLeavingRequested(event) => {
                lobby_leaving_requested.send(event);
            }
            Event::LobbyJoiningRequested(event) => {
                lobby_joining_requested.send(event);
            }
            Event::LobbyInfoRequested(event) => {
                lobby_info_requested.send(event);
            }
        },
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            panic!("Event channel disconnected");
        }
    }
}

fn new_connection_system(
    mut reader: EventReader<NewConnection>,
    mut mapper: ResMut<ClientMapping>,
    mut commands: Commands,
) {
    for NewConnection {
        id,
        username,
        sender,
    } in reader.read()
    {
        println!("Spawning new player {} with id {}", username, id);
        let player_entity = commands
            .spawn(Player {
                id: *id,
                username: username.clone(),
            })
            .id();
        mapper.map.insert(
            *id,
            RemoteClient {
                sender: sender.clone(),
                player_entity,
            },
        );
    }
}

fn connection_broken_system(
    mut reader: EventReader<ConnectionBroken>,
    mut mapper: ResMut<ClientMapping>,
    player_query: Query<(&Player, &InLobby)>,
    mut lobby_query: Query<(Entity, &mut Lobby)>,
    mut commands: Commands,
) {
    for ConnectionBroken { id } in reader.read() {
        println!("Connection with {id} broken");
        let id = *id;

        let client = mapper.map.get(&id).unwrap();
        if let Ok((_, lobby_ref)) = player_query.get(client.player_entity) {
            let (ent, mut lobby) = lobby_query.get_mut(lobby_ref.entity).unwrap();

            let pos = lobby.players.iter().position(|p| p.id == id).unwrap();
            lobby.players.remove(pos);

            if lobby.players.is_empty() {
                commands.entity(ent).despawn();
            }
        }

        commands.entity(client.player_entity).despawn();
        mapper.map.remove(&id);
    }
}

fn lobby_list_requested_system(
    mut reader: EventReader<LobbyListRequested>,
    query: Query<&Lobby>,
    mapper: Res<ClientMapping>,
) {
    for event in reader.read() {
        let lobbies = query
            .iter()
            .map(|l| ShortLobbyInfo {
                lobby_id: l.id,
                players: l.players.len(),
            })
            .collect();
        if let Some(client) = mapper.map.get(&event.id) {
            client
                .sender
                .send(LobbyServerMessage::LobbyList { lobbies })
                .unwrap();
        }
    }
}

fn lobby_creation_requested_system(
    mut reader: EventReader<LobbyCreationRequested>,
    mut lobby_info_writer: EventWriter<LobbyInfoRequested>,
    player_query: Query<&Player>,
    mapper: Res<ClientMapping>,
    mut commands: Commands,
) {
    for event in reader.read() {
        let client = mapper.map.get(&event.id).unwrap();

        let lobby_id = Uuid::new_v4();
        let ent = commands
            .spawn(Lobby {
                id: lobby_id,
                players: vec![PlayerRef {
                    id: event.id,
                    entity: client.player_entity,
                }],
            })
            .id();

        commands.entity(client.player_entity).insert(InLobby {
            id: lobby_id,
            entity: ent,
        });

        client.sender.send(LobbyServerMessage::OK).unwrap();
        client
            .sender
            .send(LobbyServerMessage::YouJoinedLobby { lobby_id })
            .unwrap();
        client.sender.send(LobbyServerMessage::LobbyInfo {
            info: LobbyInfo {
                lobby_id,
                players: vec![PlayerWithSide {
                    player_id: event.id,
                    username: player_query.get(client.player_entity).unwrap().username.clone(),
                    side: Side::Blue,
                }],
            },
        }).unwrap();
    }
}

fn lobby_joining_requested(
    mut reader: EventReader<LobbyJoiningRequested>,
    mut lobby_info_writer: EventWriter<LobbyInfoRequested>,
    player_query: Query<&Player>,
    in_lobby_query: Query<&InLobby>,
    mut lobby_query: Query<(Entity, &mut Lobby)>,
    mapper: Res<ClientMapping>,
    mut commands: Commands,
) {
    for event in reader.read() {
        let client = mapper.map.get(&event.id).unwrap();

        if in_lobby_query.get(client.player_entity).is_ok() {
            client
                .sender
                .send(LobbyServerMessage::Negative {
                    msg: "Cannot join a lobby while you are in another".to_string(),
                })
                .unwrap();
            continue;
        }

        let Some((ent, mut lobby)) = lobby_query.iter_mut().find(|(_, l)| l.id == event.lobby_id)
        else {
            client
                .sender
                .send(LobbyServerMessage::Negative {
                    msg: "Cannot join non-existant lobby".to_string(),
                })
                .unwrap();
            continue;
        };

        lobby.players.push(PlayerRef {
            id: event.id,
            entity: client.player_entity,
        });
        commands.entity(client.player_entity).insert(InLobby {
            id: event.lobby_id,
            entity: ent,
        });

        client.sender.send(LobbyServerMessage::OK).unwrap();
        client
            .sender
            .send(LobbyServerMessage::YouJoinedLobby {
                lobby_id: event.lobby_id,
            })
            .unwrap();

        for player in &lobby.players {
            lobby_info_writer.send(LobbyInfoRequested {
                id: player.id,
                lobby_id: event.lobby_id,
            });
        }
    }
}

fn lobby_leaving_requested_system(
    mut reader: EventReader<LobbyLeavingRequested>,
    mut lobby_info_writer: EventWriter<LobbyInfoRequested>,
    player_query: Query<(&Player, &InLobby)>,
    mut lobby_query: Query<(Entity, &mut Lobby)>,
    mapper: Res<ClientMapping>,
    mut commands: Commands,
) {
    for event in reader.read() {
        let client = mapper.map.get(&event.id).unwrap();

        if let Ok((player, in_lobby)) = player_query.get(client.player_entity) {
            let Ok((ent, mut lobby)) = lobby_query.get_mut(in_lobby.entity) else {
                panic!()
            };
            let pos = lobby.players.iter().position(|p| p.id == event.id).unwrap();
            lobby.players.remove(pos);

            for player in &lobby.players {
                lobby_info_writer.send(LobbyInfoRequested {
                    id: player.id,
                    lobby_id: in_lobby.id,
                });
            }

            if lobby.players.is_empty() {
                commands.entity(ent).despawn();
            }
        }

        commands.entity(client.player_entity).remove::<InLobby>();

        client.sender.send(LobbyServerMessage::OK).unwrap();
        client
            .sender
            .send(LobbyServerMessage::YouLeftLobby)
            .unwrap();
    }
}

fn lobby_info_requested_system(
    mut reader: EventReader<LobbyInfoRequested>,
    player_query: Query<&Player>,
    mapper: Res<ClientMapping>,
    lobby_query: Query<&Lobby>,
) {
    for event in reader.read() {
        let client = mapper.map.get(&event.id).unwrap();

        if let Some(lobby) = lobby_query.iter().find(|l| l.id == event.lobby_id) {
            let info = LobbyInfo {
                lobby_id: event.lobby_id,
                players: lobby
                    .players
                    .iter()
                    .map(|p| PlayerWithSide {
                        player_id: p.id,
                        username: player_query.get(p.entity).unwrap().username.clone(),
                        side: Side::Blue,
                    })
                    .collect(),
            };

            client
                .sender
                .send(LobbyServerMessage::LobbyInfo { info })
                .unwrap();
        }
    }
}

fn listen_for_incoming_requests(send_new_connection: Sender<Event>) {
    let listener = TcpListener::bind("[::]:65432").unwrap();

    loop {
        let (stream, addr) = listener.accept().unwrap();
        let id = Uuid::new_v4();
        println!("Connection received from {addr} with id {id}");
        let sender = send_new_connection.clone();
        std::thread::spawn(move || handle_connection(stream, addr, id, sender));
    }
}

fn handle_connection(mut stream: TcpStream, addr: SocketAddr, id: Uuid, send_event: Sender<Event>) {
    let new_user_message =
        stream.read_message::<LobbyClientNewConnectionMessage>(Some(Duration::from_secs(3)));

    let new_user_message = match new_user_message {
        Ok(msg) => {
            println!("Received new client connection message: {msg:?}");
            msg
        }
        Err(e) => {
            eprintln!("Error receiving new connection message: {e}");
            return;
        }
    };

    let (send_server_msg, recv_server_msg) = mpsc::channel();

    send_event
        .send(Event::NewConnection(NewConnection {
            id,
            username: new_user_message.username.clone(),
            sender: send_server_msg,
        }))
        .unwrap();
    {
        let stream = stream.try_clone().unwrap();
        std::thread::spawn(move || sender_thread(stream, id, recv_server_msg));
    }

    loop {
        let msg = match stream.read_message::<LobbyClientMessage>(None) {
            Ok(msg) => msg,
            Err(e) => {
                break;
            }
        };

        let event = match msg {
            LobbyClientMessage::StartMatchmaking => todo!(),
            LobbyClientMessage::StopMatchmaking => todo!(),
            LobbyClientMessage::CreateLobby => {
                Event::LobbyCreationRequested(LobbyCreationRequested { id })
            }
            LobbyClientMessage::ListLobbies => Event::LobbyListRequested(LobbyListRequested { id }),
            LobbyClientMessage::JoinLobby { lobby_id } => {
                Event::LobbyJoiningRequested(LobbyJoiningRequested { id, lobby_id })
            }
            LobbyClientMessage::LeaveLobby => {
                Event::LobbyLeavingRequested(LobbyLeavingRequested { id })
            }
            LobbyClientMessage::GetLobbyInfo { lobby_id } => {
                Event::LobbyInfoRequested(LobbyInfoRequested { id, lobby_id })
            }
            LobbyClientMessage::SwitchSide => todo!(),
            LobbyClientMessage::SelectChampion { champion } => todo!(),
            LobbyClientMessage::LockInChampion { champion } => todo!(),
        };

        send_event.send(event).unwrap();
    }

    // Connection has been broken; send ConnectionBroken event and exit thread
    send_event
        .send(Event::ConnectionBroken(ConnectionBroken { id }))
        .unwrap();
}

fn sender_thread(mut stream: TcpStream, id: Uuid, recv: Receiver<LobbyServerMessage>) {
    loop {
        let Ok(msg) = recv.recv() else { break };
        stream.write_message(&msg).unwrap();
    }
}

#[derive(Resource, Default)]
struct ClientMapping {
    map: HashMap<Uuid, RemoteClient>,
}

struct RemoteClient {
    sender: Sender<LobbyServerMessage>,
    player_entity: Entity,
}

#[derive(BevyEvent)]
struct NewConnection {
    id: Uuid,
    username: String,
    sender: Sender<LobbyServerMessage>,
}

#[derive(BevyEvent)]
struct ConnectionBroken {
    id: Uuid,
}

#[derive(BevyEvent)]
struct LobbyListRequested {
    id: Uuid,
}

#[derive(BevyEvent)]
struct LobbyCreationRequested {
    id: Uuid,
}

#[derive(BevyEvent)]
struct LobbyLeavingRequested {
    id: Uuid,
}

#[derive(BevyEvent)]
struct LobbyJoiningRequested {
    id: Uuid,
    lobby_id: Uuid,
}

#[derive(BevyEvent)]
struct LobbyInfoRequested {
    id: Uuid,
    lobby_id: Uuid,
}
