pub mod view;
mod widget;

use std::{
    any::{type_name, type_name_of_val, Any, TypeId},
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicU32, Ordering},
        Mutex,
    },
};

use bevy::{ecs::system::Command, prelude::*};
use bevy_mod_picking::prelude::EntityEvent;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<Message>().add_systems(Update, ui_system);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct ViewId(u32);

impl ViewId {
    fn new() -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

pub type ViewIdPath = Vec<ViewId>;

pub enum MessageResult<A> {
    Action(A),
    Command(Box<dyn FnOnce(&mut World) + Send + Sync>),
    RequestRebuild,
    Nop,
    Stale(Box<dyn Any>),
}

pub struct Cx {
    id_path: ViewIdPath,
    removed_elements: Vec<Entity>,
}

impl Cx {
    pub fn with_id<T>(&mut self, func: impl FnOnce(&mut Cx, ViewId) -> T) -> T {
        let id = ViewId::new();
        self.id_path.push(id);
        let res = func(self, id);
        self.id_path.pop();
        res
    }
}

pub trait View<T, A = ()>: Send + Sync {
    type State: Send + Sync;
    type Element: Widget + Debug;

    fn build(&self, cx: &mut Cx) -> (ViewId, Self::State, Self::Element);
    fn rebuild(
        &self,
        cx: &mut Cx,
        prev: &Self,
        id: &mut ViewId,
        state: &mut Self::State,
        element: &mut Self::Element,
    ) -> bool;
    fn message(
        &self,
        id_path: &[ViewId],
        state: &mut Self::State,
        message: Box<dyn Any>,
        app_state: &mut T,
    ) -> MessageResult<A>;
}

pub trait AnyView<T, A = ()>: Send + Sync {
    fn as_any(&self) -> &dyn Any;

    fn dyn_build(&self, cx: &mut Cx) -> (ViewId, Box<dyn Any + Send + Sync>, Box<dyn AnyWidget>);
    fn dyn_rebuild(
        &self,
        cx: &mut Cx,
        prev: &dyn AnyView<T, A>,
        id: &mut ViewId,
        state: &mut Box<dyn Any + Send + Sync>,
        element: &mut Box<dyn AnyWidget>,
    ) -> bool;

    fn dyn_message(
        &self,
        id_path: &[ViewId],
        state: &mut dyn Any,
        message: Box<dyn Any>,
        app_state: &mut T,
    ) -> MessageResult<A>;
}

impl<T, A, V: View<T, A> + 'static> AnyView<T, A> for V
where
    V::State: 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_build(&self, cx: &mut Cx) -> (ViewId, Box<dyn Any + Send + Sync>, Box<dyn AnyWidget>) {
        let (id, state, widget) = self.build(cx);
        (id, Box::new(state), Box::new(widget))
    }

    fn dyn_rebuild(
        &self,
        cx: &mut Cx,
        prev: &dyn AnyView<T, A>,
        id: &mut ViewId,
        state: &mut Box<dyn Any + Send + Sync>,
        element: &mut Box<dyn AnyWidget>,
    ) -> bool {
        self.rebuild(
            cx,
            prev.as_any().downcast_ref().unwrap(),
            id,
            state.downcast_mut().unwrap(),
            element.deref_mut().as_any_mut().downcast_mut().unwrap(),
        )
    }

    fn dyn_message(
        &self,
        id_path: &[ViewId],
        state: &mut dyn Any,
        message: Box<dyn Any>,
        app_state: &mut T,
    ) -> MessageResult<A> {
        self.message(id_path, state.downcast_mut().unwrap(), message, app_state)
    }
}

pub type BoxedView<T, A = ()> = Box<dyn AnyView<T, A>>;

impl<T, A> View<T, A> for BoxedView<T, A> {
    type State = Box<dyn Any + Send + Sync>;

    type Element = Box<dyn AnyWidget>;

    fn build(&self, cx: &mut Cx) -> (ViewId, Self::State, Self::Element) {
        self.deref().dyn_build(cx)
    }

    fn rebuild(
        &self,
        cx: &mut Cx,
        prev: &Self,
        id: &mut ViewId,
        state: &mut Self::State,
        element: &mut Self::Element,
    ) -> bool {
        self.deref()
            .dyn_rebuild(cx, prev.deref(), id, state, element)
    }

    fn message(
        &self,
        id_path: &[ViewId],
        state: &mut Self::State,
        message: Box<dyn Any>,
        app_state: &mut T,
    ) -> MessageResult<A> {
        self.deref()
            .dyn_message(id_path, state.deref_mut(), message, app_state)
    }
}

#[derive(Debug)]
pub struct Pod {
    entity: Option<Entity>,
    widget: Box<dyn AnyWidget>,
}

impl Pod {
    pub fn new(widget: Box<dyn AnyWidget>) -> Self {
        Self {
            entity: None,
            widget,
        }
    }
}

pub trait Widget: Debug + Send + Sync + 'static {
    fn build(&mut self, commands: &mut Commands) -> Entity;
    fn rebuild(&mut self, entity: Entity, commands: &mut Commands);
    fn delete(&mut self, entity: Entity, commands: &mut Commands);
}

