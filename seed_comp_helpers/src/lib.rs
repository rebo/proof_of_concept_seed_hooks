#[macro_use]
extern crate seed;
pub mod helpers;

use comp_state::Store;
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
        // clone_all!(url, state_access);
        Box::new(move || {
            if let Some(app) = topo::Env::get::<seed::App<Ms, Mdl, Node<Ms>>>() {
                app.update(msg.clone());
            }
        })
    };

    // let (once, once_access) = use_state(|| false);
    // if !once {
    seed::set_timeout(boxed_fn, 0);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
