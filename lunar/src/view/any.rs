use std::any::Any;

use godot::{
    classes::Node,
    obj::{Gd, NewAlloc},
};

use crate::{
    ctx::Context,
    view::{AnchorType, View, ViewId},
};

pub trait AnyView {
    fn as_any(&self) -> &dyn Any;
    fn dyn_build(
        &self,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) -> AnyViewState;
    fn dyn_rebuild(
        &self,
        prev: &dyn AnyView,
        state: &mut AnyViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    );
    fn dyn_teardown(
        &self,
        state: &mut AnyViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    );
    fn dyn_notify_state(
        &self,
        path: &[ViewId],
        state: &mut AnyViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    );
    fn dyn_collect_nodes(&self, state: &AnyViewState, nodes: &mut Vec<Gd<Node>>);
}

pub struct AnyViewState {
    anchor: Gd<Node>,
    inner: Box<dyn Any>,
    id: ViewId,
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

    fn dyn_build(
        &self,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) -> AnyViewState {
        let mut any_anchor = Node::new_alloc();
        anchor_type.add(anchor, &any_anchor);
        let inner_id = ctx.new_structural_id();

        let inner = ctx.with_id(inner_id, |ctx| {
            self.build(ctx, &mut any_anchor, AnchorType::Before)
        });
        AnyViewState {
            anchor: any_anchor,
            inner: Box::new(inner),
            id: inner_id,
        }
    }

    fn dyn_rebuild(
        &self,
        prev: &dyn AnyView,
        state: &mut AnyViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        let mut any_anchor = state.anchor.clone();
        if let Some(prev) = prev.as_any().downcast_ref::<V>() {
            let inner = state
                .inner
                .downcast_mut::<V::ViewState>()
                .expect("What the hell bro");

            ctx.with_id(state.id, |ctx| {
                self.rebuild(prev, inner, ctx, &mut any_anchor, AnchorType::Before);
            })
        } else {
            ctx.with_id(state.id, |ctx| {
                prev.dyn_teardown(state, ctx, &mut any_anchor, AnchorType::Before);
            });
            state.id = ctx.new_structural_id();
            let inner = ctx.with_id(state.id, |ctx| {
                self.build(ctx, &mut any_anchor, AnchorType::Before)
            });
            state.inner = Box::new(inner);
        }
    }

    fn dyn_teardown(
        &self,
        state: &mut AnyViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        let inner = state
            .inner
            .downcast_mut::<V::ViewState>()
            .expect("What the hell bro");
        let mut any_anchor = state.anchor.clone();
        ctx.with_id(state.id, |ctx| {
            self.teardown(inner, ctx, &mut any_anchor, AnchorType::Before);
        });
    }

    fn dyn_notify_state(
        &self,
        path: &[ViewId],
        state: &mut AnyViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        let inner = state
            .inner
            .downcast_mut::<V::ViewState>()
            .expect("What the hell bro");
        let mut any_anchor = state.anchor.clone();
        if let Some((start, rest)) = path.split_first()
            && *start == state.id
        {
            ctx.with_id(state.id, |ctx| {
                self.notify_state(rest, inner, ctx, &mut any_anchor, AnchorType::Before)
            });
        }
    }

    fn dyn_collect_nodes(&self, state: &AnyViewState, nodes: &mut Vec<Gd<Node>>) {
        let inner = state
            .inner
            .downcast_ref::<V::ViewState>()
            .expect("What the hell bro");
        self.collect_nodes(inner, nodes);
        nodes.push(state.anchor.clone());
    }
}

// MARK: View for dyn AnyView

macro_rules! dyn_anyview_impl {
    ($($who:tt)*) => {
        impl View for $($who)* {
            type ViewState = AnyViewState;

            fn build(
                &self,
                ctx: &mut Context,
                anchor: &mut Node,
                anchor_type: AnchorType,
            ) -> Self::ViewState {
                self.dyn_build(ctx, anchor, anchor_type)
            }

            fn rebuild(
                &self,
                prev: &Self,
                state: &mut Self::ViewState,
                ctx: &mut Context,
                anchor: &mut Node,
                anchor_type: AnchorType,
            ) {
                self.dyn_rebuild(prev, state, ctx, anchor, anchor_type);
            }

            fn teardown(
                &self,
                state: &mut Self::ViewState,
                ctx: &mut Context,
                anchor: &mut Node,
                anchor_type: AnchorType,
            ) {
                self.dyn_teardown(state, ctx, anchor, anchor_type);
            }

            fn notify_state(
                &self,
                path: &[ViewId],
                state: &mut Self::ViewState,
                ctx: &mut Context,
                anchor: &mut Node,
                anchor_type: AnchorType,
            ) {
                self.dyn_notify_state(path, state, ctx, anchor, anchor_type);
            }

            fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<Gd<Node>>) {
                self.dyn_collect_nodes(state, nodes);
            }
        }
    };
}

dyn_anyview_impl! { dyn AnyView }
dyn_anyview_impl! { dyn AnyView + Send }
dyn_anyview_impl! { dyn AnyView + Send + Sync }
