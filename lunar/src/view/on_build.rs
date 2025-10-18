use std::cell::Cell;

use crate::View;

pub struct OnBuild<Cb> {
    cb: Cell<Option<Cb>>,
}

impl<Cb> View for OnBuild<Cb>
where
    Cb: FnOnce(),
{
    type ViewState = ();

    fn build(
        &self,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) -> Self::ViewState {
        self.cb.take().unwrap()()
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) {
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) {
    }

    fn notify_state(
        &self,
        path: &[super::ViewId],
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) {
    }

    fn collect_nodes(
        &self,
        state: &Self::ViewState,
        nodes: &mut Vec<godot::prelude::Gd<godot::prelude::Node>>,
    ) {
    }
}

pub fn on_build<Cb>(cb: Cb) -> OnBuild<Cb>
where
    Cb: FnOnce(),
{
    OnBuild {
        cb: Cell::new(Some(cb)),
    }
}
