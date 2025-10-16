use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use godot::{
    classes::Node,
    obj::{Gd, Inherits},
};
use slotmap::Key;

use crate::{
    ctx::Context,
    system::{APPS, AppData, AppId},
    view::{AnchorType, View, any::AnyView},
};

pub struct App {
    id: AppId,
}

pub fn run<N, V, F>(root: Gd<N>, f: F) -> App
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
