use crate::ui::{widget, MessageResult, View};


pub struct Label {
    label: String,
}

impl Label {
    pub fn new(label: impl ToString) -> Self {
        Self {
            label: label.to_string()
        }
    }
}

impl<T, A> View<T, A> for Label {
    type State = ();

    type Element = widget::Label;

    fn build(&self, cx: &mut crate::ui::Cx) -> (crate::ui::ViewId, Self::State, Self::Element) {
        cx.with_id(|cx, id| {
            (id, (), widget::Label { label: self.label.clone() })
        })
    }

    fn rebuild(
        &self,
        cx: &mut crate::ui::Cx,
        prev: &Self,
        id: &mut crate::ui::ViewId,
        state: &mut Self::State,
        element: &mut Self::Element,
    ) -> bool {
        if self.label != prev.label {
            element.label = self.label.clone();
            true
        } else {
            false
        }
    }

    fn message(
        &self,
        id_path: &[crate::ui::ViewId],
        state: &mut Self::State,
        message: Box<dyn std::any::Any>,
        app_state: &mut T,
    ) -> crate::ui::MessageResult<A> {
        if id_path.is_empty() {
            MessageResult::Nop
        } else {
            MessageResult::Stale(Box::new(()))
        }
    }
}
