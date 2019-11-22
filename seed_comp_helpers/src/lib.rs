#[macro_use]
extern crate seed;
pub mod helpers;

pub use helpers::event_helpers::{on_click, on_input};
pub use helpers::form_state;
pub use helpers::graphql_list;
pub use helpers::two_way;
pub use helpers::use_fetch_helper;
use seed::prelude::*;

pub fn init<Ms: 'static, Mdl: 'static, O: Orders<Ms>>(orders: &mut O) {
    comp_state::init_root_context();
    if topo::Env::get::<seed::App<Ms, Mdl, Node<Ms>>>().is_none() {
        topo::Env::add(orders.clone_app());
    }
    comp_state::init_root_context();
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
