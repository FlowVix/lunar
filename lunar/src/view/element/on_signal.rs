use godot::{
    builtin::{Callable, Variant},
    classes::Node,
    meta::ToGodot,
    obj::{Inherits, NewAlloc},
    prelude::Gd,
};
use std::{marker::PhantomData, rc::Rc, sync::Arc};

use crate::view::{
    AnchorType, View,
    element::{ElementView, impl_element_view},
};

pub struct OnSignal<N, Name, Cb, Inner> {
    pub(crate) inner: Inner,
    pub(crate) name: Name,
    pub(crate) cb: Rc<Cb>,
    pub(crate) _p: PhantomData<N>,
}

pub struct OnSignalViewState<InnerViewState> {
    callable: Callable,
    inner_view_state: InnerViewState,
}

impl<N, Name, Cb, Inner> View for OnSignal<N, Name, Cb, Inner>
where
    Inner: ElementView<N>,
    Name: AsRef<str> + Clone,
    Cb: Fn(&[&Variant]) + 'static,
    N: Inherits<Node>,
{
    type ViewState = OnSignalViewState<Inner::ViewState>;

    fn build(
        &self,
        ctx: &mut crate::ctx::Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) -> Self::ViewState {
        let inner_view_state = self.inner.build(ctx, anchor, anchor_type);
        let mut node = self.inner.get_node(&inner_view_state);

        let cb = self.cb.clone();
        let callable = Callable::from_fn("boing", move |args| cb(args));

        node.upcast_mut().connect(self.name.as_ref(), &callable);
        OnSignalViewState {
            callable,
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
        let mut node = self.get_node(state);

        node.upcast_mut()
            .disconnect(prev.name.as_ref(), &state.callable);

        let cb = self.cb.clone();
        let callable = Callable::from_fn("boing", move |args| cb(args));

        node.upcast_mut().connect(self.name.as_ref(), &callable);
        state.callable = callable;
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

    // fn message(
    //     &self,
    //     msg: crate::Message,
    //     path: &[crate::ViewID],
    //     view_state: &mut Self::ViewState,
    //     app_state: &mut State,
    // ) -> MessageResult {
    //     if path.is_empty() {
    //         match msg {
    //             Message::Signal { ref name, ref args } => {
    //                 if **name == *self.name.as_ref() {
    //                     let node = self.get_node(view_state);
    //                     (self.cb)(app_state, args, node);
    //                     return MessageResult::Success;
    //                 }
    //             }
    //             _ => {}
    //         }
    //     }
    //     self.inner
    //         .message(msg, path, &mut view_state.inner_view_state, app_state)
    // }

    // fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<Gd<Node>>) {
    //     self.inner.collect_nodes(&state.inner_view_state, nodes);
    // }
}

impl<N, Name, Cb, Inner> ElementView<N> for OnSignal<N, Name, Cb, Inner>
where
    Inner: ElementView<N>,
    Name: AsRef<str> + Clone,
    Cb: Fn(&[&Variant]) + 'static,
    N: Inherits<Node>,
{
    fn get_node(&self, state: &Self::ViewState) -> Gd<N> {
        self.inner.get_node(&state.inner_view_state)
    }
}

impl<N, Name0, Cb0, Inner> OnSignal<N, Name0, Cb0, Inner> {
    impl_element_view! { N }
}
