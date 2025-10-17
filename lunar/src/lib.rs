#![deny(unused_must_use, unnameable_types)]
#![allow(clippy::too_many_arguments, clippy::single_match)]

mod app;
mod ctx;
mod system;
mod view;

pub use app::{App, run};
pub use ctx::Context;
pub use either;
pub use lunar_macro::view;
pub use view::{
    AnchorType, View, ViewId,
    any::{AnyView, AnyViewState},
    either::EitherViewState,
    element::{
        Element, ElementView, ElementViewState,
        attr::{Attr, AttrViewState},
        el,
        on_build::{OnBuild, OnBuildViewState},
        on_signal::{OnSignal, OnSignalViewState},
        on_teardown::{OnTeardown, OnTeardownViewState},
        theme_override::{
            ThemeOverride, ThemeOverrideColor, ThemeOverrideConstant, ThemeOverrideFont,
            ThemeOverrideFontSize, ThemeOverrideIcon, ThemeOverrideStylebox, ThemeOverrideType,
            ThemeOverrideViewState,
        },
    },
    option::OptionViewState,
    stateful::{Stateful, StatefulViewState, state::State, stateful},
};

// fn goy() -> impl View {
//     state(0, |count| )
// }
