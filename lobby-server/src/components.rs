use bevy::ecs::{component::Component, entity::Entity};
use uuid::Uuid;


#[derive(Component)]
pub struct Player {
    pub id: Uuid,
    pub username: String,
}

#[derive(Component)]
pub struct PlayerRef {
    pub id: Uuid,
    pub entity: Entity,
}

#[derive(Component)]
pub struct InLobby {
    pub id: Uuid,
    pub entity: Entity
}

#[derive(Component)]
pub struct Lobby {
    pub id: Uuid,
    pub players: Vec<PlayerRef>
}
