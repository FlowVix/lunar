pub mod state;

use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use crate::{
    system::{STATES, StateData, StateId},
    view::{View, ViewId, stateful::state::State},
};

pub struct Stateful<StateFn, InnerFn> {
    state_fn: StateFn,
    inner_fn: InnerFn,
}
pub struct StatefulViewState<T: 'static, Inner: View> {
    state: State<T>,
    inner: Inner,
    inner_state: Inner::ViewState,
    inner_id: ViewId,
}

impl<StateFn, InnerFn, T, Inner> View for Stateful<StateFn, InnerFn>
where
    T: 'static,
    StateFn: Fn() -> T,
    InnerFn: Fn(State<T>) -> Inner,
    Inner: View,
{
    type ViewState = StatefulViewState<T, Inner>;

    fn build(
        &self,
        ctx: &mut crate::ctx::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) -> Self::ViewState {
        let value = (self.state_fn)();
        let path: Rc<[ViewId]> = ctx.path.clone().into();
        let id = STATES.with_borrow_mut(|states| {
            states.insert(StateData {
                value: Rc::new(RefCell::new(value)),
                path,
            })
        });
        let state = State {
            state_id: id,
            app_id: ctx.app_id,
            _p: PhantomData,
        };
        let inner = (self.inner_fn)(state);
        let inner_id = ctx.new_structural_id();
        let inner_state = ctx.with_id(inner_id, |ctx| inner.build(ctx, anchor, anchor_type));
        StatefulViewState {
            state,
            inner,
            inner_state,
            inner_id,
        }
    }

    fn rebuild(
        &self,
        _prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) {
        let inner = (self.inner_fn)(state.state);
        ctx.with_id(state.inner_id, |ctx| {
            inner.rebuild(
                &state.inner,
                &mut state.inner_state,
                ctx,
                anchor,
                anchor_type,
            );
        });
        state.inner = inner;
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) {
        ctx.with_id(state.inner_id, |ctx| {
            state
                .inner
                .teardown(&mut state.inner_state, ctx, anchor, anchor_type);
        });
    }

    fn notify_state(
        &self,
        path: &[ViewId],
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) {
        if let Some((start, rest)) = path.split_first() {
            if *start == state.inner_id {
                ctx.with_id(state.inner_id, |ctx| {
                    state.inner.notify_state(
                        rest,
                        &mut state.inner_state,
                        ctx,
                        anchor,
                        anchor_type,
                    );
                });
            }
        } else {
            let new = (self.inner_fn)(state.state);
            ctx.with_id(state.inner_id, |ctx| {
                new.rebuild(
                    &state.inner,
                    &mut state.inner_state,
                    ctx,
                    anchor,
                    anchor_type,
                );
            });
            state.inner = new;
        }
    }

    fn collect_nodes(
        &self,
        state: &Self::ViewState,
        nodes: &mut Vec<godot::prelude::Gd<godot::prelude::Node>>,
    ) {
        state.inner.collect_nodes(&state.inner_state, nodes);
    }
}

pub fn stateful<T, Inner, StateFn, InnerFn>(
    init: StateFn,
    view: InnerFn,
) -> Stateful<StateFn, InnerFn>
where
    T: 'static,
    StateFn: Fn() -> T,
    InnerFn: Fn(State<T>) -> Inner,
    Inner: View,
{
    Stateful {
        state_fn: init,
        inner_fn: view,
    }
}
