// pub mod any;
pub mod option;
pub mod scoped;

use godot::{classes::Node, obj::Gd};

use crate::system::ScopeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnchorType {
    ChildOf,
    Before,
}
impl AnchorType {
    pub fn add(self, anchor: &mut Node, node: &Gd<Node>) {
        match self {
            AnchorType::ChildOf => anchor.add_child(node),
            AnchorType::Before => {
                let idx = anchor.get_index();
                let mut parent = anchor.get_parent().unwrap();
                parent.add_child(node);
                parent.move_child(node, idx);
            }
        }
    }
    pub fn remove(self, anchor: &mut Node, node: &Gd<Node>) {
        match self {
            AnchorType::ChildOf => anchor.remove_child(node),
            AnchorType::Before => anchor.get_parent().unwrap().remove_child(node),
        }
    }
}

pub trait View {
    type ViewState;

    fn build(
        &self,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) -> (Self::ViewState, Option<ScopeId>);
    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) -> Option<ScopeId>;
    fn teardown(&self, state: &mut Self::ViewState, anchor: &mut Node, anchor_type: AnchorType);
}
