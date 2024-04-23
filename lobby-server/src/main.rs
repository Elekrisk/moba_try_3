#![feature(try_blocks)]
use std::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use bevy::utils::HashMap;
use common::{
    network::{
        lobby::{
            LobbyClientMessage, LobbyClientNewConnectionMessage, LobbyId, LobbyInfo,
            LobbyServerMessage, Player as NetworkPlayer, PlayerId, ShortLobbyInfo,
        },
        TcpStreamExt,
    },
    Side,
};
use uuid::Uuid;

fn main() {
    State::new().run();
}

struct Client {
    player_id: PlayerId,
    username: String,
    sender: Sender<LobbyServerMessage>,
    in_lobby: Option<LobbyId>,
}

struct Lobby {
    id: LobbyId,
    players: HashMap<Side, Vec<PlayerId>>,
    owner: PlayerId,
}

enum Command {
    NewClient(Client),
    ClientDisconnected(PlayerId),
    MsgFromClient {
        id: PlayerId,
        msg: LobbyClientMessage,
    },
}

pub struct State {
    players: HashMap<PlayerId, Client>,
    lobbies: HashMap<LobbyId, Lobby>,
}

impl State {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            lobbies: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        let (send, recv) = mpsc::channel();

        std::thread::spawn(|| listen(send));

        loop {
            match recv.recv().unwrap() {
                Command::NewClient(client) => {
                    self.players.insert(client.player_id, client);
                }
                Command::ClientDisconnected(id) => {
                    self.leave_lobby(id);
                    self.players.remove(&id);
                }
                Command::MsgFromClient { id, msg } => {
                    self.handle_message(id, msg);
                }
            }
        }
    }

    fn handle_message(&mut self, player_id: PlayerId, msg: LobbyClientMessage) {
        let client = self.players.get_mut(&player_id).unwrap();
        match msg {
            LobbyClientMessage::StartMatchmaking => todo!(),
            LobbyClientMessage::StopMatchmaking => todo!(),
            LobbyClientMessage::CreateLobby => {
                if client.in_lobby.is_some() {
                    let send = client.sender.send(LobbyServerMessage::Negative {
                        msg: "Cannot create lobby while in one".into(),
                    });
                    return;
                }

                let lobby_id = LobbyId(Uuid::new_v4());
                let lobby = Lobby {
                    id: lobby_id,
                    players: {
                        let mut map = HashMap::new();
                        map.insert(Side::Red, vec![player_id]);
                        map.insert(Side::Blue, vec![]);
                        map
                    },
                    owner: player_id,
                };

                self.lobbies.insert(lobby_id, lobby);
                client.in_lobby = Some(lobby_id);

                let _ = client
                    .sender
                    .send(LobbyServerMessage::YouJoinedLobby { lobby_id });
            }
            LobbyClientMessage::ListLobbies => {
                let _ = client.sender.send(LobbyServerMessage::LobbyList {
                    lobbies: self
                        .lobbies
                        .values()
                        .map(|lobby| ShortLobbyInfo {
                            id: lobby.id,
                            players: lobby.players.values().map(|v| v.len()).sum(),
                        })
                        .collect(),
                });
            }
            LobbyClientMessage::JoinLobby { id } => {
                if client.in_lobby.is_some() {
                    client
                        .sender
                        .send(LobbyServerMessage::Negative {
                            msg: "Cannot join lobby while in one already".into(),
                        })
                        .unwrap();
                    return;
                }

                let Some(lobby) = self.lobbies.get_mut(&id) else {
                    let _ = client.sender.send(LobbyServerMessage::Negative {
                        msg: "Cannot join lobby; lobby does not exist".into(),
                    });
                    return;
                };

                let side = lobby
                    .players
                    .iter()
                    .fold(
                        (Side::Red, usize::MAX),
                        |(min_side, min_side_count), (side, players)| {
                            #[allow(clippy::if_same_then_else)]
                            if players.len() < min_side_count {
                                (*side, players.len())
                            } else if players.len() == min_side_count && *side < min_side {
                                (*side, players.len())
                            } else {
                                (min_side, min_side_count)
                            }
                        },
                    )
                    .0;
                let players = lobby.players.get_mut(&side).unwrap();

                players.push(player_id);
                client.in_lobby = Some(lobby.id);

                let joined_player = NetworkPlayer {
                    id: player_id,
                    username: client.username.clone(),
                };

                let _ = client
                    .sender
                    .send(LobbyServerMessage::YouJoinedLobby { lobby_id: id });

                for player in lobby.players.values().flatten() {
                    if *player == player_id {
                        continue;
                    }

                    let client = self.players.get(player).unwrap();
                    let _ = client.sender.send(LobbyServerMessage::PlayerJoinedLobby {
                        player: joined_player.clone(),
                        side,
                    });
                }
            }
            LobbyClientMessage::LeaveLobby => {
                self.leave_lobby(player_id);
            }
            LobbyClientMessage::GetLobbyInfo { id } => {
                let Some(lobby) = self.lobbies.get(&id) else {
                    let _ = client.sender.send(LobbyServerMessage::Negative {
                        msg: "Cannot get lobby info of non-existant lobby".into(),
                    });
                    return;
                };

                let lobby_info = LobbyInfo {
                    id,
                    players: lobby
                        .players
                        .iter()
                        .map(|(side, players)| {
                            (
                                *side,
                                players
                                    .iter()
                                    .map(|p| {
                                        let client = self.players.get(p).unwrap();
                                        NetworkPlayer {
                                            id: *p,
                                            username: client.username.clone(),
                                        }
                                    })
                                    .collect(),
                            )
                        })
                        .collect(),
                    lobby_owner: lobby.owner,
                };

                let _ = self
                    .players
                    .get(&player_id)
                    .unwrap()
                    .sender
                    .send(LobbyServerMessage::LobbyInfo { info: lobby_info });
            }
            LobbyClientMessage::SwitchSide => todo!(),
            LobbyClientMessage::SelectChampion { champion } => todo!(),
            LobbyClientMessage::LockInChampion { champion } => todo!(),
        }
    }

    fn leave_lobby(&mut self, player: PlayerId) {
        let Some(client) = self.players.get_mut(&player) else {
            return;
        };
        let Some(lobby_id) = client.in_lobby else {
            return;
        };
        let Some(lobby) = self.lobbies.get_mut(&lobby_id) else {
            return;
        };

        for players in lobby.players.values_mut() {
            let Some(pos) = players.iter().position(|&id| id == player) else {
                continue;
            };

            players.remove(pos);
            break;
        }

        client.in_lobby = None;

        let _ = client.sender.send(LobbyServerMessage::YouLeftLobby);

        if lobby.players.values().all(|v| v.is_empty()) {
            self.lobbies.remove(&lobby_id);
            return;
        }

        let left_player = NetworkPlayer {
            id: player,
            username: client.username.clone(),
        };

        for player in lobby.players.values().flatten() {
            let client = self.players.get(player).unwrap();

            let _ = client.sender.send(LobbyServerMessage::PlayerLeftLobby {
                player: left_player.clone(),
            });
        }
    }
}

