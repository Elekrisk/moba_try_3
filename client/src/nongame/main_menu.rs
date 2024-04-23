mod lobby;
mod lobby_list;

use bevy::{app::AppExit, prelude::*};
use bevy_mod_picking::prelude::*;

use crate::{
    ui::{button, img_on_hover_btn, stack, BuildContext, Widget as _, WidgetExt},
    DEBUG,
};

use self::{lobby::LobbyPlugin, lobby_list::LobbyListPlugin};

use super::{destroy_menu, network::Request, ConnectingState};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ConnectingState::Connected),
            (make_main_menu, enter_main_menu),
        );
        app.add_systems(OnExit(ConnectingState::Connected), destroy_menu);
        app.insert_state(LobbyState::None);

        app.add_plugins((LobbyListPlugin, LobbyPlugin));

        if DEBUG {
            app.add_systems(OnEnter(ConnectingState::Connected), |mut e: EventWriter<Request>| {
                e.send(Request::CreateLobby);
            });
        }
    }
}

fn enter_main_menu(mut next_state: ResMut<NextState<LobbyState>>) {
    next_state.set(LobbyState::NotInLobby);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum LobbyState {
    None,
    NotInLobby,
    InLobby,
}

#[derive(Component)]
pub struct MenuHolder;

pub fn make_main_menu(asset_server: Res<AssetServer>, mut commands: Commands) {
    let font = asset_server.load("fonts/Roboto-Light.ttf");

    let text_style = TextStyle {
        font,
        font_size: 16.0,
        color: Color::GOLD,
    };

    let button_img: Handle<Image> = asset_server.load("ui/button.png");

    let mut cx = BuildContext {
        asset_server: &asset_server,
        commands: &mut commands,
    };

    let lobby_tab_button = img_on_hover_btn("Lobby", || {})
        .styled(|s| {
            s.padding = UiRect::axes(Val::Px(8.0), Val::Px(8.0));
        })
        .custom(|e, cx| {
            let ec = &mut cx.commands.entity(e);
            ec.insert((
                UiImage::new(cx.asset_server.load("ui/underline.png")),
                ImageScaleMode::Sliced(TextureSlicer {
                    border: BorderRect::square(16.0),
                    ..default()
                }),
            ));
            e
        });

    let tab_bar = stack(FlexDirection::Row)
        .with(lobby_tab_button)
        .styled(|s| {
            s.flex_grow = 1.0;
        });

    let quit_button = button("Quit", |mut e: EventWriter<AppExit>| {
        e.send(AppExit);
    })
    .styled(|s| {
        s.padding = UiRect::axes(Val::Px(8.0), Val::Px(8.0));
    })
    .custom(|e, cx| {
        let mut ec = cx.commands.entity(e);
        ec.insert((
            UiImage::new(cx.asset_server.load("ui/button.png")),
            ImageScaleMode::Sliced(TextureSlicer {
                border: BorderRect::square(16.0),
                ..default()
            }),
        ));
        e
    });

    let button_group = stack(FlexDirection::Row).with(quit_button);

    let top_bar = stack(FlexDirection::Row)
        .with(tab_bar)
        .with(button_group)
        .styled(|s| {
            s.padding = UiRect::axes(Val::Px(8.0), Val::Px(8.0));
        });

    let lobby_holder = stack(FlexDirection::Column)
        .styled(|s| {
            s.flex_grow = 1.0;
        })
        .custom(|e, cx| {
            cx.commands.entity(e).insert(MenuHolder);
            e
        });

    let root = stack(FlexDirection::Column)
        .with(top_bar)
        .with(lobby_holder)
        .styled(|s| {
            s.width = Val::Percent(100.0);
            s.height = Val::Percent(100.0);
            s.align_items = AlignItems::Stretch;
        })
        .wrap_focus_root();

    root.build(&mut cx);

    // let lobby_tab = commands
    //     .spawn((
    //         ButtonBundle {
    //             style: Style {
    //                 padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
    //                 ..default()
    //             },
    //             image: button_img.clone().into(),
    //             ..default()
    //         },
    //         ImageScaleMode::Sliced(TextureSlicer {
    //             border: BorderRect::square(16.0),
    //             ..default()
    //         }),
    //     ))
    //     .with_children(|parent| {
    //         parent.spawn(TextBundle {
    //             text: Text::from_section("Lobby", text_style.clone()),
    //             ..default()
    //         });
    //     })
    //     .id();

    // let tabbar = commands
    //     .spawn(NodeBundle {
    //         style: Style {
    //             flex_grow: 1.0,
    //             flex_direction: FlexDirection::Row,
    //             ..default()
    //         },
    //         ..default()
    //     })
    //     .push_children(&[lobby_tab])
    //     .id();

    // let quit_button = commands
    //     .spawn((
    //         ButtonBundle {
    //             style: Style {
    //                 padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
    //                 ..default()
    //             },
    //             image: button_img.clone().into(),
    //             ..default()
    //         },
    //         ImageScaleMode::Sliced(TextureSlicer {
    //             border: BorderRect::square(16.0),
    //             ..default()
    //         }),
    //         On::<Pointer<Click>>::run(|mut e: EventWriter<AppExit>| {
    //             e.send(AppExit);
    //         }),
    //     ))
    //     .with_children(|parent| {
    //         parent.spawn(TextBundle {
    //             text: Text::from_section("Quit", text_style.clone()),
    //             ..default()
    //         });
    //     })
    //     .id();

    // let topbar = commands
    //     .spawn(NodeBundle {
    //         style: Style {
    //             flex_direction: FlexDirection::Row,
    //             ..default()
    //         },
    //         ..default()
    //     })
    //     .push_children(&[tabbar, quit_button])
    //     .id();

    // let menu_holder = commands
    //     .spawn((
    //         NodeBundle {
    //             style: Style {
    //                 flex_direction: FlexDirection::Column,
    //                 align_items: AlignItems::Center,
    //                 justify_content: JustifyContent::Center,
    //                 ..default()
    //             },
    //             ..default()
    //         },
    //         MenuHolder,
    //     ))
    //     .id();

    // let _root = commands
    //     .spawn(NodeBundle {
    //         style: Style {
    //             width: Val::Percent(100.0),
    //             height: Val::Percent(100.0),
    //             flex_direction: FlexDirection::Column,
    //             align_items: AlignItems::Stretch,
    //             ..default()
    //         },
    //         ..default()
    //     })
    //     .push_children(&[topbar, menu_holder])
    //     .id();
}
