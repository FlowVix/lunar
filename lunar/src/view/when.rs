use godot::{
    classes::Node,
    obj::{Gd, NewAlloc},
};

use crate::{AnchorType, View};

pub struct When<T, InnerFn> {
    value: T,
    inner_fn: InnerFn,
}

pub struct WhenViewState<Inner: View> {
    anchor: Gd<Node>,
    inner: Inner,
    inner_state: Inner::ViewState,
}

impl<T, InnerFn, Inner> View for When<T, InnerFn>
where
    T: PartialEq,
    InnerFn: Fn() -> Inner,
    Inner: View,
{
    type ViewState = WhenViewState<Inner>;

    fn build(
        &self,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) -> Self::ViewState {
        let mut when_anchor = Node::new_alloc();
        anchor_type.add(anchor, &when_anchor);

        let inner = (self.inner_fn)();
        let inner_state = inner.build(ctx, &mut when_anchor, AnchorType::Before);

        WhenViewState {
            anchor: when_anchor,
            inner,
            inner_state,
        }
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) {
        let mut when_anchor = state.anchor.clone();
        if self.value != prev.value {
            state.inner.teardown(
                &mut state.inner_state,
                ctx,
                &mut when_anchor,
                AnchorType::Before,
            );
            let inner = (self.inner_fn)();
            let inner_state = inner.build(ctx, &mut when_anchor, AnchorType::Before);
            state.inner = inner;
            state.inner_state = inner_state;
        } else {
            let inner = (self.inner_fn)();
            inner.rebuild(
                &state.inner,
                &mut state.inner_state,
                ctx,
                &mut when_anchor,
                AnchorType::Before,
            );
            state.inner = inner;
        }
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) {
        let mut when_anchor = state.anchor.clone();
        state.inner.teardown(
            &mut state.inner_state,
            ctx,
            &mut when_anchor,
            AnchorType::Before,
        );
    }

    fn notify_state(
        &self,
        path: &[super::ViewId],
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) {
        let mut when_anchor = state.anchor.clone();
        state.inner.notify_state(
            path,
            &mut state.inner_state,
            ctx,
            &mut when_anchor,
            AnchorType::Before,
        );
    }

    fn collect_nodes(
        &self,
        state: &Self::ViewState,
        nodes: &mut Vec<godot::prelude::Gd<godot::prelude::Node>>,
    ) {
        state.inner.collect_nodes(&state.inner_state, nodes);
        nodes.push(state.anchor.clone());
    }
}

pub fn when<T, InnerFn, Inner>(value: T, inner_fn: InnerFn) -> When<T, InnerFn>
where
    T: PartialEq,
    InnerFn: Fn() -> Inner,
    Inner: View,
{
    When { value, inner_fn }
}
