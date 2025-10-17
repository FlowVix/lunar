pub mod attr;
pub mod on_build;
pub mod on_signal;
pub mod on_teardown;
pub mod theme_override;

use std::marker::PhantomData;

use godot::{
    builtin::Variant,
    classes::Node,
    meta::ToGodot,
    obj::{Gd, Inherits, NewAlloc},
};

pub struct Element<N, Children> {
    children: Children,
    _p: PhantomData<N>,
}

pub fn el<N: Inherits<Node> + NewAlloc>() -> Element<N, ()> {
    Element {
        children: (),
        _p: PhantomData,
    }
}
impl<N, Children> Element<N, Children> {
    pub fn children<NewChildren>(self, children: NewChildren) -> Element<N, NewChildren> {
        Element {
            children,
            _p: PhantomData,
        }
    }
}

pub struct ElementViewState<N: Inherits<Node>, ChildViewState> {
    node: Gd<N>,
    child_view_state: ChildViewState,
}

impl<N, Children> View for Element<N, Children>
where
    N: Inherits<Node> + NewAlloc,
    Children: View,
{
    type ViewState = ElementViewState<N, Children::ViewState>;

    fn build(
        &self,
        ctx: &mut crate::ctx::Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) -> Self::ViewState {
        let mut node = N::new_alloc();
        anchor_type.add(anchor, &node.clone().upcast::<Node>());

        let child_view_state =
            self.children
                .build(ctx, node.upcast_mut::<Node>(), AnchorType::ChildOf);

        ElementViewState {
            node,
            child_view_state,
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
        self.children.rebuild(
            &prev.children,
            &mut state.child_view_state,
            ctx,
            state.node.upcast_mut::<Node>(),
            AnchorType::ChildOf,
        );
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        self.children.teardown(
            &mut state.child_view_state,
            ctx,
            state.node.upcast_mut(),
            AnchorType::ChildOf,
        );

        anchor_type.remove(anchor, &state.node.clone().upcast());
        state.node.upcast_mut::<Node>().queue_free();
    }

    fn notify_state(
        &self,
        path: &[ViewId],
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) {
        self.children
            .notify_state(path, &mut state.child_view_state, ctx, anchor, anchor_type)
    }

    fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<Gd<Node>>) {
        nodes.push(state.node.clone().upcast::<Node>());
    }
}

pub trait ElementView<N: Inherits<Node>>: View + Sized {
    fn get_node(&self, state: &Self::ViewState) -> Gd<N>;
}

impl<N, Children> ElementView<N> for Element<N, Children>
where
    N: Inherits<Node> + NewAlloc,
    Children: View,
{
    fn get_node(&self, state: &Self::ViewState) -> Gd<N> {
        state.node.clone()
    }
}

// doing this instead of the trait because rust was smelly
macro_rules! impl_element_view {
    ($node:ident) => {
        pub fn attr<Name, Value, const BUILD_ONLY: bool>(
            self,
            name: Name,
            value: Value,
        ) -> $crate::view::element::attr::Attr<$node, Name, Self, BUILD_ONLY>
        where
            Name: AsRef<str>,
            Value: ToGodot,
            $node: godot::prelude::Inherits<godot::prelude::Node>,
        {
            use std::marker::PhantomData;
            $crate::view::element::attr::Attr {
                inner: self,
                name,
                value: value.to_variant(),
                _p: PhantomData,
            }
        }
        pub fn on_signal<Name, Cb>(
            self,
            name: Name,
            cb: Cb,
        ) -> $crate::view::element::on_signal::OnSignal<$node, Name, Cb, Self>
        where
            Name: AsRef<str>,
            Cb: Fn(&[&Variant]) + 'static,
            $node: godot::prelude::Inherits<godot::prelude::Node>,
        {
            use std::marker::PhantomData;
            $crate::view::element::on_signal::OnSignal {
                inner: self,
                name,
                cb: cb.into(),
                _p: PhantomData,
            }
        }
        pub fn on_build<Cb>(self, cb: Cb) -> $crate::OnBuild<$node, Cb, Self>
        where
            Cb: Fn(Gd<$node>),
            $node: godot::prelude::Inherits<godot::prelude::Node>,
        {
            use std::marker::PhantomData;
            $crate::OnBuild {
                inner: self,
                cb,
                _p: PhantomData,
            }
        }
        pub fn on_teardown<Cb>(self, cb: Cb) -> $crate::OnTeardown<$node, Cb, Self>
        where
            Cb: Fn(),
            $node: godot::prelude::Inherits<godot::prelude::Node>,
        {
            use std::marker::PhantomData;
            $crate::OnTeardown {
                inner: self,
                cb,
                _p: PhantomData,
            }
        }
        pub fn theme_override<Typ: crate::ThemeOverrideType, Name>(
            self,
            name: Name,
            value: Typ::ValueType,
        ) -> $crate::ThemeOverride<$node, Typ, Name, Self>
        where
            Name: AsRef<str>,
            $node: godot::prelude::Inherits<godot::prelude::Node>,
        {
            use std::marker::PhantomData;
            $crate::ThemeOverride {
                inner: self,
                name,
                value,
                _p: PhantomData,
            }
        }
    };
}
pub(crate) use impl_element_view;

use crate::view::{AnchorType, View, ViewId};

impl<N, Children> Element<N, Children> {
    impl_element_view! { N }
}
