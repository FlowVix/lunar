use godot::{
    classes::Node,
    obj::{Gd, NewAlloc},
};

use crate::{
    ctx::Context,
    view::{AnchorType, View, ViewId},
};

pub struct OptionViewState<InnerViewState> {
    anchor: Gd<Node>,
    inner: Option<(InnerViewState, ViewId)>,
}

impl<Inner> View for Option<Inner>
where
    Inner: View,
{
    type ViewState = OptionViewState<Inner::ViewState>;

    fn build(
        &self,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) -> Self::ViewState {
        let mut opt_anchor = Node::new_alloc();
        anchor_type.add(anchor, &opt_anchor);
        OptionViewState {
            anchor: opt_anchor.clone(),
            inner: self.as_ref().map(|inner| {
                let inner_id = ctx.new_structural_id();
                (
                    ctx.with_id(inner_id, |ctx| {
                        inner.build(ctx, &mut opt_anchor, AnchorType::Before)
                    }),
                    inner_id,
                )
            }),
        }
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        assert_eq!(
            prev.is_some(),
            state.inner.is_some(),
            "Bruh why are they not the same"
        );
        let mut opt_anchor = state.anchor.clone();
        match (self, prev.as_ref().zip(state.inner.as_mut())) {
            (None, None) => {}
            (None, Some((prev, (inner_state, id)))) => {
                ctx.with_id(*id, |ctx| {
                    prev.teardown(inner_state, ctx, &mut opt_anchor, AnchorType::Before);
                });
                state.inner = None;
            }
            (Some(new), None) => {
                let inner_id = ctx.new_structural_id();
                state.inner = Some((
                    ctx.with_id(inner_id, |ctx| {
                        new.build(ctx, &mut opt_anchor, AnchorType::Before)
                    }),
                    inner_id,
                ));
            }
            (Some(new), Some((prev, (inner_state, id)))) => {
                ctx.with_id(*id, |ctx| {
                    new.rebuild(prev, inner_state, ctx, &mut opt_anchor, AnchorType::Before);
                });
            }
        }
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        assert_eq!(
            self.is_some(),
            state.inner.is_some(),
            "Bruh why are they not the same"
        );
        let mut opt_anchor = state.anchor.clone();

        if let Some((val, (inner, id))) = self.as_ref().zip(state.inner.as_mut()) {
            ctx.with_id(*id, |ctx| {
                val.teardown(inner, ctx, &mut opt_anchor, AnchorType::Before);
            });
        }
        anchor_type.remove(anchor, &opt_anchor);
        opt_anchor.queue_free();
    }

    fn notify_state(
        &self,
        path: &[ViewId],
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: AnchorType,
    ) {
        assert_eq!(
            self.is_some(),
            state.inner.is_some(),
            "Bruh why are they not the same"
        );
        let mut opt_anchor = state.anchor.clone();
        if let Some((start, rest)) = path.split_first()
            && let Some((val, (inner, child_id))) = self.as_ref().zip(state.inner.as_mut())
            && start == child_id
        {
            ctx.with_id(*child_id, |ctx| {
                val.notify_state(rest, inner, ctx, &mut opt_anchor, AnchorType::Before)
            })
        }
    }
}
