use std::rc::Rc;

use godot::{
    classes::{INode, Node},
    obj::{Base, Gd},
    prelude::{GodotClass, godot_api},
};

use crate::View;

#[doc(hidden)]
#[derive(GodotClass)]
#[class(base=Node, no_init)]
pub struct __LunarOnProcessNode {
    base: Base<Node>,

    cb: Rc<dyn Fn(f64)>,
}
#[godot_api]
impl INode for __LunarOnProcessNode {
    fn process(&mut self, delta: f64) {
        (self.cb)(delta);
    }
}

pub struct OnProcess<Cb> {
    cb: Rc<Cb>,
}

impl<Cb> View for OnProcess<Cb>
where
    Cb: Fn(f64) + 'static,
{
    type ViewState = Gd<__LunarOnProcessNode>;

    fn build(
        &self,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) -> Self::ViewState {
        let node = Gd::from_init_fn(|base| __LunarOnProcessNode {
            base,
            cb: self.cb.clone(),
        });
        anchor_type.add(anchor, &node.clone().upcast::<Node>());

        node
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
        anchor_type.remove(anchor, &state.clone().upcast());
        state.upcast_mut::<Node>().queue_free();
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
        nodes.push(state.clone().upcast::<Node>());
    }
}

pub fn on_process<Cb>(cb: Cb) -> OnProcess<Cb>
where
    Cb: Fn(f64),
{
    OnProcess { cb: Rc::new(cb) }
}
