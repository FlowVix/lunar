pub mod state;

use std::{any::Any, cell::RefCell, marker::PhantomData, rc::Rc};

use ahash::AHashSet;
use slotmap::{SlotMap, new_key_type};

new_key_type! {
    pub struct ScopeId;
    pub struct StateId;
}

pub struct Scope {
    pub parent: Option<ScopeId>,
    pub children: AHashSet<ScopeId>,

    pub cleanups: Vec<Box<dyn FnOnce()>>,

    pub states: Vec<Rc<RefCell<dyn Any>>>,
}

pub struct System {
    pub scopes: SlotMap<ScopeId, Scope>,
    pub current_scope: Option<(ScopeId, usize)>,
}

thread_local! {
    pub static SYSTEM: RefCell<System> = RefCell::new(System {
        scopes: SlotMap::default(),
        current_scope: None,
    });
}

impl System {
    pub fn remove_scope(&mut self, id: ScopeId) {
        let Some(scope) = self.scopes.remove(id) else {
            return;
        };
        if let Some(p) = scope.parent {
            self.scopes[p].children.remove(&id);
        }
        for f in scope.cleanups {
            f();
        }
        for child in scope.children {
            self.remove_scope(child);
        }
    }
}

pub fn child_scope<R, F: FnOnce() -> R>(f: F) -> (R, ScopeId) {
    let (prev, child) = SYSTEM.with_borrow_mut(|system| {
        let child = system.scopes.insert(Scope {
            parent: system.current_scope.map(|v| v.0),
            children: AHashSet::new(),
            cleanups: vec![],
            states: vec![],
        });
        if let Some((current, _)) = system.current_scope {
            system.scopes[current].children.insert(child);
        }
        let prev = system.current_scope;
        system.current_scope = Some((child, 0));
        (prev, child)
    });
    (
        f(),
        SYSTEM.with_borrow_mut(|system| {
            system.current_scope = prev;
            child
        }),
    )
}
pub fn in_scope<R, F: FnOnce() -> R>(scope: ScopeId, f: F) -> R {
    let prev = SYSTEM.with_borrow_mut(|system| {
        let prev = system.current_scope;
        system.current_scope = Some((scope, 0));
        prev
    });
    let ret = f();
    SYSTEM.with_borrow_mut(|system| {
        system.current_scope = prev;
    });
    ret
}
