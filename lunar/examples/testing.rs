use std::rc::Rc;

use godot::classes::Button;
use lunar::{View, view};

fn main() {
    view! {
        state a = 3;
        state b = 3;

        Button
    };
}
