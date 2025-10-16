use godot::{
    classes::Node,
    obj::{Gd, NewAlloc},
};

use crate::{
    system::ScopeId,
    view::{AnchorType, View},
};

pub struct OptionViewState<InnerViewState> {
    anchor: Gd<Node>,
    inner: Option<InnerViewState>,
}

impl<Inner> View for Option<Inner>
where
    Inner: View,
{
    type ViewState = OptionViewState<Inner::ViewState>;

    fn build(
        &self,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) -> (Self::ViewState, Option<ScopeId>) {
        let mut opt_anchor = Node::new_alloc();
        anchor_type.add(anchor, &opt_anchor);
        let mut ret_id = None;
        (
            OptionViewState {
                anchor: opt_anchor.clone(),
                inner: self.as_ref().map(|inner| {
                    let (build, scope_id) = inner.build(&mut opt_anchor, AnchorType::Before);
                    ret_id = scope_id;
                    build
                }),
            },
            ret_id,
        )
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) -> Option<ScopeId> {
        assert_eq!(
            prev.is_some(),
            state.inner.is_some(),
            "Bruh why are they not the same"
        );
        let mut opt_anchor = state.anchor.clone();
        match (self, prev.as_ref().zip(state.inner.as_mut())) {
            (None, None) => None,
            (None, Some((prev, inner_state))) => {
                prev.teardown(inner_state, &mut opt_anchor, AnchorType::Before);
                state.inner = None;
                None
            }
            (Some(new), None) => {
                let build = new.build(&mut opt_anchor, AnchorType::Before);
                state.inner = Some(build.0);
                build.1
            }
            (Some(new), Some((prev, inner_state))) => {
                new.rebuild(prev, inner_state, &mut opt_anchor, AnchorType::Before)
            }
        }
    }

    fn teardown(&self, state: &mut Self::ViewState, anchor: &mut Node, anchor_type: AnchorType) {
        assert_eq!(
            self.is_some(),
            state.inner.is_some(),
            "Bruh why are they not the same"
        );
        let mut opt_anchor = state.anchor.clone();

        if let Some((val, inner)) = self.as_ref().zip(state.inner.as_mut()) {
            val.teardown(inner, &mut opt_anchor, AnchorType::Before);
        }
        anchor_type.remove(anchor, &opt_anchor);
        opt_anchor.queue_free();
    }
}
