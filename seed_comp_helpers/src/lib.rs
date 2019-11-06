#[macro_use]
extern crate seed;
pub mod helpers;

use comp_state::{use_state, Store};
pub use helpers::form_state;
pub use helpers::list;
pub use helpers::memo;
pub use helpers::two_way;
pub use helpers::use_fetch_helper;
use seed::prelude::*;
use std::cell::RefCell;

pub fn init<Ms: 'static, Mdl: 'static, O: Orders<Ms>>(orders: &mut O) {
    if topo::Env::get::<seed::App<Ms, Mdl, Node<Ms>>>().is_none() {
        topo::Env::add(orders.clone_app());
    }

    if topo::Env::get::<RefCell<Store>>().is_none() {
        topo::Env::add(RefCell::new(Store::default()));
    }
}

pub fn schedule_update<Ms: Clone + 'static, Mdl: 'static>(msg: Ms) {
    let boxed_fn = {
        Box::new(move || {
            if let Some(app) = topo::Env::get::<seed::App<Ms, Mdl, Node<Ms>>>() {
                app.update(msg.clone());
            }
        })
    };
    seed::set_timeout(boxed_fn, 0);
}

pub fn do_once<F: Fn() -> ()>(func: F) {
    topo::call!({
        let (has_done, has_done_access) = use_state(|| false);
        if !has_done {
            func();
            has_done_access.set(true);
        }
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
