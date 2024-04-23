use bevy::prelude::*;

use crate::ui::{BuildContext, Widget};

pub struct Styled<W> {
    inner: W,
    func: Box<dyn FnOnce(&mut Style) + Send>,
}

impl<W: Widget> Styled<W> {
    pub fn new(inner: W, func: impl FnOnce(&mut Style) + Send + 'static) -> Self {
        Self {
            inner,
            func: Box::new(func),
        }
    }
}

impl<W: Widget> Widget for Styled<W> {
    fn build(self, cx: &mut BuildContext) -> bevy::prelude::Entity {
        let e = self.inner.build(cx);
        cx.commands.entity(e).add(move |mut c: EntityWorldMut| {
            (self.func)(&mut c.get_mut::<Style>().unwrap());
        });
        e
    }

    fn build_boxed(self: Box<Self>, cx: &mut BuildContext) -> bevy::prelude::Entity {
        self.build(cx)
    }
}
