use std::any::Any;

use godot::{
    classes::Node,
    obj::{Gd, NewAlloc},
};

use crate::view::{AnchorType, View};

pub struct AnyViewState {
    anchor: Gd<Node>,
    inner: Box<dyn Any>,
}

pub trait AnyView {
    fn as_any(&self) -> &dyn Any;
    fn dyn_build(&self, anchor: &mut Node, anchor_type: AnchorType) -> AnyViewState;
    fn dyn_rebuild(
        &self,
        prev: &dyn AnyView,
        state: &mut AnyViewState,
        anchor: &mut Node,
        anchor_type: AnchorType,
    );
    fn dyn_teardown(&self, state: &mut AnyViewState, anchor: &mut Node, anchor_type: AnchorType);
    // fn dyn_message(
    //     &self,
    //     msg: Message,
    //     path: &[ViewID],
    //     view_state: &mut AnyViewState,
    // ) -> MessageResult;
    // fn collect_nodes(&self, state: &AnyViewState, nodes: &mut Vec<Gd<Node>>);
}

// MARK: AnyView for View

impl<V> AnyView for V
where
    V: View + 'static,
    V::ViewState: 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_build(&self, anchor: &mut Node, anchor_type: AnchorType) -> AnyViewState {
        let mut any_anchor = Node::new_alloc();
        anchor_type.add(anchor, &any_anchor);

        let inner = self.build(&mut any_anchor, AnchorType::Before);
        AnyViewState {
            anchor: any_anchor,
            inner: Box::new(inner),
        }
    }

    fn dyn_rebuild(
        &self,
        prev: &dyn AnyView,
        state: &mut AnyViewState,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        let mut any_anchor = state.anchor.clone();
        if let Some(prev) = prev.as_any().downcast_ref::<V>() {
            let inner = state
                .inner
                .downcast_mut::<V::ViewState>()
                .expect("What the hell bro");

            self.rebuild(prev, inner, &mut any_anchor, AnchorType::Before);
        } else {
            prev.dyn_teardown(state, &mut any_anchor, AnchorType::Before);
            let inner = self.build(&mut any_anchor, AnchorType::Before);
            state.inner = Box::new(inner);
        }
    }

    fn dyn_teardown(&self, state: &mut AnyViewState, anchor: &mut Node, anchor_type: AnchorType) {
        let inner = state
            .inner
            .downcast_mut::<V::ViewState>()
            .expect("What the hell bro");
        let mut any_anchor = state.anchor.clone();
        self.teardown(inner, &mut any_anchor, AnchorType::Before);
    }

    // fn dyn_message(
    //     &self,
    //     msg: Message,
    //     path: &[ViewID],
    //     view_state: &mut AnyViewState,
    //     app_state: &mut State,
    // ) -> MessageResult {
    //     let inner = view_state
    //         .inner
    //         .downcast_mut::<V::ViewState>()
    //         .expect("What the hell bro");
    //     if let Some((start, rest)) = path.split_first() {
    //         if *start == view_state.id {
    //             self.message(msg, rest, inner, app_state)
    //         } else {
    //             MessageResult::Stale(msg)
    //         }
    //     } else {
    //         MessageResult::Stale(msg)
    //     }
    // }

    // fn collect_nodes(&self, state: &AnyViewState, nodes: &mut Vec<Gd<Node>>) {
    //     let inner = state
    //         .inner
    //         .downcast_ref::<V::ViewState>()
    //         .expect("What the hell bro");
    //     self.collect_nodes(inner, nodes);
    // }
}

// MARK: View for dyn AnyView

macro_rules! dyn_anyview_impl {
    ($($who:tt)*) => {
        impl View for $($who)* {
            type ViewState = AnyViewState;

            fn build(
                &self,
                anchor: &mut Node,
                anchor_type: AnchorType,
            ) -> Self::ViewState {
                self.dyn_build(anchor, anchor_type)
            }

            fn rebuild(
                &self,
                prev: &Self,
                state: &mut Self::ViewState,
                anchor: &mut Node,
                anchor_type: AnchorType,
            ) {
                self.dyn_rebuild(prev, state, anchor, anchor_type);
            }

            fn teardown(
                &self,
                state: &mut Self::ViewState,
                anchor: &mut Node,
                anchor_type: AnchorType,
            ) {
                self.dyn_teardown(state, anchor, anchor_type);
            }

            // fn message(
            //     &self,
            //     msg: Message,
            //     path: &[ViewID],
            //     view_state: &mut Self::ViewState,
            // ) -> MessageResult {
            //     self.dyn_message(msg, path, view_state)
            // }

            // fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<Gd<Node>>) {
            //     self.collect_nodes(state, nodes);
            // }
        }
    };
}

dyn_anyview_impl! { dyn AnyView }
dyn_anyview_impl! { dyn AnyView + Send }
dyn_anyview_impl! { dyn AnyView + Send + Sync }
