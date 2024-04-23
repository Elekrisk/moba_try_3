use std::{any::Any, marker::PhantomData, time::Duration};

use bevy::{
    ecs::{component::TableStorage, system::RunSystemOnce},
    prelude::*,
};

use super::IntoSystemAny;

pub struct UiAnimationPlugin<T> {
    hidden: PhantomData<T>,
}

impl<T> UiAnimationPlugin<T> {
    pub fn new() -> Self {
        Self {
            hidden: PhantomData,
        }
    }
}

impl<T> Default for UiAnimationPlugin<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Component> Plugin for UiAnimationPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, run_anim::<T>);
    }

    fn is_unique(&self) -> bool {
        false
    }
}

pub enum Easing {
    Linear,
    Custom(Box<dyn Fn(f32) -> f32 + Sync + Send>),
}

pub struct WrappedSystem<In, Out> {
    inner: Box<dyn System<In = In, Out = Out>>,
}

impl<In: Send + Sync + 'static, Out: Send + Sync + 'static> System for WrappedSystem<In, Out> {
    type In = In;

    type Out = Out;

    fn name(&self) -> std::borrow::Cow<'static, str> {
        self.inner.name()
    }

    fn component_access(&self) -> &bevy::ecs::query::Access<bevy::ecs::component::ComponentId> {
        self.inner.component_access()
    }

    fn archetype_component_access(
        &self,
    ) -> &bevy::ecs::query::Access<bevy::ecs::archetype::ArchetypeComponentId> {
        self.inner.archetype_component_access()
    }

    fn is_send(&self) -> bool {
        self.inner.is_send()
    }

    fn is_exclusive(&self) -> bool {
        self.inner.is_exclusive()
    }

    fn has_deferred(&self) -> bool {
        self.inner.has_deferred()
    }

    unsafe fn run_unsafe(
        &mut self,
        input: Self::In,
        world: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell,
    ) -> Self::Out {
        self.inner.run_unsafe(input, world)
    }

    fn apply_deferred(&mut self, world: &mut World) {
        self.inner.apply_deferred(world)
    }

    fn initialize(&mut self, _world: &mut World) {
        self.inner.initialize(_world)
    }

    fn update_archetype_component_access(
        &mut self,
        world: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell,
    ) {
        self.inner.update_archetype_component_access(world)
    }

    fn check_change_tick(&mut self, change_tick: bevy::ecs::component::Tick) {
        self.inner.check_change_tick(change_tick)
    }

    fn get_last_run(&self) -> bevy::ecs::component::Tick {
        self.inner.get_last_run()
    }

    fn set_last_run(&mut self, last_run: bevy::ecs::component::Tick) {
        self.inner.set_last_run(last_run)
    }

    fn type_id(&self) -> std::any::TypeId {
        self.inner.type_id()
    }

    fn run(&mut self, input: Self::In, world: &mut World) -> Self::Out {
        self.inner.run(input, world)
    }

    fn default_system_sets(&self) -> Vec<bevy::ecs::schedule::InternedSystemSet> {
        self.inner.default_system_sets()
    }
}

pub trait AnimFunc<T> = Fn(&mut T, f32) + Sync + Send + 'static;

#[derive(Component)]
pub struct Animation<T> {
    anim_func: Box<dyn AnimFunc<T>>,
    easing: Easing,
    duration: Duration,
    remaining: Duration,
    on_finish: Option<WrappedSystem<Entity, ()>>,
}

impl<T: Component> Animation<T> {
    pub fn new(anim_func: impl AnimFunc<T>, easing: Easing, duration: Duration) -> Self {
        Self {
            anim_func: Box::new(anim_func),
            easing,
            duration,
            remaining: duration,
            on_finish: None,
        }
    }

    pub fn on_finish<M>(mut self, on_finish: impl IntoSystem<Entity, (), M>) -> Self {
        self.on_finish = Some(WrappedSystem {
            inner: Box::new(IntoSystem::into_system(on_finish)) as _,
        });
        self
    }

    pub fn run_anim(&mut self, entity: Entity, comp: &mut T, time: &Time, commands: &mut Commands) {
        if self.remaining.is_zero() {
            return;
        }

        self.remaining = self.remaining.saturating_sub(time.delta());

        let t = 1.0 - self.remaining.div_duration_f32(self.duration);

        let eased_t = match &self.easing {
            Easing::Linear => t,
            Easing::Custom(func) => func(t),
        };

        (self.anim_func)(comp, eased_t);

        if self.remaining.is_zero() {
            if let Some(on_finish) = self.on_finish.take() {
                commands.add(move |w: &mut World| w.run_system_once_with(entity, on_finish));
            }
        }
    }
}

fn run_anim<T: Component>(
    mut q: Query<(Entity, &mut Animation<T>, &mut T)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (e, mut anim, mut comp) in &mut q {
        anim.run_anim(e, &mut comp, &time, &mut commands);
    }
}
