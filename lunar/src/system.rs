use std::{any::Any, cell::RefCell, rc::Rc};

use godot::{classes::Node, obj::Gd};
use slotmap::{SecondaryMap, SlotMap, new_key_type};

use crate::{
    ctx::Context,
    view::{
        ViewId,
        any::{AnyView, AnyViewState},
    },
};

new_key_type! {
    pub struct StateId;
    pub struct AppId;
}

pub struct StateData {
    pub value: Rc<RefCell<dyn Any>>,
    pub path: Rc<[ViewId]>,
}

pub struct AppData {
    pub ctx: Rc<RefCell<Context>>,
    pub view: Rc<RefCell<Box<dyn AnyView>>>,
    pub view_state: Rc<RefCell<AnyViewState>>,
    pub root: Gd<Node>,
}

pub struct System {
    pub states: SlotMap<StateId, StateData>,
    pub apps: SlotMap<AppId, AppData>,
}

thread_local! {
    pub static STATES: RefCell<SlotMap<StateId, StateData>> = RefCell::new(SlotMap::default());
    pub static APPS: RefCell<SlotMap<AppId, AppData>> = RefCell::new(SlotMap::default());
    pub static APP_NOTIFICATIONS: RefCell<SecondaryMap<AppId, Vec<Rc<[ViewId]>>>> = RefCell::new(SecondaryMap::new());
}