pub trait AnyWidget: Widget {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<W: Widget> AnyWidget for W {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        println!("{}", std::any::type_name::<W>());
        self
    }
}

impl Widget for Box<dyn AnyWidget> {
    fn build(&mut self, commands: &mut Commands) -> Entity {
        self.deref_mut().build(commands)
    }

    fn rebuild(&mut self, entity: Entity, commands: &mut Commands) {
        self.deref_mut().rebuild(entity, commands);
    }

    fn delete(&mut self, entity: Entity, commands: &mut Commands) {
        self.deref_mut().delete(entity, commands);
    }
}

pub struct UiRoot<T, V: View<T>, F: FnMut(&mut T) -> V> {
    ui_logic: F,
    data: T,
    render_result: Option<RenderResult<T, V>>,
    needs_rebuild: bool,
}

pub struct RenderResult<T, V: View<T>> {
    view: V,
    state: V::State,
    id: ViewId,
    root_pod: Pod,
}

impl<T, V: View<T>, F: FnMut(&mut T) -> V> UiRoot<T, V, F> {
    pub fn new(data: T, ui_logic: F) -> Self {
        Self {
            ui_logic,
            data,
            render_result: None,
            needs_rebuild: true,
        }
    }

    fn render(&mut self, commands: &mut Commands) {
        let mut cx = Cx {
            id_path: vec![],
            removed_elements: vec![],
        };

        let view = (self.ui_logic)(&mut self.data);
        if let Some(render_result) = &mut self.render_result {
            let widget = render_result
                .root_pod
                .widget
                .deref_mut()
                .as_any_mut()
                .downcast_mut::<V::Element>()
                .unwrap();
            let needs_rebuild = view.rebuild(
                &mut cx,
                &render_result.view,
                &mut render_result.id,
                &mut render_result.state,
                widget,
            );
            if needs_rebuild {
                let entity = render_result.root_pod.entity.unwrap();
                widget.rebuild(entity, commands);
            }

            render_result.view = view;
        } else {
            let (id, state, mut widget) = view.build(&mut cx);
            let entity = widget.build(commands);

            println!("Render resulted in a {} widget", type_name_of_val(&widget));

            let render_result = RenderResult {
                view,
                state,
                id,
                root_pod: Pod {
                    entity: Some(entity),
                    widget: Box::new(widget),
                },
            };

            self.render_result = Some(render_result);
        }

        self.needs_rebuild = false;
    }

    fn make_sure_root_exists(&mut self, commands: &mut Commands) {
        if self.render_result.is_none() {
            self.render(commands);
        }
    }

    fn message(&mut self, event: &Message, commands: &mut Commands) {
        self.make_sure_root_exists(commands);
        let render_result = self.render_result.as_mut().unwrap();

        if event.path[0] != render_result.id {
            return;
        }

        let state = &mut render_result.state;
        self.needs_rebuild = match render_result.view.message(
            &event.path[1..],
            state,
            event.extract_data(),
            &mut self.data,
        ) {
            MessageResult::Action(_) => false,
            MessageResult::Command(command) => {
                commands.add(command);
                true
            }
            MessageResult::RequestRebuild => true,
            MessageResult::Nop => false,
            MessageResult::Stale(_) => todo!(),
        };
    }
}

#[derive(Debug, Event)]
pub struct Message {
    path: ViewIdPath,
    data: Mutex<Option<Box<dyn Any + Send + Sync>>>,
}

impl Message {
    pub fn new(path: ViewIdPath, data: impl Any + Send + Sync) -> Self {
        Self {
            path,
            data: Mutex::new(Option::Some(Box::new(data))),
        }
    }

    pub fn extract_data(&self) -> Box<dyn Any + Send + Sync> {
        self.data.lock().unwrap().take().unwrap()
    }
}

pub trait AnyUiRoot: Send + Sync {
    fn render(&mut self, commands: &mut Commands);
    fn make_sure_root_exists(&mut self, commands: &mut Commands);
    fn message(&mut self, event: &Message, commands: &mut Commands);
    fn needs_rebuild(&self) -> bool;
}

impl<T: Send + Sync, V: View<T> + Send + Sync, F: FnMut(&mut T) -> V + Send + Sync> AnyUiRoot
    for UiRoot<T, V, F>
{
    fn render(&mut self, commands: &mut Commands) {
        UiRoot::render(self, commands);
    }

    fn make_sure_root_exists(&mut self, commands: &mut Commands) {
        UiRoot::make_sure_root_exists(self, commands);
    }

    fn message(&mut self, event: &Message, commands: &mut Commands) {
        UiRoot::message(self, &event, commands);
    }

    fn needs_rebuild(&self) -> bool {
        self.needs_rebuild || self.render_result.is_none()
    }
}

#[derive(Component)]
pub struct UiRootComponent(pub Box<dyn AnyUiRoot>);

fn ui_system(
    mut query: Query<&mut UiRootComponent>,
    mut events: EventReader<Message>,
    mut commands: Commands,
) {
    let events = events.read().collect::<Vec<_>>();

    for mut ui_root in &mut query {
        for event in &events {
            ui_root.0.message(event, &mut commands);
        }

        if ui_root.0.needs_rebuild() {
            ui_root.0.render(&mut commands);
        }
    }
}
