use std::{
    any::{type_name, type_name_of_val, Any, TypeId},
    marker::PhantomData,
};

use bevy::prelude::*;

pub struct Bindings {}

pub struct Binding<Source, SourceData, Target, TargetData> {
    to: Entity,
    getter: fn(&Source) -> &SourceData,
    setter: fn(&mut Target, TargetData),
    transformer: fn(&SourceData) -> TargetData,
    _marker: PhantomData<(Source, SourceData, Target, TargetData)>,
}

pub struct Change {
    entity: Entity,
    change: Box<dyn FnOnce(EntityMut)>,
}

impl<S: Component, SD, T: Component, TD: 'static> Binding<S, SD, T, TD> {
    pub fn new(
        to: Entity,
        getter: fn(&S) -> &SD,
        setter: fn(&mut T, TD),
        transformer: fn(&SD) -> TD,
    ) -> Self {
        Self {
            to,
            getter,
            setter,
            transformer,
            _marker: PhantomData,
        }
    }

    pub fn propagate(&self, source_ent: EntityRef) -> Option<Change> {
        let source = source_ent.get_ref::<S>().unwrap();
        if !source.is_changed() {
            return None;
        }
        let source_data = (self.getter)(&*source);
        let target_data = (self.transformer)(source_data);
        let setter = self.setter;
        Some(Change {
            entity: self.to,
            change: Box::new(move |mut e| {
                let mut target = e.get_mut::<T>().unwrap();
                (setter)(&mut *target, target_data);
            }),
        })
    }
}

impl<S: Component, T: Component, D: Copy + 'static> Binding<S, D, T, D> {
    pub fn copy_field(to: Entity, getter: fn(&S) -> &D, setter: fn(&mut T, D)) -> Self {
        Self::new(to, getter, setter, |data| *data)
    }
}

impl<S: Component, T: Component, D: Clone + 'static> Binding<S, D, T, D> {
    pub fn clone_field(to: Entity, getter: fn(&S) -> &D, setter: fn(&mut T, D)) -> Self {
        Self::new(to, getter, setter, |data| data.clone())
    }
}

impl<C: Component + Clone> Binding<C, C, C, C> {
    pub fn clone_component(to: Entity) -> Self {
        Self::new(to, |c| c, |c, d| *c = d, |c| c.clone())
    }
}

pub macro binding($s:ty {$($stt:tt)*} >> $to:ident . $t:tt {$($ttt:tt)*} $f:expr) {
    BoxedBinding(Box::new(Binding::new($to, |s: &$s| &s $($stt)*, |t: &mut $t, d| t $($ttt)* = d, $f)))
}

unsafe impl<S, SD, T, TD> Send for Binding<S, SD, T, TD> {}
unsafe impl<S, SD, T, TD> Sync for Binding<S, SD, T, TD> {}

impl<S, SD: 'static, T, TD: 'static> AnyBinding for Binding<S, SD, T, TD>
where
    S: Component,
    T: Component,
{
    fn propagate(&self, source_ent: EntityRef) -> Option<Change> {
        Binding::propagate(&self, source_ent)
    }
}

pub trait AnyBinding: Send + Sync + 'static {
    fn propagate(&self, source_ent: EntityRef) -> Option<Change>;
}

#[derive(Component)]
pub struct BoxedBinding(Box<dyn AnyBinding>);

pub struct BindingPlugin;

impl Plugin for BindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, binding_propagator);
    }
}

fn binding_propagator(world: &mut World) {
    let mut query = world.query::<(EntityRef, &BoxedBinding)>();

    loop {
        let mut changes = vec![];

        for (source_ent, binding) in query.iter(world) {
            let change = binding.0.propagate(source_ent);
            if let Some(change) = change {
                changes.push(change);
            }
        }

        if changes.is_empty() {
            break;
        }

        for change in changes {
            let e = world.entity_mut(change.entity);
            (change.change)(e.into());
        }
    }
}

#[derive(Component)]
pub struct Label(pub String);

pub trait Widget {
    fn build(&self, commands: &mut Commands);
}

pub trait WidgetExt : Widget {
    type Options;
    fn make(options: Self::Options) -> Self;
}

pub struct Button {
    label: String,
}

impl Widget for Button {
    fn build(&self, commands: &mut Commands) {
        let text = commands.spawn(TextBundle {
            text: Text::from_section(
                "",
                TextStyle {
                    font_size: 16.0,
                    color: Color::BLACK,
                    ..default()
                },
            ),
            ..default()
        });
        let id = text.id();

        let base = commands
            .spawn((
                ButtonBundle { ..default() },
                Label(self.label.clone()),
                binding!(Label {.0} >> id.Text { .sections[0].value } String::clone),
            ))
            .add_child(id);
    }
}

pub struct Stack {
    children: Vec<Box<dyn Widget>>
}

impl Stack {
    pub fn add_child(&mut self, child: impl Widget + 'static) {
        self.children.push(Box::new(child));
    }
}

impl Widget for Stack {
    fn build(&self, commands: &mut Commands) {
        todo!()
    }
}

fn test() {
    
}
