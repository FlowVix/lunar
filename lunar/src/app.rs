use std::{cell::RefCell, marker::PhantomData, mem, rc::Rc};

use godot::{
    classes::Node,
    obj::{Gd, Inherits},
};
use slotmap::Key;

use crate::{
    ctx::Context,
    system::{APP_NOTIFICATIONS, APPS, AppData, AppId},
    view::{AnchorType, View, any::AnyView},
};

pub struct App {
    id: AppId,
}

pub fn start<N, V, F>(root: Gd<N>, f: F) -> App
where
    N: Inherits<Node>,
    V: View + 'static,
    F: FnOnce() -> V,
{
    let mut root = root.upcast::<Node>();
    let view = Box::new(f()) as Box<dyn AnyView>;

    let id = APPS.with_borrow_mut(|apps| {
        apps.insert_with_key(|id| {
            let mut ctx = Context {
                app_id: id,
                id_counter: 0,
                path: vec![],
            };

            let view_state = view.build(&mut ctx, &mut root, AnchorType::ChildOf);
            let ctx = Rc::new(RefCell::new(ctx));
            AppData {
                ctx: ctx.clone(),
                view: Rc::new(RefCell::new(view)),
                view_state: Rc::new(RefCell::new(view_state)),
                root,
            }
        })
    });
    App { id }
}
impl App {
    pub fn run(&self) {
        let Some(paths) =
            APP_NOTIFICATIONS.with_borrow_mut(|map| map.get_mut(self.id).map(mem::take))
        else {
            return;
        };

        let (ctx, view, view_state, mut root) = APPS.with_borrow(|apps| {
            let v = &apps[self.id];
            (
                v.ctx.clone(),
                v.view.clone(),
                v.view_state.clone(),
                v.root.clone(),
            )
        });

        for path in paths {
            view.borrow().notify_state(
                &path,
                &mut view_state.borrow_mut(),
                &mut ctx.borrow_mut(),
                &mut root,
                AnchorType::ChildOf,
            );
        }
    }
}
