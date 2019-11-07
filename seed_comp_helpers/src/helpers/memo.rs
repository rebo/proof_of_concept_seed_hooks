// use crate::store::*;
// use anymap::any::Any;
// // use seed::prelude::*;
// use std::rc::Rc;

// pub fn memoize_value<T: Sync + Send + 'static + Clone>(value: T) -> T {
//     let mut map = anymap::Map::new();
//     map.insert::<T>(value);
//     let arc_anymap: Arc<anymap::Map<dyn Any + Sync + Send>> = Arc::new(map);
//     let (value, set_value) = use_state(arc_anymap);
//     value.get::<T>().cloned().unwrap()
// }

use comp_state::*;

#[derive(Clone)]
pub struct MemoControl(StateAccess<bool>);

impl MemoControl {
    pub fn recalc(&self, trigger: bool) {
        self.0.set(trigger);
    }
}

pub struct Watch<T: Clone + 'static> {
    state_access: StateAccess<T>,
    pub changed: bool,
}
impl<T> Watch<T>
where
    T: Clone + 'static,
{
    pub fn new(changed: bool, state_access: StateAccess<T>) -> Watch<T> {
        Watch {
            changed,
            state_access,
        }
    }

    pub fn hard_get(&self) -> T {
        self.state_access.get().unwrap()
    }
}

use std::fmt::Debug;
pub fn watch<T: 'static + Clone + Debug + PartialEq>(current_watched: &T) -> Watch<T> {
    topo::call!({
        let current_watched_clone = current_watched.clone();
        let (watched, watch_access) = use_state(|| current_watched.clone());
        if watched != current_watched_clone {
            // log!(format!(
            //     "watched:{:#?} changed to {:#?}",
            //     watched, current_watched_clone
            // ));
            watch_access.set(current_watched.clone());
            Watch::new(true, watch_access)
        } else {
            Watch::new(false, watch_access)
        }
    })
}

pub fn use_memo<T: 'static + Clone, F: Fn() -> T>(recalc: bool, func: F) -> (T, MemoControl) {
    topo::call!({
        let (update, recalc_trigger_access) = use_state(|| false);

        // let arc_func = Arc::new(func);
        let new_func = || func();

        // by definition this will keep returning 'value'
        // until update is set to true.

        let (mut value, value_access) = use_state(new_func);

        if update || recalc {
            value = func();
            value_access.set(value.clone());
            recalc_trigger_access.set(false);
        }
        (value, MemoControl(recalc_trigger_access))
    })
}
