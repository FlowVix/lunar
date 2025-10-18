use std::{marker::PhantomData, rc::Rc};

use godot::global::godot_print;

use crate::{
    system::{APP_NOTIFICATIONS, APPS, AppId, STATES, StateId},
    view::AnchorType,
};

pub struct State<T: 'static> {
    pub(crate) state_id: StateId,
    pub(crate) app_id: AppId,
    pub(crate) quiet: bool,
    pub(crate) _p: PhantomData<*const T>,
}
impl<T> Copy for State<T> {}
impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> State<T> {
    pub fn notify(&self) {
        if self.quiet {
            return;
        }
        let path = STATES.with_borrow_mut(|states| states[self.state_id].path.clone());
        APP_NOTIFICATIONS
            .with_borrow_mut(|map| map.entry(self.app_id).unwrap().or_default().push(path));
    }
    pub fn get(&self) -> T
    where
        T: Copy,
    {
        STATES.with_borrow(|states| *states[self.state_id].value.borrow().downcast_ref().unwrap())
    }
    pub fn get_clone(&self) -> T
    where
        T: Clone,
    {
        STATES.with_borrow(|states| {
            states[self.state_id]
                .value
                .borrow()
                .downcast_ref::<T>()
                .unwrap()
                .clone()
        })
    }
    pub fn with<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let value = STATES.with_borrow_mut(|states| states[self.state_id].value.clone());
        f(value
            .try_borrow()
            .expect("cannot get value during an `update` call")
            .downcast_ref::<T>()
            .unwrap())
    }
    pub fn set(&self, to: T) {
        STATES.with_borrow(|states| {
            *states[self.state_id]
                .value
                .borrow_mut()
                .downcast_mut()
                .unwrap() = to;
        });
        self.notify();
    }
    pub fn update<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let value = STATES.with_borrow_mut(|states| states[self.state_id].value.clone());
        let ret = f(value
            .try_borrow_mut()
            .expect("cannot update value during another `update` call")
            .downcast_mut::<T>()
            .unwrap());
        self.notify();
        ret
    }
}
