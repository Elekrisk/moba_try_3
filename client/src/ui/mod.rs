mod core;
mod img_on_hover_btn;
mod animation;

pub use core::*;
pub use img_on_hover_btn::*;
pub use animation::*;

use std::{
    any::Any,
    ops::{Deref, DerefMut},
};

use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_mod_picking::prelude::*;

use self::{animation::UiAnimationPlugin, img_on_hover_btn::ImgOnHoverBtnPlugin};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((TextEditPlugin, ButtonPlugin, ImgOnHoverBtnPlugin, UiAnimationPlugin::<Style>::new()));
    }
}

pub struct BuildContext<'a, 'w, 's> {
    pub asset_server: &'a AssetServer,
    pub commands: &'a mut Commands<'w, 's>,
}

pub trait Widget {
    fn build(self, cx: &mut BuildContext) -> Entity;
    fn build_boxed(self: Box<Self>, cx: &mut BuildContext) -> Entity;
}

impl Widget for Entity {
    fn build(self, cx: &mut BuildContext) -> Entity {
        self
    }

    fn build_boxed(self: Box<Self>, cx: &mut BuildContext) -> Entity {
        self.build(cx)
    }
}

impl<'a> Widget for &'a str {
    fn build(self, cx: &mut BuildContext) -> Entity {
        label(self).build(cx)
    }

    fn build_boxed(self: Box<Self>, cx: &mut BuildContext) -> Entity {
        self.build(cx)
    }
}

impl Widget for String {
    fn build(self, cx: &mut BuildContext) -> Entity {
        label(self).build(cx)
    }

    fn build_boxed(self: Box<Self>, cx: &mut BuildContext) -> Entity {
        self.build(cx)
    }
}

pub trait AnyComponent: Any {
    fn insert_into(self: Box<Self>, commands: &mut EntityCommands);
}

impl<T: Component + Any> AnyComponent for T {
    fn insert_into(self: Box<Self>, commands: &mut EntityCommands) {
        commands.insert(*self);
    }
}

pub struct WidgetWrapper<W> {
    extra: Vec<Box<dyn AnyComponent>>,
    widget: W,
}

impl<W: Widget> WidgetWrapper<W> {
    pub fn new(widget: W) -> Self {
        Self {
            extra: vec![],
            widget,
        }
    }

    pub fn with_extra(&mut self, extra: impl AnyComponent) -> &mut Self {
        self.extra.push(Box::new(extra));
        self
    }

    pub fn build(self, cx: &mut BuildContext) -> Entity {
        let e = self.widget.build(cx);
        let mut c = cx.commands.entity(e);
        for extra in self.extra {
            extra.insert_into(&mut c);
        }
        e
    }
}

impl<W> Deref for WidgetWrapper<W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl<W> DerefMut for WidgetWrapper<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}

/// Component inserted on focused UI elements.
#[derive(Component)]
pub struct Focused;

pub struct FocusRoot<W> {
    inner: W,
}

impl<W: Widget> Widget for FocusRoot<W> {
    fn build(self, cx: &mut BuildContext) -> Entity {
        let child = self.inner.build(cx);
        cx.commands
            .spawn((
                NodeBundle { style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                }, ..default() },
                On::<Pointer<Click>>::run(focus_handler),
            ))
            .add_child(child)
            .id()
    }

    fn build_boxed(self: Box<Self>, cx: &mut BuildContext) -> Entity {
        self.build(cx)
    }
}

fn focus_handler(
    input: Listener<Pointer<Click>>,
    focus_query: Query<Entity, With<Focused>>,
    mut commands: Commands,
) {
    println!("FOCUS HANDLER");
    let target = input.target();

    if let Ok(e) = focus_query.get_single() {
        if e == target {
            return;
        }

        commands.entity(e).remove::<Focused>();
    }

    commands.entity(target).insert(Focused);
}

pub trait WidgetExt: Sized {
    fn wrap_focus_root(self) -> FocusRoot<Self>;
    fn styled(self, func: impl FnOnce(&mut Style) + Send + 'static) -> Styled<Self>;
    fn custom(
        self,
        func: impl FnOnce(Entity, &mut BuildContext) -> Entity + Send + 'static,
    ) -> Custom<Self>;

    fn insert(self, bundle: impl Bundle) -> Custom<Self>;
}

impl<W: Widget> WidgetExt for W {
    fn wrap_focus_root(self) -> FocusRoot<Self> {
        FocusRoot { inner: self }
    }

    fn styled(self, func: impl FnOnce(&mut Style) + Send + 'static) -> Styled<Self> {
        Styled::new(self, func)
    }

    fn custom(
        self,
        func: impl FnOnce(Entity, &mut BuildContext) -> Entity + Send + 'static,
    ) -> Custom<Self> {
        Custom {
            inner: self,
            func: Box::new(func),
        }
    }
    
    fn insert(self, bundle: impl Bundle) -> Custom<Self> {
        Custom::new_as_insert(self, || bundle)
    }
}
