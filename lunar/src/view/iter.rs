use godot::{
    classes::Node,
    obj::{Gd, NewAlloc},
};
use std::{cell::Cell, collections::HashMap, hash::Hash};

use crate::{AnchorType, View, ViewId, util::hash};

pub struct VecViewState<InnerViewState> {
    anchor: Gd<Node>,
    inner: Vec<InnerViewState>,
}

impl<K, Inner> View for Vec<(K, Inner)>
where
    Inner: View,
    K: Hash + Eq,
{
    type ViewState = VecViewState<Inner::ViewState>;

    fn build(
        &self,
        ctx: &mut crate::Context,
        anchor: &mut Node,
        anchor_type: super::AnchorType,
    ) -> Self::ViewState {
        let mut vec_anchor = Node::new_alloc();
        thread_local! {
            static NAME_COUNTER: Cell<usize> = const { Cell::new(0) };
        }
        vec_anchor.set_name(&format!(
            "__VEC_ANCHOR_{}",
            NAME_COUNTER.replace(NAME_COUNTER.get() + 1)
        ));
        anchor_type.add(anchor, &vec_anchor);
        VecViewState {
            anchor: vec_anchor.clone(),
            inner: self
                .iter()
                .map(|(k, inner)| {
                    ctx.with_id(ViewId::Key(hash(k)), |ctx| {
                        inner.build(ctx, &mut vec_anchor, AnchorType::Before)
                    })
                })
                .collect(),
        }
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut Node,
        anchor_type: super::AnchorType,
    ) {
        assert_eq!(
            prev.len(),
            state.inner.len(),
            "Bruh why are they not the same"
        );
        let mut vec_anchor = state.anchor.clone();

        let mut total_nodes = 0;

        let mut prev_map = state
            .inner
            .drain(..)
            .enumerate()
            .map(|(idx, inner)| {
                let mut nodes = vec![];
                prev[idx].1.collect_nodes(&inner, &mut nodes);
                total_nodes += nodes.len();
                (&prev[idx].0, (inner, nodes, &prev[idx].1))
            })
            .collect::<HashMap<_, _>>();

        let mut move_idx = vec_anchor.get_index() as usize - total_nodes;
        for (k, v) in self {
            if let Some((mut inner, nodes, prev)) = prev_map.remove(k) {
                for node in &nodes {
                    node.get_parent().unwrap().move_child(node, move_idx as i32);
                    move_idx += 1;
                }
                ctx.with_id(ViewId::Key(hash(k)), |ctx| {
                    v.rebuild(prev, &mut inner, ctx, &mut vec_anchor, AnchorType::Before);
                });
                state.inner.push(inner);
            } else {
                let inner = ctx.with_id(ViewId::Key(hash(k)), |ctx| {
                    v.build(ctx, &mut vec_anchor, AnchorType::Before)
                });
                let mut nodes = vec![];
                v.collect_nodes(&inner, &mut nodes);
                for node in &nodes {
                    node.get_parent().unwrap().move_child(node, move_idx as i32);
                    move_idx += 1;
                }
                state.inner.push(inner);
            }
        }
        for (k, (mut inner, _, prev)) in prev_map.drain() {
            ctx.with_id(ViewId::Key(hash(k)), |ctx| {
                prev.teardown(&mut inner, ctx, &mut vec_anchor, AnchorType::Before);
            });
        }
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut Node,
        anchor_type: super::AnchorType,
    ) {
        assert_eq!(
            self.len(),
            state.inner.len(),
            "Bruh why are they not the same"
        );
        let mut vec_anchor = state.anchor.clone();

        for ((k, inner), state) in self.iter().zip(&mut state.inner) {
            ctx.with_id(ViewId::Key(hash(k)), |ctx| {
                inner.teardown(state, ctx, &mut vec_anchor, AnchorType::Before);
            });
        }
        anchor_type.remove(anchor, &vec_anchor);
        vec_anchor.queue_free();
    }

    fn notify_state(
        &self,
        path: &[super::ViewId],
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) {
        assert_eq!(
            self.len(),
            state.inner.len(),
            "Bruh why are they not the same"
        );
        let mut vec_anchor = state.anchor.clone();
        if let Some((start, rest)) = path.split_first() {
            for ((k, inner), state) in self.iter().zip(&mut state.inner) {
                if *start == ViewId::Key(hash(k)) {
                    ctx.with_id(*start, |ctx| {
                        inner.notify_state(rest, state, ctx, &mut vec_anchor, AnchorType::Before);
                    });
                }
            }
        }
    }

    fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<Gd<Node>>) {
        assert_eq!(
            self.len(),
            state.inner.len(),
            "Bruh why are they not the same"
        );
        for ((_, inner), state) in self.iter().zip(&state.inner) {
            inner.collect_nodes(state, nodes);
        }
        nodes.push(state.anchor.clone());
    }
}
