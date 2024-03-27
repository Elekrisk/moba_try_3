use std::fmt::Display;

use bevy::ecs::system::Command;

use crate::ui::{widget, MessageResult, View};


pub struct Button<T> {
    label: String,
    action: ButtonAction<T>
}

pub enum ButtonAction<T> {
    Nop,
    ModifyState(Box<dyn Fn(&mut T) + Send + Sync>),
    Command(Box<dyn Command + Sync>)
}

impl<T> Button<T> {
    pub fn new(label: impl Display) -> Self {
        Self {
            label: label.to_string(),
            action: ButtonAction::Nop
        }
    }

    pub fn with_action(mut self, action: ButtonAction<T>) -> Self {
        self.action = action;
        self
    }
}

impl<T, A> View<T, A> for Button<T> {
    type State = ();

    type Element = widget::Button;

    fn build(&self, cx: &mut crate::ui::Cx) -> (crate::ui::ViewId, Self::State, Self::Element) {
        cx.with_id(|cx, id| {
            (id, (), widget::Button::new(cx.id_path.clone(), self.label.clone()))
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
            println!("New label {:?} is different from as old label {:?}", self.label, prev.label);
            element.label = self.label.clone();
            true
        } else {
            println!("New label {:?} is the same as old label {:?}", self.label, prev.label);
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
        let event = *message.downcast::<ButtonEvent>().unwrap();

        match event {
            ButtonEvent::Click => {
                match &self.action {
                    ButtonAction::Nop => MessageResult::Nop,
                    ButtonAction::ModifyState(func) => {
                        func(app_state);
                        MessageResult::RequestRebuild
                    },
                    ButtonAction::Command(_) => todo!(),
                }
            },
        }
    }
}

pub enum ButtonEvent {
    Click
}
