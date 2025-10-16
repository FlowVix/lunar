use crate::{system::AppId, view::ViewId};

pub struct Context {
    pub(crate) app_id: AppId,
    pub(crate) id_counter: u64,
    pub(crate) path: Vec<ViewId>,
}

impl Context {
    pub(crate) fn new_structural_id(&mut self) -> ViewId {
        let out = ViewId::Structural(self.id_counter);
        self.id_counter += 1;
        out
    }
    pub(crate) fn with_id<R>(&mut self, id: ViewId, f: impl FnOnce(&mut Self) -> R) -> R {
        self.path.push(id);
        let out = f(self);
        self.path.pop();
        out
    }
}
