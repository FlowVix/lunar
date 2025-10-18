use crate::View;

pub struct OnRebuild<Cb> {
    cb: Cb,
}

impl<Cb> View for OnRebuild<Cb>
where
    Cb: Fn(),
{
    type ViewState = ();

    fn build(
        &self,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) -> Self::ViewState {
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) {
        (self.cb)()
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

pub fn on_rebuild<Cb>(cb: Cb) -> OnRebuild<Cb>
where
    Cb: Fn(),
{
    OnRebuild { cb }
}
