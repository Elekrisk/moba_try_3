use std::any::Any;

use bevy::ui::FlexDirection;

use crate::ui::{widget, AnyView, AnyWidget, BoxedView, MessageResult, Pod, View, ViewId};

pub struct Stack<T, A> {
    direction: FlexDirection,
    children: Vec<BoxedView<T, A>>
}

impl<T, A> Stack<T, A> {
    pub fn new(direction: FlexDirection) -> Self {
        Self {
            direction,
            children: vec![],
        }
    }

    pub fn with_child<V: View<T, A> + 'static>(mut self, child: V) -> Self where V::State: 'static {
        self.children.push(Box::new(child));
        self
    }
}

impl<T, A> View<T, A> for Stack<T, A> {
    type State = Vec<(ViewId, Box<dyn Any + Send + Sync>)>;

    type Element = widget::Stack;

    fn build(&self, cx: &mut crate::ui::Cx) -> (crate::ui::ViewId, Self::State, Self::Element) {
        cx.with_id(|cx, id| {
            let mut state = vec![];
            
            let mut children = vec![];

            for child in &self.children {
                let (id, st, el) = child.build(cx);

                state.push((id, st));
                children.push(Pod::new(el));
            }

            let element = widget::Stack {
                direction: self.direction,
                children
            };

            (id, state, element)
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
        let mut needs_rebuild = false;

        let children = &self.children;
        let prev_children = &prev.children;
        let elem_children = &mut element.children;

        let shared_count = children.len().min(prev_children.len());

        for i in 0..shared_count {
            let child = &children[i];
            let prev_child = &prev_children[i];
            let (id, state) = &mut state[i];
            let elem_child = &mut elem_children[i].widget;

            println!("Stack rebuilding {:?}", id);
            needs_rebuild |= child.rebuild(cx, prev_child, id, state, elem_child);
        }

        for i in shared_count..children.len() {
            let (id, st, el) = children[i].build(cx);
            state.push((id, st));
            elem_children.push(Pod::new(el));
            needs_rebuild = true;
            println!("Stack creating {:?}", id);
        }

        for i in shared_count..prev_children.len() {
            let pod = &mut elem_children[i];
            cx.removed_elements.push(pod.entity.unwrap());
            needs_rebuild = true;
            println!("Stack removing child");
        }

        if self.direction != prev.direction {
            element.direction = self.direction;
            needs_rebuild = true;
        }

        needs_rebuild
    }

    fn message(
        &self,
        id_path: &[crate::ui::ViewId],
        state: &mut Self::State,
        message: Box<dyn std::any::Any>,
        app_state: &mut T,
    ) -> crate::ui::MessageResult<A> {
        match id_path {
            [] => MessageResult::Nop,
            [id, rest @ ..] => {
                for i in 0..state.len() {
                    let (child_id, state) = &mut state[i];
                    if *child_id == *id {
                        return self.children[i].message(rest, state, message, app_state);
                    }
                }
                MessageResult::Stale(Box::new(()))
            }
        }
    }
}
