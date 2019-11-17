use seed::{prelude::*, *};

pub fn on_click<Ms, F>(func: F) -> events::Listener<Ms>
where
    F: FnOnce(web_sys::MouseEvent) -> Ms + 'static + Clone,
{
    mouse_ev(Ev::Click, func)
}
