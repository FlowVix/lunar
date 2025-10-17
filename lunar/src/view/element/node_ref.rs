use std::marker::PhantomData;

use godot::{
    classes::Node,
    obj::{Gd, Inherits},
};

use crate::{ElementView, State, View, view::element::impl_element_view};

pub struct NodeRef<N: Inherits<Node>, Inner> {
    pub(crate) inner: Inner,
    pub(crate) state: State<Option<Gd<N>>>,
}

pub struct NodeRefViewState<InnerViewState> {
    inner_view_state: InnerViewState,
}

impl<N, Inner> View for NodeRef<N, Inner>
where
    Inner: ElementView<N>,
    N: Inherits<Node>,
{
    type ViewState = NodeRefViewState<Inner::ViewState>;

    fn build(
        &self,
        ctx: &mut crate::Context,
        anchor: &mut Node,
        anchor_type: crate::AnchorType,
    ) -> Self::ViewState {
        let inner_view_state = self.inner.build(ctx, anchor, anchor_type);

        let node = self.inner.get_node(&inner_view_state);
        self.state.set(Some(node));

        NodeRefViewState { inner_view_state }
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut Node,
        anchor_type: crate::AnchorType,
    ) {
        self.inner.rebuild(
            &prev.inner,
            &mut state.inner_view_state,
            ctx,
            anchor,
            anchor_type,
        );
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut Node,
        anchor_type: crate::AnchorType,
    ) {
        self.inner
            .teardown(&mut state.inner_view_state, ctx, anchor, anchor_type);
    }

    fn notify_state(
        &self,
        path: &[crate::ViewId],
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: crate::AnchorType,
    ) {
        self.inner
            .notify_state(path, &mut state.inner_view_state, ctx, anchor, anchor_type);
    }

    fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<godot::prelude::Gd<Node>>) {
        self.inner.collect_nodes(&state.inner_view_state, nodes);
    }
}

impl<N, Inner> ElementView<N> for NodeRef<N, Inner>
where
    Inner: ElementView<N>,
    N: Inherits<Node>,
{
    fn get_node(&self, state: &Self::ViewState) -> Gd<N> {
        self.inner.get_node(&state.inner_view_state)
    }
}

impl<N: Inherits<Node>, Inner> NodeRef<N, Inner> {
    impl_element_view! { N }
}
