use std::cell::Cell;

use crate::View;

pub struct OnChange<T, Cb> {
    value: T,
    initial: bool,
    cb: Cell<Option<Cb>>,
}

impl<T, Cb> View for OnChange<T, Cb>
where
    T: PartialEq,
    Cb: Fn(),
{
    type ViewState = ();

    fn build(
        &self,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) -> Self::ViewState {
        if self.initial {
            self.cb.take().unwrap()();
        }
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) {
        if self.value != prev.value {
            self.cb.take().unwrap()();
        }
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

pub fn on_change<T, Cb>(value: T, cb: Cb) -> OnChange<T, Cb>
where
    T: PartialEq,
    Cb: Fn(),
{
    OnChange {
        value,
        initial: false,
        cb: Cell::new(Some(cb)),
    }
}
pub fn on_change_init<T, Cb>(value: T, cb: Cb) -> OnChange<T, Cb>
where
    T: PartialEq,
    Cb: Fn(),
{
    OnChange {
        value,
        initial: true,
        cb: Cell::new(Some(cb)),
    }
}
