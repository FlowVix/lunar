#![deny(unused_must_use, unnameable_types)]
#![allow(clippy::too_many_arguments, clippy::single_match)]

mod app;
mod ctx;
mod system;
mod util;
mod view;

use std::rc::Rc;

pub use app::{App, start};
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
        node_ref::{NodeRef, NodeRefViewState},
        on_signal::{OnSignal, OnSignalViewState},
        theme_override::{
            ThemeOverride, ThemeOverrideColor, ThemeOverrideConstant, ThemeOverrideFont,
            ThemeOverrideFontSize, ThemeOverrideIcon, ThemeOverrideStylebox, ThemeOverrideType,
            ThemeOverrideViewState,
        },
    },
    iter::VecViewState,
    memo::{Memo, MemoViewState, memo},
    on_build::{OnBuild, on_build},
    on_change::{OnChange, on_change, on_change_init},
    on_physics_process::{__LunarOnPhysicsProcessNode, OnPhysicsProcess, on_physics_process},
    on_process::{__LunarOnProcessNode, OnProcess, on_process},
    on_ready::{__LunarOnReadyNode, OnReady, on_ready},
    on_rebuild::{OnRebuild, on_rebuild},
    on_teardown::{OnTeardown, on_teardown},
    option::OptionViewState,
    stateful::{Stateful, StatefulViewState, state::State, stateful, stateful_quiet},
    when::{When, WhenViewState, when},
};
