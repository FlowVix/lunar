use std::rc::Rc;

use lunar::view;

fn main() {
    view! {
        let bob: i32 = 6 * {
            3;
            4 + 8
        } + 9;
    };
}
