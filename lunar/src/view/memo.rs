use godot::{
    classes::Node,
    obj::{Gd, NewAlloc},
};

use crate::{AnchorType, View};

pub struct Memo<T, InnerFn> {
    value: T,
    inner_fn: InnerFn,
}

pub struct MemoViewState<Inner: View> {
    inner: Inner,
    inner_state: Inner::ViewState,
}

impl<T, InnerFn, Inner> View for Memo<T, InnerFn>
where
    T: PartialEq,
    InnerFn: Fn() -> Inner,
    Inner: View,
{
    type ViewState = MemoViewState<Inner>;

    fn build(
        &self,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) -> Self::ViewState {
        let inner = (self.inner_fn)();
        let inner_state = inner.build(ctx, anchor, anchor_type);

        MemoViewState { inner, inner_state }
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) {
        if self.value != prev.value {
            let inner = (self.inner_fn)();
            inner.rebuild(
                &state.inner,
                &mut state.inner_state,
                ctx,
                anchor,
                anchor_type,
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
        state
            .inner
            .teardown(&mut state.inner_state, ctx, anchor, anchor_type);
    }

    fn notify_state(
        &self,
        path: &[super::ViewId],
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) {
        state
            .inner
            .notify_state(path, &mut state.inner_state, ctx, anchor, anchor_type);
    }

    fn collect_nodes(
        &self,
        state: &Self::ViewState,
        nodes: &mut Vec<godot::prelude::Gd<godot::prelude::Node>>,
    ) {
        state.inner.collect_nodes(&state.inner_state, nodes);
    }
}

pub fn memo<T, InnerFn, Inner>(value: T, inner_fn: InnerFn) -> Memo<T, InnerFn>
where
    T: PartialEq,
    InnerFn: Fn() -> Inner,
    Inner: View,
{
    Memo { value, inner_fn }
}
