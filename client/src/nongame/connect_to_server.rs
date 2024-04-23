use std::{net::SocketAddr, sync::mpsc};

use bevy::prelude::*;

use crate::{ui::{
    button, label, stack, textedit, BuildContext, TextEditComponent, Widget, WidgetExt,
}, DEBUG};

use super::{destroy_menu, network, quit, ConnectingState, EventChannel, Menu, RequestChannel};

#[derive(Clone, Event)]
pub struct ConnectToServer(SocketAddr);

pub struct InConnectToServerPlugin;

impl Plugin for InConnectToServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ConnectToServer>()
            .add_systems(OnEnter(ConnectingState::NotConnected), make_connect_menu)
            .add_systems(OnExit(ConnectingState::NotConnected), destroy_menu)
            .add_systems(
                Update,
                connect_to_server_system.run_if(in_state(ConnectingState::NotConnected)),
            );



        if DEBUG {
            app.add_systems(Startup, |mut e: EventWriter<ConnectToServer>| {
                e.send(ConnectToServer("[::]:65432".parse().unwrap()));
            });
        }
    }
}

fn make_connect_menu(asset_server: Res<AssetServer>, mut commands: Commands) {
    let mut cx = BuildContext {
        asset_server: &asset_server,
        commands: &mut commands,
    };
    let cx = &mut cx;

    let te = textedit("[::]:65432").build(cx);

    stack(FlexDirection::Column)
        .with(label("Connect to server:"))
        .with(te)
        .with(button(
            label("Connect"),
            move |mut e: EventWriter<ConnectToServer>, q: Query<&TextEditComponent>| {
                let te = q.get(te).unwrap();
                let addr = te.text.parse().unwrap();
                e.send(ConnectToServer(addr));
            },
        ))
        .wrap_focus_root()
        .styled(|style| {
            style.width = Val::Percent(100.0);
            style.height = Val::Percent(100.0);
            style.align_items = AlignItems::Center;
            style.justify_content = JustifyContent::Center;
        })
        .custom(|e, cx| {
            cx.commands.entity(e).insert(Menu);
            e
        })
        .build(cx);
}

fn connect_to_server_system(
    mut events: EventReader<ConnectToServer>,
    mut next_state: ResMut<NextState<ConnectingState>>,
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

    next_state.set(ConnectingState::Connecting);

    event_channel.channel = Some(recv_event);
    request_channel.channel = Some(send_request);

    std::thread::spawn(move || network::connect_to_server(addr, send_event, recv_request));
}
