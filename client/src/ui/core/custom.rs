use bevy::prelude::*;

use crate::ui::{BuildContext, Widget};

pub trait CustomBuilder = FnOnce(Entity, &mut BuildContext) -> Entity;

pub struct Custom<W> {
    pub inner: W,
    pub func: Box<dyn CustomBuilder>,
}

impl<W: Widget> Custom<W> {
    pub fn new(inner: W, func: impl CustomBuilder + 'static) -> Self {
        Self {
            inner,
            func: Box::new(func)
        }
    }

    pub fn new_as_insert<B: Bundle>(inner: W, func: impl FnOnce() -> B + 'static) -> Self {
        Self::new(inner, |e, cx| cx.commands.entity(e).insert(func()).id())
    }
}

impl<W: Widget> Widget for Custom<W> {
    fn build(self, cx: &mut crate::ui::BuildContext) -> Entity {
        let e = self.inner.build(cx);
        (self.func)(e, cx)
    }

    fn build_boxed(self: Box<Self>, cx: &mut crate::ui::BuildContext) -> Entity {
        self.build(cx)
    }
}
