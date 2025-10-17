use either::Either::{self, Left, Right};
use godot::{
    classes::Node,
    obj::{Gd, NewAlloc},
};

use crate::{AnchorType, Context, View, ViewId};

pub struct EitherViewState<AViewState, BViewState> {
    anchor: Gd<Node>,
    inner: Either<AViewState, BViewState>,
    id: ViewId,
}

impl<A, B> View for Either<A, B>
where
    A: View,
    B: View,
{
    type ViewState = EitherViewState<A::ViewState, B::ViewState>;

    fn build(
        &self,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) -> Self::ViewState {
        let mut eit_anchor = Node::new_alloc();
        anchor_type.add(anchor, &eit_anchor);
        let inner_id = ctx.new_structural_id();
        EitherViewState {
            inner: self.as_ref().map_either_with(
                (ctx, &mut eit_anchor),
                |(ctx, opt_anchor), v| {
                    ctx.with_id(inner_id, |ctx| v.build(ctx, opt_anchor, AnchorType::Before))
                },
                |(ctx, opt_anchor), v| {
                    ctx.with_id(inner_id, |ctx| v.build(ctx, opt_anchor, AnchorType::Before))
                },
            ),
            anchor: eit_anchor,
            id: inner_id,
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
            prev.is_left(),
            state.inner.is_left(),
            "Bruh why are they not the same"
        );
        let mut eit_anchor = state.anchor.clone();
        match (self, prev, &mut state.inner) {
            (Left(new), Left(prev), Left(inner)) => {
                ctx.with_id(state.id, |ctx| {
                    new.rebuild(prev, inner, ctx, &mut eit_anchor, AnchorType::Before);
                });
            }
            (Right(new), Right(prev), Right(inner)) => {
                ctx.with_id(state.id, |ctx| {
                    new.rebuild(prev, inner, ctx, &mut eit_anchor, AnchorType::Before);
                });
            }
            (Right(new), Left(prev), Left(inner)) => {
                ctx.with_id(state.id, |ctx| {
                    prev.teardown(inner, ctx, &mut eit_anchor, AnchorType::Before);
                });
                state.id = ctx.new_structural_id();
                state.inner = Right(ctx.with_id(state.id, |ctx| {
                    new.build(ctx, &mut eit_anchor, AnchorType::Before)
                }));
            }
            (Left(new), Right(prev), Right(inner)) => {
                ctx.with_id(state.id, |ctx| {
                    prev.teardown(inner, ctx, &mut eit_anchor, AnchorType::Before);
                });
                state.id = ctx.new_structural_id();
                state.inner = Left(ctx.with_id(state.id, |ctx| {
                    new.build(ctx, &mut eit_anchor, AnchorType::Before)
                }));
            }
            _ => unreachable!(),
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
            self.is_left(),
            state.inner.is_left(),
            "Bruh why are they not the same"
        );
        let mut eit_anchor = state.anchor.clone();

        match (self, &mut state.inner) {
            (Left(val), Left(inner)) => {
                ctx.with_id(state.id, |ctx| {
                    val.teardown(inner, ctx, &mut eit_anchor, AnchorType::Before);
                });
            }
            (Right(val), Right(inner)) => {
                ctx.with_id(state.id, |ctx| {
                    val.teardown(inner, ctx, &mut eit_anchor, AnchorType::Before);
                });
            }
            _ => unreachable!(),
        }

        anchor_type.remove(anchor, &eit_anchor);
        eit_anchor.queue_free();
    }

    fn notify_state(
        &self,
        path: &[ViewId],
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) {
        assert_eq!(
            self.is_left(),
            state.inner.is_left(),
            "Bruh why are they not the same"
        );
        let mut eit_anchor = state.anchor.clone();
        if let Some((start, rest)) = path.split_first() {
            match (self, &mut state.inner) {
                (Left(val), Left(inner)) => {
                    if *start == state.id {
                        ctx.with_id(state.id, |ctx| {
                            val.notify_state(rest, inner, ctx, &mut eit_anchor, AnchorType::Before);
                        })
                    }
                }
                (Right(val), Right(inner)) => {
                    if *start == state.id {
                        ctx.with_id(state.id, |ctx| {
                            val.notify_state(rest, inner, ctx, &mut eit_anchor, AnchorType::Before);
                        })
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<Gd<Node>>) {
        assert_eq!(
            self.is_left(),
            state.inner.is_left(),
            "Bruh why are they not the same"
        );
        match (self, &state.inner) {
            (Left(val), Left(inner)) => {
                val.collect_nodes(inner, nodes);
            }
            (Right(val), Right(inner)) => {
                val.collect_nodes(inner, nodes);
            }
            _ => unreachable!(),
        }
        nodes.push(state.anchor.clone());
    }
}
