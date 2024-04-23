use std::{
    net::{SocketAddr, TcpStream},
    sync::mpsc::{Receiver, Sender},
};

use common::{network::{
    lobby::{
        LobbyClientMessage, LobbyClientNewConnectionMessage, LobbyId, LobbyInfo, LobbyServerMessage, Player, PlayerWithSide, ShortLobbyInfo
    },
    TcpStreamExt,
}, Side};
use uuid::Uuid;

use bevy::prelude::Event as BevyEvent;

#[derive(Debug, Clone, BevyEvent)]
pub enum Request {
    GetLobbyList,
    CreateLobby,
    GetLobbyInfo { id: LobbyId },
    JoinLobby { id: LobbyId },
    LeaveLobby,
}

pub enum Event {
    ServerConnectionStatus(ServerConnectionStatus),
    UpdateLobbyList(UpdateLobbyList),
    UpdateLobbyInfo(UpdateLobbyInfo),
    PlayerJoinedLobby(PlayerJoinedLobby),
    PlayerLeftLobby(PlayerLeftLobby),
    JoinedLobby(JoinedLobby),
    LeftLobby(LeftLobby),
}

#[derive(BevyEvent)]
pub enum ServerConnectionStatus {
    Connected,
    ConnectionFailed,
}

#[derive(BevyEvent)]
pub struct UpdateLobbyList {
    pub lobbies: Vec<ShortLobbyInfo>,
}

#[derive(BevyEvent)]
pub struct UpdateLobbyInfo {
    pub lobby_info: LobbyInfo,
}

#[derive(BevyEvent)]
pub struct PlayerJoinedLobby {
    pub player: Player,
    pub side: Side,
}

#[derive(BevyEvent)]
pub struct PlayerLeftLobby {
    pub player: Player,
}

#[derive(BevyEvent)]
pub struct JoinedLobby {
    pub lobby_id: LobbyId,
}

#[derive(BevyEvent)]
pub struct LeftLobby;

pub fn connect_to_server(
    addr: SocketAddr,
    send_event: Sender<Event>,
    recv_request: Receiver<Request>,
) {
    println!("Connecting to server...");
    let mut stream = match TcpStream::connect(addr) {
        Ok(stream) => {
            send_event
                .send(Event::ServerConnectionStatus(
                    ServerConnectionStatus::Connected,
                ))
                .unwrap();
            stream
        }
        Err(e) => {
            send_event
                .send(Event::ServerConnectionStatus(
                    ServerConnectionStatus::ConnectionFailed,
                ))
                .unwrap();
            eprintln!("Connection failed: {e}");
            return;
        }
    };
    println!("Connected!");
    stream
        .write_message(&LobbyClientNewConnectionMessage {
            username: "Guest".to_string(),
        })
        .unwrap();

    {
        let stream = stream.try_clone().unwrap();
        std::thread::spawn(move || event_listener(send_event, stream));
    }
    event_sender(recv_request, stream);
}

pub fn event_sender(recv_request: Receiver<Request>, mut stream: TcpStream) {
    loop {
        println!("Waiting for request to send...");
        let request = recv_request.recv().unwrap();
        println!("Request {request:?} received");

        let msg = match request {
            Request::GetLobbyList => LobbyClientMessage::ListLobbies,
            Request::GetLobbyInfo { id } => LobbyClientMessage::GetLobbyInfo { id },
            Request::JoinLobby { id } => LobbyClientMessage::JoinLobby { id },
            Request::LeaveLobby => LobbyClientMessage::LeaveLobby,
            Request::CreateLobby => LobbyClientMessage::CreateLobby,
        };

        stream.write_message(&msg).unwrap();
    }
}

pub fn event_listener(send_event: Sender<Event>, mut stream: TcpStream) {
    loop {
        println!("Listening for events...");
        let msg = stream.read_message::<LobbyServerMessage>(None).unwrap();
        println!("{msg:?}");
        let event = match msg {
            LobbyServerMessage::OK => None,
            LobbyServerMessage::Negative { msg } => {
                eprintln!("Negative received: {msg}");
                None
            }
            LobbyServerMessage::StopMatchmaking => todo!(),
            LobbyServerMessage::LobbyList { lobbies } => {
                Some(Event::UpdateLobbyList(UpdateLobbyList { lobbies }))
            }
            LobbyServerMessage::LobbyInfo { info } => {
                Some(Event::UpdateLobbyInfo(UpdateLobbyInfo { lobby_info: info }))
            }
            LobbyServerMessage::MatchmakingDone { lobby_id } => todo!(),
            LobbyServerMessage::PlayerJoinedLobby { player, side } => Some(Event::PlayerJoinedLobby(PlayerJoinedLobby { player, side })),
            LobbyServerMessage::PlayerLeftLobby { player } => Some(Event::PlayerLeftLobby(PlayerLeftLobby { player })),
            LobbyServerMessage::PlayerSwitchedSide { player, side } => todo!(),
            LobbyServerMessage::PlayerSelectedChampion { player, champion } => todo!(),
            LobbyServerMessage::PlayerLockedInChampion { player, champion } => todo!(),
            LobbyServerMessage::YouJoinedLobby { lobby_id } => {
                Some(Event::JoinedLobby(JoinedLobby { lobby_id }))
            }
            LobbyServerMessage::YouLeftLobby => Some(Event::LeftLobby(LeftLobby)),
        };

        if let Some(event) = event {
            send_event.send(event).unwrap();
        }
    }
}
