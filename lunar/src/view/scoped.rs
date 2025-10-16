use crate::{
    system::{ScopeId, child_scope, in_scope},
    view::View,
};

pub struct Scoped<InnerFn> {
    inner_fn: InnerFn,
}
pub struct ScopedViewState<Inner: View> {
    inner: Inner,
    inner_state: Inner::ViewState,
    scope: ScopeId,
}

impl<InnerFn, Inner> View for Scoped<InnerFn>
where
    InnerFn: Fn() -> Inner,
    Inner: View,
{
    type ViewState = ScopedViewState<Inner>;

    fn build(
        &self,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) -> (Self::ViewState, Option<crate::system::ScopeId>) {
        let ((inner, inner_state), scope) = child_scope(|| {
            let inner = (self.inner_fn)();
            let (build, _) = inner.build(anchor, anchor_type);
            (inner, build)
        });
        (
            ScopedViewState {
                inner,
                inner_state,
                scope,
            },
            Some(scope),
        )
    }

    fn rebuild(
        &self,
        _prev: &Self,
        state: &mut Self::ViewState,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) -> Option<crate::system::ScopeId> {
        in_scope(state.scope, || {
            let inner = (self.inner_fn)();
            inner.rebuild(&state.inner, &mut state.inner_state, anchor, anchor_type);
            state.inner = inner;
        });
        Some(state.scope)
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) {
        todo!()
    }
}
