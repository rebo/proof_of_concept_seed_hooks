use seed::{prelude::*, *};

pub fn on_click<Ms, F>(func: F) -> events::Listener<Ms>
where
    Ms: Default + Clone,
    F: FnOnce(web_sys::MouseEvent) -> () + 'static + Clone,
{
    mouse_ev(Ev::Click, |a| {
        func(a);
        Ms::default()
    })
}
