use std::marker::PhantomData;

use crate::system::{SYSTEM, StateId};

pub struct State<T: 'static> {
    pub(crate) id: StateId,
    pub(crate) _p: PhantomData<T>,
}
impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for State<T> {}

// impl<T> State<T> {
//     pub fn get(&self) -> T
//     where
//         T: Copy,
//     {
//         SYSTEM.with_borrow(|s| *s.states[self.id].value.borrow().downcast_ref().unwrap())
//     }
//     pub fn get_clone(&self) -> T
//     where
//         T: Clone,
//     {
//         SYSTEM.with_borrow(|s| {
//             s.states[self.id]
//                 .value
//                 .borrow()
//                 .downcast_ref::<T>()
//                 .unwrap()
//                 .clone()
//         })
//     }
// }
