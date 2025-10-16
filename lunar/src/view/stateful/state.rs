use std::marker::PhantomData;

use godot::global::godot_print;

use crate::{
    system::{APPS, AppId, STATES, StateId},
    view::AnchorType,
};

pub struct State<T: 'static> {
    pub(crate) state_id: StateId,
    pub(crate) app_id: AppId,
    pub(crate) _p: PhantomData<T>,
}
impl<T> Copy for State<T> {}
impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> State<T> {
    pub fn notify(&self) {
        let (ctx, view, view_state, mut root) = APPS.with_borrow(|apps| {
            let v = &apps[self.app_id];
            (
                v.ctx.clone(),
                v.view.clone(),
                v.view_state.clone(),
                v.root.clone(),
            )
        });
        let path = STATES.with_borrow_mut(|states| states[self.state_id].path.clone());

        view.borrow().dyn_notify_state(
            &path,
            &mut view_state.borrow_mut(),
            &mut ctx.borrow_mut(),
            &mut root,
            AnchorType::ChildOf,
        );
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
}
