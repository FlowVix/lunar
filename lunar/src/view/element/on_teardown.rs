use std::marker::PhantomData;

use godot::{builtin::Variant, classes::Node, meta::ToGodot, obj::Inherits, prelude::Gd};

use crate::{AnchorType, ElementView, View, view::element::impl_element_view};

pub struct OnTeardown<N, Cb, Inner> {
    pub(crate) inner: Inner,
    pub(crate) cb: Cb,
    pub(crate) _p: PhantomData<N>,
}

pub struct OnTeardownViewState<InnerViewState> {
    inner_view_state: InnerViewState,
}

impl<N, Cb, Inner> View for OnTeardown<N, Cb, Inner>
where
    Inner: ElementView<N>,
    Cb: Fn(),
    N: Inherits<Node>,
{
    type ViewState = OnTeardownViewState<Inner::ViewState>;

    fn build(
        &self,
        ctx: &mut crate::Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) -> Self::ViewState {
        let inner_view_state = self.inner.build(ctx, anchor, anchor_type);

        OnTeardownViewState { inner_view_state }
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
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
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        (self.cb)();

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
}

impl<N, Cb, Inner> ElementView<N> for OnTeardown<N, Cb, Inner>
where
    Inner: ElementView<N>,
    Cb: Fn(),
    N: Inherits<Node>,
{
    fn get_node(&self, state: &Self::ViewState) -> Gd<N> {
        self.inner.get_node(&state.inner_view_state)
    }
}

impl<N, Cb0, Inner> OnTeardown<N, Cb0, Inner> {
    impl_element_view! { N }
}