fn listen(sender: Sender<Command>) {
    let listener = TcpListener::bind("[::]:65432").unwrap();

    loop {
        let (mut stream, addr) = listener.accept().unwrap();

        let id = PlayerId(Uuid::new_v4());

        let Ok(msg) =
            stream.read_message::<LobbyClientNewConnectionMessage>(Some(Duration::from_secs(3)))
        else {
            continue;
        };

        let (send1, recv1) = mpsc::channel();

        let client = Client {
            player_id: id,
            username: msg.username,
            sender: send1,
            in_lobby: None,
        };

        sender.send(Command::NewClient(client)).unwrap();

        let sender = sender.clone();
        std::thread::spawn(move || listen_connection(id, stream, sender, recv1));
    }
}

fn listen_connection(
    id: PlayerId,
    mut stream: TcpStream,
    sender: Sender<Command>,
    receiver: Receiver<LobbyServerMessage>,
) {
    let s = stream.try_clone().unwrap();
    std::thread::spawn(move || send_connection(s, receiver));

    loop {
        let read_message = stream.read_message::<LobbyClientMessage>(None);
        println!("{:?}", read_message);
        match read_message {
            Ok(msg) => sender.send(Command::MsgFromClient { id, msg }).unwrap(),
            Err(_) => {
                sender.send(Command::ClientDisconnected(id)).unwrap();
                break;
            }
        }
    }
}

fn send_connection(mut stream: TcpStream, receiver: Receiver<LobbyServerMessage>) {
    let _: anyhow::Result<_> = try {
        loop {
            let msg = receiver.recv()?;
            stream.write_message(&msg)?;
        }
    };
}
