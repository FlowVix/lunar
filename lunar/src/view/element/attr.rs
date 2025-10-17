use godot::{builtin::Variant, classes::Node, meta::ToGodot, obj::Inherits, prelude::Gd};
use std::marker::PhantomData;

use crate::view::{
    AnchorType, View,
    element::{ElementView, impl_element_view},
};

pub struct Attr<N, Name, Inner, const BUILD_ONLY: bool> {
    pub(crate) inner: Inner,
    pub(crate) name: Name,
    pub(crate) value: Variant,
    pub(crate) _p: PhantomData<N>,
}

pub struct AttrViewState<InnerViewState> {
    prev_value: Variant,
    inner_view_state: InnerViewState,
}

impl<N, Name, Inner, const BUILD_ONLY: bool> View for Attr<N, Name, Inner, BUILD_ONLY>
where
    Inner: ElementView<N>,
    Name: AsRef<str> + Clone,
    N: Inherits<Node>,
{
    type ViewState = AttrViewState<Inner::ViewState>;

    fn build(
        &self,
        ctx: &mut crate::ctx::Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) -> Self::ViewState {
        let inner_view_state = self.inner.build(ctx, anchor, anchor_type);
        let mut node = self.inner.get_node(&inner_view_state);
        let prev_value = node.upcast_ref().get(self.name.as_ref());
        node.upcast_mut().set(self.name.as_ref(), &self.value);
        AttrViewState {
            prev_value,
            inner_view_state,
        }
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        self.inner.rebuild(
            &prev.inner,
            &mut state.inner_view_state,
            ctx,
            anchor,
            anchor_type,
        );

        if !BUILD_ONLY {
            let mut node = self.get_node(state);
            if self.name.as_ref() != prev.name.as_ref() {
                node.upcast_mut().set(prev.name.as_ref(), &state.prev_value);
            }
            state.prev_value = node.upcast_ref().get(self.name.as_ref());
            node.upcast_mut().set(self.name.as_ref(), &self.value);
        }
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        self.inner
            .teardown(&mut state.inner_view_state, ctx, anchor, anchor_type);
    }

    fn notify_state(
        &self,
        path: &[crate::view::ViewId],
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: crate::view::AnchorType,
    ) {
        self.inner
            .notify_state(path, &mut state.inner_view_state, ctx, anchor, anchor_type);
    }

    fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<Gd<Node>>) {
        self.inner.collect_nodes(&state.inner_view_state, nodes);
    }
}

impl<N, Name, Inner, const BUILD_ONLY: bool> ElementView<N> for Attr<N, Name, Inner, BUILD_ONLY>
where
    Inner: ElementView<N>,
    Name: AsRef<str> + Clone,
    N: Inherits<Node>,
{
    fn get_node(&self, state: &Self::ViewState) -> Gd<N> {
        self.inner.get_node(&state.inner_view_state)
    }
}

impl<N, Name0, Inner, const BUILD_ONLY0: bool> Attr<N, Name0, Inner, BUILD_ONLY0> {
    impl_element_view! { N }
}
