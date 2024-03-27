use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Side;


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
    JoinLobby {
        lobby_id: Uuid,
    },
    LeaveLobby,
    GetLobbyInfo {
        lobby_id: Uuid,
    },
    SwitchSide,
    SelectChampion {
        champion: String,
    },
    LockInChampion {
        champion: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LobbyServerMessage {
    OK,
    Negative {
        msg: String,
    },
    StopMatchmaking,
    LobbyList {
        lobbies: Vec<ShortLobbyInfo>,
    },
    LobbyInfo {
        info: LobbyInfo
    },
    MatchmakingDone {
        lobby_id: Uuid,
    },
    PlayerJoinedLobby {
        player: Player,
        side: Side,
    },
    PlayerLeftLobby {
        player: Player,
    },
    PlayerSwitchedSide {
        player: Player,
        side: Side,
    },
    PlayerSelectedChampion {
        player: Player,
        champion: String,
    },
    PlayerLockedInChampion {
        player: Player,
        champion: String,
    },
    YouJoinedLobby {
        lobby_id: Uuid,
    },
    YouLeftLobby,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShortLobbyInfo {
    pub lobby_id: Uuid,
    pub players: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LobbyInfo {
    pub lobby_id: Uuid,
    pub players: Vec<PlayerWithSide>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub player_id: Uuid,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerWithSide {
    pub player_id: Uuid,
    pub username: String,
    pub side: Side,
}
