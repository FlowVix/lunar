pub mod any;
pub mod either;
pub mod element;
pub mod iter;
pub mod memo;
pub mod on_build;
pub mod on_change;
pub mod on_physics_process;
pub mod on_process;
pub mod on_ready;
pub mod on_rebuild;
pub mod on_teardown;
pub mod option;
pub mod stateful;
pub mod when;

use std::ops::Deref;

use godot::{classes::Node, obj::Gd};

use crate::ctx::Context;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewId {
    Structural(u64),
    Key(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnchorType {
    ChildOf,
    Before,
}
impl AnchorType {
    pub fn add(self, anchor: &mut Node, node: &Gd<Node>) {
        match self {
            AnchorType::ChildOf => anchor.add_child(node),
            AnchorType::Before => {
                let idx = anchor.get_index();
                let mut parent = anchor.get_parent().unwrap();
                parent.add_child(node);
                parent.move_child(node, idx);
            }
        }
    }
    pub fn remove(self, anchor: &mut Node, node: &Gd<Node>) {
        match self {
            AnchorType::ChildOf => anchor.remove_child(node),
            AnchorType::Before => anchor.get_parent().unwrap().remove_child(node),
        }
    }
}

pub trait View {
    type ViewState;

    fn build(
        &self,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) -> Self::ViewState;
    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    );
    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    );
    fn notify_state(
        &self,
        path: &[ViewId],
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: AnchorType,
    );
    fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<Gd<Node>>);
}

impl<Inner> View for Box<Inner>
where
    Inner: View + ?Sized,
{
    type ViewState = Inner::ViewState;

    fn build(
        &self,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) -> Self::ViewState {
        self.deref().build(ctx, anchor, anchor_type)
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        self.deref().rebuild(prev, state, ctx, anchor, anchor_type);
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        self.deref().teardown(state, ctx, anchor, anchor_type);
    }

    fn notify_state(
        &self,
        path: &[ViewId],
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: AnchorType,
    ) {
        self.deref()
            .notify_state(path, state, ctx, anchor, anchor_type);
    }

    fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<Gd<Node>>) {
        self.deref().collect_nodes(state, nodes);
    }
}

impl<Inner> View for &Inner
where
    Inner: View + ?Sized,
{
    type ViewState = Inner::ViewState;

    fn build(
        &self,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) -> Self::ViewState {
        (*self).build(ctx, anchor, anchor_type)
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        (*self).rebuild(prev, state, ctx, anchor, anchor_type);
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        (*self).teardown(state, ctx, anchor, anchor_type);
    }

    fn notify_state(
        &self,
        path: &[ViewId],
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: AnchorType,
    ) {
        (*self).notify_state(path, state, ctx, anchor, anchor_type);
    }

    fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<Gd<Node>>) {
        (*self).collect_nodes(state, nodes);
    }
}

macro_rules! tuple_impl {
    ($($v:literal)*) => {
        paste::paste! {
            impl<$( [< V $v >] ,)*> View for ($( [< V $v >] ,)*) where $( [< V $v >] : View,)* {
                type ViewState = ($( ([<V $v>]::ViewState, ViewId), )*);

                #[allow(clippy::unused_unit)]
                #[allow(unused_variables)]
                fn build(&self, ctx: &mut Context, anchor: &mut Node, anchor_type: AnchorType) -> Self::ViewState {
                    (
                        $(
                            {
                                let child_id = ctx.new_structural_id();
                                (ctx.with_id(child_id, |ctx| {
                                    self.$v.build(ctx, anchor, anchor_type)
                                }), child_id)
                            },
                        )*
                    )
                }
                #[allow(unused_variables)]
                fn rebuild(
                    &self,
                    prev: &Self,
                    state: &mut Self::ViewState,
                    ctx: &mut Context,
                    anchor: &mut Node,
                    anchor_type: AnchorType,
                ) {
                    $(
                        ctx.with_id(state.$v.1, |ctx| {
                            self.$v.rebuild(&prev.$v, &mut state.$v.0, ctx, anchor, anchor_type);
                        });
                    )*
                }
                #[allow(unused_variables)]
                fn teardown(&self, state: &mut Self::ViewState, ctx: &mut Context, anchor: &mut Node, anchor_type: AnchorType) {
                    $(
                        ctx.with_id(state.$v.1, |ctx| {
                            self.$v.teardown(&mut state.$v.0, ctx, anchor, anchor_type);
                        });
                    )*
                }

                #[allow(unused_variables)]
                fn notify_state(
                    &self,
                    path: &[ViewId],
                    state: &mut Self::ViewState,
                    ctx: &mut crate::ctx::Context,
                    anchor: &mut godot::prelude::Node,
                    anchor_type: AnchorType,
                ) {
                    if let Some((start, rest)) = path.split_first() {
                        $(
                            if *start == state.$v.1 {
                                ctx.with_id(state.$v.1, |ctx| {
                                    self.$v.notify_state(rest, &mut state.$v.0, ctx, anchor, anchor_type);
                                });
                            }
                        )*
                    }
                }

                #[allow(unused_variables)]
                fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<Gd<Node>>) {
                    $(
                        self.$v.collect_nodes(&state.$v.0, nodes);
                    )*
                }
            }
        }
    };
}

tuple_impl! {}
tuple_impl! { 0 }
tuple_impl! { 0 1 }
tuple_impl! { 0 1 2 }
tuple_impl! { 0 1 2 3 }
tuple_impl! { 0 1 2 3 4 }
tuple_impl! { 0 1 2 3 4 5 }
tuple_impl! { 0 1 2 3 4 5 6 }
tuple_impl! { 0 1 2 3 4 5 6 7 }
tuple_impl! { 0 1 2 3 4 5 6 7 8 }
tuple_impl! { 0 1 2 3 4 5 6 7 8 9 }
tuple_impl! { 0 1 2 3 4 5 6 7 8 9 10 }
