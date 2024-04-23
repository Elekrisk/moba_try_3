use bevy::prelude::*;

use crate::ui::{button, label, stack, BuildContext, Widget, WidgetExt};

use super::{destroy_menu, network::ServerConnectionStatus, ConnectingState, Menu};

pub struct InConnectingToServerPlugin;

impl Plugin for InConnectingToServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ConnectingState::Connecting), make_connecting_menu);
        app.add_systems(OnExit(ConnectingState::Connecting), destroy_menu);

        app.add_systems(
            Update,
            connected_to_server.run_if(in_state(ConnectingState::Connecting)),
        );
    }
}

fn make_connecting_menu(asset_server: Res<AssetServer>, mut commands: Commands) {
    stack(FlexDirection::Column)
        .with("Connecting...")
        .with(button("Cancel", || {}))
        .wrap_focus_root()
        .custom(|e, cx| cx.commands.entity(e).insert(Menu).id())
        .build(&mut BuildContext {
            asset_server: &asset_server,
            commands: &mut commands,
        });
}

fn connected_to_server(
    mut reader: EventReader<ServerConnectionStatus>,
    mut next_state: ResMut<NextState<ConnectingState>>,
) {
    for event in reader.read() {
        match event {
            ServerConnectionStatus::Connected => next_state.set(ConnectingState::Connected),
            ServerConnectionStatus::ConnectionFailed => {
                next_state.set(ConnectingState::NotConnected)
            }
        }
    }
}
