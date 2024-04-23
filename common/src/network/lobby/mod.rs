use std::fmt::Display;

use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Side;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LobbyId(pub Uuid);

impl Display for LobbyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub Uuid);

impl Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LobbyClientNewConnectionMessage {
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LobbyClientMessage {
    StartMatchmaking,
    StopMatchmaking,
    CreateLobby,
    ListLobbies,
    JoinLobby { id: LobbyId },
    LeaveLobby,
    GetLobbyInfo { id: LobbyId },
    SwitchSide,
    SelectChampion { champion: String },
    LockInChampion { champion: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LobbyServerMessage {
    OK,
    Negative { msg: String },
    StopMatchmaking,
    LobbyList { lobbies: Vec<ShortLobbyInfo> },
    LobbyInfo { info: LobbyInfo },
    MatchmakingDone { lobby_id: LobbyId },
    PlayerJoinedLobby { player: Player, side: Side },
    PlayerLeftLobby { player: Player },
    PlayerSwitchedSide { player: Player, side: Side },
    PlayerSelectedChampion { player: Player, champion: String },
    PlayerLockedInChampion { player: Player, champion: String },
    YouJoinedLobby { lobby_id: LobbyId },
    YouLeftLobby,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShortLobbyInfo {
    pub id: LobbyId,
    pub players: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyInfo {
    pub id: LobbyId,
    pub players: HashMap<Side, Vec<Player>>,
    pub lobby_owner: PlayerId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerWithSide {
    pub player: Player,
    pub side: Side,
}
